# Recently Deleted

Move a named Note to Recently Deleted and restore it to the active collection.

## Sub-features

- `deleted-section-visual` distinguishes the recovery section from active Notes in both themes.
- `delete-confirm` names the selected Note and supports cancellation.
- `delete-recoverable` moves the Note into Recently Deleted.
- `restore-note` returns the same Note to the collection.
- `delete-permanent` removes one Recently Deleted Note immediately.
- `clear-all` confirms before clearing all Recently Deleted Notes.

## How to get to it (user POV)

- Open the three-dot Note actions menu on an active Note and choose Delete.
- Use Restore or Delete beside a Note in Recently Deleted.
- Choose Clear All in the Recently Deleted header when Notes are present.

## Driving it with CUA

Preconditions: Create only synthetic Notes using [Writing](writing.md). Record their UUIDs and contents.

- Empty section. Before deleting a Note, require the Recently Deleted header to remain visible and Clear All to be hidden.
- Confirmation. From a fresh screenshot, click the row's Note actions, then Delete. Require a dialog naming that Note and Cancel as default focus. Click Cancel and require the active collection unchanged.
- Soft delete. Repeat and accept the named confirmation. Require the row to leave the active list and appear in Recently Deleted. Copy the collection and require the same UUID and content in the deleted collection.
- Section appearance. Move a second disposable Note to Recently Deleted. Inspect Light and Dark Themes. Require a full-width contrasting neutral header, distinct recovery rows, a visible separator between rows, neutral Restore, and readable red Delete and Clear All controls. Include a long title and a compact window to check wrapping and clipping.
- Restore. Click Restore beside the original Note, not the Backup Restore control. Require its active row and content to return with the same UUID. Restart and confirm persistence.
- Permanent deletion. For a dedicated disposable Note, move it to Recently Deleted. Satisfy any active tool approval requirement before clicking Delete. Nota removes it immediately without an app confirmation. Require it absent from both collections. Record a skip if required approval is unavailable.
- Clear All. With disposable deleted Notes present, open Clear All. Capture its count, cancel, and confirm nothing changed. Repeat and confirm only when the active tool's confirmation policy allows.
- Proof. Save the named confirmation, deleted row, restored editor, and before/after collection copies. Report permanent operations separately.

## Gotchas

- Recently Deleted Delete is immediate. Only Clear All adds another confirmation.
- The sidebar Backup Restore imports a desktop-transition file; it does not restore a deleted Note.
- Deleting fixtures through the UI is feature verification. Final cleanup removes the whole disposable profile after preserving evidence.

See the [dated Recently Deleted verification record](../../../../docs/recently-deleted-verification.md) for prior observations and coverage limits. It does not replace a run against the current build.
