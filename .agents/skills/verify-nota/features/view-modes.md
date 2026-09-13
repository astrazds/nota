# Write, Preview, and Split

Read the selected Markdown Note in Preview or beside its Writing Surface.

## Sub-features

- `mode-write` keeps the editable title and body available.
- `mode-preview` renders the title, metadata, and Markdown body.
- `mode-split` shows editor and rendered Preview side by side in a wide window.
- `markdown-help` opens and closes the syntax reference.
- `compact-navigation` switches between Notes and Writing in compact windows.

## How to get to it (user POV)

- Choose Write, Preview, or Split in the editor footer.
- Choose the ? button beside the mode controls for Markdown help.
- In compact windows, use Notes and Writing to switch between the list and editor.

## Driving it with CUA

Preconditions: Select the synthetic `verify` Note from [Writing](writing.md). Use the default build with preview-webkit.

- Write. Click Write from the latest screenshot. Require the editable title and body.
- Preview. Click Preview. Require the Note Title and body to render, plus read-only Tags if present. Capture the entire window.
- Split. In a wide window, click Split. Require both views of the same Note, equally divided editor space, and one footer. Edit the body using key events and require Preview to update.
- Help. Click ?. Require Markdown examples, then click Close or send `Escape`. Require focus to return to the app.
- Compact. Drag the native window's edge using `notaTab.drag([x1,y1], [x2,y2])` from observed bounds. Use Notes and Writing when they appear. Confirm the selected Note and edits survive navigation.
- Proof. Save a screenshot for each mode and any compact state. Inspect title origins, line measure, footer alignment, clipping, and focus. Restore the window size after the check.

## Gotchas

- Split is unavailable at compact widths.
- Broadway does not exercise native GPU rendering. A WebKit or portal failure cannot be reported as a passing Preview.
- Avoid activating external links during this local fixture. Such links invoke the system handler.
