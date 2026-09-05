# Product display name is Nota; keep existing technical identifiers

The product is **Nota**, and the repository is `astrazds/nota` after leaving the Leptos-era `noter-leptos-md` slug. User-facing copy, desktop `Name`, metainfo name, AppImage filename, and documentation use Nota.

Application ID `net.astrazds.Noter`, crate names `noter-core` / `noter-web` / `noter-desktop`, CSS class prefixes, and the `noter-desktop` binary stay as they are. Renaming those now would force another collection-directory migration on top of the existing `$XDG_DATA_HOME/noter` → `net.astrazds.Noter` path, and would churn every in-tree identifier without changing product behaviour.

A later identifier migration can move the application ID to `net.astrazds.Nota` if the display/technical split becomes more costly than a data-path rename.
