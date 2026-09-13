# Capture and save Notes

Create a Note, write its title and Markdown body, and recover the same content after restarting Nota.

## Sub-features

- `capture-button` creates and selects a Note using New Note.
- `capture-empty` creates the first Note using Create a Note.
- `capture-keyboard` creates a Note with Ctrl+N.
- `write-save` saves edits automatically and preserves identity across restart.
- `write-format` applies toolbar formatting and supports undo and redo.

## How to get to it (user POV)

- Choose New Note in the sidebar.
- In an Empty Collection, choose Create a Note.
- Press Ctrl+N.
- Select a Note row to edit it. Formatting tools appear above its body.

## Driving it with CUA

Preconditions: Launch an empty profile and pass Doctor. Use the main skill's key-event recipe.

- Empty entry. Click the visible Create a Note control using `notaTab.click([x, y])`. The count becomes one and the Note Title receives focus.
- Write. Type `verify` with `for (const key of 'verify') await notaTab.pressKey(key)`. Click the body and send `['s','a','v','e','space','p','r','o','o','f']` through `pressKey`. Require the title, body, sidebar snippet, and Saved state to agree.
- Sidebar entry. Click New Note and type `second`. Require count two and focus in its title. Re-select `verify` and require its body unchanged.
- Keyboard entry. With focus inside Nota, call `await notaTab.pressKey('ctrl+n')`. Require one additional Note and title focus. Record a skip if the browser consumes the shortcut.
- Formatting. In a separate synthetic Note, type `word` in the body, then `ctrl+a`. Click B and require `**word**`. Focus the body, send `ctrl+z`, then `ctrl+shift+z`, and verify the text and caret after each. Type a further character and check its exact placement. Use I, S, task-list, and table controls when those commands are in scope.
- Persistence. Re-select `verify`, capture the visible content and collection copy, then perform the main skill's native close and restart sequence. Reopen `verify` and require title `verify`, body `save proof`, and the same stored UUID. Capture the restarted state and collection.

## Gotchas

- Browser text filling can silently do nothing to the GTK canvas. Use key events and inspect the result.
- New Note creates a saved Note immediately. There is no draft Cancel or Save button.
- Autosave is debounced. Observe Saved and the actual collection before claiming persistence.
- On the first Broadway creation, key events may not enter the title until you click it explicitly. Record automatic focus as unverified if that fallback is needed. Later New Note and Ctrl+N paths must be checked independently.
- A browser reload is not an app restart.
- Unicode formatting requires a native input path that actually emits the requested characters. Do not claim the Unicode caret regression from an ASCII check; supplement with `mise run test:gtk` on a supported display.
