# Noter

A local-first Markdown note-taking web app built with **Rust**, **Leptos**, and **Tailwind CSS**.

Current app version: **0.3.0**.

## Features

- **Local-First Note Identity**: Quiet note-app structure with a scannable sidebar, calm writing surface, and warm accents.
- **Markdown Support**: Markdown writing with explicit Write, Preview, and desktop Split view modes.
  - Supports CommonMark plus tables, footnotes, strikethrough, and task lists.
  - Raw HTML in notes is rendered as text for safety.
  - Use 2 spaces at line-end for hard line breaks.
- **Organisation Tools**:
  - **Search**: Real-time search bar (with debounce) to filter notes by title, content, or tags with text highlighting.
  - **Tags**: Lightweight note metadata for secondary filtering without folders or notebooks.
  - **Pinning**: Pin important notes to the top of your list.
- **Local Persistence**: Automatically persists notes to your browser's `LocalStorage` with debounced saves while typing.
- **Tuned Themes**: Supports Light and Dark themes with coherent surfaces, borders, selection states, and accents.
- **Responsive Design**: Optimised for desktop and mobile, with top-bar navigation between the Note List and Writing Surface.
- **First-Run Flow**: Empty collections show a direct path to create the first note and focus the note title.
- **Enhanced Editing**:
  - Contextual formatting tools (Bold, Italic, Strikethrough, Task List, Insert Table)
  - Markdown help modal
  - Save status near the active editing context
- **Stable Note Actions**: Pin/unpin and delete are available from a note action menu instead of hover-only controls.
- **Delete Confirmation**: Modal confirmation names the target note before deleting it.
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
   ```

### Running the App

Start the development server:
```bash
trunk serve --open
```
The app will be available at `http://localhost:8080`.

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
```

Tests cover core domain logic including:
- **Filtering & Sorting**: Real-time search, text highlighting, render-ready note list projection, active tag filtering, and note pinning logic.
- **Formatting**: Named Markdown commands and UTF-16/UTF-8-safe selection handling.
- **Note Logic**: Workspace behaviours for note creation, selected note editing, delete confirmation, title extraction, date formatting, preview truncation, and deserialisation.
- **Tags**: Parsing, display formatting, case-insensitive matching, collection, and sorting.
- **Persistence**: Save lifecycle and save session behaviour for debounced LocalStorage saves.
- **Preview Safety**: Raw HTML escaping, safe URL policy, and supported Markdown preview dialect.
- **Unicode Support**: Proper handling of multi-byte characters in character counting, preview truncation, formatting, and search highlighting.

## Architecture Notes

The app keeps high-leverage behaviour behind focused Rust Modules:

- `tag_rules`: tag parsing, display formatting, collection, sorting, and case-insensitive matching.
- `note_workspace`: selected note lookup, empty collection display state, note creation, selected note editing, delete confirmation target naming, and pinning behaviours.
- `note_discovery`: note list projection, search, active tag filtering, ordering, display fields, and highlight segments.
- `editor_view`: explicit Write, Preview, and Split view modes with viewport-aware behaviour.
- `storage`: debounced save session, save lifecycle, LocalStorage persistence, and page lifecycle flushing.
- `markdown_editing`: named Markdown commands, cheatsheet sections, and Unicode-safe caret handling.
- `markdown_preview`: supported Markdown preview rendering and preview safety policy.

## Domain Docs

- `CONTEXT.md` captures the product language used by agents and contributors.
- `docs/adr/` records accepted design and architecture decisions.

## License

This project is licensed under the [MIT License](./LICENSE).

## Code Guidelines

For AI agents and contributors, please refer to [AGENTS.md](./AGENTS.md) for detailed coding standards and project conventions.
