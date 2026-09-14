# Recently Deleted visual verification

The native GTK app was exercised on 2026-09-14 with the full-width Recently
Deleted section. The screenshots use synthetic notes created through the UI
in a disposable profile. They show the rebuilt application without external
CSS overrides.

The build passed with:

```sh
mise exec -- cargo build -p nota-desktop --features preview-webkit --locked
```

Before publication, the repository tasks `fmt`, `check`, `clippy`, `test`, and
`test:package` passed through mise. The Rust suites passed 171 tests, with one
display-dependent GTK test ignored by the default suite. The AppDir contract
suite passed all 11 tests.

## Observed results

- The section has a full-width header, a distinct background, and readable
  controls in both light and dark themes.
- Moving `old draft` to Recently Deleted preserved its identity and the body
  `save proof`.
- Clear All displayed the count of one deleted note and focused Cancel.
  Cancellation preserved the note.
- Restore returned the same note and text to the active collection.
- After the native window closed with exit status zero and the app restarted,
  the note reopened with the same text. The collection snapshots were
  byte-identical before and after restart.
- The runtime logs contained no GTK CSS parser warnings.

![Recently Deleted in light theme](assets/verification/recently-deleted-light.jpg)

![Recently Deleted in dark theme](assets/verification/recently-deleted-dark.jpg)

## Coverage limits

The run used GTK Broadway. It does not establish Wayland integration, GPU
rendering, desktop portal behavior, or AppImage packaging. The isolated
runtime logged portal and accessibility service warnings.

This smoke check used one deleted note. The earlier visual prototype covered
the separator between two deleted rows. Permanent Delete, confirmed Clear All,
and cancellation of the initial move were not exercised in this run.
