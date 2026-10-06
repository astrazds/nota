# Nota domain context

Nota is a local-first Markdown note app for writing, finding, and organising
personal Notes. Linux and Windows share the domain model and storage formats.
The [user guide](docs/usage.md) describes current controls and platform
differences. [DESIGN.md](DESIGN.md) defines the visual system.

## Note vocabulary

| Term | Meaning |
| --- | --- |
| Note | A personal Markdown document with a stable UUID, a separate title, Tags, a pin state, and timestamps. Use Note in product copy rather than document or file. |
| Note Title | The Note's explicit name. It is not derived from the first Markdown heading. |
| Markdown Note App | A note-taking product where Markdown supports writing. It is not a developer editor or Markdown workbench. |
| Flat Collection | One active collection discovered through Search, the Note List, and Tags. There are no folders or separate notebooks. |
| Empty Collection | A collection with no active Notes. The editor offers New note. A complete notebook restore also requires Recently Deleted to be empty. |
| Quick Capture | Create a Note, select it, close the Notes drawer, switch to Write, and focus the Note Title. |
| Note Actions | Explicit controls for pinning, editing Tags, and moving a Note to Recently Deleted. They remain available without hover. |
| Delete Confirmation | A confirmation that names the Note before moving it to Recently Deleted. |
| Recently Deleted | Recoverable Notes outside the active collection. Expand the section in the Notes drawer to restore Notes or delete them permanently. |
| Clear All | Permanently delete every Note in Recently Deleted after confirmation. The action is below the expanded deleted rows. |
| Tag | A lightweight label used for secondary organisation and filtering. It is not a folder or a primary navigation item. |
| Note Metadata | Tags and the modification date shown below the Note Title. |

## Workspace vocabulary

| Term | Meaning |
| --- | --- |
| Focus | The current layout. One Note occupies the main workspace, and navigation opens in an overlay drawer. |
| Local-First Note Identity | Nota's visual identity, with paper colours, warm accents, a serif reading face, and native controls. |
| Notes Drawer | The on-demand navigation panel containing Search, filters, the Note List, Recently Deleted, Backup, and About Nota. |
| Writing Surface | The editable Markdown body in Write or Split. |
| Preview | The rendered Markdown body below the shared native Note Title and metadata header. Preview does not duplicate that header inside the webview. |
| View Mode | Write, Preview, or Split. A nonempty collection opens in Preview. Quick Capture opens Write. |
| View Mode Controls | Write, Preview, and Split controls in the editor footer. Markdown help lives in Settings. |
| Pane Rhythm | The shared alignment, spacing, and reading width across view modes. Split has its own compact body typography. |
| Formatting Tools | Controls between the Note header and Markdown body, visible while the Writing Surface is available. |
| Save Status | The editor footer's Saved locally, Saving, or Save failed indicator. |
| Global Notification | Transient operation feedback separate from the editor's Save Status. Linux shows an overlay; Windows uses an InfoBar below the top bar. |
| Theme | A separately tuned treatment of surfaces, text, borders, selection, and accents. Light and Dark are not colour inversions. |
| Responsive Navigation | The Notes drawer adapts to window width. It overlays the editor on larger windows and occupies the full width on compact windows. |
| Product Metadata | Version or build information for support, outside the main writing workflow. |
| Diagnostics Surface | A secondary support view for product and storage details. The available details differ by frontend. |

## Discovery vocabulary

| Term | Meaning |
| --- | --- |
| Search | The primary way to find Notes by title, content, Tags, or pin state. It is not a command palette. |
| Note List | Rows that help users recognise and select Notes. Each row includes a title, preview text, and modification date. |
| Discovery Depth | Improvements that explain Search results within the Flat Collection, before introducing a new organisation model. |
| Match Snippet | A compact body excerpt around matched plain or quoted Search terms when the title or Tags do not already explain the match. |
| Search Hint | Syntax help associated with Search. GTK shows a focus-time popover; Windows provides a tooltip. |

Search, the Note List, and lightweight Tags remain the primary discovery
model. All notes and Pinned filters live above the list. A selected Tag adds
a removable filter. Creating a Note clears Search and filters so the new Note
is discoverable.

The shared core computes match snippets and highlight ranges. GTK renders
highlighted matches; Windows currently shows plain result text. Do not infer
identical frontend presentation from shared Search behavior. See the
[user guide](docs/usage.md#write-and-find-notes) for supported syntax.

## Storage vocabulary

| Term | Meaning |
| --- | --- |
| Native Collection | Active Notes and Recently Deleted stored together in one validated, versioned `collection.json`. Preferences and Backup Health are separate. |
| Backup | A versioned local export of active Notes. It is distinct from sync and from a complete notebook export. |
| Backup Health | Metadata about the last successful notes Backup export. It describes a local recovery point, not cloud status. |
| Backup Controls | The Backup menu in the Notes drawer footer. Both frontends offer notes Backup export and import, complete notebook export, and complete notebook restore. |
| Backup Import Preview | Validation and a count of Notes to add or replace, shown before Merge Import. |
| Merge Import | Add missing identities and replace matching identities without clearing unrelated Notes. A matching Recently Deleted Note returns to the active collection. |
| Desktop Transition | A versioned complete notebook file containing active Notes, Recently Deleted, Theme, and optional Backup Health. Restore requires both destination collections to be empty. |
| Storage Recovery | The startup state when the saved collection cannot be read or validated. Editing stays blocked until an explicit recovery action succeeds. |
| Previous Snapshot | The last valid active and Recently Deleted collection pair retained before the next save. It is one recovery point, not version history. |
| Corrupt Payload Quarantine | The unreadable saved bytes preserved before an explicit recovery action replaces the current collection. It is diagnostic evidence, not a Backup. |

## Interaction contracts

The Notes drawer starts closed at every window width. The top bar provides
Notes, New note, the Nota wordmark, and Settings. Selecting a Note closes the
drawer. Opening the drawer blocks editor input, and dismissing it restores
the previous focus when that control is still available.

The Note Title and metadata stay above the body in Write, Preview, and Split.
The title remains a native editable control in Preview. Tags link to filters,
and the adjacent add control opens Tag editing. A leading top-level Markdown heading
that repeats the Note Title is suppressed in Preview to avoid a duplicate
heading.

Write and Preview use a centered reading area. Split divides the available
body area into equal columns on larger windows. At 560 logical pixels or
less, the editor appears above Preview. Both panes share the native Note
header and one editor footer. The footer contains the word count, View Mode
Controls, and Save Status.

Formatting Tools appear only in Write or Split. Settings contains Theme
controls and Markdown help. Backup and About Nota remain secondary utilities
in the Notes drawer footer. Recently Deleted is a collapsed section below
the active Note List in the same scroll area.

Selection must preserve Note identity and keep the chosen row in view.
Editing a Note must not reset its native selection or undo history when a
save response arrives. Destructive confirmations identify their target or
impact and put default focus on Cancel.

## Data contracts

Notes stay local. The app has no accounts, telemetry, cloud sync, or remote
Note store. Exported files are user-managed recovery and transfer tools.

A notes Backup includes active Notes only. Merge Import requires an import
preview and preserves unrelated Notes. Complete notebook export includes
Recently Deleted, Theme, and optional Backup Health. It excludes window
geometry, Preview cache, and the profile lock.

Storage Recovery never silently resets unreadable data. Restoring a Previous
Snapshot recovers both collections. Starting empty preserves the corrupt
payload in quarantine. Linux also offers Backup import within recovery.
Windows requires recovery to finish before Import opens; Start empty provides
a path when the user has only a notes Backup.

The product name is Nota, the repository is `astrazds/nota`, the application
ID is `net.astrazds.Nota`, and Rust crates use the `nota-` prefix. Legacy
`noter.flat_collection` Backup files and `noter.desktop_transition` files
remain accepted. Browser LocalStorage belongs to the historical migration
source, not the current runtime.

See [the architecture guide](docs/architecture.md) for code ownership and
[the ADR index](docs/adr/README.md) for decisions and their original context.
