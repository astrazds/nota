# Backup and storage recovery

Export active Notes, preview a merge import, and recover a collection through explicit user choices.

## Sub-features

- `backup-export` writes a Backup and updates Backup Health.
- `backup-import` previews add and replace counts before a merge.
- `transition-restore` restores a desktop-transition file only into empty collections.
- `storage-recovery` offers Previous Snapshot, Start Empty, and Import Backup for corrupt saved state.

## How to get to it (user POV)

- Open Notes > Backup > Export notes backup or Import notes backup.
- Use Export complete notebook or Restore complete notebook in that menu for desktop transition.
- At corrupt startup, choose Restore Previous Snapshot, Start Empty, or Import Backup.

## Driving it with CUA

Preconditions: Create synthetic Notes through [Writing](writing.md). Keep exported files under the evidence directory. Use separate fresh profiles for transition and corrupt-storage scenarios.

- Export. Open Notes > Backup > Export notes backup, operate the visible GTK file chooser, and save `backup.json` inside `nota_run`. Type paths using key events. Require success feedback and updated Backup Health. Read the exported JSON and compare active Note IDs, titles, bodies, and Tags.
- Merge. Edit a backed-up Note through the live editor and create another unrelated Note. Leave the exported JSON unchanged. Use Backup > Import notes backup and select that file. Require the Merge Import this Backup? dialog with the observed new and replace counts. Cancel once and compare state. Repeat and confirm; require the backed-up Note restored and the unrelated Note preserved.
- Transition. In a fresh Empty Collection, use Backup > Restore complete notebook to select `crates/nota-core/tests/fixtures/desktop-transition-v1.json`. Confirm the restored active and Recently Deleted Notes against the fixture, then restart. Repeat Restore and require rejection without mutation.
- Corrupt startup. Only after orderly shutdown of a disposable run, preserve its collection and previous snapshot as evidence, then replace that run's collection with invalid JSON. This is failure injection, never a successful-edit shortcut. Restart and require Storage Recovery to block editing. Exercise each offered action in a separate profile with the same failure precondition. Require the previous collection, quarantine copy, or previewed import outcome appropriate to the action.
- Proof. Capture file chooser actions and resulting dialogs, Backup JSON, collection copies, and relevant preference or quarantine files. Verify that canceled and rejected operations leave the collection unchanged.

## Gotchas

- Broadway has no browser HTML file input for GTK dialogs. Browser upload APIs cannot operate a native file chooser.
- Desktop portals may not work in an isolated Broadway session. If a chooser cannot open, record that path as blocked and use the real desktop AppImage rehearsal for its remaining proof.
- Backup contains active Notes only. Desktop Transition also includes Recently Deleted, Theme, and optional Backup Health.
- Both native collections must be empty for Desktop Transition.
- A corrupt file is permitted only as an explicit failure fixture in the disposable profile.
