# AppImage as the first native distribution artifact

ADR-0009 originally packaged native Noter with Meson plus a GNOME 50 Flatpak, and withheld 2.0.0 until Flatpak build/install/run. Flatpak tooling is not the first local gate, and the first shippable artifact needs to be something this repository can actually build and run. Native Noter wraps the existing Meson prefix in an x86_64 AppImage (GTK4 plus bundled WebKitGTK 6 helpers) as the first distribution path. `cargo run -p noter-desktop` stays the development path. The Devel Flatpak manifest remains in tree for later and is not retired. 2.0.0 stays a prerelease until that AppImage runs a clean-profile web-to-desktop rehearsal.

`build-aux/package_appimage.py` is the packager: it copies WebKit helpers into the AppDir, writes a runtime hook for bundled fonts and a `bwrap` overlay of `/usr/lib/webkitgtk-6.0` (this distro's WebKitGTK 6 ignores `WEBKIT_EXEC_PATH`), and invokes linuxdeploy. AppDir contract tests live beside the packager.

This amends the ADR-0009 packaging gate only. Browser hosting retirement, tagging, and publication remain separate approvals.
