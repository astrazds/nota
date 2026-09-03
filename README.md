# Noter

A local-first Markdown note-taking app. The stable 1.0 line is the existing **Leptos** browser app; the repository now also contains the in-progress Linux-native **Relm4/GTK4** replacement.

Current app version: **1.0.2**.

The native crates use `2.0.0-alpha.1`. This is not a 2.0 cutover: the browser remains the migration source until native parity, packaging, and a manual migration rehearsal pass.

## Native migration preview

- `noter-core` owns toolkit-independent Notes, Flat Collection behavior, Search, Tags, Backup v1, Storage Recovery rules, Markdown Preview generation, and UTF-8 byte-range formatting.
- `noter-web` remains the final browser migration Adapter and now exposes **Export for desktop**.
- `noter-desktop` provides the Relm4 root component, GTK Writing Surface, stable-UUID Note factories, native Backup/transition file workflows, Light/Dark Theme preferences, XDG collection storage, atomic Previous Snapshot replacement, corrupt-payload quarantine, and close-time persistence flush.

Run the native development build on Linux with GTK 4.22 or newer:

```bash
cargo run -p noter-desktop
```

WebKitGTK 6 Preview support is represented by the `preview-webkit` feature and requires the `webkitgtk-6.0` development package. Flatpak build/install/run gates additionally require Flatpak and Flatpak Builder. Those system prerequisites are intentionally not installed by Cargo.

## Features

- **Local-First Note Identity**: Quiet note-app structure with a scannable Frame A sidebar, calm writing surface, aligned main frame/sidebar borders, compact editor footer, warm selected Note state, and paper-neutral popup models.
- **Brand App Icons**: Browser tab, Apple touch, PWA, maskable, monochrome, and favicon-safe icons use the Noter folded-note mark from the brand toolkit.
- **Documented Design System**: `PRODUCT.md`, `DESIGN.md`, `docs/brand-toolkit.md`, and `.impeccable/design.json` capture the product register, brand direction, typography, palette, component rules, and visual anti-patterns used by agents and contributors.
- **Self-Hosted Typography**: Source Sans 3 carries the product UI, while Source Code Pro is reserved for Markdown/source editing. Fonts are bundled locally through Trunk and Tailwind, with no remote font provider.
- **Markdown Support**: Markdown writing with explicit Write, Preview, and desktop Split view modes in a stable editor-area footer that matches the sidebar footer height and compact control rhythm.
  - Supports CommonMark plus tables, footnotes, strikethrough, and task lists.
  - Raw HTML in notes is rendered as text for safety.
  - Use 2 spaces at line-end for hard line breaks.
  - Preview suppresses a duplicate first content heading when it matches the Note Title.
  - Preview renders the Note Title and read-only Tags before the Markdown body, matching the editor header order.
  - Write, Preview, and Split share a left-aligned pane rhythm, `72ch` reading measure, and Note Title scale so view switches do not reframe the note.
- **Organisation Tools**:
  - **Search**: Real-time search bar (with debounce) to filter notes by title, content, or tags with title/Tag highlighting, compact body Match Snippets, lightweight result status, and filtered-empty explanations.
  - **Scoped Search**: Optional syntax for quoted phrases, `title:`, `tag:`, and `is:pinned` filters, shown as a focus-time hint while keeping Search focused on Notes.
  - **Tags**: Lightweight Note Metadata with autocomplete, normalization, compact read-only Tag pills, preview visibility beneath the Note Title, and reviewed cleanup for secondary filtering without folders or notebooks.
  - **Pinning**: Pin important notes to the top of your list.
- **Local Persistence**: Automatically persists notes to your browser's `LocalStorage` with debounced saves while typing, preserving the previous valid active/Recently Deleted collection snapshot before each safe save.
- **Quick Capture**: Create a new Note from the sidebar, empty state, or `Ctrl/Cmd+N`; compact viewports return directly to the Writing Surface with the Note Title focused.
- **Recoverable Delete**: Deleted Notes move to Recently Deleted so accidental deletes can be restored, individually cleared, or cleared all at once after a count-specific confirmation.
- **Storage Recovery**: If saved Notes or Recently Deleted payloads become corrupt, Noter starts in a recovery state with Restore previous snapshot, Start empty, and Import Backup paths before normal editing resumes.
- **Backup & Restore**: Export a versioned Flat Collection backup, track the last successful export with actionable stale/missing Backup Health nudges, and preview add/replace impact before safely merge-importing backups from compact sidebar footer controls.
- **Diagnostics**: About Noter opens from the main frame and exposes version, storage mode, Backup Health, and corrupt-payload quarantine state without adding persistent metadata to the main note workflow.
- **Debug Starter Notes**: Debug builds seed three representative notes when the browser has no saved notes yet, giving manual testing coverage for pinning, tags, rich Markdown, preview safety, search, and responsive editing.
- **Tuned Themes**: Supports Light and Dark themes with coherent surfaces, borders, selection states, and accents.
- **Polished Sidebar Utilities**: Search has a clear affordance, Recently Deleted actions use explicit recovery/destructive copy, and Backup controls sit in a compact labelled footer row.
- **Responsive Design**: Optimised for desktop and mobile, with compact Responsive Navigation and editor-area View Mode Controls.
- **First-Run Flow**: Empty collections show a direct path to create the first note and focus the note title.
- **Enhanced Editing**:
  - Matched Writing Surface and Preview body text scale for lower visual friction.
  - Preview prose uses the same readable body scale in Light, Dark, full Preview, and Split modes, with tinted dark-theme prose rather than pure white.
  - Tag chips stay compact near the Note header, defer removal to the Edit tags flow, preserve 44px mobile touch targets where they become controls, and switch to a single edit input only when editing.
  - Contextual formatting tools (Bold, Italic, Strikethrough, Task List, Insert Table) inside the Writing Surface after Note Metadata.
  - Markdown syntax modal uses the shared paper-neutral popup model with dialog semantics on the popup panel.
  - Floating global notification outlet for save, Backup, and import feedback, inset from the app chrome so it does not sit on the viewport corner.
- **Stable Note Actions**: Pin/unpin and delete are available from a note action menu instead of hover-only controls.
- **Delete Confirmation**: Modal confirmation uses the "Move to Recently Deleted?" frame, names the target Note, and defaults keyboard focus to Cancel before recoverable or permanent removal.
- **Accessibility**: ARIA labels on interactive elements, panel-owned dialog semantics for popups, and mobile touch-target coverage for compact editor controls.

## Technology Stack

- **Frontend**: [Leptos](https://leptos.dev/) (Rust Full-stack Framework)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/) with Typography plugin
- **Parsing**: [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
- **Build Tool**: [Trunk](https://trunkrs.dev/)

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.95 or newer
- Rust WASM target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/#install)
- [Node.js](https://nodejs.org/) (for Tailwind CSS processing)

### Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd noter
   ```

2. Install dependencies:
   ```bash
   npm install
   npx playwright install chromium
   ```

### Running the App

Start the development server:
```bash
npm run dev
```
The app will be available at `http://localhost:8080`.

In debug builds, a browser with no saved `noter-notes` LocalStorage entry starts with three representative testing notes. Release builds and browsers with an existing saved note collection keep the normal empty/saved collection behaviour.

## Testing

Run the Rust unit tests:
```bash
cargo test
```

Check browser-target compilation:
```bash
cargo check --target wasm32-unknown-unknown --all-features
```

Full pre-merge verification used by agents:
```bash
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-features
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
- **Formatting**: Named Markdown commands and UTF-16/UTF-8-safe selection handling.
- **Note Logic**: Workspace behaviours for quick capture, note creation, selected note editing, recoverable delete/restore/individual clear/count-confirmed Clear All, delete confirmation, title extraction, date formatting, preview truncation, and deserialisation.
- **Tags**: Parsing, display formatting, autocomplete suggestions, normalization, individual removal, cleanup planning, case-insensitive matching, collection, and sorting.
- **Persistence**: Save lifecycle and save session behaviour for debounced LocalStorage saves, previous snapshot preservation, corrupt-payload recovery, Recently Deleted storage, and Backup Health metadata.
- **Starter Notes**: Debug-only sample notes cover pinning, tags, rich Markdown, preview safety, long previews, and responsive editing checks.
- **Preview Rendering & Safety**: Title/body separation, duplicate heading suppression, raw HTML escaping, safe URL policy, and supported Markdown preview dialect on the same body-rendering path used by the app.
- **Unicode Support**: Proper handling of multi-byte characters in character counting, preview truncation, formatting, and search highlighting.

Coverage sweeps should follow the repo's TDD convention: add or tighten one behavior-focused test for each uncovered risk, run it, then run the full pre-merge verification gates. Numeric coverage is not currently part of the local toolchain; `cargo llvm-cov` and `cargo tarpaulin` are not installed by default.

## Architecture Notes

The app keeps high-leverage behaviour behind focused Rust Modules:

- `src/app/`: startup construction, runtime persistence orchestration, save snapshots, Theme/sidebar persistence, page flush wiring, and viewport reclassification.
- `src/backup/`: versioned Flat Collection Backup export/import, import preview, Backup Health assessment, validation, merge behavior, sidebar Backup Controls, browser download/FileReader adapters, pending import preview state, and Backup Global Notification outcomes.
- `src/notes/`: Note identity and collection logic, scoped Search parsing, Note List projection, Discovery Depth render models, selected Note workspace behavior, Tags, Quick Capture, recoverable delete/restore, Delete Confirmation, Clear All, and debug starter Notes.
- `src/storage/`: debounced save session, save lifecycle, active Notes persistence, Recently Deleted persistence, previous snapshot persistence, Storage Recovery startup choices, Corrupt Payload Quarantine, Backup Health metadata, and page lifecycle flushing.
- `src/ui/`: View Mode logic, named Markdown commands, Markdown preview rendering/safety, Responsive Navigation, semantic Theme recipes, load-bearing visual recipes, and Writing Surface render models.
- `src/components/`: Leptos rendering Modules for the app shell, sidebar, editor, popups, and shared modal model. These should stay thin and call the deeper product Modules above.

Crate-level aliases preserve the established Module names (`note_workspace`, `note_discovery`, `ui_recipes`, `storage_recovery`, and similar) so existing tests and call sites can use the product vocabulary while the physical file structure stays grouped by area.

## Domain Docs

- `CONTEXT.md` captures the product language used by agents and contributors.
- `PRODUCT.md` captures the product register, users, purpose, brand personality, anti-references, design principles, and accessibility expectations.
- `DESIGN.md` captures the Local Notebook visual system, including palette, typography, elevation, component rules, and do/don't guidance.
- `docs/brand-toolkit.md` captures Noter's brand promise, voice, logo/mark direction, external surface guidance, and brand checks.
- `.impeccable/design.json` mirrors the reusable design tokens and component examples used for UI review.
- `docs/adr/` records accepted design and architecture decisions.

## License

This project is licensed under the [MIT License](./LICENSE).

## Code Guidelines

For AI agents and contributors, please refer to [AGENTS.md](./AGENTS.md) for detailed coding standards and project conventions.
