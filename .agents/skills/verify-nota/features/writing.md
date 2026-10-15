# Capture and save Notes

Create a Note, write its title and Markdown body, and recover the same content after restarting Nota.

## Sub-features

- `capture-button` creates and selects a Note using the top-bar New note control.
- `capture-empty` creates the first Note using the empty-editor **New note** button.
- `capture-keyboard` creates a Note with Ctrl+N.
- `write-save` saves edits automatically and preserves identity across restart.
- `write-format` applies toolbar formatting and supports undo and redo.

## How to get to it (user POV)

- Choose the top-bar plus (tooltip **New note (Ctrl+N)**).
- In an Empty Collection, choose **New note** in the empty editor (heading **A quiet place for your notes**). The Notes drawer empty state is only **No notes yet.** — it has no create control.
- Press Ctrl+N.
- Select a Note row to open it. Formatting tools appear above its body only in **Write** or **Split** (existing Notes reopen in Preview).

## Driving it with CUA

Preconditions: Launch an empty profile and pass Doctor. Use the main skill's key-event recipe.

- Empty entry. Click the centered **New note** control using `notaTab.click([x, y])` (or the browser driver's screenshot-coordinate click). The count becomes one, Write mode activates, and the title Entry (placeholder **Untitled note**) receives focus.
- Write. Type `verify` with `for (const key of 'verify') await notaTab.pressKey(null, key)`. Click the body and send `['s','a','v','e','space','p','r','o','o','f']` through `pressKey(null, key)`. Require the title, body, sidebar snippet, and footer **Saved locally** to agree.
- Top-bar entry. Click the plus (`New note (Ctrl+N)`) and type `second`. Require title focus. Open Notes and require count two. Re-select `verify` and require its body unchanged.
- Keyboard entry. With focus inside Nota, call `await notaTab.pressKey(null, 'ctrl+n')`. Require one additional Note and title focus. Record a skip if the browser consumes the shortcut.
- Formatting. Switch to Write if needed. In a separate synthetic Note, type `word` in the body, then `ctrl+a`. Click B and require `**word**`. Focus the body, send `ctrl+z`, then `ctrl+shift+z`, and verify the text and caret after each. Type a further character and check its exact placement. Check I, Heading, Bullet list, Task list, and Link in the toolbar. Strikethrough and Insert table are in Note actions.
- Persistence. Re-select `verify`, capture the visible content and collection copy, then perform the main skill's native close and restart sequence. Reopen `verify` and require title `verify`, body `save proof`, and the same stored UUID. Capture the restarted state and collection.

## Gotchas

- Browser text filling can silently do nothing to the rendered GTK app. Use key events and inspect the result. Send keys serially; parallel key tool calls scramble order. Caps Lock in the driving browser uppercases GTK input — toggle it off before fixtures.
- New Note immediately creates and selects a Note, then autosaves it. There is no draft Cancel or Save button.
- Autosave is debounced. Observe **Saved locally** and the actual collection before claiming persistence.
- On the first Broadway creation, key events may not enter the title until you click it explicitly. Record automatic focus as unverified if that fallback is needed. Later New Note and Ctrl+N paths must be checked independently.
- A browser reload is not an app restart.
- Unicode formatting requires a native input path that actually emits the requested characters. Do not claim the Unicode caret regression from an ASCII check; supplement with `mise run test:gtk` on a supported display.
