# Contribute to Nota

Use small, focused changes with behavior and verification that a reviewer can
follow. Discuss substantial behavior changes in a GitHub issue before you
implement them.

## Prerequisites

Use [mise](https://mise.jdx.dev/) for Rust, the Windows .NET SDK, and repository tasks.
`mise.toml` pins the development toolchain. The workspace declares its minimum
Rust version in `Cargo.toml`.

A Linux build needs a C toolchain, `pkg-config`, and development
headers for GTK 4.22 or newer, libadwaita 1.9 or newer, Pango 1.56 or newer, and
WebKitGTK 6. Install these through your Linux distribution. Package names and
availability vary by distribution.

For Windows, follow the [Windows build guide](docs/windows.md). WinUI uses
the pinned .NET SDK and a Rust DLL, without GTK or WebKitGTK. Visual Studio
Build Tools and the Windows SDK remain machine prerequisites.

AppImage packaging also needs Meson, Ninja, Python 3, and network access for
the linuxdeploy downloads. The runtime uses Bubblewrap when available to map
the bundled WebKitGTK helpers into the path expected by WebKitGTK.

## Set up and run

From the repository root:

```sh
mise install
mise run setup:rust
mise run dev
```

`mise run dev` starts GTK on Linux and WinUI on Windows. It uses the normal
application data directory. Use an [isolated profile](#use-an-isolated-profile)
for tests with synthetic Notes.

On Linux, compile a release binary with:

```sh
mise run build:desktop
```

The binary is `target/release/nota-desktop`. A Linux write-only development build
can omit WebKitGTK:

```sh
mise exec -- cargo run -p nota-desktop --no-default-features --features gui --locked
```

A headless core or shared-application check does not need GTK:

```sh
mise run test:core
mise exec -- cargo check -p nota-app --locked
```

On Windows, `mise run build:windows` builds the debug app and shared Rust DLL.
Use `mise run package:windows` for the release folder and ZIP. The complete
commands and prerequisites are in [the Windows guide](docs/windows.md).

## Use an isolated profile

On Linux, build before setting runtime profile variables:

```sh
mise exec -- cargo build -p nota-desktop --locked
nota_profile=$(mktemp -d)
XDG_DATA_HOME="$nota_profile" dbus-run-session -- ./target/debug/nota-desktop
```

The private D-Bus session prevents this launch from activating a running
personal notebook. Notes go into `$nota_profile/net.astrazds.Nota`. Reuse
the same directory and command to check persistence after a normal close.
If you set `CARGO_TARGET_DIR`, use its binary path instead of `./target/debug/`.

On Windows, package the app, then launch a unique test profile from PowerShell:

```powershell
mise run package:windows
$notaProfile = Join-Path ([System.IO.Path]::GetTempPath()) ('nota-verify-' + [guid]::NewGuid())
.\dist\Nota-windows-x64\Nota.Windows.exe --data-dir "$notaProfile"
```

Use synthetic Notes and preserve the profile until you have compared its data
after restart. The [GTK verification procedure](.agents/skills/verify-nota/SKILL.md)
also isolates configuration and cache directories for UI checks. AppImage
runtime checks use [the AppImage rehearsal](docs/agents/appimage-rehearsal.md).

## Verify a change

```sh
mise run verify
mise run doc
```

On Linux, `verify` runs formatting, Cargo checks, Clippy, workspace tests, and the
AppImage directory contract tests. It does not build an AppImage or run the
ignored GTK widget test.

On Windows, `verify` checks the portable Rust crates, builds WinUI, and runs
the C# integration checks against the real Rust DLL. Windows CI also publishes
the self-contained package. Native UI checks are described in the
[Windows guide](docs/windows.md#keep-verification-separate-from-personal-notes).

On Linux with a working GTK display, run:

```sh
mise run test:gtk
```

The [CI workflow](.github/workflows/ci.yml) has `check`, `native`, and `windows`
jobs. The Linux native job uses the gtk4-rs container, installs WebKitGTK,
and runs workspace and GTK tests under Xvfb. The Windows job verifies the
WinUI build and bindings, then uploads the x64 ZIP as a workflow artifact.

`mise run doc` generates API documentation under `target/doc/`. Linux includes
all Rust workspace crates; Windows includes `nota-core`, `nota-app`, and
`nota-ffi`. These generated files are not hand-edited or committed.

For a GTK UI change, inspect the real app in Light and Dark Themes and at
wide and compact window sizes. Check focus, selection, scroll position, and
keyboard behavior. Follow the [native verification procedure](.agents/skills/verify-nota/SKILL.md)
for isolated profiles, screenshots, and persistence proof. Broadway can expose
the real GTK app to a browser driver. Source-level contracts and web mockups
alone do not prove native rendering.

For a Windows UI change, run the real WinUI app with an isolated profile.
Check native keyboard input, file pickers, WebView2 Preview, Light and Dark
Themes, scaling, and close/reopen persistence. Follow [the Windows procedure](docs/windows.md#keep-verification-separate-from-personal-notes).
The GTK Broadway procedure does not verify Windows controls.

For an AppImage or migration change:

```sh
mise run package:appimage
```

Complete [the AppImage rehearsal](docs/agents/appimage-rehearsal.md) and record
which automated and human checks passed. Packaging produces
`dist/Nota-x86_64.AppImage` from the current sources.

## Preserve the product and data contracts

Read [the architecture map](docs/architecture.md) before moving behavior
between `nota-core`, `nota-app`, and the platform frontends. Use the domain language in
[CONTEXT.md](CONTEXT.md) and the design rules in [DESIGN.md](DESIGN.md).

Keep Search and the Note List as the primary discovery tools. Preserve stable
Note identities, merge-only Backup import, explicit Storage Recovery, and
legacy format compatibility. Keep Notes local and avoid telemetry, cloud sync,
remote Note storage, or analytics.

Keep generated AppImages, `build/`, `dist/`, `target/`, local agent settings, and
editor state out of commits. The tracked `.agents/skills/verify-nota/` procedure
is maintained repository documentation. Preserve font and icon licenses when
changing bundled assets.

## Open a pull request

Explain the user-visible problem, the final change, and the checks you ran.
Separate automated results from manual observations and list missing tools or
unverified behavior. Add a focused behavior test when it protects a real
regression. Keep commits ordered and independently reviewable.

By contributing, you agree that your contribution is licensed under the
repository's MIT license.
