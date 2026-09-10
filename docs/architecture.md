# Architecture

Nota is a virtual Cargo workspace with a toolkit-independent domain core and
one Relm4/GTK4 desktop frontend. The browser frontend was removed after the
native cutover described in [ADR-0009](adr/0009-relm4-native-replacement.md).
Backup v1 and desktop-transition import remain permanent compatibility
contracts.

## Dependency direction

`nota-desktop` depends on `nota-core`. The core owns its source files and has
no dependency on GTK, WebKitGTK, Linux paths, or native persistence. The
desktop crate owns platform state, effects, widgets, file selection, and XDG
storage.

| Change | Owner | Behavior checks |
| --- | --- | --- |
| Note mutation, selection, deletion, and recovery | `crates/nota-core/src/note_workspace.rs` and `note_collection.rs` | Core workspace tests |
| Search results, snippets, and Tags | `crates/nota-core/src/note_discovery.rs`, `note_list_interaction.rs`, and `tag_rules.rs` | Core discovery tests and native workflow tests |
| Backup and desktop-transition formats | `crates/nota-core/src/backup.rs` and `transition.rs` | Backup workflows and compatibility fixtures |
| Markdown parsing and formatting | `crates/nota-core/src/markdown_preview.rs` and `markdown_editing.rs` | Core Markdown tests and native visual contracts |
| Desktop state transitions | `crates/nota-desktop/src/app.rs` | Native workflow tests |
| Desktop effects and shutdown | `crates/nota-desktop/src/ui/mod.rs` | Persistence tests and native workflow tests |
| Desktop widgets and input conversion | `crates/nota-desktop/src/ui/workspace.rs` and `selection.rs` | Native visual contracts and live GTK verification |
| Desktop row identity and synchronization | `crates/nota-desktop/src/ui/note_list.rs` | Row identity tests and live GTK selection |
| Desktop dialogs and JSON file selection | `crates/nota-desktop/src/ui/dialogs.rs` and `files.rs` | Native import workflows and live GTK dialogs |
| Atomic native storage | `crates/nota-desktop/src/storage.rs` and `persistence.rs` | Native storage, shutdown, and transition tests |
| Preview HTML and navigation policy | `crates/nota-desktop/src/preview.rs` and `webkit_preview.rs` | Preview tests and native workflow tests |

## Follow a change through the app

User actions enter through `AppMsg` in `crates/nota-desktop/src/app.rs`.
`AppState` delegates Note mutations to `NoteWorkspace` and increments its save
revision after a persistent change. The desktop shell schedules
`PersistenceWorker`, which writes through `NativeStore` and flushes on
shutdown.

Search uses `NoteListInteraction` to produce a render-ready projection. The
desktop `NoteLists` owns its factories and updates existing row widgets when
UUID order is unchanged. Dialogs and file selection emit `AppMsg` through a
channel. They do not depend on the root Relm4 component type.

Core Markdown formatting accepts UTF-8 byte ranges. GTK character conversion
stays in `crates/nota-desktop/src/selection.rs`. Preview policy produces
generated HTML in the core, while the desktop crate owns WebKitGTK navigation
and external-link handling.

## Boundaries worth preserving

Keep `NoteWorkspace` and `NoteListInteraction` as the domain behavior
boundaries. Keep GTK focus, page composition, persistence scheduling, and
native shutdown in `nota-desktop`.

Keep Backup v1, desktop-transition v1, legacy kind acceptance, UUID identity,
and merge-only import compatible. Keep current and previous native snapshots
as complete active and Recently Deleted collection pairs. The acceptance
fixtures live in `crates/nota-core/tests/fixtures/`.

Keep the GTK widget hierarchy stable during structural changes. Module tests
cover domain behavior and storage contracts. Native visual contract tests
check declared layout rules, so a GTK UI change also needs a live run.

## Verification entrypoints

`mise.toml` defines tool versions and common tasks. `mise run verify` runs
formatting, Cargo checks, Clippy, workspace tests, and the AppImage directory
contract in order. Run `mise run test:gtk` separately on a live display for
GTK widget behavior. CI runs both workspace and GTK tests under Xvfb.

Use `mise run test:core` for the toolkit-independent core. Use `mise run test`
for all workspace behavior and compatibility tests.

Development prerequisites and installation commands are in
[CONTRIBUTING.md](../CONTRIBUTING.md).
