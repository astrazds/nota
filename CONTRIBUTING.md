# Contributing to Nota

Thanks for taking an interest in Nota. Small, focused changes are easiest
to review.

## Before opening a pull request

1. Open an issue for behavior changes or substantial new work so the scope can
   be agreed first.
2. Preserve Nota's local-first boundary. Changes must not add telemetry, cloud
   sync, remote note storage, or analytics.
3. Keep the product a Markdown Note App. Search and the Note List remain the
   discovery system; do not introduce folders, notebooks, or a command palette
   as primary navigation.
4. Do not commit generated AppImages, `dist/`, `target/`, browser reports, local
   agent files, or editor state.
5. Run the complete local check:

   ```sh
   mise install
   mise run setup:rust
   mise run setup:browser
   mise run verify
   ```

   Native GTK work also needs GTK 4.22, libadwaita 1.9, and, for
   Preview/Split, WebKitGTK 6. GitHub Actions runs that native job in the
   [gtk4-rs GTK 4 container](https://relm4.org/book/stable/continuous_integration.html)
   rather than Ubuntu LTS packages. Browser visual and workflow contracts
   use the Trunk version pinned in `mise.toml`.

Read [the architecture map](docs/architecture.md) before moving behavior
between the core, browser, and native modules. `mise.toml` owns tool versions
and common tasks. `mise tasks` lists focused checks and development commands.

## Pull requests

Explain the problem, the chosen approach, and how you verified it. Add or
update a behavior-focused test for domain changes. By contributing, you agree
that your contribution is licensed under the repository's MIT license.
