use nota_core::backup::export_flat_collection_backup;
use nota_core::editor_view::EditorViewMode;
use nota_core::note_list_interaction::NoteListDisplayState;
use nota_core::note_workspace::FocusIntent;
use nota_core::transition::ThemePreference;
use nota_desktop::app::{AppModel, AppMsg};
use nota_desktop::storage::CollectionEnvelope;

#[test]
fn merge_import_does_not_apply_before_backup_import_preview_confirmation() {
    let existing = nota_core::Note::new("Local".to_string(), "Keep".to_string());
    let imported = nota_core::Note::new("Backup".to_string(), "New".to_string());
    let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
    let mut app = AppModel::new(
        CollectionEnvelope::new(vec![existing.clone()], Vec::new()),
        ThemePreference::Light,
        None,
    );

    app.apply(AppMsg::ImportBackupJson(json));
    assert_eq!(app.workspace.notes(), std::slice::from_ref(&existing));
    assert!(app.pending_backup_import().is_some());
}

#[test]
fn storage_recovery_keeps_import_backup_and_does_not_start_empty_silently() {
    let imported = nota_core::Note::new("Recovered".to_string(), "From Backup".to_string());
    let json = export_flat_collection_backup(std::slice::from_ref(&imported)).unwrap();
    let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
    app.set_storage_recovery(true);

    assert!(!app.apply(AppMsg::QuickCapture));
    app.apply(AppMsg::ImportBackupJson(json));
    assert!(app.apply(AppMsg::ConfirmBackupImport));
    assert!(!app.is_in_storage_recovery());
}

#[test]
fn filtered_search_does_not_use_empty_collection_copy() {
    let note = nota_core::Note::new("Roadmap".to_string(), "Ship native".to_string());
    let mut app = AppModel::new(
        CollectionEnvelope::new(vec![note], Vec::new()),
        ThemePreference::Light,
        None,
    );
    app.apply(AppMsg::EditSearch("zzzz".to_string()));
    app.apply(AppMsg::CommitSearch);
    let model = app.note_list_render_model();
    assert_eq!(model.display_state, NoteListDisplayState::FilteredEmpty);
}

#[test]
fn quick_capture_requests_note_title_focus() {
    let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
    assert!(app.apply(AppMsg::QuickCapture));
    assert_eq!(app.workspace.focus_intent(), FocusIntent::NoteTitle);
}

#[test]
fn preview_and_split_expose_documented_view_mode_surfaces() {
    let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::Light, None);
    app.apply(AppMsg::SetViewMode(EditorViewMode::Preview));
    assert!(app.view_mode.surfaces().preview);
    assert!(!app.view_mode.surfaces().writing);

    app.apply(AppMsg::SetViewMode(EditorViewMode::Split));
    assert!(app.view_mode.surfaces().preview);
    assert!(app.view_mode.surfaces().writing);

    app.apply(AppMsg::Resize(600.0));
    assert_eq!(app.view_mode, EditorViewMode::Write);
}
