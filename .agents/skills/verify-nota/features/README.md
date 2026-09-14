# Nota verification map

Read this index before driving a feature. This is a starter map of the native GTK app. Use the launch, Doctor, evidence, and cleanup procedure in [verify-nota](../SKILL.md).

## Preconditions and conventions

- Start with an empty disposable profile and the current build. Create synthetic Notes through the UI.
- Use one Broadway tab per instance. Choose an unused loopback port for another run.
- CUA coordinates come from the latest screenshot. The rendered GTK names below are visual handles, not browser ARIA selectors.
- Enter lowercase ASCII fixtures with `for (const key of 'verify') await notaTab.pressKey(key)`. Translate spaces to `space` and newlines to `Return`.
- After each batch, call `getAXState()` and inspect a screenshot. Record actual actions and resulting state.
- For a feature-level claim, cover every listed entry point and relevant sub-feature. Report skips individually. One passing recipe does not establish whole-app coverage.

## Features

| Feature | Recipe | Observable proof |
| --- | --- | --- |
| Capture and save | [Writing](writing.md) | Title and body survive native restart with the same identity |
| Find and organise | [Search and tags](search.md) | Queries and tag filters explain results; pin state persists |
| Read Markdown | [View modes](view-modes.md) | Write, Preview, and Split show the same Note |
| Recover deletion | [Recently Deleted](deletion.md) | Named confirmation, distinct recovery section in both themes, and restoration |
| Keep recovery copies | [Backup and recovery](backup.md) | Export contents, previewed merge, and explicit recovery decisions |

## Coverage limits

This initial map names but does not prove each path. Theme and About diagnostics are additional visible controls. Check them when related changes touch appearance or diagnostics. AppImage packaging and full migration acceptance belong to [the AppImage rehearsal](../../../../docs/agents/appimage-rehearsal.md).

Current behavior and labels come from `docs/usage.md`, `CONTEXT.md`, and `crates/nota-desktop/src/ui/`. Update recipes against the live app when those controls change.
