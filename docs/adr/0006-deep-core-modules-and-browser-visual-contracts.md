# Deep core modules and browser visual contracts

Noter should stay easy to change while keeping the product focused on one thing well: creating, writing, finding, organising, previewing, deleting, and locally backing up Notes.

Accepted decisions:

- Keep the Flat Collection model and deepen existing product Modules rather than redesigning the app.
- Move startup construction, persistence snapshots, Theme/sidebar persistence, page-flush wiring, and viewport reclassification into an `app_runtime` Module.
- Treat Note List projection as the public discovery Interface for Search, active Tags, selected Note visibility, ordering, render keys, and highlights. Lower-level filtering/highlighting helpers are Implementation details unless a real caller needs them.
- Keep `theme` semantic. Put load-bearing layout and visual contracts such as compact footers, View Mode Controls, Search Hint, Tag pills, selected Note rows, and Backup Controls into `ui_recipes`.
- Keep Backup domain semantics in `backup`, but move sidebar Backup Controls, browser download/FileReader Adapter details, pending import state, and related Global Notification outcomes into `backup_controls`.
- Deepen the Writing Surface through a render-ready model that preserves Note Title, Note Metadata, Markdown body, Preview/Split ordering, hidden-by-filter messaging, and formatting command application without leaking browser selection concerns.
- Add Playwright browser coverage for the visual contracts most likely to regress: Light/Dark readability, Search Hint contrast, selected Note state, footer height parity, compact controls, Preview/Split Note Metadata order, Backup Controls placement, and Global Notification layering.
- Keep browser workflow coverage alongside visual coverage for user-visible regressions in Quick Capture, Note Title editing, Note creation/edit/save, scoped Search, pinning, Tags, Formatting Tools, Preview safety, Backup export/import, Responsive Navigation, Markdown syntax help, recoverable delete/restore, and Clear All.

Implementation notes:

- Browser tests live under `tests/browser/` and run with `npm run test:browser`.
- Workflow specs are grouped by product area: Backup, editor, Note workflows, Responsive Navigation, and visual contracts.
- The browser coverage starts a local `trunk serve` process on port 1420 through `playwright.config.js`.
- Browser coverage runs with one Playwright worker so Trunk/Tailwind startup and rebuild work cannot race visual-contract pages that assert emitted CSS, footer height, Search Hint contrast, touch targets, and Pane Rhythm.
- Backup Health persistence remains browser storage on wasm targets; native tests verify AppState updates without touching `web_sys::window()`.
- The architecture Modules are intentionally conservative: each wraps existing product behaviour behind a deeper Interface instead of adding folders, sync, command palette behaviour, or destructive Backup replacement.

This keeps Noter aligned with `CONTEXT.md`: Search and the Note List remain primary discovery, Tags remain lightweight Note Metadata, Backup stays a local utility, Preview remains a View Mode for the same Note, and Global Notifications stay transient above app chrome.
