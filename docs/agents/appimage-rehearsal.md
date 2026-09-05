# AppImage rehearsal

ADR-0009/0010 gate: an AppImage that runs, plus a **manual** clean-profile web-to-desktop pass. `cargo test -p nota-desktop --test web_to_desktop_rehearsal` is the automated seam; it is not the manual pass.

Agents: follow this file when the user runs `/appimage-rehearsal` or asks to rehearse the AppImage.

## Done when

- `dist/Nota-x86_64.AppImage` mtime is after this run’s meson install.
- `cargo test -p nota-desktop --test web_to_desktop_rehearsal` passed.
- A window exists with class `net.astrazds.Nota` and title `Nota`, pid in the AppImage process tree just launched.
- That process uses a temp `HOME` / `XDG_DATA_HOME` / `XDG_CONFIG_HOME` / `XDG_CACHE_HOME`, not `~/.local/share/net.astrazds.Nota`.
- The human restored a desktop-transition JSON into that Empty Collection, confirmed the Notes, confirmed a second restore is rejected, and confirmed Merge Import still adds.
- The report names AppImage path, profile directory, and which human checks happened. 2.0.0 and publication stay unclaimed.

If the human is not at the machine, stop after the isolated window is up.

## 1. Build

Packaging commands are in `README.md`. After `meson install` into `build/AppDir`:

```bash
python3 build-aux/package_appimage.py package build/AppDir --output dist/Nota-x86_64.AppImage
```

Done when `dist/Nota-x86_64.AppImage` is newer than the install, `--appimage-extract usr/bin` lists `nota-desktop`, and the desktop file `Exec=` is `nota-desktop`.

`package_appimage.py` ignores a pre-existing `dist/*.AppImage` and keeps the file linuxdeploy just wrote. If a launch still execs `noter-desktop`, delete `dist/*.AppImage` and package again.

## 2. Automated seam

```bash
cargo test -p nota-desktop --test web_to_desktop_rehearsal
```

Fixture: `crates/nota-core/tests/fixtures/desktop-transition-v1.json`.

## 3. Isolated launch

Create a temp profile. Point `HOME`, `XDG_DATA_HOME`, `XDG_CONFIG_HOME`, and `XDG_CACHE_HOME` at subdirs of it. Keep `XDG_RUNTIME_DIR` and `DISPLAY`. Launch `dist/Nota-x86_64.AppImage`.

Done when the window class/title match `net.astrazds.Nota` / `Nota` and there is no `collection.json` under that `XDG_DATA_HOME`.

## 4. Human restore

1. Browser Adapter: **Export for desktop**.
2. AppImage: restore that JSON (Empty Collection only).
3. Confirm the Notes (and Recently Deleted / Theme if present).
4. Restore the same file again — it must refuse.
5. **Import Backup** a different Note via Merge Import — it must add, not wipe.

Stop here if the human is away. Leave the isolated window running and print the profile path.

## 5. Report

AppImage path + mtime, test result, profile path, window class/title/pid, whether `collection.json` appeared after restore, and the five human checks (done / blocked).
