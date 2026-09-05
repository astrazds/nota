# Fast capture and local recovery

Nota should make the common local-first risks explicit: users need to capture a Note quickly, avoid accidental data loss, and know whether they have a recent local recovery point.

Accepted decisions:

- Quick Capture creates a new Note, selects it, focuses the Note Title, and returns compact viewports to the Writing Surface.
- Delete moves a Note to Recently Deleted instead of immediately removing it from all app state.
- Recently Deleted supports explicit Restore and Clear actions so recovery is visible but permanent removal remains intentional.
- Backup Health records the last successful local Backup export and handles missing or malformed metadata as "No backup yet" rather than blocking the app.
- Backup Import Preview validates a selected Backup and shows add/replace impact before a Merge Import mutates the Flat Collection.

Implementation notes:

- Active Notes and Recently Deleted Notes are persisted separately in LocalStorage so the existing Flat Collection storage shape remains compatible.
- Backup Health is lightweight local metadata, not sync status.
- Merge Import remains the only Backup import behavior; destructive replace import is still out of scope.
