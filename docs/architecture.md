# Architecture

Nota has one domain core and two platform adapters. The desktop app is the
product. The browser app remains available for the migration described in
[ADR-0009](adr/0009-relm4-native-replacement.md).

## Dependency direction

Both frontends depend on `nota-core`. The core owns its source files and has
no dependency on either frontend, Leptos, GTK, browser storage, or Linux paths.
Its public modules retain their existing names so moving an implementation
does not require an API migration.

| Change | Owner | Behavior checks |
| --- | --- | --- |
| Note mutation, selection, deletion, and recovery | `crates/nota-core/src/note_workspace.rs` and `note_collection.rs` | Core workspace tests |
| Search results, snippets, and Tags | `crates/nota-core/src/note_discovery.rs`, `note_list_interaction.rs`, and `tag_rules.rs` | Core discovery tests and browser Search workflows |
| Backup and desktop transition formats | `crates/nota-core/src/backup.rs` and `transition.rs` | Core Backup workflows and transition fixtures |
| Browser state and save revisions | `src/app/state.rs` | Browser state tests |
| Browser startup, event listeners, and persistence effects | `src/app/runtime.rs` | Runtime tests and browser save workflows |
| Browser composition and notifications | `src/app/mod.rs` and `src/components/notification.rs` | Browser visual contracts |
| Browser storage and downloads | `src/storage/mod.rs` and `src/backup/controls.rs` | Browser recovery and Backup workflows |
| Desktop state transitions | `crates/nota-desktop/src/app.rs` | Native workflow tests |
| Desktop effects and shutdown | `crates/nota-desktop/src/ui/mod.rs` | Persistence tests and native rehearsal |
| Desktop widgets and input conversion | `crates/nota-desktop/src/ui/workspace.rs` and `selection.rs` | Native visual contracts and live GTK verification |
| Desktop row identity and synchronization | `crates/nota-desktop/src/ui/note_list.rs` | Row identity tests and live GTK selection |
| Desktop dialogs and JSON file selection | `crates/nota-desktop/src/ui/dialogs.rs` and `files.rs` | Native import workflows and live GTK dialogs |
| Atomic native storage | `crates/nota-desktop/src/storage.rs` and `persistence.rs` | Native storage, shutdown, and transition tests |

## Follow a change through the app

Quick Capture and editing enter through `AppState` in the browser or `AppMsg`
on desktop. Both delegate Note mutations to `NoteWorkspace`. The frontend
increments its save revision after a persistent change. Browser runtime
effects schedule LocalStorage writes. The desktop shell schedules
`PersistenceWorker`, which writes through `NativeStore` and flushes on shutdown.

Search uses `NoteListInteraction` to produce a render-ready projection.
The desktop `NoteLists` owns its factories and updates existing row widgets
when UUID order is unchanged. Dialogs and file selection emit `AppMsg` through
a channel. They do not depend on the root Relm4 component type.

Core Markdown formatting accepts UTF-8 byte ranges. Browser UTF-16 selection
conversion remains in `src/ui/markdown_editing.rs`. GTK character conversion
remains in `crates/nota-desktop/src/selection.rs`.

## Boundaries worth preserving

Keep the existing `NoteWorkspace` and `NoteListInteraction` interfaces as the
shared behavior boundary. Frontend state remains separate because Leptos
signals, GTK focus, page lifecycle, and native shutdown have different rules.
A shared application reducer would couple the temporary browser adapter to
the desktop's presentation lifecycle without removing those differences.

Keep Backup v1, desktop-transition v1, legacy kind acceptance, UUID identity,
and merge-only import compatible. Keep current and previous native snapshots
as complete active and Recently Deleted collection pairs. The acceptance
fixtures live in `crates/nota-core/tests/fixtures/`.

Keep GTK widget hierarchy and browser DOM hierarchy stable during structural
changes. Module tests cover domain behavior and storage contracts. The browser
suite also checks rendered workflows and layout. Native visual contract tests
check declared layout rules, so a GTK UI change also needs a live run.

## Verification entrypoints

`mise.toml` defines tool versions and common tasks. Run `mise tasks` to list
them and `mise run verify` for the complete local gate. Use `mise run test`
for workspace behavior, `mise run check:web` for WebAssembly, and
`mise run test:browser -- --grep '<workflow>'` for a focused browser check.

Development prerequisites and installation commands are in
[CONTRIBUTING.md](../CONTRIBUTING.md).
