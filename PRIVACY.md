# Privacy

Effective date: 2026-09-10

Nota is a local-first Markdown note app. The native Linux app stores Notes on
the device. This policy also covers legacy browser builds.

## Data collection

Nota does not collect, sell, transmit, or remotely store personal information,
note content, titles, Tags, Backup files, analytics events, or crash reports.

There is no account, no backend, and no sync service.

## Where Notes live

The native app writes a versioned `collection.json` under
`$XDG_DATA_HOME/net.astrazds.Nota` (typically
`~/.local/share/net.astrazds.Nota`). Preferences, Backup Health, a previous
valid snapshot, and any corrupt-payload quarantine files share that directory.
These are local JSON files. Nota does not encrypt them or exported Backups.

Legacy browser builds store Notes in LocalStorage under `nota-*` keys in that
browser profile, with a fallback read of legacy `noter-*` keys.

## Backup and export

A Backup is a JSON file the user chooses to export. Merge Import reads a file
the user chose. A desktop-transition file is a local migration format for
moving a browser-era collection into the native app. Native Nota keeps this
import compatible after the source cutover.

Nota does not upload those files. If you copy, email, or otherwise share an
exported Backup, that sharing is controlled by you and by the destination you
choose.

## Network

The native app does not require network access to create, edit, search,
preview, delete, restore, or back up Notes. A hosted browser build uses the
network to load its static app files. Note operations remain in the browser,
and neither app loads remote fonts, remote scripts, or analytics pixels into
the Note workflow.

Native Preview blocks scripts, remote images, and network resource loads.
When you activate an allowed HTTP, HTTPS, or email link, Nota passes it to the
system handler. The browser or email application then handles that link and
may make network requests.

Building and packaging Nota can download dependencies and linuxdeploy tools.
These development operations are separate from the app's local Note workflow.

## Contact

For privacy or support questions, use the repository issue tracker:

[GitHub issues](https://github.com/astrazds/nota/issues).

Use synthetic examples in public reports. Keep Note content, Backups, and
quarantined payloads private. Report suspected vulnerabilities through the
[security reporting process](SECURITY.md#reporting-a-vulnerability).
