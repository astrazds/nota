---
name: verify-nota
description: Verify Nota's native GTK desktop UI after behavior or visual changes. Use for isolated launch, keyboard and mouse workflows, persistence proof, and screenshots. Route AppImage checks to appimage-rehearsal.
---

# Verify Nota

Drive the real GTK app through its Broadway display backend and the Codex CUA browser tool. This checkout has no browser frontend or product CLI. Cargo tests supplement UI proof.

For packaging or migration checks, also use [the AppImage rehearsal](../../../docs/agents/appimage-rehearsal.md). Broadway does not prove Wayland integration, GPU rendering, desktop portals, or the packaged runtime.

## Launch

Run from the repository root. Use `mise.toml` and the prerequisites in `CONTRIBUTING.md`. Require `gtk4-broadwayd`, `dbus-run-session`, `curl`, `ss`, and a CUA-controlled browser. Report missing prerequisites explicitly.

Build with the same features as `mise run dev`, then launch the binary directly to track its PID:

```bash
mise exec -- cargo build -p nota-desktop --features preview-webkit --locked
```

Keep one Bash PTY alive using `exec_command` with `tty=true`, then `write_stdin`. If the sandbox blocks sockets, use normal execution escalation for this isolated session. A socket denial is an environment failure.

```bash
umask 077
mkdir -p .scratch
nota_run=$(mktemp -d "$PWD/.scratch/verify-nota-XXXXXXXX")
nota_profile=$(mktemp -d /tmp/nota-verify-XXXXXXXX)
mkdir -p "$nota_profile"/{home,data,config,cache,runtime}
export nota_run nota_profile
nota_binary="$PWD/target/debug/nota-desktop"
nota_port=18085
nota_display=:85
printf '%s\n' "$nota_run" "$nota_profile"
git rev-parse HEAD >"$nota_run/revision.txt"
git status --short >"$nota_run/worktree.txt"
sha256sum "$nota_binary" >"$nota_run/binary.sha256"
```

Require `ss -ltnp "sport = :$nota_port"` to show no listener. If occupied, choose another port and update the browser URL. Runs have private profiles, runtime directories, and D-Bus sessions. Never attach to an existing listener or drive one run from two tabs.

```bash
XDG_RUNTIME_DIR="$nota_profile/runtime" gtk4-broadwayd \
  --address=127.0.0.1 --port="$nota_port" "$nota_display" \
  >"$nota_run/broadway.log" 2>&1 &
nota_broadway_pid=$!
```

Require that PID to remain alive and `curl --fail --silent --max-time 3 "http://127.0.0.1:$nota_port/"` to return the Broadway page. Then start Nota:

```bash
env HOME="$nota_profile/home" XDG_DATA_HOME="$nota_profile/data" \
  XDG_CONFIG_HOME="$nota_profile/config" XDG_CACHE_HOME="$nota_profile/cache" \
  XDG_RUNTIME_DIR="$nota_profile/runtime" GDK_BACKEND=broadway \
  BROADWAY_DISPLAY="$nota_display" dbus-run-session -- \
  sh -c 'echo $$ > "$nota_run/app.pid"; exec "$1"' sh "$nota_binary" \
  >"$nota_run/app.log" 2>&1 &
nota_session_pid=$!
```

Apply profile overrides only to the runtime command, after mise builds. Outer `HOME` or `XDG_DATA_HOME` overrides can redirect mise's tool installation.

In `mcp__cua_repl.js`, follow the tool's first-call rules, then retain this tab:

```javascript
var notaTab = await cua.createBrowserTab('chrome', 'http://127.0.0.1:18085', { sessionName: '📝 Nota verification' });
```

In the next tool call, inspect the window:

```javascript
await notaTab.getScreenshot();
```

Ready means the native window shows `Nota`, `New Note`, and an Empty Collection with count zero. The browser title is `broadway 2.0`. Capture this state before mutation. On failure, inspect logs and run Cleanup before retrying.

## Doctor

Run this read-only check before driving and whenever the window stops responding. Require success and confirm the listener PID matches `nota_broadway_pid`:

```bash
nota_app_pid=$(cat "$nota_run/app.pid")
{
  kill -0 "$nota_broadway_pid" && kill -0 "$nota_app_pid" &&
  test "$(readlink -f "/proc/$nota_app_pid/exe")" = "$nota_binary" &&
  sha256sum --check "$nota_run/binary.sha256" &&
  tr '\0' '\n' <"/proc/$nota_app_pid/environ" | rg -F -x "XDG_DATA_HOME=$nota_profile/data" &&
  ss -ltnp "sport = :$nota_port" &&
  curl --fail --silent --max-time 3 "http://127.0.0.1:$nota_port/" >/dev/null
} >"$nota_run/doctor.txt" 2>&1
cat "$nota_run/doctor.txt"
```

Also inspect `app.log` and the screenshot. HTTP success alone does not prove GTK is alive. Broadway lacks OpenGL and may log portal or AT-SPI failures. Record those limits; a blank Preview or broken file picker remains unverified.

## Drive

Broadway paints GTK controls into a canvas. Its AX tree may contain only `AXWebArea`; GTK labels are not DOM selectors. Prefer accessible handles when exposed. Otherwise take a fresh screenshot and use `notaTab.click([x, y])` at the visible control. Derive coordinates from that screenshot, never from an earlier run.

Use `pressKey` for GTK text. Browser `typeText`, `paste`, and form filling may leave the canvas unchanged. For the synthetic title `verify`:

```javascript
for (const key of ['v', 'e', 'r', 'i', 'f', 'y']) await notaTab.pressKey(key);
await notaTab.getAXState();
await notaTab.getScreenshot();
```

Use `space`, `Return`, `BackSpace`, `ctrl+a`, and `ctrl+z` for key events. Click the intended field before `ctrl+a`. After each action batch, inspect AX state and the screenshot before selecting the next target. A frame can lag key events. Wait for the expected visible value with bounded observations before asserting or typing again. After 60 seconds without progress, run Doctor.

Follow every entry point for the requested feature or report the precise skip. Browser-reserved shortcuts can prevent GTK receiving a key. Passing a mouse path does not verify its shortcut.

## Evidence

Keep proof in the printed `nota_run` directory. Set `notaEvidence` in CUA to that exact path. Give each capture a distinct name. This CUA backend returns JPEG bytes; use the matching extension and confirm the format with `file` before handing over evidence:

```javascript
var notaFs = await import('node:fs/promises');
await notaFs.writeFile(notaEvidence + '/state.jpg', await notaTab.getScreenshot());
await notaFs.writeFile(notaEvidence + '/state.ax.txt', await notaTab.getAXState({ disableDiffing: true }));
```

Write `actions.md` with feature IDs, entry points, actual coordinates or key calls, expected and observed results, artifact names, skips, and warnings. Capture the state before actions and the resulting state. Inspect screenshots for content, focus, clipping, and spacing.

Exercise real controls, not internal setters, direct JSON writes, or test-only endpoints. For persistence, observe `Saved`, copy the synthetic collection, close the native window using its title-bar button, require exit zero, and relaunch with the same profile. Browser reload does not restart Nota.

```bash
cp "$nota_profile/data/net.astrazds.Nota/collection.json" "$nota_run/collection.before-restart.json"
```

After UI close, `wait "$nota_session_pid"` must succeed. Preserve the first log with `cp "$nota_run/app.log" "$nota_run/app.before-restart.log"`. Repeat only the Nota launch block, keep Broadway running, run Doctor, and reopen the Note. Save `collection.after-restart.json` and a screenshot of its title and body. Check identities and contents, not just file existence.

Backups need exported file contents and merge results alongside UI evidence. Use synthetic state only. Mocks belong only at existing production boundaries. If a recipe adds a dry-run mode, observe its file and network effects instead of trusting the flag.

## Cleanup

Close the native window through the UI. Wait for `nota_session_pid` and record its exit status. Close only this tab with `await notaTab.close()`.

If a failed attempt leaves Nota alive, confirm the recorded app PID still identifies the binary in `/proc` before sending `TERM` to that PID. Wait for the recorded D-Bus wrapper. Forced termination does not prove shutdown flush. Never kill by process name or signal stale PIDs.

```bash
kill "$nota_broadway_pid"
wait "$nota_broadway_pid"
ss -ltnp "sport = :$nota_port"
```

Broadway termination may return status 143. Require that its listener and the recorded app process are gone. Copy remaining synthetic data needed for proof, then remove only this run's profile:

```bash
test -d "$nota_profile" && rm -rf -- "$nota_profile"
test ! -e "$nota_profile"
ls -l "$nota_run"
test -s "$nota_run/actions.md"
```

Keep `nota_run` with screenshots, collection copies, logs, and receipts. Run Cleanup after failed iterations too. Report surviving evidence and any unverified behavior.

## Helpers

No helper scripts ship with this skill. Use the shell blocks and CUA calls directly. Keep the feature map current with `/maintain-verification-skill`.
