# Use Nota

Nota keeps personal Markdown Notes in one local Flat Collection. Search and
Tags help you find Notes without folders or notebooks.

Linux uses GTK and Windows uses WinUI 3. Both use the same Note identities,
Markdown rules, and saved collection formats. Some controls differ.

## Write and find Notes

Select **New note** in the top bar or press `Ctrl+N`. Nota selects the new Note and focuses
its title. Edit the title and Markdown body directly. Formatting buttons wrap
the selected text or insert Markdown at the caret.

Use **Write**, **Preview**, and **Split** in the editor footer. Write edits
plain Markdown in a reading font. Preview renders the Markdown body and hides
the formatting tools. The title remains editable. Split shows a monospace Markdown editor alongside Preview in wide
windows and stacks them in compact windows.
Open **Notes** to find another Note. The list overlays the editor without
moving its reading column. Selecting a Note closes the list.

Press `Ctrl+F` to open Notes and focus Search. Combine ordinary words with quoted phrases,
`title:`, `tag:`, or `is:pinned`. Click a Tag below the Note Title to filter
the Note List. Use **Pinned** to show pinned Notes with the current Search.
Both **Pinned** and **All notes** clear the active Tag filter. Clicking a
Tag below the title clears the Pinned button filter but keeps Search.
A filter can hide the selected Note while it remains open in the editor.
GTK also shows a message explaining why that Note is absent from the list.

Choose **+** beside the Tags or **Edit tags** in the Note actions menu to change Tags.
Windows shows **Add tags** when the Note has no Tags.
On Linux, **Review Tag cleanup** appears during Tag editing when the collection
has Tags that can be normalized. Review the proposed changes before applying
them. On Windows, edit the comma-separated Tags in the dialog. Windows does
not expose the collection-wide Tag cleanup dialog.

Use the pin button or Note actions menu to pin a Note. The Note actions menu
also moves a Note to Recently Deleted and contains additional Markdown commands.
Open **Settings** for Theme and Markdown help.

## Save and recover a deleted Note

Nota saves edits automatically after a short idle period and flushes pending
edits during orderly shutdown. Check the editor's Save Status before you
close the app if it reports a save error. A failed shutdown save keeps the
window open so you can retry. On Windows, `Ctrl+S` also retries the save.

Open a Note's actions menu and choose **Move to recently deleted**. Confirm
with **Move** on Linux or **Move note** on Windows. The confirmation names
the Note. Choose **Cancel** to keep the Note active.

Open **Notes**, then expand **Recently deleted** below the active Note List.
Its header shows the deleted count. Scroll the drawer if the section is
below the visible list.

Choose **Restore** beside a deleted Note to return it to the active collection.
The adjacent **Delete** permanently removes that Note. Linux applies this
action immediately; Windows asks for confirmation. Below the deleted rows,
**Clear All** on Linux or **Clear all** on Windows asks for confirmation
before permanently clearing every deleted Note. Cancel leaves them in place.
The clear action is hidden when the section is empty.

To preserve Notes before permanently removing them, choose **Backup / Export**.
That file includes active Notes and Recently Deleted.

## Export a notebook

1. Open **Notes**, then **Backup** in the drawer footer.
2. Choose **Backup / Export**. On Windows, the item is **Backup / Export…**.
3. Save the JSON file. The suggested name is `nota-backup-YYYY-MM-DD.json`.
4. Keep that file outside the live application data directory if you want a
   separate recovery copy.

**Backup / Export** writes one complete notebook. The file contains active
Notes, Recently Deleted, Theme, and a new Backup Health time. It does not
contain window dimensions, the Preview cache, or edit history. After the file
is written, Nota stores that same Backup Health time locally. The indicator
does not check that the file still exists.

## Import or restore a notebook

1. Open **Notes**, then **Backup**.
2. Choose **Import / Restore**. On Windows, the item is **Import / Restore…**.
3. Select the JSON file and read the confirmation.
4. Choose the named action to apply the file, or choose **Cancel** to leave
   the notebook unchanged.

Nota chooses the confirmation from the file.

- A notes-only Backup always combines notes. New identities are added.
  Matching identities are replaced. A matching Note in Recently Deleted
  returns to the active collection. Other Notes stay. Theme and Backup Health
  stay as they are.
- A complete notebook replaces the notebook only when the active collection
  and Recently Deleted are both empty, and Storage Recovery is not open.
  Restore applies the file's active Notes, Recently Deleted, Theme, and
  Backup Health. If the file has no Backup Health, restore clears the local
  Backup Health.
- A complete notebook opened into a notebook that already has Notes, or
  opened during Storage Recovery, combines only the active Notes from the
  file. Recently Deleted, Theme, and Backup Health in that file are not
  applied.

An older `noter.flat_collection` file and an older `noter.desktop_transition`
file follow the same choices. Keep the original export until you have checked
the notebook and reopened Nota.

On Windows, open a separate empty profile with `--data-dir` when you want an
exact restore without changing the current notebook. See [the Windows guide](windows.md#keep-verification-separate-from-personal-notes).
On Linux, use a separate XDG data directory as described in
[Contributing](../CONTRIBUTING.md#use-an-isolated-profile).

## Restore a browser-era collection

Use a complete notebook JSON exported by a legacy browser build, or import a
notes-only Backup with **Import / Restore**. Follow [Import or restore a notebook](#import-or-restore-a-notebook).
Native Nota cannot read a browser profile's LocalStorage directly.

## Recover unreadable saved data

When Nota cannot parse its current collection, it shows Storage Recovery and
keeps normal editing disabled. Linux offers these actions:

- **Restore Previous Snapshot** restores the last valid active and Recently
  Deleted pair, when a valid snapshot exists.
- **Start Empty** preserves the corrupt payload in a quarantine file and
  starts an empty collection.
- **Import / Restore** opens the same import flow. A complete notebook
  imported here combines its active Notes. It does not replace the notebook.

Windows opens a recovery dialog with **Restore previous**, **Start empty**,
and **Close Nota**. To recover from a file on Windows, first choose
**Start empty**, confirm, then use **Import / Restore**. Both platforms
preserve the unreadable collection in a quarantine file before replacing it
during recovery.

A Previous Snapshot is one recovery copy, not a full edit history.

## Storage and Preview

Linux data lives under `$XDG_DATA_HOME/net.astrazds.Nota`, or
`~/.local/share/net.astrazds.Nota` when `XDG_DATA_HOME` is unset. Windows uses
`%LOCALAPPDATA%\net.astrazds.Nota`, unless you supply `--data-dir`.
On Linux, Nota migrates a predecessor `net.astrazds.Noter` or `noter`
directory only when the canonical directory is absent. A failed migration
is reported instead of silently using the predecessor directory. Windows
opens its selected path directly without this directory migration.

Each open session holds an exclusive `profile.lock`. A second writer cannot
open the same profile. On Linux, an ordinary second launch activates the
existing window through the desktop session. Close Nota before copying or
editing its profile files.

Nota does not encrypt its collection or Backup files. Preview blocks scripts,
remote images, and network resource loads. Activating an allowed web or email
link opens the system handler, whose network and privacy behavior is separate.
See [the privacy policy](../PRIVACY.md) for details.
