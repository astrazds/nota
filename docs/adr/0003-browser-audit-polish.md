# Browser audit polish

The May 2026 browser audit resolved a set of visual and interaction issues across desktop, split, and mobile views.

Accepted decisions:

- Keep the Writing Surface body text at the same scale as Preview body text so switching between Write, Preview, and Split does not create a jarring size change.
- Keep the selected Note visible in the editor even when Search or active Tag filters remove it from the Note List, but show a clear filter-hidden banner.
- Treat the first Markdown `h1` as duplicate content when it exactly matches the Note Title, and suppress that duplicate in Preview.
- Show scoped Search syntax as a focus-time Search Hint instead of permanent sidebar content.
- Move Backup Controls to one compact horizontal row in the sidebar footer.
- Present Tags as read-only Note Metadata chips until the user chooses to edit them, then show one tag input rather than duplicating chips and input text.
- Show read-only Tags under the Note Title in Preview and Split view so Note Metadata remains visible outside the Writing Surface and matches the editor header order.
- Preserve mobile layout integrity with a full-width Note List, touch-sized editor controls, wrapped Note Titles, and truncated long Note List titles.

Implementation notes:

- The Leptos preview pane owns the generated Note Title and read-only Tags.
- `markdown_preview` renders the sanitized Markdown body and still suppresses a first content `h1` that duplicates the Note Title.
- Coverage for this split lives on the body renderer path used by the app, including duplicate-heading behavior and preview safety policy.

This keeps Noter aligned with the domain model in `CONTEXT.md`: Search and the Note List remain primary, Tags stay lightweight, Backup remains a secondary local utility, and Preview is a View Mode for the same Note rather than a separate document frame.
