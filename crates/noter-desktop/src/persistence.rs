use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::storage::{CollectionEnvelope, NativeStore, StorageError};

const SAVE_DEBOUNCE: Duration = Duration::from_millis(300);

#[derive(Debug)]
enum Command {
    Save {
        revision: u64,
        collection: CollectionEnvelope,
    },
    Flush(Sender<Result<u64, StorageError>>),
    Shutdown(Sender<Result<u64, StorageError>>),
}

#[derive(Debug)]
pub struct PersistenceWorker {
    sender: Sender<Command>,
    handle: Option<JoinHandle<()>>,
}

impl PersistenceWorker {
    pub fn start(store: NativeStore) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || run_worker(store, receiver));
        Self {
            sender,
            handle: Some(handle),
        }
    }

    pub fn schedule(&self, revision: u64, collection: CollectionEnvelope) -> bool {
        self.sender
            .send(Command::Save {
                revision,
                collection,
            })
            .is_ok()
    }

    pub fn flush(&self) -> Result<u64, StorageError> {
        let (reply_sender, reply_receiver) = mpsc::channel();
        self.sender
            .send(Command::Flush(reply_sender))
            .map_err(|_| worker_stopped())?;
        reply_receiver.recv().map_err(|_| worker_stopped())?
    }

    pub fn shutdown(mut self) -> Result<u64, StorageError> {
        let (reply_sender, reply_receiver) = mpsc::channel();
        self.sender
            .send(Command::Shutdown(reply_sender))
            .map_err(|_| worker_stopped())?;
        let result = reply_receiver.recv().map_err(|_| worker_stopped())?;
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| worker_stopped())?;
        }
        result
    }
}

fn run_worker(store: NativeStore, receiver: Receiver<Command>) {
    let mut persisted_revision = 0;
    let mut pending: Option<(u64, CollectionEnvelope)> = None;

    loop {
        match receiver.recv_timeout(if pending.is_some() {
            SAVE_DEBOUNCE
        } else {
            Duration::from_secs(60)
        }) {
            Ok(Command::Save {
                revision,
                collection,
            }) => {
                let pending_revision = pending.as_ref().map_or(0, |(revision, _)| *revision);
                if revision > persisted_revision && revision > pending_revision {
                    pending = Some((revision, collection));
                }
            }
            Ok(Command::Flush(reply)) => {
                let result = flush_pending(&store, &mut pending, &mut persisted_revision);
                let _send_result = reply.send(result);
            }
            Ok(Command::Shutdown(reply)) => {
                let result = flush_pending(&store, &mut pending, &mut persisted_revision);
                let _send_result = reply.send(result);
                return;
            }
            Err(RecvTimeoutError::Timeout) if pending.is_some() => {
                let _save_result = flush_pending(&store, &mut pending, &mut persisted_revision);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                let _save_result = flush_pending(&store, &mut pending, &mut persisted_revision);
                return;
            }
        }
    }
}

fn flush_pending(
    store: &NativeStore,
    pending: &mut Option<(u64, CollectionEnvelope)>,
    persisted_revision: &mut u64,
) -> Result<u64, StorageError> {
    let Some((revision, collection)) = pending.take() else {
        return Ok(*persisted_revision);
    };
    if let Err(error) = store.save_collection(&collection) {
        *pending = Some((revision, collection));
        return Err(error);
    }
    *persisted_revision = revision;
    Ok(revision)
}

fn worker_stopped() -> StorageError {
    StorageError::InvalidCollection("persistence worker stopped unexpectedly".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AppModel, AppMsg};
    use crate::storage::LoadOutcome;
    use noter_core::Note;
    use noter_core::transition::ThemePreference;

    #[test]
    fn close_time_flush_persists_only_the_latest_revision() {
        let temp = tempfile::tempdir().unwrap();
        let store = NativeStore::at(temp.path());
        let worker = PersistenceWorker::start(store.clone());
        let revision_one = CollectionEnvelope::new(
            vec![Note::new("Old".to_string(), String::new())],
            Vec::new(),
        );
        let revision_two = CollectionEnvelope::new(
            vec![Note::new("Latest".to_string(), String::new())],
            Vec::new(),
        );

        assert!(worker.schedule(1, revision_one));
        assert!(worker.schedule(2, revision_two.clone()));
        assert!(worker.schedule(1, CollectionEnvelope::empty()));
        assert_eq!(worker.shutdown().unwrap(), 2);
        assert_eq!(
            store.load_collection().unwrap(),
            LoadOutcome::Ready(revision_two)
        );
    }

    #[test]
    fn native_create_edit_close_and_relaunch_restores_identical_state() {
        let temp = tempfile::tempdir().unwrap();
        let store = NativeStore::at(temp.path());
        let mut app = AppModel::new(CollectionEnvelope::empty(), ThemePreference::System, None);

        assert!(app.apply(AppMsg::QuickCapture));
        assert!(app.apply(AppMsg::UpdateTitle("Native note".to_string())));
        assert!(app.apply(AppMsg::UpdateContent(
            "Persists through an orderly shutdown.".to_string(),
        )));
        let expected = app.collection();

        let worker = PersistenceWorker::start(store.clone());
        assert!(worker.schedule(app.revision(), expected.clone()));
        assert_eq!(worker.shutdown().unwrap(), app.revision());

        let LoadOutcome::Ready(reloaded) = store.load_collection().unwrap() else {
            panic!("a valid native collection must relaunch without recovery");
        };
        assert_eq!(reloaded, expected);
        let relaunched = AppModel::new(reloaded, ThemePreference::System, None);
        assert_eq!(relaunched.collection(), expected);
    }
}
