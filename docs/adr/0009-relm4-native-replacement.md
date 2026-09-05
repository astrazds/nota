# Relm4 native replacement after 1.0

Noter will replace its browser application with a Linux-first Relm4/GTK4 desktop application after the 1.0 line. This supersedes ADR-0008 for the post-1.0 product while preserving ADR-0008 as the accepted browser-release decision and migration source. The browser release remains available only as a temporary migration Adapter until native parity and data transfer are verified.

Accepted decisions:

- Use a temporary Cargo workspace with `noter-core`, `noter-web`, and `noter-desktop`. Keep toolkit-independent Note, Flat Collection, Search, Tags, Backup, Storage Recovery, View Mode, Markdown parsing, and formatting behavior in `noter-core`.
- Continue ADR-0006's deep-Module and vertical-slice discipline. `noter-desktop` has one Relm4 root `AppModel`/`AppMsg` Component; stable Note UUIDs cross widget and persistence boundaries.
- Target Linux for the first native release with application ID `net.astrazds.Noter`. Do not couple shared domain Modules to Linux APIs unnecessarily.
- Keep native persistence concrete. Store a versioned `collection.json` under the XDG application data directory `$XDG_DATA_HOME/net.astrazds.Noter` (typically `~/.local/share/net.astrazds.Noter`). Preserve the last valid state as `collection.previous.json`, and keep Theme/window preferences and Backup Health in separate files. If that canonical directory is absent and a legacy `$XDG_DATA_HOME/noter` directory exists, rename it into the application-id path on first launch.
- Validate and serialize the whole active/Recently Deleted pair before mutation. Write and sync a same-directory temporary file, atomically rename it, and sync the directory. A revision-aware worker coalesces the 300 ms debounce and flushes the latest revision during orderly shutdown.
- Treat corrupt current state as Storage Recovery rather than silently resetting. Offer the valid Previous Snapshot when present and preserve corrupt bytes in a timestamped quarantine file when the user explicitly starts empty.
- Preserve `noter.flat_collection` Backup v1 and merge-only import indefinitely. Add `noter.desktop_transition` v1 for an exact one-time restore of active Notes, Recently Deleted, Theme, and Backup Health.
- Allow desktop-transition restore only into a first-run or Empty Collection. Reject a non-empty native collection without mutation and direct the user to normal merge Backup import.
- The final browser release exposes an explicit Export for desktop action. Transition import remains supported after the browser frontend is retired.
- Build the native interface from GTK4 plus bundled CSS/resources instead of adopting libadwaita's visual conventions. Preserve the Local-First Note Identity, Pane Rhythm, accessible names, keyboard paths, focus restoration, and minimum target sizing. About, Markdown help, Delete Confirmation, Clear All, and Backup Import Preview are paper-neutral product windows, not stock `GtkAlertDialog`. The Note List updates existing row widgets in place when row identity is unchanged so selecting a Note does not steal sidebar focus or scroll. Split divides the editor area equally.
- Convert GTK character offsets to validated UTF-8 byte ranges at the toolkit boundary. Shared formatting Interfaces use byte ranges and contain no browser UTF-16 concepts.
- Render Preview in a non-editable WebKitGTK 6 `WebView`. Disable JavaScript and persistent storage, load only generated HTML, apply a deny-by-default CSP, allow same-document footnote anchors, prevent in-view external navigation, and pass only validated HTTP(S)/mailto targets to the system handler after user activation.
- Package with a thin Meson installation layer. Keep `cargo run -p noter-desktop` as the development path. The first shippable native artifact is an AppImage wrapping that Meson prefix (ADR-0010). The GNOME 50 Devel Flatpak manifest stays in tree for later.
- Do not call the migration complete or release 2.0.0 until parity, native smoke, metadata validation, an AppImage that runs, and a manual clean-profile web-to-desktop rehearsal pass. Until then, native crates use a 2.0.0 prerelease version. Flatpak build/install/run is no longer the first packaging gate.

Consequences:

- The repository temporarily carries two frontends, but only `noter-web` owns browser storage, DOM selection conversion, and the transition download Adapter.
- `noter-core` becomes the durable behavior boundary rather than a generic storage or UI abstraction layer.
- Native collection files live under the application id, not a short `noter` directory name. Alpha installs that already wrote `$XDG_DATA_HOME/noter` are renamed on first launch when the canonical path is absent.
- WebKitGTK 6 development headers are a build prerequisite for Preview. AppImage packaging downloads linuxdeploy tools on demand. Flatpak tooling remains optional until that later gate.
- Browser hosting retirement, publication, tagging, branch creation, and release actions remain separate approvals.
