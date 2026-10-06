# Nota design system

Nota uses a compact Note List, a clear Writing Surface, warm functional accents,
and paper-neutral dialogs. Linux uses GTK and Windows uses WinUI 3. Preview is
an embedded, read-only view of the same Note, using WebKitGTK on Linux and
WebView2 on Windows.

## Sources of truth

Keep design intent here and exact implementation values in their owners:

| Concern | Owner |
| --- | --- |
| GTK palette, controls, focus, and Light and Dark Themes | [nota.css](crates/nota-desktop/resources/nota.css) |
| GTK window composition and widget behavior | [workspace.rs](crates/nota-desktop/src/ui/workspace.rs) |
| GTK layout dimensions | [visual_contract.rs](crates/nota-desktop/src/visual_contract.rs) |
| Centered GTK reading column | [writing_plane.rs](crates/nota-desktop/src/ui/writing_plane.rs) and [workspace.rs](crates/nota-desktop/src/ui/workspace.rs) |
| Preview HTML, typography, and content policy | [preview.rs](crates/nota-app/src/preview.rs) |
| Windows composition and native input | [MainWindow.xaml](windows/Nota.Windows/MainWindow.xaml) and [MainWindow.xaml.cs](windows/Nota.Windows/MainWindow.xaml.cs) |
| Windows palette and control styles | [App.xaml](windows/Nota.Windows/App.xaml) |
| GTK product dialogs | [dialogs.rs](crates/nota-desktop/src/ui/dialogs.rs) |
| Windows product dialogs | [MainWindow.Dialogs.cs](windows/Nota.Windows/MainWindow.Dialogs.cs) |
| Bundled fonts | [fonts.rs](crates/nota-desktop/src/fonts.rs) and [assets/fonts](assets/fonts) |

Browser-era Tailwind utilities and copied token tables are not native styling
APIs. Change the source owner and inspect its rendered result.

## Color

Use Warm Capture Yellow for primary actions. Use the softer signal color for
selection, active View Mode controls, and focus. Light Theme uses warm paper
surfaces; Dark Theme has its own palette. Preserve visible borders and readable
text in each theme rather than deriving one by inversion.

The `.nota-root` and `.nota-root.nota-dark` variables in `nota.css` define the
GTK palette. Windows defines theme brushes in `App.xaml` and applies the
native titlebar colors in `MainWindow.xaml.cs`. Use the existing roles before
introducing another color.
Reserve red for destructive actions and errors, and green for success feedback.
Selected Notes must also have a visible border or other non-color cue.

## Typography and reading measure

Use bundled Gelasio for the Note Title, Write body, and Preview prose. Use
Source Sans 3 for app controls and Preview headings. Split uses Source Code
Pro for the Markdown editor; Preview code uses the same monospace family.
Write remains a plain Markdown editor even though it uses a reading font.

Both frontends load bundled TrueType fonts for native controls. The shared
Preview document embeds WOFF2 fonts without network requests. Preserve the
upstream font licenses in both distribution formats. See the
[font sources](assets/fonts/SOURCES.md) for the exact files and origins.

Center the reading column while keeping its text left-aligned. The GTK
`WritingPlane` and Windows XAML cap the Write and Preview body measure at
600 logical pixels, with surrounding insets owned by the native layout.
GTK does not support CSS `max-width`, so `WritingPlane` enforces the widget
allocation. Avoid adding the outer inset again inside Preview HTML.
Split expands the body area across the available window width. The Note
Title, metadata, and formatting tools keep their centered column.

Use size and weight to establish hierarchy. Keep utility labels secondary to
the Note and avoid decorative letter spacing or a monospace product identity.

## Layout and controls

The **Notes** control opens an overlay drawer without shifting the reading
column. The drawer covers the full content width in compact windows. The
editor is unavailable until the drawer closes. Selecting a Note closes it.
Split divides the body area equally in wide windows and stacks the editor
above Preview in compact windows.

Keep View Mode controls in one stable editor footer. Split uses one footer
across both panes. The sidebar footer holds Backup controls. Exact dimensions
belong in GTK's `NATIVE_VISUAL_CONTRACT` and the Windows XAML and layout code,
not in duplicate prose token tables.

Formatting controls sit between Note Metadata and the Markdown body. They
appear only when writing is available. Preserve native selection and undo history
when a formatting action changes text.

Tags appear as links below the Note Title. Selecting a Tag opens Notes with
that filter. The drawer shows the active Tag filter beside **All notes** and
**Pinned**. GTK also exposes each Note's Tags in its row actions menu.
Keep existing row widgets when identities are unchanged so updates preserve
focus and scroll position. GTK edits comma-separated Tags inline and offers
suggestions and collection-wide cleanup. Windows edits Tags in a dialog.

GTK's Search Hint appears temporarily below Search. Global Notifications provide
short-lived save, Backup, import, and error feedback without adding permanent
header chrome. Windows keeps error notifications visible until dismissed.
Storage Recovery is an explicit app state with named actions. GTK shows it
inside the workspace; Windows uses a dialog while the workspace is disabled.

### Recently Deleted

Keep Recently Deleted below the active Note List, separated by space and a
top border. Its header shows the deleted count and expands the recovery
rows. Both frontends start with the section collapsed.

Place **Restore** and **Delete** beside each deleted Note. Put **Clear All**
below the rows on GTK and **Clear all** below them on Windows. Restore is
neutral; destructive actions use the theme's red. Preserve these distinctions
in both themes.

The header remains visible when Recently Deleted is empty. The clear action
appears only when it contains Notes. The section remains part of the drawer's
scroll content rather than a separate navigation destination. Exact values belong
to the `.nota-deleted-*` rules in `nota.css` for GTK and to `MainWindow.xaml`
and `ReconcileDeleted` in `MainWindow.xaml.cs` for Windows.

## Dialogs and accessibility

GTK uses paper product windows for About, Markdown help, deletion, Clear All,
and Backup Import Preview. Windows uses themed WinUI `ContentDialog` controls.
Keep a clear title, readable body, and compact action row. Use GTK accessible
roles and names and WinUI automation names. Cancel is the default focus for destructive
confirmations. Escape and closing a dialog must return the user to the app.

Use visible native focus states. GTK-owned titlebar nodes need styling alongside
the application widgets. Test with keyboard navigation and pointer input in
both themes. Keep Note actions discoverable without hover.

## Visual verification

Inspect the affected native frontend after changing its layout or styling. Check wide and
compact sizes, Light and Dark Themes, empty and populated collections, and
Write, Preview, and Split. Include long titles, multiple Tags, Search matches,
and dialogs when those areas change.

Source-level visual contracts protect declared dimensions and relationships.
They do not prove rendered typography, focus, wrapping, or spacing. Use the
real AppImage for packaging claims. Browser automation can display GTK through
Broadway, but a web mockup does not verify native GTK styling.
For WinUI, inspect the Windows app at the target display scaling and use its
native keyboard input and file pickers. A GTK run cannot establish Windows
layout, and passing C# binding tests does not establish rendered UI behavior.

See [CONTRIBUTING.md](CONTRIBUTING.md#verify-a-change) for commands and
[the brand toolkit](docs/brand-toolkit.md) for screenshot and external copy rules.
The [Recently Deleted verification record](docs/recently-deleted-verification.md)
records the earlier section layout and the recovery paths exercised at that time.
Use current captures to assess the drawer and collapsible section.
