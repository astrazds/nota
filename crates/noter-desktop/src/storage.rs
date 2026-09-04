use std::error::Error;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::{DateTime, Utc};
use noter_core::Note;
use noter_core::backup::BackupHealthRecord;
use noter_core::transition::ThemePreference;
use serde::{Deserialize, Serialize};

const COLLECTION_VERSION: u32 = 1;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollectionEnvelope {
    pub version: u32,
    pub notes: Vec<Note>,
    pub recently_deleted_notes: Vec<Note>,
}

impl CollectionEnvelope {
    pub fn new(notes: Vec<Note>, recently_deleted_notes: Vec<Note>) -> Self {
        Self {
            version: COLLECTION_VERSION,
            notes,
            recently_deleted_notes,
        }
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), Vec::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Preferences {
    pub theme: ThemePreference,
    pub window_width: i32,
    pub window_height: i32,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            window_width: 1180,
            window_height: 760,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeRecovery {
    pub corrupt_collection_path: PathBuf,
    pub previous_snapshot: Option<CollectionEnvelope>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadOutcome {
    Ready(CollectionEnvelope),
    Recovery(NativeRecovery),
}

#[derive(Debug)]
pub enum StorageError {
    DataDirectoryUnavailable,
    Io { path: PathBuf, source: io::Error },
    InvalidCollection(String),
    Serialize(serde_json::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataDirectoryUnavailable => {
                write!(formatter, "XDG data directory is unavailable")
            }
            Self::Io { path, source } => write!(
                formatter,
                "storage operation failed for {}: {source}",
                path.display()
            ),
            Self::InvalidCollection(reason) => write!(formatter, "invalid collection: {reason}"),
            Self::Serialize(error) => write!(formatter, "could not serialize collection: {error}"),
        }
    }
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Serialize(error) => Some(error),
            Self::DataDirectoryUnavailable | Self::InvalidCollection(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NativeStore {
    data_dir: PathBuf,
}

impl NativeStore {
    pub fn discover() -> Result<Self, StorageError> {
        let data_home = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .map(|home| home.join(".local/share"))
            })
            .ok_or(StorageError::DataDirectoryUnavailable)?;
        Ok(Self::at(data_home.join("noter")))
    }

    pub fn at(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn load_collection(&self) -> Result<LoadOutcome, StorageError> {
        let collection_path = self.collection_path();
        let raw = match fs::read(&collection_path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(LoadOutcome::Ready(CollectionEnvelope::empty()));
            }
            Err(source) => {
                let previous_snapshot = fs::read(self.previous_collection_path())
                    .ok()
                    .and_then(|previous| parse_collection(&previous).ok());
                return Ok(LoadOutcome::Recovery(NativeRecovery {
                    corrupt_collection_path: collection_path,
                    previous_snapshot,
                    reason: source.to_string(),
                }));
            }
        };

        match parse_collection(&raw) {
            Ok(collection) => Ok(LoadOutcome::Ready(collection)),
            Err(error) => {
                let previous_snapshot = fs::read(self.previous_collection_path())
                    .ok()
                    .and_then(|previous| parse_collection(&previous).ok());
                Ok(LoadOutcome::Recovery(NativeRecovery {
                    corrupt_collection_path: collection_path,
                    previous_snapshot,
                    reason: error.to_string(),
                }))
            }
        }
    }

    pub fn save_collection(&self, collection: &CollectionEnvelope) -> Result<(), StorageError> {
        validate_collection(collection)?;
        let next = serde_json::to_vec_pretty(collection).map_err(StorageError::Serialize)?;
        let current_path = self.collection_path();

        if let Ok(current) = fs::read(&current_path)
            && parse_collection(&current).is_ok()
        {
            write_atomic(&self.previous_collection_path(), &current)?;
        }
        write_atomic(&current_path, &next)
    }

    pub fn restore_previous(
        &self,
        recovery: &NativeRecovery,
    ) -> Result<CollectionEnvelope, StorageError> {
        let previous = recovery.previous_snapshot.clone().ok_or_else(|| {
            StorageError::InvalidCollection("no previous snapshot is available".to_string())
        })?;
        self.save_collection(&previous)?;
        Ok(previous)
    }

    pub fn start_empty(
        &self,
        recovery: &NativeRecovery,
        now: DateTime<Utc>,
    ) -> Result<(CollectionEnvelope, PathBuf), StorageError> {
        fs::create_dir_all(&self.data_dir)
            .map_err(|source| io_error(self.data_dir.clone(), source))?;
        let quarantine = self.data_dir.join(format!(
            "collection.corrupt-{}.json",
            now.format("%Y%m%dT%H%M%SZ")
        ));
        fs::rename(&recovery.corrupt_collection_path, &quarantine)
            .map_err(|source| io_error(recovery.corrupt_collection_path.clone(), source))?;
        let empty = CollectionEnvelope::empty();
        self.save_collection(&empty)?;
        Ok((empty, quarantine))
    }

    pub fn load_preferences(&self) -> Preferences {
        fs::read(self.preferences_path())
            .ok()
            .and_then(|raw| serde_json::from_slice(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save_preferences(&self, preferences: &Preferences) -> Result<(), StorageError> {
        let raw = serde_json::to_vec_pretty(preferences).map_err(StorageError::Serialize)?;
        write_atomic(&self.preferences_path(), &raw)
    }

    pub fn load_backup_health(&self) -> Option<BackupHealthRecord> {
        fs::read(self.backup_health_path())
            .ok()
            .and_then(|raw| serde_json::from_slice(&raw).ok())
    }

    pub fn save_backup_health(&self, health: &BackupHealthRecord) -> Result<(), StorageError> {
        let raw = serde_json::to_vec_pretty(health).map_err(StorageError::Serialize)?;
        write_atomic(&self.backup_health_path(), &raw)
    }

    pub fn has_quarantined_corrupt_payloads(&self) -> bool {
        fs::read_dir(&self.data_dir)
            .ok()
            .map(|entries| {
                entries.filter_map(Result::ok).any(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("collection.corrupt-")
                })
            })
            .unwrap_or(false)
    }

    fn collection_path(&self) -> PathBuf {
        self.data_dir.join("collection.json")
    }

    fn previous_collection_path(&self) -> PathBuf {
        self.data_dir.join("collection.previous.json")
    }

    fn preferences_path(&self) -> PathBuf {
        self.data_dir.join("preferences.json")
    }

    fn backup_health_path(&self) -> PathBuf {
        self.data_dir.join("backup-health.json")
    }
}

fn parse_collection(raw: &[u8]) -> Result<CollectionEnvelope, StorageError> {
    let collection: CollectionEnvelope = serde_json::from_slice(raw)
        .map_err(|error| StorageError::InvalidCollection(error.to_string()))?;
    validate_collection(&collection)?;
    Ok(collection)
}

fn validate_collection(collection: &CollectionEnvelope) -> Result<(), StorageError> {
    if collection.version != COLLECTION_VERSION {
        return Err(StorageError::InvalidCollection(format!(
            "unsupported version {}",
            collection.version
        )));
    }
    let mut identities = std::collections::HashSet::new();
    for note in collection
        .notes
        .iter()
        .chain(&collection.recently_deleted_notes)
    {
        if !identities.insert(note.id) {
            return Err(StorageError::InvalidCollection(format!(
                "duplicate Note identity {}",
                note.id
            )));
        }
    }
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), StorageError> {
    let parent = path.parent().ok_or_else(|| {
        StorageError::InvalidCollection("storage path has no parent directory".to_string())
    })?;
    fs::create_dir_all(parent).map_err(|source| io_error(parent.to_path_buf(), source))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            StorageError::InvalidCollection("storage path has no UTF-8 file name".to_string())
        })?;
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let temp_path = parent.join(format!(
        ".{file_name}.tmp-{}-{sequence}",
        std::process::id()
    ));

    let write_result = (|| {
        let mut temp = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)
            .map_err(|source| io_error(temp_path.clone(), source))?;
        temp.write_all(bytes)
            .map_err(|source| io_error(temp_path.clone(), source))?;
        temp.sync_all()
            .map_err(|source| io_error(temp_path.clone(), source))?;
        fs::rename(&temp_path, path).map_err(|source| io_error(path.to_path_buf(), source))?;
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|source| io_error(parent.to_path_buf(), source))
    })();

    if write_result.is_err() {
        let _cleanup_result = fs::remove_file(&temp_path);
    }
    write_result
}

fn io_error(path: PathBuf, source: io::Error) -> StorageError {
    StorageError::Io { path, source }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn atomic_save_keeps_the_last_valid_collection_as_previous() {
        let temp = tempfile::tempdir().unwrap();
        let store = NativeStore::at(temp.path());
        let first = CollectionEnvelope::new(
            vec![Note::new("First".to_string(), "one".to_string())],
            Vec::new(),
        );
        let second = CollectionEnvelope::new(
            vec![Note::new("Second".to_string(), "two".to_string())],
            Vec::new(),
        );

        store.save_collection(&first).unwrap();
        store.save_collection(&second).unwrap();

        assert_eq!(store.load_collection().unwrap(), LoadOutcome::Ready(second));
        let previous: CollectionEnvelope = serde_json::from_slice(
            &fs::read(temp.path().join("collection.previous.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(previous, first);
    }

    #[cfg(unix)]
    #[test]
    fn write_failure_leaves_the_current_collection_readable() {
        let temp = tempfile::tempdir().unwrap();
        let store = NativeStore::at(temp.path());
        let current = CollectionEnvelope::new(
            vec![Note::new("Current".to_string(), "safe".to_string())],
            Vec::new(),
        );
        let replacement = CollectionEnvelope::new(
            vec![Note::new("Replacement".to_string(), "new".to_string())],
            Vec::new(),
        );
        store.save_collection(&current).unwrap();

        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o500)).unwrap();
        let result = store.save_collection(&replacement);
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();

        assert!(matches!(result, Err(StorageError::Io { .. })));
        assert_eq!(
            store.load_collection().unwrap(),
            LoadOutcome::Ready(current)
        );
    }

    #[test]
    fn corrupt_current_state_offers_previous_recovery_and_quarantine() {
        let temp = tempfile::tempdir().unwrap();
        let store = NativeStore::at(temp.path());
        let previous = CollectionEnvelope::new(
            vec![Note::new("Previous".to_string(), "safe".to_string())],
            Vec::new(),
        );
        store.save_collection(&previous).unwrap();
        store.save_collection(&CollectionEnvelope::empty()).unwrap();
        fs::write(temp.path().join("collection.json"), b"{broken").unwrap();

        let LoadOutcome::Recovery(recovery) = store.load_collection().unwrap() else {
            panic!("corrupt current state must enter Storage Recovery");
        };
        assert_eq!(recovery.previous_snapshot, Some(previous));

        let now = Utc.with_ymd_and_hms(2026, 9, 3, 12, 0, 0).unwrap();
        let (empty, quarantine) = store.start_empty(&recovery, now).unwrap();
        assert_eq!(empty, CollectionEnvelope::empty());
        assert_eq!(fs::read_to_string(quarantine).unwrap(), "{broken");
        assert!(store.has_quarantined_corrupt_payloads());
    }

    #[test]
    fn corrupt_current_state_without_previous_snapshot_still_enters_recovery() {
        let temp = tempfile::tempdir().unwrap();
        let store = NativeStore::at(temp.path());
        fs::write(temp.path().join("collection.json"), b"{broken").unwrap();

        let LoadOutcome::Recovery(recovery) = store.load_collection().unwrap() else {
            panic!("corrupt current state must enter Storage Recovery");
        };
        assert!(recovery.previous_snapshot.is_none());
        assert!(matches!(
            store.restore_previous(&recovery),
            Err(StorageError::InvalidCollection(_))
        ));
    }

    #[test]
    fn preferences_and_backup_health_do_not_replace_collection_history() {
        let temp = tempfile::tempdir().unwrap();
        let store = NativeStore::at(temp.path());
        let collection = CollectionEnvelope::new(
            vec![Note::new("Kept".to_string(), String::new())],
            Vec::new(),
        );
        store.save_collection(&collection).unwrap();
        store
            .save_preferences(&Preferences {
                theme: ThemePreference::Dark,
                ..Preferences::default()
            })
            .unwrap();
        store
            .save_backup_health(&BackupHealthRecord {
                last_successful_export_at: Utc::now(),
            })
            .unwrap();

        assert_eq!(
            store.load_collection().unwrap(),
            LoadOutcome::Ready(collection)
        );
        assert!(!temp.path().join("collection.previous.json").exists());
    }
}
