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
| Pango-based reading measure | [writing_plane.rs](crates/nota-desktop/src/ui/writing_plane.rs) and [style.rs](crates/nota-desktop/src/ui/style.rs) |
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

Use bundled Source Sans 3 for app controls and reading text. Use Source Code
Pro for Markdown editing and code examples. Register the local fonts through
Pango for GTK and include their installed files in the AppImage.
Windows bundles the TrueType versions for native controls. Preserve the
upstream font licenses in both distribution formats.

Keep the Note Title, Tags, body, and footer aligned across Write, Preview, and
Split. The GTK writing plane is left-aligned and capped at a Pango-measured `72ch`.
GTK does not support CSS `max-width`, so `WritingPlane` enforces the widget
allocation. Preview owns its HTML reading measure; the outer GTK layout owns
its horizontal inset. Avoid adding the inset again inside Preview HTML.
Windows measures a Source Code Pro glyph through WinUI and limits the editor
to 72 glyph widths. Its XAML layout owns the surrounding inset.

Use size and weight to establish hierarchy. Keep utility labels secondary to
the Note and avoid decorative letter spacing or a monospace product identity.

## Layout and controls

The Note List stays visible in wide windows. Compact windows switch between
the Note List and editor through a normal navigation control. Split divides
the editor area equally and is available only in wide windows.

Keep View Mode controls in one stable editor footer. Split uses one footer
across both panes. The sidebar footer holds Backup controls. Exact dimensions
belong in GTK's `NATIVE_VISUAL_CONTRACT` and the Windows XAML and layout code,
not in duplicate prose token tables.

Formatting controls sit between Note Metadata and the Markdown body. They
appear only when writing is available. Preserve native selection and undo history
when a formatting action changes text.

Tag chips remain compact metadata. GTK Note List Tags are filter buttons
and show every matching Tag. Their FlowBox children must not add default
padding that changes chip spacing. Keep existing row and Tag widgets when
identities are unchanged so updates preserve focus and scroll position.
Windows exposes Tag filters above the Note List in a horizontally scrollable
row and uses an editable comma-separated field below the Note Title.

GTK's Search Hint appears temporarily below Search. Global Notifications provide
short-lived save, Backup, import, and error feedback without adding permanent
header chrome. Windows keeps error notifications visible until dismissed.
Storage Recovery is an explicit app state with named actions. GTK shows it
inside the workspace; Windows uses a dialog while the workspace is disabled.

### Recently Deleted

Keep Recently Deleted visibly separate from the active Note List. Its header
spans the sidebar width on a contrasting neutral band. The deleted rows use
the theme's surface background, with a fine separator between rows. Leave
space above the section so the boundary remains clear without relying on color.

Use a stronger heading and readable utility text while keeping the Writing
Surface primary. Place **Restore** and **Delete** beside each deleted Note,
and **Clear All** in the section header. Restore is neutral; destructive
actions use the theme's red. Preserve these distinctions in both themes.

The header remains visible when Recently Deleted is empty; Clear All appears
only when it contains Notes. The section remains part of the sidebar's scroll
content rather than a separate navigation destination. Exact values belong
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
contains native light and dark captures and the exercised recovery paths.
