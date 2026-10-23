use nota_app::app::{AppModel, AppliedImport};
use nota_app::storage::CollectionEnvelope;
use nota_app::{
    EditorViewMode, FocusIntent, ImportDestination, Note, NoteListDisplayState, ThemePreference,
    export_flat_collection_backup, stage_import,
};

#[test]
fn merge_import_does_not_apply_before_backup_import_preview_confirmation() {
    let existing = Note::new("Local".to_string(), "Keep".to_string());
    let imported = Note::new("Backup".to_string(), "New".to_string());
    let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
    let mut app = AppModel::new(
        CollectionEnvelope::new(vec![existing.clone()], Vec::new()),
        ThemePreference::Light,
        None,
    );

    stage_json(&mut app, &json);
    assert_eq!(app.workspace.notes(), std::slice::from_ref(&existing));
    assert_eq!(
        app.pending_import().unwrap().body,
        "Notes in this file: 1\n\nNotes to add: 1\nNotes to replace: 0"
    );
}

#[test]
fn merge_import_recovers_a_deleted_note_and_keeps_the_collection_valid() {
    let note = Note::new("Recovered".to_string(), "From Backup".to_string());
    let note_id = note.id;
    let json = export_flat_collection_backup(std::slice::from_ref(&note)).unwrap();
    let mut app = AppModel::new(
        CollectionEnvelope::new(vec![note.clone()], Vec::new()),
        ThemePreference::Light,
        None,
    );
    app.request_delete(note_id);
    assert!(app.confirm_delete());

    stage_json(&mut app, &json);
    assert_eq!(
        app.pending_import().unwrap().body,
        "Notes in this file: 1\n\nNotes to add: 0\nNotes to replace: 1\nRecently Deleted notes that will return: 1"
    );
    assert_eq!(
        app.commit_staged_import().unwrap(),
        Some(AppliedImport::Merged)
    );

    assert_eq!(app.workspace.notes(), &[note]);
    assert!(app.workspace.recently_deleted_notes().is_empty());
    let temp = tempfile::tempdir().unwrap();
    nota_app::storage::NativeStore::at(temp.path())
        .save_collection(&app.collection())
        .unwrap();
}

#[test]
fn storage_recovery_keeps_import_backup_and_does_not_start_empty_silently() {
    let imported = Note::new("Recovered".to_string(), "From Backup".to_string());
    let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
    let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
    app.set_storage_recovery(true);

    app.quick_capture();
    assert!(app.is_in_storage_recovery());
    stage_json(&mut app, &json);
    assert_eq!(
        app.commit_staged_import().unwrap(),
        Some(AppliedImport::Merged)
    );
    assert!(!app.is_in_storage_recovery());
}

fn stage_json(app: &mut AppModel, json: &str) {
    let staged = stage_import(
        json,
        ImportDestination {
            notes: app.workspace.notes(),
            recently_deleted: app.workspace.recently_deleted_notes(),
            storage_recovery: app.is_in_storage_recovery(),
        },
    )
    .unwrap();
    app.stage_import(staged);
}

#[test]
fn filtered_search_does_not_use_empty_collection_copy() {
    let note = Note::new("Roadmap".to_string(), "Ship native".to_string());
    let mut app = AppModel::new(
        CollectionEnvelope::new(vec![note], Vec::new()),
        ThemePreference::Light,
        None,
    );
    app.edit_search("zzzz".to_string());
    app.commit_search();
    let model = app.note_list_render_model();
    assert_eq!(model.display_state, NoteListDisplayState::FilteredEmpty);
    let copy = app
        .note_list
        .list_copy(app.workspace.notes(), app.workspace.selected_id());
    assert_ne!(copy.title, "No notes yet.");
}

#[test]
fn tag_cleanup_message_applies_a_reviewed_plan_as_one_saved_change() {
    let mut note = Note::new("Dirty tags".to_string(), String::new());
    note.tags = vec![" Work ".to_string(), "work".to_string()];
    let mut app = AppModel::new(
        CollectionEnvelope::new(vec![note], Vec::new()),
        ThemePreference::Light,
        None,
    );

    assert!(app.apply_tag_cleanup());

    assert_eq!(app.workspace.notes()[0].tags, ["Work"]);
    assert_eq!(app.revision(), 1);
    assert_eq!(app.save_status, nota_app::app::SaveStatus::Saving);
    assert_eq!(
        app.notification.as_ref().unwrap().message,
        "Tags cleaned up"
    );
    let notification_generation = app.notification_generation();
    assert!(!app.apply_tag_cleanup());
    assert_eq!(app.revision(), 1);
    assert_eq!(app.notification_generation(), notification_generation);
}

#[test]
fn quick_capture_requests_note_title_focus() {
    let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
    assert!(app.quick_capture());
    assert_eq!(app.workspace.focus_intent(), FocusIntent::NoteTitle);
}

#[test]
fn preview_and_split_expose_documented_view_mode_surfaces() {
    let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
    app.set_view_mode(EditorViewMode::Preview);
    assert!(app.layout.show_preview);
    assert!(!app.layout.show_writing);

    app.set_view_mode(EditorViewMode::Split);
    assert!(app.layout.show_preview);
    assert!(app.layout.show_writing);

    app.resize(600.0);
    assert_eq!(app.view_mode, EditorViewMode::Split);
    assert_eq!(app.layout.editor_min_height_px, 0.0);
    app.resize(560.0);
    assert_eq!(app.layout.preview_min_height_px, 500.0);
    assert_eq!(app.layout.editor_min_height_px, 340.0);
}
