# Editor-first controls and Clear All recovery

Noter should keep the Note content as the primary workspace while still making writing, previewing, and recovery actions quick to reach.

Accepted decisions:

- Move View Mode Controls (`Write`, `Preview`, `Split`, and Markdown help) into one stable-height editor-area footer.
- Match the editor-area footer to the sidebar footer's compact visual rhythm: same rendered height, font size, footer padding, and button padding.
- Use one editor-area footer for Split so the editor and Preview panes still read as one View Mode for the same Note.
- Remove persistent desktop editor header chrome once View Mode Controls no longer need it.
- Keep compact Responsive Navigation as a minimal overlay affordance so small viewports can return to the Note List.
- Move Formatting Tools into the Writing Surface after Note Title and Note Metadata and before the Markdown body.
- Show Formatting Tools only when writing is available: Write mode and the writing side of Split, never Preview.
- Keep Global Notifications as floating overlay feedback above app chrome instead of turning save, Backup, or import feedback into persistent header/footer content.
- Add `Clear All` to the Recently Deleted summary row only when recoverable Notes exist.
- Require count-specific Delete Confirmation before `Clear All` permanently removes Recently Deleted Notes.

Implementation notes:

- `note_workspace` owns the clear-all confirmation state so bulk clearing remains testable through the workspace and app-state paths.
- The shared confirmation modal handles both note-specific delete and count-specific Recently Deleted clearing.
- The sidebar still owns Note List, Recently Deleted, and Backup Controls; the editor-area footer owns only View Mode Controls.
- The shared footer height is enforced with the local `noter-footer-height` utility because the build pipeline did not emit the Tailwind `min-h-12` utility reliably.
- Browser verification should cover Write, Preview, Split, compact navigation, Light/Dark contrast, floating notifications, and the Clear All confirmation/cancel/confirm flow.

This keeps Noter aligned with the domain model in `CONTEXT.md`: Formatting Tools are contextual writing aids, View Mode Controls belong with the editor area, and Recently Deleted remains recoverable until the user explicitly confirms permanent removal.
