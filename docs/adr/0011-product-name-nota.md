# Product, repository, and identifiers are Nota

The product is **Nota**, and the repository is `astrazds/nota` after leaving the Leptos-era `noter-leptos-md` slug. User-facing copy, desktop `Name`, metainfo, AppImage filename, crates, CSS prefixes, LocalStorage keys, and the `nota-desktop` binary use Nota.

The Linux application ID is `net.astrazds.Nota`. Native collection data lives at `$XDG_DATA_HOME/net.astrazds.Nota`. On first launch, if that directory is absent, the store renames `$XDG_DATA_HOME/net.astrazds.Noter` or a still-older `$XDG_DATA_HOME/noter` directory into the canonical path.

New Backup exports use kind `nota.flat_collection` and filename `nota-backup-YYYY-MM-DD.json`. Import still accepts `noter.flat_collection`. New desktop-transition exports use `nota.desktop_transition`; import still accepts `noter.desktop_transition`. Browser LocalStorage reads `nota-*` keys and falls back to `noter-*` keys.

This supersedes ADR-0009's application ID `net.astrazds.Noter` for new installs while preserving that path as a one-time migration source.
