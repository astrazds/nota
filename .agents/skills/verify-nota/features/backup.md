# Backup and storage recovery

Export active Notes, preview a merge import, and recover a collection through explicit user choices.

## Sub-features

- `backup-export` writes a complete notebook and updates Backup Health.
- `backup-import` confirms from the session text, then merges or restores.
- `storage-recovery` offers Previous Snapshot, Start Empty, and Import / Restore. A complete file imported there merges active Notes.

## How to get to it (user POV)

- Open Notes > Backup > Backup / Export or Import / Restore.
- At corrupt startup, choose Restore Previous Snapshot, Start Empty, or Import / Restore.

## Driving it with CUA

Preconditions: Create synthetic Notes through [Writing](writing.md). Keep exported files under the evidence directory. Use separate fresh profiles for transition and corrupt-storage scenarios.

- Export. Open Notes > Backup > Backup / Export, operate the visible GTK file chooser, and save `backup.json` inside `nota_run`. Type paths using key events. Require success feedback and updated Backup Health. Read the exported JSON and require `nota.desktop_transition`, the minted health time, active Note IDs, titles, bodies, Tags, and Recently Deleted.
- Merge. Edit a backed-up Note through the live editor and create another unrelated Note. Leave the exported JSON unchanged. Use Backup > Import / Restore and select that file. Require the Import these notes? dialog, with the observed add and replace counts, and the sentence that Recently Deleted, theme, and Backup Health from the file are not applied. Cancel once and compare state. Repeat and confirm. Require the backed-up Note restored, the unrelated Note preserved, and local Theme unchanged.
- Exact restore. In a fresh Empty Collection, use Backup > Import / Restore to select `crates/nota-core/tests/fixtures/desktop-transition-v1.json`. Require the Restore this backup? dialog. Confirm the restored active and Recently Deleted Notes against the fixture, then restart. Import the same file again. Require Import these notes?, then cancel and require the collection unchanged.
- Corrupt startup. Only after orderly shutdown of a disposable run, preserve its collection and previous snapshot as evidence, then replace that run's collection with invalid JSON. This is failure injection, never a successful-edit shortcut. Restart and require Storage Recovery to block editing. Exercise each offered action in a separate profile with the same failure precondition. Require the previous collection, quarantine copy, or previewed import outcome appropriate to the action.
- Proof. Capture file chooser actions and resulting dialogs, Backup JSON, collection copies, and relevant preference or quarantine files. Verify that canceled and rejected operations leave the collection unchanged.

## Gotchas

- Broadway has no browser HTML file input for GTK dialogs. Browser upload APIs cannot operate a native file chooser.
- Desktop portals may not work in an isolated Broadway session. If a chooser cannot open, record that path as blocked and use the real desktop AppImage rehearsal for its remaining proof.
- Backup / Export writes a complete notebook, including Recently Deleted, Theme, and a new Backup Health time.
- Exact restore runs only for a complete file when both collections are empty and Storage Recovery is closed. A second import of that file offers a merge.
- A corrupt file is permitted only as an explicit failure fixture in the disposable profile.
