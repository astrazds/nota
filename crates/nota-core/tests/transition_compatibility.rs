use nota_core::Note;
use nota_core::backup::import_flat_collection_backup;
use nota_core::transition::{ThemePreference, TransitionError, import_desktop_transition};

const TRANSITION_V1: &str = include_str!("fixtures/desktop-transition-v1.json");
const BACKUP_V1: &str = include_str!("fixtures/flat-collection-backup-v1.json");

#[test]
fn native_importer_consumes_the_final_web_transition_fixture() {
    let restored = import_desktop_transition(&[], &[], TRANSITION_V1).unwrap();

    assert_eq!(restored.notes[0].title, "Desktop handoff");
    assert_eq!(restored.recently_deleted_notes[0].title, "Recover later");
    assert_eq!(restored.theme, ThemePreference::Dark);
    assert!(restored.backup_health.is_some());
}

#[test]
fn backup_v1_fixture_remains_merge_compatible() {
    let existing = Note::new("Keep existing".to_string(), String::new());
    let mut notes = vec![existing.clone()];

    import_flat_collection_backup(&mut notes, BACKUP_V1).unwrap();

    assert_eq!(notes[0], existing);
    assert_eq!(notes[1].title, "Existing Backup");
}

#[test]
fn malformed_and_future_transition_fixtures_do_not_mutate_native_state() {
    let existing = Note::new("Keep me".to_string(), String::new());
    let malformed = import_desktop_transition(&[], &[], "{not json");
    assert!(matches!(malformed, Err(TransitionError::Deserialize(_))));

    let future = TRANSITION_V1.replacen("\"version\": 1", "\"version\": 99", 1);
    let error = import_desktop_transition(&[], &[], &future).unwrap_err();
    assert!(matches!(error, TransitionError::UnsupportedVersion(99)));

    let rejected = import_desktop_transition(std::slice::from_ref(&existing), &[], TRANSITION_V1);
    assert!(matches!(rejected, Err(TransitionError::CollectionNotEmpty)));
    assert_eq!(existing.title, "Keep me");
}
