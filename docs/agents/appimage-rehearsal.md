# AppImage rehearsal

Use this procedure to verify the current AppImage and the permanent migration
contracts. The original manual web-to-desktop transfer gate in ADR-0009 and
ADR-0010 passed before the native source cutover. Repeating this procedure does
not publish a release or retire a hosted browser app.

Agents: follow this file when the user runs `/appimage-rehearsal` or asks to
rehearse the AppImage.

## Done when

- `mise run package:appimage` produced `dist/Nota-x86_64.AppImage` from the
  current checkout.
- `mise exec -- cargo test -p nota-desktop --test web_to_desktop_rehearsal
  --locked` passed.
- A window exists with class `net.astrazds.Nota` and title `Nota`. Its process
  belongs to the AppImage process tree from this run.
- The process uses temporary `HOME`, `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, and
  `XDG_CACHE_HOME` directories.
- A human restored a desktop-transition JSON into the Empty Collection,
  confirmed the Notes, confirmed that a second restore is rejected, and
  confirmed that Merge Import still adds a Note.
- The report names the AppImage path, the profile directory, and each human
  check. The report does not claim a 2.0.0 release or publication.

If the human is not at the machine, stop after the isolated window is open.

## 1. Build

```bash
mise run package:appimage
```

Done when `dist/Nota-x86_64.AppImage` is newer than the Meson install,
`--appimage-extract usr/bin` lists `nota-desktop`, and the desktop file has
`Exec=nota-desktop`.

`package_appimage.py` ignores a pre-existing `dist/*.AppImage` and keeps the
file that linuxdeploy wrote during the current run.

## 2. Verify the automated compatibility path

```bash
mise exec -- cargo test -p nota-desktop --test web_to_desktop_rehearsal --locked
```

The desktop-transition fixture is
`crates/nota-core/tests/fixtures/desktop-transition-v1.json`.

## 3. Launch with an isolated profile

Create a temporary profile. Point `HOME`, `XDG_DATA_HOME`, `XDG_CONFIG_HOME`,
and `XDG_CACHE_HOME` at subdirectories of the profile. Keep `XDG_RUNTIME_DIR`
and `DISPLAY`. Launch `dist/Nota-x86_64.AppImage`.

Done when the window class is `net.astrazds.Nota`, the title is `Nota`, and the
profile has no `collection.json` before restore.

## 4. Check migration and Backup import

Use the committed desktop-transition fixture or a transition export supplied
from a legacy browser build. This source tree no longer produces browser
exports.

1. In the AppImage, restore the desktop-transition JSON into the Empty
   Collection.
2. Confirm the restored Notes. If the file contains Recently Deleted Notes or
   a Theme preference, confirm those values too.
3. Restore the same file again. Nota must reject the second restore without
   changing the collection.
4. Import a Backup that contains a different Note. Merge Import must add the
   Note without clearing the restored collection.

If the human is away, leave the isolated window running and print the profile
path.

## 5. Report

Report the AppImage path and modification time, the test result, the profile
path, the window class, the title, the process ID, and whether
`collection.json` appeared after restore. Report each human check as done or
blocked.
