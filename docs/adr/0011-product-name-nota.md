# Product display name is Nota; migrate the application ID

The product is **Nota**, and the repository is `astrazds/nota` after leaving the Leptos-era `noter-leptos-md` slug. User-facing copy, desktop `Name`, metainfo, AppImage filename, and documentation use Nota.

The Linux application ID is `net.astrazds.Nota`. Native collection data lives at `$XDG_DATA_HOME/net.astrazds.Nota`. On first launch, if that directory is absent, the store renames `$XDG_DATA_HOME/net.astrazds.Noter` or a still-older `$XDG_DATA_HOME/noter` directory into the canonical path. Backup v1 keeps the `noter.flat_collection` kind so existing Backup files remain importable. Download filenames use `nota-backup-YYYY-MM-DD.json`.

Crate names `noter-core` / `noter-web` / `noter-desktop`, CSS class prefixes, LocalStorage keys, and the `noter-desktop` binary stay as they are. Those are in-tree identifiers, not the XDG data path, and renaming them does not change product behaviour.

This supersedes ADR-0009's application ID `net.astrazds.Noter` for new installs while preserving that path as a one-time migration source.
