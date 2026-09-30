# Run Nota on Windows

Nota's Windows frontend uses WinUI 3 and the same Rust note model, Markdown
rules, storage formats, and backup compatibility as the Linux app. The initial
Windows package targets x64. ARM64 packages are not provided yet.
The recorded live verification used Windows 11 x64. Windows 10 and clean
machines without development tools have not been verified; see
[the verification limits](windows-verification.md#limits).

## Run the packaged app

Extract `Nota-windows-x64.zip` to a writable folder and run `Nota.Windows.exe`.
Keep the extracted files together. The package includes the .NET and Windows
App SDK runtimes and the Rust library.

Build the ZIP with the steps below, or download the `Nota-windows-x64`
artifact from a successful [CI run](https://github.com/astrazds/nota/actions/workflows/ci.yml).
Extract the workflow artifact, then extract the app ZIP inside it. Workflow
artifacts are development builds, not signed releases.

Preview requires Microsoft's WebView2 Runtime. It is normally present on
Windows 11. If Nota reports that it is missing, install the
[Evergreen WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
and reopen Nota. Note editing and storage do not require an internet connection.

This development package is unsigned. It is not a Microsoft Store release or
an installer, and it does not register file associations or an automatic updater.

## Build from source

Install [mise](https://mise.jdx.dev/). Install Visual Studio 2022 Build Tools
with the **Desktop development with C++** workload, including the MSVC x64
tools and Windows SDK. These are native system prerequisites. The repository
pins Rust and .NET through `mise.toml`.

From PowerShell in the repository root:

```powershell
mise install
mise run setup:rust
mise run dev
```

To verify the Rust application, compile WinUI, and exercise the C# bindings
against the real Rust library:

```powershell
mise run verify
```

To publish a self-contained folder and ZIP:

```powershell
mise run package:windows
```

The outputs are `dist/Nota-windows-x64/` and `dist/Nota-windows-x64.zip`.
NuGet package lockfiles and `Cargo.lock` pin application dependencies.
Update them through the package managers when changing dependencies.

## Keep verification separate from personal Notes

By default, Nota stores its collection under
`%LOCALAPPDATA%\net.astrazds.Nota`. To test with an isolated collection, supply
an absolute directory:

```powershell
.\dist\Nota-windows-x64\Nota.Windows.exe --data-dir C:\Temp\nota-verification
```

Use synthetic notes. Check title and body editing, formatting and undo,
Unicode text, Search, Tags, pinning, Recently Deleted, Backup, and all view
modes. Close the app normally and reopen the same profile to prove saving.
Check Light and Dark Themes, keyboard focus, and display scaling in the real
Windows app. Binding tests alone do not prove native rendering or input.

Only one session can write a profile at a time. If Nota reports that the
profile is already open, return to its existing window or use a different
`--data-dir`. Do not delete `profile.lock` to bypass the running session.

To verify Storage Recovery, close the isolated app first. Preserve its
`collection.json`, then replace that test file with malformed JSON. Reopen
Nota and verify that normal editing is blocked until recovery is resolved.
Restoring a previous snapshot must recover its Notes. Starting empty must
preserve the corrupt bytes in a quarantine file.

## Move Notes between Linux and Windows

Use **Export** on the source machine and **Import** on the destination.
On Windows, select **Export notes backup…** from the Export menu. Review the
merge counts before confirming. Existing Backup v1 files and
legacy `noter.*` format identifiers remain supported. A desktop-transition
file restores an entire collection only into an empty destination.

Nota does not synchronize machines. A normal Backup contains active Notes.
On Windows, **Export complete notebook state…** creates a desktop-transition
file with active Notes, Recently Deleted, Theme, and optional Backup Health.
Use **Restore** on either platform to load it into an empty notebook. Linux's
Export button creates only a notes Backup. Window dimensions and Preview cache
are not part of either export. See [the restore procedure](usage.md#restore-a-complete-notebook).
