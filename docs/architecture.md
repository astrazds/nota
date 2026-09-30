# Architecture

Nota has a toolkit-independent Rust domain core and shared application layer.
Linux uses Relm4/GTK4; Windows uses C#/WinUI 3. The browser frontend was removed after the
native cutover described in [ADR-0009](adr/0009-relm4-native-replacement.md).
Backup v1 and desktop-transition import remain permanent compatibility
contracts.

## Dependency direction

`nota-app` depends on `nota-core` and owns application state, persistence,
recovery, and preview generation. `nota-desktop` consumes those crates for GTK.
`nota-ffi` exposes the shared application to WinUI through a small C ABI.
The domain core has no dependency on UI toolkits or native persistence.
Each frontend owns widgets, input, native file selection, and webview hosting.

| Change | Owner | Behavior checks |
| --- | --- | --- |
| Note mutation, selection, recoverable deletion, and restoration | `crates/nota-core/src/note_workspace.rs` and `note_collection.rs` | Core workspace tests |
| Search results, snippets, and Tags | `crates/nota-core/src/note_discovery.rs`, `note_list_interaction.rs`, and `tag_rules.rs` | Core discovery tests and native workflow tests |
| Backup and desktop-transition formats | `crates/nota-core/src/backup.rs` and `transition.rs` | Backup workflows and compatibility fixtures |
| Markdown parsing and formatting | `crates/nota-core/src/markdown_preview.rs` and `markdown_editing.rs` | Core Markdown tests and native visual contracts |
| Application state transitions | `crates/nota-app/src/app.rs` | Shared application tests |
| Storage Recovery and save lifecycle | `crates/nota-app` | Persistence and session behavior tests |
| Desktop widgets and input conversion | `crates/nota-desktop/src/ui/workspace.rs` and `selection.rs` | Native visual contracts and live GTK verification |
| Desktop row identity and synchronization | `crates/nota-desktop/src/ui/note_list.rs` | Row identity tests and live GTK selection |
| Native styling and layout | `crates/nota-desktop/resources/nota.css`, `src/visual_contract.rs`, `src/ui/style.rs`, and `src/ui/writing_plane.rs` within the desktop crate | Native visual contract tests and live GTK verification |
| Desktop dialogs and JSON file selection | `crates/nota-desktop/src/ui/dialogs.rs` and `files.rs` | Native import workflows and live GTK dialogs |
| Atomic native storage | `crates/nota-app/src/storage.rs` and `persistence.rs` | Native storage, shutdown, and transition tests |
| Preview HTML and navigation policy | `crates/nota-app/src/preview.rs` | Preview tests and native workflow tests |
| Windows ABI and buffer ownership | `crates/nota-ffi` | ABI tests and C# integration checks |
| Windows widgets, native editing, and WebView2 | `windows/Nota.Windows` | Windows build and live UI verification |

## Follow a change through the app

User actions enter through `AppMsg` in `crates/nota-app/src/app.rs`.
`AppModel` delegates Note mutations to `NoteWorkspace` and increments its save
revision after a persistent change. The desktop shell schedules
`PersistenceWorker`, which coalesces edits, writes through `NativeStore`, and
reports completion back as an `AppMsg`. The GTK thread does not wait for a
normal save. Orderly shutdown flushes the latest pending revision.

Search uses `NoteListInteraction` to produce a render-ready projection. The
desktop `NoteLists` owns its factories and updates existing row widgets when
UUID order is unchanged. Dialogs and file selection emit `AppMsg` through a
channel. They do not depend on the root Relm4 component type.

Core Markdown formatting accepts UTF-8 byte ranges. The GTK toolbar applies
the changed range as one TextBuffer user action so undo, redo, and caret
placement remain owned by GTK. GTK character conversion stays in
`crates/nota-desktop/src/selection.rs`. The core renders the Markdown body.
Shared `preview.rs` wraps it with the
Note Title, Tags, CSS, and content security policy. `webkit_preview.rs` owns
WebKitGTK settings and navigation; the GTK shell opens allowed external links
through the system handler.

The Windows frontend serializes DLL calls on a background queue. Typed JSON
requests cross the C ABI as borrowed UTF-8 bytes. Rust allocates replies;
C# copies them and returns each buffer to Rust. A SafeHandle owns the session.
The ABI crate alone permits the unsafe code needed for pointer conversion.
Domain and application code continue to forbid unsafe Rust.

Windows edits carry a note UUID and an edit sequence. Snapshot responses must
not reset a newer native edit or its undo history. Formatting converts UTF-16
selection offsets into the core's UTF-8 byte ranges. Close waits for a
successful flush before releasing the session, so a failed save can be retried.
WebView2 displays shared generated HTML with restricted scripts, resource
requests, and navigation.

## Boundaries worth preserving

Keep `NoteWorkspace` and `NoteListInteraction` as the domain behavior
boundaries. Keep platform focus and page composition in the frontends.
Keep save and recovery invariants in `nota-app`.

Keep Backup v1, desktop-transition v1, legacy kind acceptance, UUID identity,
and merge-only import compatible. Keep current and previous native snapshots
as complete active and Recently Deleted collection pairs. The acceptance
fixtures live in `crates/nota-core/tests/fixtures/`.

Keep the GTK widget hierarchy stable during structural changes. Module tests
cover domain behavior and storage contracts. Native visual contract tests
check declared layout rules, so a GTK UI change also needs a live run.

## Storage and compatibility

`NativeStore` validates the active and Recently Deleted collections together.
It writes `collection.json` atomically and retains `collection.previous.json`
as the last valid pair. Preferences and Backup Health have separate files.
Unreadable collection data enters `LoadOutcome::Recovery`; it is never
silently replaced with starter Notes.

Merge Import operates through `NoteWorkspace`, including identities currently
in Recently Deleted. Desktop-transition restore requires both collections to
be empty and transfers optional Backup Health exactly. These formats have
separate fixtures and tests because their restore semantics differ.

See [the user guide](usage.md) for the user-facing recovery choices.

## Verification entrypoints

`mise.toml` defines tool versions and common tasks. `mise run verify` runs
formatting, Cargo checks, Clippy, workspace tests, and the AppImage directory
contract in order. Run `mise run test:gtk` separately on a live display for
GTK widget behavior. CI runs both workspace and GTK tests under Xvfb.

On Windows, `mise run verify` checks the portable Rust crates, builds WinUI,
and runs `mise run test:windows` against the real DLL. `mise run package:windows`
publishes a self-contained x64 folder and ZIP. Native UI verification remains
separate from those automated checks; follow [the Windows guide](windows.md).

Use `mise run test:core` for the toolkit-independent core. Use `mise run test`
for all workspace behavior and compatibility tests.

Use the [native verification procedure](../.agents/skills/verify-nota/SKILL.md)
for isolated UI runs and persistence proof. The [documentation index](README.md)
links dated verification records and their coverage limits.

Development prerequisites and installation commands are in
[CONTRIBUTING.md](../CONTRIBUTING.md).
