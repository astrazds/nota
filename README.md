<p align="center">
  <img src="assets/icons/nota-192.png" width="112" height="112" alt="Nota folded-note icon">
</p>

<h1 align="center">Nota</h1>

<p align="center">
  A local-first Markdown note app for Linux.
</p>

<p align="center">
  <a href="https://github.com/astrazds/nota/actions/workflows/ci.yml"><img alt="CI status" src="https://github.com/astrazds/nota/actions/workflows/ci.yml/badge.svg?branch=main"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

Capture a Note, write Markdown, and find it again through Search and Tags.
Nota keeps one Flat Collection on your device, with Recently Deleted for
recovery and local Backup files you control. It has no accounts, telemetry,
backend, or sync service.

<p align="center">
  <img src="docs/assets/readme/nota-main-window.png" alt="Nota native window showing the Note List, a selected Note, and the Writing Surface">
</p>

The Relm4/GTK4 app is the only frontend in this source tree. The current version
is `2.0.0-alpha.1`. Native source migration is complete; a stable release and
retirement of any hosted browser build are separate steps.

## Build and run

Install [mise](https://mise.jdx.dev/) and the native dependencies listed in
[CONTRIBUTING.md](CONTRIBUTING.md#prerequisites), then run:

```sh
git clone https://github.com/astrazds/nota.git
cd nota
mise install
mise run dev
```

Preview and Split are included by default. The project pins Rust through
`mise.toml`; Cargo declares Rust 1.95 as the minimum version.

To build the x86_64 AppImage with the additional packaging prerequisites:

```sh
mise run package:appimage
./dist/Nota-x86_64.AppImage
```

The packager downloads linuxdeploy tools and bundles the native app, fonts,
and WebKitGTK helpers. See [the AppImage rehearsal](docs/agents/appimage-rehearsal.md)
for a clean-profile check.

## Use Nota

- Create a Note with **New Note** or `Ctrl+N`.
- Write Markdown and use **Write**, **Preview**, or **Split** in the editor
  footer. Split is available in wide windows.
- Focus Search with `Ctrl+F`. Search supports words, quoted phrases, `title:`,
  `tag:`, and `is:pinned`.
- Use Note actions to pin or delete a Note. Deleted Notes remain in
  **Recently Deleted** until you restore or permanently remove them.
- Use **Export** for a Backup and **Import** for a Merge Import. Review the
  add and replace counts before applying an import.

[The user guide](docs/usage.md) covers Tags, saving, Backup, recovery, and
migration from browser-era exports. Remote images and active content are
blocked in Preview. Links you activate open through the system handler.

## Data and compatibility

The app stores data under `$XDG_DATA_HOME/net.astrazds.Nota`, normally
`~/.local/share/net.astrazds.Nota`. Notes and Backup files are not encrypted by
Nota. See [PRIVACY.md](PRIVACY.md) for storage and network details, and
[SECURITY.md](SECURITY.md) to report a vulnerability.

Backup v1 and desktop-transition v1 remain supported, including legacy
`noter.*` format identifiers. A native build cannot read another browser's
LocalStorage directly. Use a previously exported Backup or desktop-transition
file to move that collection into Nota.

## Contribute

```sh
mise run setup:rust
mise run verify
```

The GTK widget regression requires a display and runs separately with
`mise run test:gtk`. [CONTRIBUTING.md](CONTRIBUTING.md) describes the toolchain,
feature checks, packaging, and evidence expected in a pull request.

[Documentation](docs/README.md) links the architecture, domain, design, and
decision records. Nota is licensed under [MIT](LICENSE).
