# Noter

A local-first Markdown note-taking web app built with **Rust**, **Leptos**, and **Tailwind CSS**.

Current app version: **0.9.3**.

## Features

- **Local-First Note Identity**: Quiet note-app structure with a scannable sidebar, calm writing surface, and warm accents.
- **Documented Design System**: `PRODUCT.md`, `DESIGN.md`, and `.impeccable/design.json` capture the product register, typography, palette, component rules, and visual anti-patterns used by agents and contributors.
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
- **Local Persistence**: Automatically persists notes to your browser's `LocalStorage` with debounced saves while typing.
- **Quick Capture**: Create a new Note from the sidebar, empty state, or `Ctrl/Cmd+N`; compact viewports return directly to the Writing Surface with the Note Title focused.
- **Recoverable Delete**: Deleted Notes move to Recently Deleted so accidental deletes can be restored, individually cleared, or cleared all at once after a count-specific confirmation.
- **Backup & Restore**: Export a versioned Flat Collection backup, track the last successful export, and preview add/replace impact before safely merge-importing backups from compact sidebar footer controls.
- **Debug Starter Notes**: Debug builds seed three representative notes when the browser has no saved notes yet, giving manual testing coverage for pinning, tags, rich Markdown, preview safety, search, and responsive editing.
- **Tuned Themes**: Supports Light and Dark themes with coherent surfaces, borders, selection states, and accents.
- **Polished Sidebar Utilities**: Search has a clear affordance, Recently Deleted actions use explicit recovery/destructive copy, and Backup controls sit in a compact labelled footer row.
- **Responsive Design**: Optimised for desktop and mobile, with compact Responsive Navigation and editor-area View Mode Controls.
- **First-Run Flow**: Empty collections show a direct path to create the first note and focus the note title.
- **Enhanced Editing**:
  - Matched Writing Surface and Preview body text scale for lower visual friction.
  - Preview prose uses the same readable body scale in Light, Dark, full Preview, and Split modes.
  - Tag chips stay compact near the Note header, defer removal to the Edit tags flow, preserve mobile touch targets, and switch to a single edit input only when editing.
  - Contextual formatting tools (Bold, Italic, Strikethrough, Task List, Insert Table) inside the Writing Surface after Note Metadata.
  - Markdown syntax modal
  - Floating global notification outlet for save, Backup, and import feedback above app chrome
- **Stable Note Actions**: Pin/unpin and delete are available from a note action menu instead of hover-only controls.
- **Delete Confirmation**: Modal confirmation uses the "Move to Recently Deleted?" frame, names the target Note, and defaults keyboard focus to Cancel before recoverable or permanent removal.
- **Accessibility**: ARIA labels on interactive elements.

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
- **Backup & Restore**: Versioned Flat Collection backup export, validation, import preview, merge import, duplicate identity handling, backup health, and all-or-nothing failure behavior.
- **Filtering & Sorting**: Real-time search, scoped query parsing, quoted phrase matching, title/Tag highlighting, compact body Match Snippets, render-ready note list projection, active Search/Tag result status, filtered-empty explanations, active tag filtering, and note pinning logic.
- **Browser Visual Regressions**: Playwright coverage verifies Light/Dark Theme readability, Search Hint contrast and placement, selected Note state, emitted Tailwind/style contracts, local Source font loading, editor/sidebar footer height parity, compact footer controls, compact desktop Tag chips, labelled desktop actions, mobile touch targets, startup notification quietness, Preview/Split Note Title consistency, Note Metadata ordering, dark Preview/Split prose, Write/Preview/Split pane alignment, Backup Controls placement, and floating Global Notification layering.
- **Browser Workflow Regressions**: Playwright coverage exercises Quick Capture, Note Title editing, Note creation/edit/save, scoped Search, pinning, Tags, Formatting Tools, Preview safety, Backup export/import, Responsive Navigation, Markdown syntax help, recoverable delete/restore, and Clear All.
- **Formatting**: Named Markdown commands and UTF-16/UTF-8-safe selection handling.
- **Note Logic**: Workspace behaviours for quick capture, note creation, selected note editing, recoverable delete/restore/individual clear/count-confirmed Clear All, delete confirmation, title extraction, date formatting, preview truncation, and deserialisation.
- **Tags**: Parsing, display formatting, autocomplete suggestions, normalization, individual removal, cleanup planning, case-insensitive matching, collection, and sorting.
- **Persistence**: Save lifecycle and save session behaviour for debounced LocalStorage saves, Recently Deleted storage, and Backup Health metadata.
- **Starter Notes**: Debug-only sample notes cover pinning, tags, rich Markdown, preview safety, long previews, and responsive editing checks.
- **Preview Rendering & Safety**: Title/body separation, duplicate heading suppression, raw HTML escaping, safe URL policy, and supported Markdown preview dialect on the same body-rendering path used by the app.
- **Unicode Support**: Proper handling of multi-byte characters in character counting, preview truncation, formatting, and search highlighting.

Coverage sweeps should follow the repo's TDD convention: add or tighten one behavior-focused test for each uncovered risk, run it, then run the full pre-merge verification gates. Numeric coverage is not currently part of the local toolchain; `cargo llvm-cov` and `cargo tarpaulin` are not installed by default.

## Architecture Notes

The app keeps high-leverage behaviour behind focused Rust Modules:

- `tag_rules`: tag parsing, display formatting, collection, sorting, and case-insensitive matching.
- `backup`: versioned Flat Collection backup export/import, import preview, backup health assessment, validation, and merge behavior.
- `backup_controls`: sidebar Backup Controls, browser download/FileReader adapters, pending import preview state, and Backup Global Notification outcomes.
- `search_query`: scoped Search parsing and Note matching for quoted phrases, `title:`, `tag:`, and `is:pinned`.
- `note_workspace`: selected note lookup, empty collection display state, note creation, selected note editing, recoverable delete/restore/individual clear/count-confirmed Clear All, delete confirmation target naming, and pinning behaviours.
- `note_discovery`: the primary Note List projection Interface for Search integration, active Tag filtering, selected Note visibility, ordering, display fields, render keys, and highlight segments.
- `app_runtime`: startup construction, runtime persistence orchestration, save snapshots, Theme/sidebar persistence, page flush wiring, and viewport reclassification.
- `editor_view`: explicit Write, Preview, and Split view modes with viewport-aware behaviour.
- `storage`: debounced save session, save lifecycle, active Notes persistence, Recently Deleted persistence, Backup Health metadata, and page lifecycle flushing.
- `ui_recipes`: load-bearing visual recipes for shared footer rhythm, compact controls, typography roles, pane measure, Search, Search Hint, Tag pills, selected Note rows, recovery controls, and Backup Controls while keeping `theme` semantic.
- `writing_surface`: render-ready Writing Surface model, Preview Note Title/Note Metadata/body ordering, hidden-by-filter messaging, and formatting command application behind a selection-safe Interface.
- `markdown_editing`: named Markdown commands, cheatsheet sections, and Unicode-safe caret handling.
- `markdown_preview`: supported Markdown body rendering, duplicate title suppression, and preview safety policy. The Leptos preview pane owns the Note Title and read-only Tags so Preview and Split match the editor header order.

## Domain Docs

- `CONTEXT.md` captures the product language used by agents and contributors.
- `PRODUCT.md` captures the product register, users, purpose, brand personality, anti-references, design principles, and accessibility expectations.
- `DESIGN.md` captures the Local Notebook visual system, including palette, typography, elevation, component rules, and do/don't guidance.
- `.impeccable/design.json` mirrors the reusable design tokens and component examples used for UI review.
- `docs/adr/` records accepted design and architecture decisions.

## License

This project is licensed under the [MIT License](./LICENSE).

## Code Guidelines

For AI agents and contributors, please refer to [AGENTS.md](./AGENTS.md) for detailed coding standards and project conventions.
