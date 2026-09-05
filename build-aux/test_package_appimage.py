#!/usr/bin/env python3
"""AppDir contract tests for the native AppImage packager."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from package_appimage import (
    AppDirError,
    prepare_appdir,
    verify_appdir,
    write_custom_apprun,
)


REQUIRED_RELATIVE_PATHS = (
    "usr/bin/noter-desktop",
    "usr/share/applications/net.astrazds.Noter.desktop",
    "usr/share/icons/hicolor/scalable/apps/net.astrazds.Noter.svg",
    "usr/share/metainfo/net.astrazds.Noter.metainfo.xml",
    "usr/share/net.astrazds.Noter/fonts/source-sans-3-latin-wght-normal.woff2",
    "usr/share/net.astrazds.Noter/fonts/source-sans-3-latin-wght-italic.woff2",
    "usr/share/net.astrazds.Noter/fonts/source-code-pro-latin-wght-normal.woff2",
    "usr/share/net.astrazds.Noter/fonts/source-code-pro-latin-wght-italic.woff2",
    "usr/lib/webkitgtk-6.0/WebKitWebProcess",
    "usr/lib/webkitgtk-6.0/WebKitNetworkProcess",
    "usr/lib/webkitgtk-6.0/WebKitGPUProcess",
    "apprun-hooks/noter-runtime.sh",
)


def write_file(root: Path, relative: str, contents: str = "x") -> None:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(contents)


def complete_appdir(root: Path) -> None:
    for relative in REQUIRED_RELATIVE_PATHS:
        if relative.endswith(".desktop"):
            write_file(
                root,
                relative,
                "\n".join(
                    [
                        "[Desktop Entry]",
                        "Name=Nota",
                        "Exec=noter-desktop",
                        "Icon=net.astrazds.Noter",
                        "Type=Application",
                    ]
                ),
            )
        elif relative.endswith("noter-runtime.sh"):
            write_file(
                root,
                relative,
                "\n".join(
                    [
                        "export NOTER_FONT_DIR=\"$APPDIR/usr/share/net.astrazds.Noter/fonts\"",
                        "export WEBKIT_EXEC_PATH=\"$APPDIR/usr/lib/webkitgtk-6.0\"",
                        "export WEBKIT_DISABLE_DMABUF_RENDERER=1",
                    ]
                ),
            )
        else:
            write_file(root, relative)


class VerifyAppdirTests(unittest.TestCase):
    def test_empty_appdir_is_rejected_with_the_installed_layout_paths(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            with self.assertRaises(AppDirError) as raised:
                verify_appdir(root)
            message = str(raised.exception)
            self.assertIn("usr/bin/noter-desktop", message)
            self.assertIn("net.astrazds.Noter.desktop", message)
            self.assertIn("source-sans-3-latin-wght-normal.woff2", message)
            self.assertIn("WebKitWebProcess", message)
            self.assertIn("noter-runtime.sh", message)

    def test_complete_appdir_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            complete_appdir(root)
            verify_appdir(root)

    def test_desktop_file_must_launch_noter_desktop_by_name(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            complete_appdir(root)
            write_file(
                root,
                "usr/share/applications/net.astrazds.Noter.desktop",
                "\n".join(
                    [
                        "[Desktop Entry]",
                        "Name=Nota",
                        "Exec=/usr/bin/noter-desktop",
                        "Icon=net.astrazds.Noter",
                        "Type=Application",
                    ]
                ),
            )
            with self.assertRaises(AppDirError) as raised:
                verify_appdir(root)
            self.assertIn("Exec=noter-desktop", str(raised.exception))

    def test_runtime_hook_must_point_webkit_and_fonts_inside_the_appdir(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            complete_appdir(root)
            write_file(root, "apprun-hooks/noter-runtime.sh", "export PATH=/usr/bin\n")
            with self.assertRaises(AppDirError) as raised:
                verify_appdir(root)
            message = str(raised.exception)
            self.assertIn("NOTER_FONT_DIR", message)
            self.assertIn("WEBKIT_EXEC_PATH", message)


class PrepareAppdirTests(unittest.TestCase):
    def test_prepare_copies_webkit_helpers_and_writes_the_runtime_hook(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw) / "AppDir"
            webkit = Path(raw) / "webkit"
            webkit.mkdir()
            for helper in (
                "WebKitWebProcess",
                "WebKitNetworkProcess",
                "WebKitGPUProcess",
            ):
                write_file(webkit, helper, "helper")
            meson_paths = [
                path
                for path in REQUIRED_RELATIVE_PATHS
                if not path.startswith("usr/lib/webkitgtk-6.0/")
                and path != "apprun-hooks/noter-runtime.sh"
            ]
            for relative in meson_paths:
                if relative.endswith(".desktop"):
                    write_file(
                        root,
                        relative,
                        "\n".join(
                            [
                                "[Desktop Entry]",
                                "Name=Nota",
                                "Exec=noter-desktop",
                                "Icon=net.astrazds.Noter",
                                "Type=Application",
                            ]
                        ),
                    )
                else:
                    write_file(root, relative)
            prepare_appdir(root, webkit)
            verify_appdir(root)
            hook = (root / "apprun-hooks/noter-runtime.sh").read_text()
            self.assertIn("NOTER_FONT_DIR", hook)
            self.assertIn("WEBKIT_EXEC_PATH", hook)
            self.assertIn("unset GDK_BACKEND", hook)
            self.assertIn("bwrap", hook)
            self.assertIn("/usr/lib/webkitgtk-6.0", hook)

    def test_prepare_rejects_a_webkit_libdir_without_helpers(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            with self.assertRaises(AppDirError) as raised:
                prepare_appdir(Path(raw) / "AppDir", Path(raw) / "missing")
            self.assertIn("WebKit helper not found", str(raised.exception))


class CustomApprunTests(unittest.TestCase):
    def test_apprun_sources_hooks_and_overlays_bundled_webkit_helpers(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            complete_appdir(root)
            write_custom_apprun(root)
            text = (root / "AppRun").read_text()
            self.assertTrue((root / "AppRun").stat().st_mode & 0o100)
            self.assertIn("apprun-hooks", text)
            self.assertIn("bwrap", text)
            self.assertIn("/usr/lib/webkitgtk-6.0", text)
            self.assertIn("usr/bin/noter-desktop", text)


if __name__ == "__main__":
    unittest.main()


