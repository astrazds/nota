#!/usr/bin/env python3
"""Package a Meson DESTDIR tree as a Nota AppImage AppDir."""

from __future__ import annotations

import argparse
import os
import shutil
import stat
import subprocess
import sys
import urllib.request
from pathlib import Path

APPLICATION_ID = "net.astrazds.Nota"
BINARY_NAME = "nota-desktop"
DESKTOP_FILE = f"usr/share/applications/{APPLICATION_ID}.desktop"
ICON_FILE = f"usr/share/icons/hicolor/scalable/apps/{APPLICATION_ID}.svg"
METAINFO_FILE = f"usr/share/metainfo/{APPLICATION_ID}.metainfo.xml"
FONT_DIR = f"usr/share/{APPLICATION_ID}/fonts"
WEBKIT_LIBDIR = "usr/lib/webkitgtk-6.0"
HOOK_FILE = "apprun-hooks/nota-runtime.sh"
HOST_WEBKIT_LIBDIR = Path("/usr/lib/webkitgtk-6.0")

FONT_FILES = (
    "source-sans-3-latin-wght-normal.woff2",
    "source-sans-3-latin-wght-italic.woff2",
    "source-code-pro-latin-wght-normal.woff2",
    "source-code-pro-latin-wght-italic.woff2",
)
WEBKIT_HELPERS = (
    "WebKitWebProcess",
    "WebKitNetworkProcess",
    "WebKitGPUProcess",
)

REQUIRED_RELATIVE_PATHS = (
    f"usr/bin/{BINARY_NAME}",
    DESKTOP_FILE,
    ICON_FILE,
    METAINFO_FILE,
    *(f"{FONT_DIR}/{name}" for name in FONT_FILES),
    *(f"{WEBKIT_LIBDIR}/{name}" for name in WEBKIT_HELPERS),
    HOOK_FILE,
)

LINUXDEPLOY = "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
LINUXDEPLOY_GTK = "https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh"
LINUXDEPLOY_APPIMAGE = "https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-x86_64.AppImage"

RUNTIME_HOOK = """\
# Nota AppImage runtime: bundled fonts, WebKit helpers, and Wayland.
APPDIR="${APPDIR:-"$(dirname "$(readlink -f "$0")")"}"
unset GDK_BACKEND
export NOTA_FONT_DIR="$APPDIR/usr/share/net.astrazds.Nota/fonts"
export NOTER_FONT_DIR="$NOTA_FONT_DIR"
export WEBKIT_EXEC_PATH="$APPDIR/usr/lib/webkitgtk-6.0"
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
# Arch/Omarchy WebKitGTK 6 ignores WEBKIT_EXEC_PATH. Overlay bundled helpers once.
if [ -z "${NOTA_WEBKIT_OVERLAY:-}" ] && command -v bwrap >/dev/null 2>&1; then
  if [ -x "$APPDIR/usr/lib/webkitgtk-6.0/WebKitWebProcess" ]; then
    export NOTA_WEBKIT_OVERLAY=1
    exec bwrap --bind / / --dev-bind /dev /dev --proc /proc \\
      --bind "$APPDIR/usr/lib/webkitgtk-6.0" /usr/lib/webkitgtk-6.0 \\
      "$APPDIR/AppRun" "$@"
  fi
fi
"""


class AppDirError(Exception):
    """The AppDir is missing required Nota or WebKit files."""


def verify_appdir(root: Path) -> None:
    missing = [
        relative
        for relative in REQUIRED_RELATIVE_PATHS
        if not (root / relative).is_file()
    ]
    problems: list[str] = []
    if missing:
        problems.append("missing " + ", ".join(missing))

    desktop = root / DESKTOP_FILE
    if desktop.is_file() and not _desktop_launches_by_name(desktop.read_text()):
        problems.append("desktop file must use Exec=nota-desktop")

    hook = root / HOOK_FILE
    if hook.is_file():
        text = hook.read_text()
        for token in ("NOTA_FONT_DIR", "WEBKIT_EXEC_PATH"):
            if token not in text:
                problems.append(f"runtime hook must set {token}")

    if problems:
        raise AppDirError("; ".join(problems))


def prepare_appdir(root: Path, webkit_libdir: Path = HOST_WEBKIT_LIBDIR) -> None:
    """Copy WebKit helpers and write the AppRun hook into a Meson DESTDIR tree."""
    helper_dir = root / WEBKIT_LIBDIR
    helper_dir.mkdir(parents=True, exist_ok=True)
    for name in WEBKIT_HELPERS:
        source = webkit_libdir / name
        if not source.is_file():
            raise AppDirError(f"WebKit helper not found: {source}")
        target = helper_dir / name
        shutil.copy2(source, target)
        target.chmod(target.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    hook = root / HOOK_FILE
    hook.parent.mkdir(parents=True, exist_ok=True)
    hook.write_text(RUNTIME_HOOK)
    hook.chmod(hook.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    verify_appdir(root)


APPRUN_SCRIPT = """\
#!/bin/sh
set -eu
SELF="$(readlink -f "$0")"
APPDIR="$(dirname "$SELF")"
export APPDIR
if [ -d "$APPDIR/apprun-hooks" ]; then
  for hook in "$APPDIR"/apprun-hooks/*; do
    if [ -f "$hook" ]; then
      # shellcheck disable=SC1090
      . "$hook"
    fi
  done
fi
BIN="$APPDIR/usr/bin/nota-desktop"
WEBKIT="$APPDIR/usr/lib/webkitgtk-6.0"
if command -v bwrap >/dev/null 2>&1 && [ -x "$WEBKIT/WebKitWebProcess" ]; then
  exec bwrap --bind / / --dev-bind /dev /dev --proc /proc \\
    --bind "$WEBKIT" /usr/lib/webkitgtk-6.0 \\
    "$BIN" "$@"
fi
exec "$BIN" "$@"
"""


def write_custom_apprun(root: Path) -> None:
    apprun = root / "AppRun"
    apprun.write_text(APPRUN_SCRIPT)
    apprun.chmod(apprun.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def package_appimage(
    appdir: Path,
    output: Path,
    tools_dir: Path,
    webkit_libdir: Path = HOST_WEBKIT_LIBDIR,
) -> Path:
    appdir = appdir.resolve()
    output = output.resolve()
    tools_dir = tools_dir.resolve()
    webkit_libdir = webkit_libdir.resolve()
    prepare_appdir(appdir, webkit_libdir)
    linuxdeploy = _ensure_linuxdeploy(tools_dir)
    output.parent.mkdir(parents=True, exist_ok=True)
    previous_appimages = {
        path: path.stat().st_mtime for path in output.parent.glob("*.AppImage")
    }
    env = os.environ.copy()
    env["LINUXDEPLOY"] = str(linuxdeploy)
    env["DEPLOY_GTK_VERSION"] = "4"
    env["NO_STRIP"] = "1"
    env["APPIMAGE_EXTRACT_AND_RUN"] = "1"
    env["PATH"] = f"{tools_dir}:{env.get('PATH', '')}"
    command = [
        str(linuxdeploy),
        "--appdir",
        str(appdir),
        "--executable",
        str(appdir / "usr/bin" / BINARY_NAME),
        "--desktop-file",
        str(appdir / DESKTOP_FILE),
        "--icon-file",
        str(appdir / ICON_FILE),
        "--plugin",
        "gtk",
        "--output",
        "appimage",
    ]
    for helper in WEBKIT_HELPERS:
        command.extend(["--executable", str(appdir / WEBKIT_LIBDIR / helper)])
    subprocess.run(command, check=True, cwd=output.parent, env=env)
    produced = _find_produced_appimage(output.parent, previous_appimages)
    output.parent.mkdir(parents=True, exist_ok=True)
    if produced.resolve() != output.resolve():
        shutil.move(str(produced), str(output))
    return output


def _desktop_launches_by_name(text: str) -> bool:
    for line in text.splitlines():
        if line.startswith("Exec="):
            command = line.split("=", 1)[1].split()
            return bool(command) and command[0] == BINARY_NAME
    return False


def _ensure_linuxdeploy(tools_dir: Path) -> Path:
    tools_dir.mkdir(parents=True, exist_ok=True)
    linuxdeploy = _download(tools_dir / "linuxdeploy-x86_64.AppImage", LINUXDEPLOY)
    _download(tools_dir / "linuxdeploy-plugin-gtk.sh", LINUXDEPLOY_GTK)
    _download(
        tools_dir / "linuxdeploy-plugin-appimage-x86_64.AppImage",
        LINUXDEPLOY_APPIMAGE,
    )
    return linuxdeploy


def _download(destination: Path, url: str) -> Path:
    if destination.is_file() and destination.stat().st_size > 0:
        destination.chmod(destination.stat().st_mode | stat.S_IXUSR)
        return destination
    print(f"downloading {url}", file=sys.stderr)
    with urllib.request.urlopen(url) as response:
        destination.write_bytes(response.read())
    destination.chmod(destination.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return destination


def _find_produced_appimage(
    directory: Path, previous: dict[Path, float] | None = None
) -> Path:
    previous = previous or {}
    matches = [
        path
        for path in directory.glob("*.AppImage")
        if path not in previous or path.stat().st_mtime > previous[path]
    ]
    if not matches:
        raise AppDirError(f"linuxdeploy did not produce an AppImage in {directory}")
    return max(matches, key=lambda path: path.stat().st_mtime)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    verify = sub.add_parser("verify-appdir", help="check an AppDir against the Nota contract")
    verify.add_argument("appdir", type=Path)

    prepare = sub.add_parser("prepare-appdir", help="add WebKit helpers and the runtime hook")
    prepare.add_argument("appdir", type=Path)
    prepare.add_argument("--webkit-libdir", type=Path, default=HOST_WEBKIT_LIBDIR)

    package = sub.add_parser("package", help="wrap a Meson DESTDIR AppDir with linuxdeploy")
    package.add_argument("appdir", type=Path)
    package.add_argument(
        "--output",
        type=Path,
        default=Path("dist/Nota-x86_64.AppImage"),
    )
    package.add_argument(
        "--tools-dir",
        type=Path,
        default=Path(__file__).resolve().parent / ".tool-cache",
    )
    package.add_argument("--webkit-libdir", type=Path, default=HOST_WEBKIT_LIBDIR)

    args = parser.parse_args(argv)
    try:
        if args.command == "verify-appdir":
            verify_appdir(args.appdir)
        elif args.command == "prepare-appdir":
            prepare_appdir(args.appdir, args.webkit_libdir)
        elif args.command == "package":
            package_appimage(args.appdir, args.output, args.tools_dir, args.webkit_libdir)
            print(args.output)
    except AppDirError as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
