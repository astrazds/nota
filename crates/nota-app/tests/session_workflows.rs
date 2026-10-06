use nota_app::session::{OpenOptionsJson, Request, Session};
use nota_app::storage::{CollectionEnvelope, NativeStore};
use nota_core::Note;
use nota_core::backup::export_flat_collection_backup;
use serde_json::{Value, json};

fn open(path: &std::path::Path) -> Session {
    Session::open(OpenOptionsJson {
        data_directory: Some(path.to_path_buf()),
    })
    .unwrap()
}

fn command(session: &mut Session, value: Value) -> Value {
    session
        .execute(serde_json::from_value::<Request>(value).unwrap())
        .unwrap()
}

#[test]
fn preview_layout_changes_type_size_without_changing_markdown_content_or_policy() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = open(temp.path());
    for (layout, font) in [
        ("reading", "font:18px/1.9 'Gelasio'"),
        ("split", "font:15px/1.9 'Gelasio'"),
        ("split_narrow", "font:14px/1.9 'Gelasio'"),
    ] {
        let reply = command(
            &mut session,
            json!({"command":"preview", "title":"Notebook", "content":"# Notebook\n\nBody **text**", "dark":false, "layout":layout}),
        );
        let html = reply["result"]["html"].as_str().unwrap();
        assert!(html.contains(font));
        assert!(html.contains("<p>Body <strong>text</strong></p>"));
        assert!(!html.contains("<h1>Notebook</h1>"));
        assert!(html.contains("script-src 'none'"));
    }
    assert!(serde_json::from_value::<Request>(json!({"command":"preview", "title":"Notebook", "content":"Body", "dark":false, "layout":"editing"})).is_err());
}

#[test]
fn new_note_clears_search_tag_and_pinned_filters_so_the_note_is_visible() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = open(temp.path());
    let existing =
        command(&mut session, json!({"command":"new_note"}))["snapshot"]["selected_note"]["id"]
            .clone();
    command(
        &mut session,
        json!({"command":"edit_note", "id":existing, "edit_sequence":1, "title":"Release plan", "content":"", "tags_input":"Work"}),
    );
    command(&mut session, json!({"command":"toggle_pin", "id":existing}));
    command(
        &mut session,
        json!({"command":"search", "query":"title:release"}),
    );
    command(&mut session, json!({"command":"filter_tag", "tag":"Work"}));
    let filtered = command(
        &mut session,
        json!({"command":"filter_pinned", "pinned":true}),
    );
    assert_eq!(filtered["snapshot"]["rows"].as_array().unwrap().len(), 1);
    let created = command(&mut session, json!({"command":"new_note"}));
    let snapshot = &created["snapshot"];
    let new_id = &snapshot["selected_note"]["id"];
    assert_ne!(new_id, &existing);
    assert_eq!(snapshot["rows"].as_array().unwrap().len(), 2);
    assert!(
        snapshot["rows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| &row["id"] == new_id)
    );
    assert_eq!(snapshot["search_input"], "");
    assert_eq!(snapshot["active_tag"], Value::Null);
    assert_eq!(snapshot["pinned_only"], false);
    assert_eq!(snapshot["view_mode"], "write");
}

#[test]
fn pinned_filter_changes_visible_rows_without_changing_selection_search_or_revision() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = open(temp.path());
    let first =
        command(&mut session, json!({"command":"new_note"}))["snapshot"]["selected_note"]["id"]
            .clone();
    command(
        &mut session,
        json!({"command":"edit_note", "id":first, "edit_sequence":1, "title":"Release plan", "content":"", "tags_input":"Work"}),
    );
    command(&mut session, json!({"command":"toggle_pin", "id":first}));
    let second = command(&mut session, json!({"command":"new_note"}))["snapshot"]["selected_note"]
        ["id"]
        .clone();
    command(
        &mut session,
        json!({"command":"edit_note", "id":second, "edit_sequence":2, "title":"Release checklist", "content":"", "tags_input":"Work"}),
    );
    command(
        &mut session,
        json!({"command":"search", "query":"title:release"}),
    );
    let before = command(&mut session, json!({"command":"filter_tag", "tag":"Work"}));
    assert_eq!(before["snapshot"]["rows"].as_array().unwrap().len(), 2);
    let filtered = command(
        &mut session,
        json!({"command":"filter_pinned", "pinned":true}),
    );
    assert_eq!(filtered["snapshot"]["pinned_only"], true);
    assert_eq!(filtered["snapshot"]["rows"].as_array().unwrap().len(), 1);
    assert_eq!(filtered["snapshot"]["rows"][0]["id"], first);
    assert_eq!(filtered["snapshot"]["selected_note"]["id"], second);
    assert_eq!(filtered["snapshot"]["search_input"], "title:release");
    assert_eq!(filtered["snapshot"]["active_tag"], "Work");
    assert_eq!(
        filtered["snapshot"]["revision"],
        before["snapshot"]["revision"]
    );
    let all = command(
        &mut session,
        json!({"command":"filter_pinned", "pinned":false}),
    );
    assert_eq!(all["snapshot"]["pinned_only"], false);
    assert_eq!(all["snapshot"]["rows"].as_array().unwrap().len(), 2);
}

#[test]
fn edits_follow_the_note_identity_and_reopen_with_unicode_and_tags() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = open(temp.path());
    let first = command(&mut session, json!({"command": "new_note"}))["snapshot"]["selected_note"]
        ["id"]
        .clone();
    let second = command(&mut session, json!({"command": "new_note"}))["snapshot"]["selected_note"]
        ["id"]
        .clone();
    let edited = command(
        &mut session,
        json!({"command":"edit_note", "id":first, "edit_sequence": 2,
        "title":"日本語 📝", "content":"café\n😀", "tags_input":"Work, Rust"}),
    );
    assert_eq!(edited["snapshot"]["selected_note"]["id"], second);
    assert_eq!(edited["snapshot"]["edit_sequence"], 2);
    let stale = serde_json::from_value(
        json!({"command":"edit_note", "id":first, "edit_sequence": 1,
        "title":"stale", "content":"lost", "tags_input":""}),
    )
    .unwrap();
    assert_eq!(session.execute(stale).unwrap_err().code, "stale_edit");
    command(&mut session, json!({"command":"flush"}));
    drop(session);
    let mut reopened = open(temp.path());
    let selected = command(&mut reopened, json!({"command":"select_note", "id":first}));
    assert_eq!(selected["snapshot"]["selected_note"]["title"], "日本語 📝");
    assert_eq!(selected["snapshot"]["selected_note"]["content"], "café\n😀");
    assert_eq!(
        selected["snapshot"]["selected_note"]["tags"],
        json!(["Work", "Rust"])
    );
}

#[test]
fn formatting_uses_utf16_selection_and_rejects_a_split_surrogate() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = open(temp.path());
    let reply = command(
        &mut session,
        json!({"command":"format", "content":"a😀é中z",
        "start_utf16":1, "end_utf16":5, "kind":"bold"}),
    );
    assert_eq!(reply["result"]["content"], "a**😀é中**z");
    assert_eq!(reply["result"]["caret_utf16"], 9);
    let split = serde_json::from_value(json!({"command":"format", "content":"😀",
        "start_utf16":1, "end_utf16":2, "kind":"italic"}))
    .unwrap();
    assert_eq!(
        session.execute(split).unwrap_err().code,
        "invalid_selection"
    );
    assert!(session.app.workspace.notes().is_empty());
}

#[test]
fn failed_flush_keeps_pending_edits_and_same_session_can_retry() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = open(temp.path());
    let id =
        command(&mut session, json!({"command":"new_note"}))["snapshot"]["selected_note"]["id"]
            .clone();
    command(
        &mut session,
        json!({"command":"edit_note", "id":id, "edit_sequence":1,
        "title":"Must survive", "content":"latest text", "tags_input":""}),
    );
    let collection_path = temp.path().join("collection.json");
    std::fs::create_dir(&collection_path).unwrap();
    assert!(session.flush().is_err());
    assert_eq!(session.snapshot()["save_status"], "failed");
    std::fs::remove_dir(&collection_path).unwrap();
    session.flush().unwrap();
    assert_eq!(session.snapshot()["save_status"], "saved");
    drop(session);
    let reopened = open(temp.path());
    assert_eq!(reopened.app.workspace.notes()[0].title, "Must survive");
    assert_eq!(reopened.app.workspace.notes()[0].content, "latest text");
}

#[test]
fn recovery_gates_mutations_and_preserves_corrupt_bytes_when_restoring_previous() {
    let temp = tempfile::tempdir().unwrap();
    let store = NativeStore::at(temp.path());
    let previous =
        CollectionEnvelope::new(vec![Note::new("Previous".into(), "safe".into())], vec![]);
    store.save_collection(&previous).unwrap();
    store.save_collection(&CollectionEnvelope::empty()).unwrap();
    std::fs::write(temp.path().join("collection.json"), b"{broken").unwrap();
    let mut session = open(temp.path());
    assert_eq!(
        session.execute(Request::NewNote).unwrap_err().code,
        "storage_recovery"
    );
    assert_eq!(session.snapshot()["recovery"]["can_restore_previous"], true);
    command(&mut session, json!({"command":"restore_previous"}));
    assert!(session.snapshot()["recovery"].is_null());
    assert_eq!(session.app.collection(), previous);
    let preserved = std::fs::read_dir(temp.path())
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("collection.corrupt-")
        })
        .unwrap();
    assert_eq!(std::fs::read(preserved.path()).unwrap(), b"{broken");
}

#[test]
fn merge_import_previews_counts_then_replaces_identity_and_adds_new_note() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = open(temp.path());
    command(&mut session, json!({"command":"new_note"}));
    let mut replacement = session.app.workspace.notes()[0].clone();
    replacement.title = "Replacement".into();
    let extra = Note::new("Extra".into(), "body".into());
    let backup = export_flat_collection_backup(&[replacement.clone(), extra]).unwrap();
    let preview = command(
        &mut session,
        json!({"command":"import_backup", "json":backup}),
    );
    assert_eq!(
        preview["snapshot"]["pending_import"],
        json!({"total_imported_notes":2,"notes_to_add":1,"notes_to_replace":1})
    );
    assert_ne!(session.app.workspace.notes()[0].title, "Replacement");
    command(&mut session, json!({"command":"confirm_import"}));
    session.flush().unwrap();
    assert_eq!(session.app.workspace.notes().len(), 2);
    assert!(session.app.workspace.notes().contains(&replacement));
}

#[test]
fn backup_health_changes_only_after_export_is_written_and_profiles_are_exclusive() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = open(temp.path());
    assert_eq!(
        Session::open(OpenOptionsJson {
            data_directory: Some(temp.path().to_path_buf())
        })
        .err()
        .unwrap()
        .code,
        "profile_in_use"
    );
    let blocked = temp.path().join("directory.json");
    std::fs::create_dir(&blocked).unwrap();
    assert!(
        session
            .execute(Request::ExportBackup { path: blocked })
            .is_err()
    );
    assert!(session.app.backup_health.is_none());
    let exported = temp.path().join("backup.json");
    session
        .execute(Request::ExportBackup {
            path: exported.clone(),
        })
        .unwrap();
    assert!(exported.is_file());
    assert!(session.app.backup_health.is_some());
}

#[test]
fn transition_restore_keeps_accepted_state_visible_until_metadata_save_can_retry() {
    use nota_core::transition::{ThemePreference, export_desktop_transition};
    let temp = tempfile::tempdir().unwrap();
    let (completed, events) = std::sync::mpsc::channel();
    let mut session = Session::open_with_completion(
        OpenOptionsJson {
            data_directory: Some(temp.path().to_path_buf()),
        },
        move || {
            let _ = completed.send(());
        },
    )
    .unwrap();
    std::fs::create_dir(temp.path().join("preferences.json")).unwrap();
    let note = Note::new("Imported".into(), "Never discard this".into());
    let transition = export_desktop_transition(
        std::slice::from_ref(&note),
        &[],
        ThemePreference::Dark,
        None,
    )
    .unwrap();
    let reply = session
        .execute(Request::ImportTransition { json: transition })
        .unwrap();
    assert_eq!(reply["snapshot"]["selected_note"]["id"], json!(note.id));
    assert_eq!(reply["snapshot"]["save_status"], "saving");
    events
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    let reply = session.execute(Request::Snapshot).unwrap();
    assert_eq!(reply["snapshot"]["save_status"], "failed");
    assert_eq!(reply["snapshot"]["notification"]["tone"], "error");
    assert_eq!(session.app.workspace.notes(), std::slice::from_ref(&note));
    assert!(session.flush().is_err());
    std::fs::remove_dir(temp.path().join("preferences.json")).unwrap();
    session.flush().unwrap();
    assert_eq!(session.snapshot()["save_status"], "saved");
    drop(session);
    let reopened = open(temp.path());
    assert_eq!(reopened.app.workspace.notes(), &[note]);
    assert_eq!(reopened.app.theme, ThemePreference::Dark);
}

#[test]
fn gtk_transition_restore_reports_backup_health_failure_and_retries_on_close() {
    use nota_app::app::AppMsg;
    use nota_core::backup::BackupHealthRecord;
    use nota_core::transition::{ThemePreference, export_desktop_transition};
    let temp = tempfile::tempdir().unwrap();
    let (completed, events) = std::sync::mpsc::channel();
    let mut session = Session::open_with_completion(
        OpenOptionsJson {
            data_directory: Some(temp.path().to_path_buf()),
        },
        move || {
            let _ = completed.send(());
        },
    )
    .unwrap();
    std::fs::create_dir(temp.path().join("backup-health.json")).unwrap();
    let note = Note::new("From web".into(), "Preserved".into());
    let health = BackupHealthRecord {
        last_successful_export_at: chrono::Utc::now(),
    };
    let transition = export_desktop_transition(
        std::slice::from_ref(&note),
        &[],
        ThemePreference::Light,
        Some(health),
    )
    .unwrap();
    assert!(
        session
            .apply_message(AppMsg::ImportTransitionJson(transition))
            .unwrap()
    );
    assert_eq!(session.app.workspace.notes(), std::slice::from_ref(&note));
    events
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    session.apply_message(AppMsg::PollPersistence).unwrap();
    assert_eq!(session.snapshot()["save_status"], "failed");
    assert_eq!(session.snapshot()["notification"]["tone"], "error");
    assert!(session.flush().is_err());
    std::fs::remove_dir(temp.path().join("backup-health.json")).unwrap();
    session.flush().unwrap();
    drop(session);
    let reopened = open(temp.path());
    assert_eq!(reopened.app.workspace.notes(), &[note]);
    assert_eq!(reopened.app.theme, ThemePreference::Light);
    assert_eq!(reopened.app.backup_health, Some(health));
}
