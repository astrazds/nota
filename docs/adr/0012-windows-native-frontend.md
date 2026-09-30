# Windows frontend with shared Rust application logic

Status: Accepted.

Nota needs a native Windows application while retaining the Linux GTK
interface and existing collection and backup formats.

## Decision

Use C#/WinUI 3 for Windows. Keep domain behavior in `nota-core` and extract
application state, persistence, recovery, and preview generation into
`nota-app`. Both frontends consume the shared Rust implementation.

Expose the application to C# through `nota-ffi`, an in-process DLL with a
small C ABI. Requests and replies contain UTF-8 JSON. Rust owns session
handles and response allocations. C# uses a SafeHandle and returns buffers
through the matching Rust free function. Only this ABI crate permits unsafe
Rust, with explicit pointer contracts and panic containment.

Serialize session calls on a background queue. Include a note identity and
edit sequence in edits, and do not replace newer editor text with an older
snapshot. The native editor owns selection and undo. Formatting crosses the
boundary as validated UTF-16 offsets and uses the shared Markdown rules.

A successful flush is the close gate. A failed flush keeps pending data and
the session available for retry. Normal editing remains blocked during
Storage Recovery. Recovery preserves corrupt bytes before replacing the
current collection.

Use WebView2 for Windows Preview, retaining the shared Markdown escaping,
deny-by-default content policy, and user-activated external-link policy.
Native widgets, file pickers, and webview hosting remain frontend concerns.

Publish an unsigned, self-contained x64 folder and ZIP first. Keep Rust and
.NET versions and common tasks in `mise.toml`. Installer signing, Microsoft
Store publication, ARM64 packaging, and automatic updates are separate work.

## Alternatives

GTK supports Windows and would retain more widget code, but its existing
WebKitGTK preview is a Linux dependency. WinUI gives Windows its own native
input, controls, and WebView2 integration while preserving shared behavior.

A Rust child process would avoid an unsafe ABI boundary and isolate crashes.
It would also require process supervision, stream framing, backpressure,
and recovery after pipe failure. There is no current product requirement for
that isolation. The DLL has a smaller lifecycle contract.

Replacing both frontends with a webview framework would establish one UI
implementation, but would require rebuilding the Linux interface and
revisiting ADR-0009. This decision retains the native GTK interface.

## Consequences

There are two interfaces to verify, but one implementation of note and
storage rules. Windows builds do not require GTK or WebKitGTK. Linux builds
do not require .NET. Platform behavior tests and live UI checks supplement
shared domain tests; successful compilation alone is not a release gate.

See [the Windows guide](../windows.md) for build and verification commands.
