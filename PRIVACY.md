# Privacy

Effective date: 2026-09-05

Nota is a local-first Markdown note app. The native Linux app stores Notes on
the device. The in-tree browser Adapter is a migration source and also stays
on the device.

## Data collection

Nota does not collect, sell, transmit, or remotely store personal information,
note content, titles, Tags, Backup files, analytics events, or crash reports.

There is no account, no backend, and no sync service.

## Where Notes live

The native app writes a versioned `collection.json` under
`$XDG_DATA_HOME/net.astrazds.Nota` (typically
`~/.local/share/net.astrazds.Nota`). Preferences, Backup Health, a previous
valid snapshot, and any corrupt-payload quarantine files share that directory.

The browser Adapter stores Notes in LocalStorage under `nota-*` keys on the
same machine, with a fallback read of legacy `noter-*` keys.

## Backup and export

A Backup is a JSON file the user chooses to export. Merge Import reads a file
the user chose. Desktop-transition export is the same kind of local file for
moving a collection onto the native app.

Nota does not upload those files. If you copy, email, or otherwise share an
exported Backup, that sharing is controlled by you and by the destination you
choose.

## Network

The native app and the browser Adapter do not require network access to create,
edit, search, preview, delete, restore, or back up Notes. They do not load
remote fonts, remote scripts, or analytics pixels.

## Contact

For privacy or support questions, use the repository issue tracker:

https://github.com/astrazds/nota/issues
