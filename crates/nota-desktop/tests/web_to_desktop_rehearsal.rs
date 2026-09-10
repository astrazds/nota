use chrono::{TimeZone, Utc};
use nota_core::backup::{BackupHealthRecord, export_flat_collection_backup};
use nota_core::transition::{ThemePreference, TransitionError, export_desktop_transition};
use nota_desktop::app::{AppModel, AppMsg};
use nota_desktop::storage::{CollectionEnvelope, LoadOutcome, NativeStore, Preferences};

const WEB_TRANSITION_V1: &str =
    include_str!("../../nota-core/tests/fixtures/desktop-transition-v1.json");

#[test]
fn clean_profile_restores_web_transition_once_then_uses_merge_import() {
    let temp = tempfile::tempdir().unwrap();
    let store = NativeStore::at(temp.path());
    assert_eq!(
        store.load_collection().unwrap(),
        LoadOutcome::Ready(CollectionEnvelope::empty()),
        "a clean native profile must start as an Empty Collection"
    );

    let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);
    app.import_transition(WEB_TRANSITION_V1).unwrap();

    assert_eq!(app.workspace.notes()[0].title, "Desktop handoff");
    assert_eq!(
        app.workspace.recently_deleted_notes()[0].title,
        "Recover later"
    );
    assert_eq!(app.theme, ThemePreference::Dark);
    assert!(app.backup_health.is_some());

    store.save_collection(&app.collection()).unwrap();
    store
        .save_preferences(&Preferences {
            theme: app.theme,
            ..Preferences::default()
        })
        .unwrap();
    if let Some(health) = app.backup_health {
        store.save_backup_health(&health).unwrap();
    }

    let LoadOutcome::Ready(reloaded) = store.load_collection().unwrap() else {
        panic!("restored native collection must relaunch without recovery");
    };
    let relaunched = AppModel::new(
        reloaded,
        store.load_preferences().theme,
        store.load_backup_health(),
    );
    assert_eq!(relaunched.workspace.notes()[0].title, "Desktop handoff");
    assert_eq!(relaunched.theme, ThemePreference::Dark);
    assert!(relaunched.backup_health.is_some());

    let before = relaunched.collection();
    let error = AppModel::new(before.clone(), relaunched.theme, relaunched.backup_health)
        .import_transition(WEB_TRANSITION_V1)
        .expect_err("a second desktop transition restore must be rejected");
    assert!(matches!(error, TransitionError::CollectionNotEmpty));

    let extra = nota_core::Note::new("After migration".to_string(), "Merge path".to_string());
    let backup = export_flat_collection_backup(std::slice::from_ref(&extra)).unwrap();
    let mut migrated = AppModel::new(before.clone(), relaunched.theme, relaunched.backup_health);
    migrated.apply(AppMsg::ImportBackupJson(backup));
    assert!(migrated.apply(AppMsg::ConfirmBackupImport));
    assert_eq!(migrated.workspace.notes().len(), 2);
    assert!(
        migrated
            .workspace
            .notes()
            .iter()
            .any(|note| note.title == "After migration")
    );
}

#[test]
fn transition_without_backup_health_clears_stale_health_on_relaunch() {
    let temp = tempfile::tempdir().unwrap();
    let store = NativeStore::at(temp.path());
    store
        .save_backup_health(&BackupHealthRecord {
            last_successful_export_at: Utc.with_ymd_and_hms(2026, 8, 1, 9, 0, 0).unwrap(),
        })
        .unwrap();
    let transferred = nota_core::Note::new("Transferred".to_string(), "Exact state".to_string());
    let transition = export_desktop_transition(
        std::slice::from_ref(&transferred),
        &[],
        ThemePreference::Light,
        None,
    )
    .unwrap();
    let mut app = AppModel::new(
        CollectionEnvelope::empty(),
        ThemePreference::System,
        store.load_backup_health(),
    );

    app.import_transition(&transition).unwrap();
    store.save_collection(&app.collection()).unwrap();
    store
        .persist_backup_health(app.backup_health.as_ref())
        .unwrap();

    let LoadOutcome::Ready(reloaded) = store.load_collection().unwrap() else {
        panic!("restored native collection must relaunch without recovery");
    };
    let relaunched = AppModel::new(reloaded, ThemePreference::Light, store.load_backup_health());
    assert_eq!(relaunched.workspace.notes(), &[transferred]);
    assert!(relaunched.backup_health.is_none());
}
