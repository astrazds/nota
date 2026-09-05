# Nota

A local-first Markdown note-taking app. The Linux-native **Relm4/GTK4** app is the post-1.0 product (`2.0.0-alpha.1`). The stable **1.0.2** line is the **Leptos** browser Adapter, kept as the migration source until native cutover.

This is not a 2.0.0 release. ADR-0009 and ADR-0010 withhold a 2.0.0 tag until a clean-profile web-to-desktop rehearsal on the AppImage is recorded and publication is approved. Native already presents the 1.0 Markdown Note App: Backup Import Preview, Storage Recovery, Search-led discovery, Tag pills with Edit tags and collection Tag autocomplete, paper-neutral About / Markdown help / confirmations, Preview and 50/50 Split, and installed fonts. The first shippable native artifact is an x86_64 AppImage wrapping the Meson prefix.

## Native migration

- `nota-core` owns toolkit-independent Notes, Flat Collection behavior, Search, Tags, Backup v1, Storage Recovery rules, Markdown Preview generation, UTF-8 byte-range formatting, and the Markdown syntax cheatsheet.
- `nota-web` remains the final browser migration Adapter and exposes **Export for desktop** (`nota.desktop_transition` v1; import still accepts `noter.desktop_transition`).
- `nota-desktop` is the Relm4 root `AppModel`/`AppMsg` Component: GTK Writing Surface, Note List factories that update in place on selection, Backup Import Preview then Merge Import, desktop-transition Restore into an Empty Collection, Storage Recovery, Search Hint and Discovery Depth UI, compact Tag pills with an Edit tags flow and collection Tag autocomplete, paper-neutral About / Markdown help / Delete Confirmation / Clear All / Backup Import Preview windows, Light/Dark Theme, XDG collection storage under `net.astrazds.Nota`, atomic Previous Snapshot replacement, Corrupt Payload Quarantine, Diagnostics, and close-time persistence flush.

Production desktop, icon, and binary use application ID `net.astrazds.Nota`. Collection data lives at `$XDG_DATA_HOME/net.astrazds.Nota` (typically `~/.local/share/net.astrazds.Nota`). A first launch migrates `$XDG_DATA_HOME/net.astrazds.Noter` or a legacy `$XDG_DATA_HOME/noter` directory when the canonical path is absent. Devel Flatpak uses `net.astrazds.Nota.Devel` and stays in tree for later. Source Sans 3 and Source Code Pro ship in `assets/fonts` and install with the app; installed runs do not read `node_modules`.

The first native distribution path is an AppImage wrapping the Meson prefix (ADR-0010):

```bash
meson setup build --prefix=/usr --buildtype=release
meson compile -C build
DESTDIR="$PWD/build/AppDir" meson install -C build
python3 build-aux/package_appimage.py package build/AppDir --output dist/Nota-x86_64.AppImage
```

That packager downloads linuxdeploy tools into `build-aux/.tool-cache` on demand, bundles WebKitGTK 6 helpers, and verifies the AppDir contract (`python3 build-aux/test_package_appimage.py`). It keeps the AppImage linuxdeploy just wrote, even if `dist/` already has an older `*.AppImage`. The Devel Flatpak manifest stays in tree for later; Flatpak Builder is not required for the AppImage.

Clean-profile web-to-desktop rehearsal (ADR-0009/0010) is `/appimage-rehearsal`. The procedure is `docs/agents/appimage-rehearsal.md`.

## Features

- **Local-First Note Identity**: Quiet note-app structure with a scannable Frame A sidebar, calm writing surface, aligned main frame/sidebar borders, compact editor footer, warm selected Note state, and paper-neutral popup models.
- **Brand App Icons**: Browser tab, Apple touch, PWA, maskable, monochrome, and favicon-safe icons use the Nota folded-note mark from the brand toolkit.
- **Documented Design System**: `PRODUCT.md`, `DESIGN.md`, `docs/brand-toolkit.md`, and `.impeccable/design.json` capture the product register, brand direction, typography, palette, component rules, and visual anti-patterns used by agents and contributors.
- **Self-Hosted Typography**: Source Sans 3 carries the product UI, while Source Code Pro is reserved for Markdown/source editing. The browser bundles fonts through Trunk and Tailwind. Native installs the same Source families from `assets/fonts`. There is no remote font provider.
- **Markdown Support**: Markdown writing with explicit Write, Preview, and desktop Split view modes in a stable editor-area footer that matches the sidebar footer height and compact control rhythm.
  - Supports CommonMark plus tables, footnotes, strikethrough, and task lists.
  - Raw HTML in notes is rendered as text for safety.
  - Use 2 spaces at line-end for hard line breaks.
  - Preview suppresses a duplicate first content heading when it matches the Note Title.
  - Preview renders the Note Title and read-only Tags before the Markdown body, matching the editor header order.
  - Write, Preview, and Split share a left-aligned pane rhythm, `72ch` reading measure, and Note Title scale so view switches do not reframe the note. Split divides the editor area 50/50 of the current viewport.
- **Organisation Tools**:
  - **Search**: Real-time search bar (with debounce) to filter notes by title, content, or tags with title/Tag highlighting, compact body Match Snippets, lightweight result status, and filtered-empty explanations.
  - **Scoped Search**: Optional syntax for quoted phrases, `title:`, `tag:`, and `is:pinned` filters, shown as a focus-time hint while keeping Search focused on Notes.
  - **Tags**: Lightweight Note Metadata with normalization, compact read-only Tag pills on both the Note List and Writing Surface, preview visibility beneath the Note Title, and an Edit tags flow with collection Tag autocomplete. The browser Adapter also offers collection-wide Tag cleanup.
  - **Pinning**: Pin important notes to the top of your list.
- **Local Persistence**: Debounced saves while typing, preserving the previous valid active/Recently Deleted collection snapshot before each safe save. The browser Adapter uses `LocalStorage`. Native stores a versioned `collection.json` under `$XDG_DATA_HOME/net.astrazds.Nota`.
- **Quick Capture**: Create a new Note from the sidebar, empty state, or `Ctrl/Cmd+N`; compact viewports return directly to the Writing Surface with the Note Title focused.
- **Recoverable Delete**: Deleted Notes move to Recently Deleted so accidental deletes can be restored, individually cleared, or cleared all at once after a count-specific confirmation.
- **Storage Recovery**: If saved Notes or Recently Deleted payloads become corrupt, Nota starts in a recovery state with Restore previous snapshot, Start empty, and Import Backup paths before normal editing resumes.
- **Backup & Restore**: Export a versioned Flat Collection backup, track the last successful export with actionable stale/missing Backup Health nudges, and preview add/replace impact before safely merge-importing backups from compact sidebar footer controls.
- **Diagnostics**: About Nota is a paper-neutral product window from the sidebar and exposes version, storage path, Backup Health, and corrupt-payload quarantine state without adding persistent metadata to the main note workflow.
- **Debug Starter Notes**: Browser Adapter debug builds seed three representative notes when there is no saved collection yet, giving manual testing coverage for pinning, tags, rich Markdown, preview safety, search, and responsive editing.
- **Tuned Themes**: Supports Light and Dark themes with coherent surfaces, borders, selection states, and accents.
- **Polished Sidebar Utilities**: Search has a clear affordance, Recently Deleted actions use explicit recovery/destructive copy, and Backup controls sit in a compact labelled footer row.
- **Responsive Design**: Optimised for desktop and mobile, with compact Responsive Navigation and editor-area View Mode Controls.
- **First-Run Flow**: Empty collections show a direct path to create the first note and focus the note title.
- **Enhanced Editing**:
  - Matched Writing Surface and Preview body text scale for lower visual friction.
  - Preview prose uses the same readable body scale in Light, Dark, full Preview, and Split modes, with tinted dark-theme prose rather than pure white.
  - Tag chips stay compact near the Note header, defer removal to the Edit tags flow, preserve 44px mobile touch targets where they become controls, and switch to a single edit input only when editing.
  - Contextual formatting tools (Bold, Italic, Strikethrough, Task List, Insert Table) inside the Writing Surface after Note Metadata.
  - Markdown syntax help uses the shared paper-neutral popup model: a notebook header, two-column cheatsheet, and dialog semantics on the popup panel.
  - Floating global notification outlet for save, Backup, and import feedback, inset from the app chrome so it does not sit on the viewport corner.
- **Stable Note Actions**: Pin/unpin and delete are available from a note action menu instead of hover-only controls.
- **Delete Confirmation**: Paper-neutral confirmation uses the "Move to Recently Deleted?" frame, names the target Note, and defaults keyboard focus to Cancel before recoverable or permanent removal. Clear All and Backup Import Preview use the same paper dialog model.
- **Accessibility**: Discoverable controls for keyboard, pointer, and touch. The browser Adapter uses ARIA labels and panel-owned dialog semantics. Native uses GTK accessible roles and names.

## Technology Stack

Native (post-1.0 product):

- **UI**: [Relm4](https://relm4.org/) on GTK 4.22+ (GNOME 50)
- **Preview**: WebKitGTK 6 when the `preview-webkit` feature is enabled
- **Install**: thin Meson layer
- **Distribution**: x86_64 AppImage via `build-aux/package_appimage.py` (ADR-0010)

Shared:

- **Domain**: `nota-core` (Notes, Search, Tags, Backup v1, Storage Recovery, View Mode, Markdown)
- **Parsing**: [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)

Browser Adapter (1.0.2 migration source):

- **UI**: [Leptos](https://leptos.dev/) (CSR)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/) with Typography plugin
- **Build**: [Trunk](https://trunkrs.dev/)

## Getting Started

### Native (Linux)

Prerequisites: [Rust](https://www.rust-lang.org/tools/install) 1.95 or newer, GTK 4.22 or newer. Preview/Split also need the `webkitgtk-6.0` development package. Default `cargo run -p nota-desktop` does not enable Preview; Meson and AppImage builds do.

```bash
cargo run -p nota-desktop
cargo run -p nota-desktop --features preview-webkit
```

### Browser Adapter

Prerequisites: Rust 1.95, `wasm32-unknown-unknown`, [Trunk](https://trunkrs.dev/#install), and [Node.js](https://nodejs.org/).

```bash
npm install
npx playwright install chromium
npm run dev
```

The Adapter is at `http://localhost:8080`. In debug builds, a browser with no saved `nota-notes` LocalStorage entry starts with three representative testing notes.

## Testing

Run the Rust unit tests:
```bash
cargo test
cargo test -p nota-desktop --all-targets
python3 build-aux/test_package_appimage.py
```

Check browser-target compilation:
```bash
cargo check --target wasm32-unknown-unknown --all-features
```

Full pre-merge verification used by agents:
```bash
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-features
python3 build-aux/test_package_appimage.py
cargo check --target wasm32-unknown-unknown --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --workspace --no-deps
npm run build
npx impeccable detect --json --fast src index.html
npm run test:browser
```

Browser tests run through a single Playwright worker because Trunk/Tailwind emits load-bearing CSS for visual contracts during startup; serial browser coverage keeps the rendered UI checks deterministic.

Tests cover core domain logic including:
- **Backup & Restore**: Versioned Flat Collection backup export, validation, import preview, merge import, duplicate identity handling, actionable backup health, and all-or-nothing failure behavior.
- **Storage Recovery**: Startup corrupt-payload detection, previous snapshot restore, start-empty quarantine, and Backup import availability during recovery.
- **Filtering & Sorting**: Real-time search, scoped query parsing, quoted phrase matching, title/Tag highlighting, compact body Match Snippets, render-ready note list projection, active Search/Tag result status, filtered-empty explanations, active tag filtering, and note pinning logic.
- **Browser Visual Regressions**: Playwright coverage verifies Light/Dark Theme readability, Search Hint contrast and placement, selected Note state, emitted Tailwind/style contracts, local Source font loading, app icon/manifest assets, Frame A material surfaces, editor/sidebar footer height parity, compact footer controls, compact desktop Tag chips, labelled desktop actions, 44px mobile touch targets, popup panel dialog semantics, startup notification quietness, Preview/Split Note Title consistency, Note Metadata ordering, dark Preview/Split prose, Write/Preview/Split pane alignment, Backup Controls placement, and floating Global Notification layering.
- **Browser Workflow Regressions**: Playwright coverage exercises Quick Capture, Note Title editing, Note creation/edit/save, scoped Search, pinning, Tags, Formatting Tools, Preview safety, Backup export/import, Responsive Navigation, Markdown syntax help, recoverable delete/restore, and Clear All.
- **Native Workflow Smoke**: `nota-desktop` tests cover Backup Import Preview then Merge Import, Storage Recovery including Import Backup, Search filtered-empty vs Empty Collection, Quick Capture Note Title focus, Preview/Split View Mode surfaces, bundled fonts without `node_modules`, XDG `net.astrazds.Nota` discovery with `net.astrazds.Noter` and legacy `noter` migration, Tag suggestions, Note List in-place selection, paper-dialog visual contracts, and a clean-profile web-to-desktop transition restore that rejects a second exact restore and then uses Merge Import.
- **AppImage AppDir contract**: `build-aux/test_package_appimage.py` (also `meson test`) checks the installed layout, bundled WebKitGTK 6 helpers, font files, desktop file `Exec=nota-desktop`, and the runtime hook that sets `NOTA_FONT_DIR` and overlays WebKit helpers. The clean-profile web-to-desktop rehearsal is `docs/agents/appimage-rehearsal.md`.
- **Formatting**: Named Markdown commands and UTF-16/UTF-8-safe selection handling.
- **Note Logic**: Workspace behaviours for quick capture, note creation, selected note editing, recoverable delete/restore/individual clear/count-confirmed Clear All, delete confirmation, title extraction, date formatting, preview truncation, and deserialisation.
- **Tags**: Parsing, display formatting, autocomplete suggestions, normalization, individual removal, cleanup planning, case-insensitive matching, collection, and sorting.
- **Persistence**: Debounced saves, previous snapshot preservation, corrupt-payload recovery, Recently Deleted storage, and Backup Health metadata. Browser Adapter tests cover LocalStorage; native tests cover XDG `$XDG_DATA_HOME/net.astrazds.Nota/collection.json` and close-time flush.
- **Starter Notes**: Debug-only sample notes cover pinning, tags, rich Markdown, preview safety, long previews, and responsive editing checks.
- **Preview Rendering & Safety**: Title/body separation, duplicate heading suppression, raw HTML escaping, safe URL policy, and supported Markdown preview dialect on the same body-rendering path used by the app.
- **Unicode Support**: Proper handling of multi-byte characters in character counting, preview truncation, formatting, and search highlighting.

Coverage sweeps should follow the repo's TDD convention: add or tighten one behavior-focused test for each uncovered risk, run it, then run the full pre-merge verification gates. Numeric coverage is not currently part of the local toolchain; `cargo llvm-cov` and `cargo tarpaulin` are not installed by default.

## Architecture Notes

Toolkit-independent behaviour lives in `nota-core` (some Modules are still `#[path]`-included from `src/` until the browser Adapter is retired):

- `src/notes/` / `nota-core`: Note identity, Flat Collection, scoped Search, Note List projection, Tags, Quick Capture, recoverable delete, Clear All.
- `src/backup/` / `nota-core`: Backup v1 export/import, import preview, Merge Import, Backup Health.
- `src/storage/` / `nota-core`: Storage Recovery rules, Previous Snapshot, Corrupt Payload Quarantine.
- `src/ui/` / `nota-core`: View Mode, Markdown commands, Markdown Preview generation, Responsive Navigation.

The Relm4 desktop app lives in `crates/nota-desktop/`: `AppModel`/`AppMsg`, XDG `NativeStore` under `net.astrazds.Nota`, bundled font lookup, WebKit Preview when enabled, paper-neutral GTK dialogs, and the GTK shell. Selecting a Note updates existing Note List row widgets in place so the sidebar does not jump to the top. Meson installs the binary, desktop file, icon, metainfo, and Source fonts. `build-aux/package_appimage.py` wraps that prefix into an AppImage and bundles WebKitGTK 6 helpers.

The browser Adapter lives under `src/app/`, `src/components/`, and the remaining `src/storage/` / `src/backup/` Adapters (LocalStorage, download/FileReader, Trunk/Tailwind). Crate-level aliases preserve Module names (`note_workspace`, `note_discovery`, `ui_recipes`, `storage_recovery`) so tests keep the product vocabulary.

## Domain Docs

- `CONTEXT.md` captures the product language used by agents and contributors.
- `PRODUCT.md` captures the product register, users, purpose, brand personality, anti-references, design principles, and accessibility expectations.
- `DESIGN.md` captures the Local Notebook visual system, including palette, typography, elevation, component rules, and do/don't guidance.
- `docs/brand-toolkit.md` captures Nota's brand promise, voice, logo/mark direction, external surface guidance, and brand checks.
- `.impeccable/design.json` mirrors the reusable design tokens and component examples used for UI review.
- `docs/adr/` records accepted design and architecture decisions, including ADR-0009 (Relm4 native replacement) and ADR-0010 (AppImage as the first native distribution artifact).

## License

This project is licensed under the [MIT License](./LICENSE).

## Code Guidelines

For AI agents and contributors, please refer to [AGENTS.md](./AGENTS.md) for detailed coding standards and project conventions.
