# Search and organise Notes

Find Notes by title, body, Tags, or pin state without changing their content.

## Sub-features

- `search-entry` opens Notes and focuses its search input, including Ctrl+F.
- `search-query` covers plain words, quoted phrases, title:, tag:, and is:pinned.
- `search-empty-clear` shows **Try a different search term.** and restores the list on clear.
- `tags-edit-filter` edits Tags through metadata or Note actions and filters through a metadata tag.
- `pin` toggles Pin and Unpin from Note actions.
- `tags-cleanup` previews and applies available Tag normalization while editing Tags.

## How to get to it (user POV)

- Open Notes and click **Search notes**, or press Ctrl+F.
- Select a Note (drawer closes), then choose editor **Note actions** > **Edit tags**, or the metadata **+**.
- Click a metadata tag in the selected Note (drawer must be closed first so the editor is sensitive).
- Open a Note row's three-dot Note actions menu for **Pin** / **Unpin**, or the editor pin control / menu for **Pin note** / **Unpin note**.
- While editing Tags, choose **Review Tag cleanup** when it appears.

## Driving it with CUA

Preconditions: Through [Writing](writing.md), create `verify` with body `save proof` and `second` with body `other`.

- Search entry. Screenshot, open Notes and click **Search notes**, then require input focus. Repeat with `await notaTab.pressKey(null, 'ctrl+f')` from the closed drawer. Verify GTK receives the shortcut.
- Query. Focus the search input, send `ctrl+a`, and enter `proof` through key events. Wait at least 200 ms for debounced commit. Require only `verify` and a body-match snippet. Repeat with `title:verify` and a quoted `"save proof"`; use `pressKey(null, 'colon')` and `pressKey(null, 'quotedbl')` for punctuation.
- Empty and clear. Replace the query with `volcano`, require `No notes match search: volcano` and **Try a different search term.**, then use `ctrl+a` and `BackSpace`. Wait >200 ms. Require both Notes again.
- Tags. Select `verify` so the drawer closes. Use the metadata plus or editor Note actions > **Edit tags**, and enter `work` in **Add tags, separated by commas**. Click the title or body to finish editing. Require a metadata tag link. Click it and require the drawer to open with the filtered result and inline `#work ×` chip. Clear that chip and require both Notes again. Repeat via `tag:work` in Search. Close Notes, choose **Pinned**, close Notes again, click a metadata tag, and require **All notes** plus that tag filter.
- Pin. Open Notes; on the `verify` row Note actions choose **Pin** (row label is **Pin**, not **Pin note**). Search `is:pinned`. Require only `verify`. Choose **Unpin** and require it to disappear from that query.
- Cleanup. If **Review Tag cleanup** is visible during tag editing, open **Clean up Tags?**, capture the proposed changes, cancel and confirm unchanged Tags, then repeat and apply when authorized for the disposable fixture.
- Proof. Capture query and results together. Compare stored titles and bodies before and after search. Tag and pin operations must change only their intended metadata and survive restart.

## Gotchas

- A selected Note can stay open while its row is filtered out. Verify results in the drawer; do not treat the editor as a search result.
- The editor (metadata, Edit tags, editor pin) is insensitive while Notes is open. Close the drawer before those paths.
- Clear text search and Tag filters before starting an unrelated recipe.
- If cleanup is unavailable, record that the fixture did not expose that path.
