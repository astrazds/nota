# Use Nota

Nota keeps personal Markdown Notes in one local Flat Collection. Search and
Tags help you find Notes without folders or notebooks.

## Write and find Notes

Select **New Note** or press `Ctrl+N`. Nota selects the new Note and focuses
its title. Edit the title and Markdown body directly. Formatting buttons wrap
the selected text or insert Markdown at the caret.

Use **Write**, **Preview**, and **Split** in the editor footer. Split is
available in wide windows. In a compact window, use the navigation control to
switch between the Note List and the selected Note.

Press `Ctrl+F` to focus Search. Combine ordinary words with quoted phrases,
`title:`, `tag:`, or `is:pinned`. A Tag button in the Note List applies a Tag
filter. Clear the filter to show the full list again. If a filter hides the
selected Note, the editor keeps it open and explains why it is absent from
the list.

Choose **Edit tags** to change a Note's Tags. **Review Tag cleanup** appears
when the collection has Tags that can be normalized. Review the proposed
changes before applying them. Use the Note actions menu to pin a Note or move
it to Recently Deleted.

## Save and recover a deleted Note

Nota saves edits automatically after a short idle period and flushes pending
edits during orderly shutdown. Check the editor's Save Status before you
close the app if it reports a save error.

To undo a deletion, choose **Restore** beside the Note in Recently Deleted.
**Delete** permanently removes that Note immediately, without another
confirmation. **Clear All** asks for confirmation before it permanently clears
Recently Deleted.

## Export and import a Backup

1. Choose **Export** in the sidebar footer and save the JSON file.
2. Keep that file outside the live application data directory if you want a
   separate recovery copy.
3. To import it, choose **Import**, select the file, and review the add and
   replace counts.
4. Confirm the Merge Import to apply it.

A Backup contains active Notes, including their identities and Tags. It does
not contain Recently Deleted, preferences, or Backup Health. Merge Import
adds new identities and replaces matching ones. If a matching Note is in
Recently Deleted, import restores it to the active collection. Other Notes
remain in place. The most recent successful export updates Backup Health;
that indicator does not verify that the exported file still exists.

## Restore a browser-era collection

Use a desktop-transition JSON exported by a legacy browser build, or use a
normal Backup with **Import**. This native-only source tree cannot create a
browser export or read a browser profile's LocalStorage.

Choose **Restore** in the sidebar Backup controls to select a desktop-transition
file. The native active collection and Recently Deleted must both be empty.
Restore transfers active Notes, Recently Deleted, Theme, and optional Backup
Health. A restore into a non-empty collection is rejected without mutation.
Use Merge Import to add Notes to an existing collection.

Keep the original export until you have checked the restored collection and
reopened the native app. The file formats still accept legacy `noter.*`
identifiers.

## Recover unreadable saved data

When Nota cannot parse its current collection, it shows Storage Recovery and
keeps normal editing disabled. Choose one of the available actions:

- **Restore previous snapshot** restores the last valid active and Recently
  Deleted pair, when a valid snapshot exists.
- **Start empty** preserves the corrupt payload in a quarantine file and
  starts an empty collection.
- **Import Backup** opens the normal preview and merge flow for a Backup you
  choose.

A Previous Snapshot is one recovery copy, not a full edit history. A Backup
contains active Notes only. These recovery paths preserve different state.

## Storage and Preview

Data lives under `$XDG_DATA_HOME/net.astrazds.Nota`, or
`~/.local/share/net.astrazds.Nota` when `XDG_DATA_HOME` is unset. Nota migrates a
predecessor `net.astrazds.Noter` or `noter` directory only when the canonical
directory is absent. A failed migration is reported instead of silently using
the predecessor directory.

Nota does not encrypt its collection or Backup files. Preview blocks scripts,
remote images, and network resource loads. Activating an allowed web or email
link opens the system handler, whose network and privacy behavior is separate.
See [the privacy policy](../PRIVACY.md) for details.
