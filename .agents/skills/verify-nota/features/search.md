# Search and organise Notes

Find Notes by title, body, Tags, or pin state without changing their content.

## Sub-features

- `search-entry` focuses Search through the sidebar command, input, and Ctrl+F.
- `search-query` covers plain words, quoted phrases, title:, tag:, and is:pinned.
- `search-empty-clear` shows no matches and restores the list on clear.
- `tags-edit-filter` edits Tags and filters through a Note List Tag button.
- `pin` toggles Pin and Unpin from Note actions.
- `tags-cleanup` previews and applies available Tag normalization.

## How to get to it (user POV)

- Click Search, click its input, or press Ctrl+F.
- Select a Note and choose Edit tags.
- Click a Tag in a Note List row.
- Open a Note row's three-dot Note actions menu for Pin or Unpin.
- Choose Review Tag cleanup when it appears.

## Driving it with CUA

Preconditions: Through [Writing](writing.md), create `verify` with body `save proof` and `second` with body `other`.

- Search entry. Screenshot, click Search, then require input focus and the Search Hint. Repeat by clicking the input and by `await notaTab.pressKey('ctrl+f')`. Verify GTK receives the shortcut.
- Query. Focus the search input, send `ctrl+a`, and enter `proof` through key events. Require only `verify` and a body-match snippet. Repeat with `title:verify` and a quoted `"save proof"`; use `pressKey('colon')` and `pressKey('quotedbl')` for punctuation.
- Empty and clear. Replace the query with `volcano`, require zero matches, then use `ctrl+a` and `BackSpace`. Require both Notes again.
- Tags. Select `verify`, click Edit tags, and enter `work` in the visible Tags field. Send `Return`. Require a Tag pill. Click its Note List Tag button and require the filtered result. Click the filter chip to clear it. Repeat via `tag:work` in Search.
- Pin. Open Note actions on `verify`, choose Pin, then search `is:pinned`. Require only `verify`. Choose Unpin and require it to disappear from that query.
- Cleanup. If Review Tag cleanup is visible, open it, capture the proposed changes, cancel and confirm unchanged Tags, then repeat and apply when authorized for the disposable fixture.
- Proof. Capture query and results together. Compare stored titles and bodies before and after search. Tag and pin operations must change only their intended metadata and survive restart.

## Gotchas

- A selected Note can stay open while its row is filtered out. Require the explanatory message; do not treat the editor as a search result.
- Clear text search and Tag filters before starting an unrelated recipe.
- If cleanup is unavailable, record that the fixture did not expose that path.
