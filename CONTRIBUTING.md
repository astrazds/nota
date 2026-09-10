# Contribute to Nota

Use small, focused changes with behavior and verification that a reviewer can
follow. Discuss substantial behavior changes in a GitHub issue before you
implement them.

## Prerequisites

Use [mise](https://mise.jdx.dev/) for the Rust version and repository tasks.
`mise.toml` pins the development toolchain. The workspace declares its minimum
Rust version in `Cargo.toml`.

A default native build needs a C toolchain, `pkg-config`, and development
headers for GTK 4.22 or newer, libadwaita 1.9 or newer, Pango 1.56 or newer, and
WebKitGTK 6. Install these through your Linux distribution. Package names and
availability vary by distribution.

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

`mise run dev` starts the GTK app with Preview and Split. It uses the normal
application data directory. Use the [isolated profile procedure](docs/agents/appimage-rehearsal.md#3-launch-with-an-isolated-profile)
when you need a disposable collection.

To compile a release binary:

```sh
mise run build:desktop
```

The binary is `target/release/nota-desktop`. A write-only development build
can omit WebKitGTK:

```sh
mise exec -- cargo run -p nota-desktop --no-default-features --features gui --locked
```

A headless core or desktop-library check does not need GTK:

```sh
mise run test:core
mise exec -- cargo check -p nota-desktop --no-default-features --locked
```

## Verify a change

```sh
mise run verify
mise run doc
```

`verify` runs formatting, Cargo checks, Clippy, workspace tests, and the
AppImage directory contract tests. It does not build an AppImage or run the
ignored GTK widget test.

On a working GTK display, run:

```sh
mise run test:gtk
```

The [CI workflow](.github/workflows/ci.yml) runs core checks separately from
native checks. Its native job uses the gtk4-rs container, installs WebKitGTK,
and runs workspace and GTK tests under Xvfb.

For a GTK UI change, inspect the real app in Light and Dark Themes and at
wide and compact window sizes. Check focus, selection, scroll position, and
keyboard behavior. Source-level visual contracts and browser automation alone
do not prove native rendering.

For an AppImage or migration change:

```sh
mise run package:appimage
```

Complete [the AppImage rehearsal](docs/agents/appimage-rehearsal.md) and record
which automated and human checks passed. Packaging produces
`dist/Nota-x86_64.AppImage` from the current sources.

## Preserve the product and data contracts

Read [the architecture map](docs/architecture.md) before moving behavior
between `nota-core` and `nota-desktop`. Use the domain language in
[CONTEXT.md](CONTEXT.md) and the design rules in [DESIGN.md](DESIGN.md).

Keep Search and the Note List as the primary discovery tools. Preserve stable
Note identities, merge-only Backup import, explicit Storage Recovery, and
legacy format compatibility. Keep Notes local and avoid telemetry, cloud sync,
remote Note storage, or analytics.

Keep generated AppImages, `build/`, `dist/`, `target/`, local agent files, and
editor state out of commits. Preserve font and icon licenses when changing
bundled assets.

## Open a pull request

Explain the user-visible problem, the final change, and the checks you ran.
Separate automated results from manual observations and list missing tools or
unverified behavior. Add a focused behavior test when it protects a real
regression. Keep commits ordered and independently reviewable.

By contributing, you agree that your contribution is licensed under the
repository's MIT license.
