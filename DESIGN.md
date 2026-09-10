# Nota design system

Nota uses a compact Note List, a clear Writing Surface, warm functional accents,
and paper-neutral dialogs. The Linux GTK window is the product. Preview is an
embedded, read-only WebKitGTK view of the same Note.

## Sources of truth

Keep design intent here and exact implementation values in their owners:

| Concern | Owner |
| --- | --- |
| GTK palette, controls, focus, and Light and Dark Themes | [nota.css](crates/nota-desktop/resources/nota.css) |
| Window composition and widget behavior | [workspace.rs](crates/nota-desktop/src/ui/workspace.rs) |
| Shared layout dimensions | [visual_contract.rs](crates/nota-desktop/src/visual_contract.rs) |
| Pango-based reading measure | [writing_plane.rs](crates/nota-desktop/src/ui/writing_plane.rs) and [style.rs](crates/nota-desktop/src/ui/style.rs) |
| Preview HTML, typography, and content policy | [preview.rs](crates/nota-desktop/src/preview.rs) |
| Product dialogs | [dialogs.rs](crates/nota-desktop/src/ui/dialogs.rs) |
| Bundled fonts | [fonts.rs](crates/nota-desktop/src/fonts.rs) and [assets/fonts](assets/fonts) |

Browser-era Tailwind utilities and copied token tables are not native styling
APIs. Change the source owner and inspect its rendered result.

## Color

Use Warm Capture Yellow for primary actions. Use the softer signal color for
selection, active View Mode controls, and focus. Light Theme uses warm paper
surfaces; Dark Theme has its own palette. Preserve visible borders and readable
text in each theme rather than deriving one by inversion.

The `.nota-root` and `.nota-root.nota-dark` variables in `nota.css` define the
current palette. Use the existing roles before introducing another color.
Reserve red for destructive actions and errors, and green for success feedback.
Selected Notes must also have a visible border or other non-color cue.

## Typography and reading measure

Use bundled Source Sans 3 for app controls and reading text. Use Source Code
Pro for Markdown editing and code examples. Register the local fonts through
Pango for GTK and include their installed files in the AppImage.

Keep the Note Title, Tags, body, and footer aligned across Write, Preview, and
Split. The writing plane is left-aligned and capped at a Pango-measured `72ch`.
GTK does not support CSS `max-width`, so `WritingPlane` enforces the widget
allocation. Preview owns its HTML reading measure; the outer GTK layout owns
its horizontal inset. Avoid adding the inset again inside Preview HTML.

Use size and weight to establish hierarchy. Keep utility labels secondary to
the Note and avoid decorative letter spacing or a monospace product identity.

## Layout and controls

The Note List stays visible in wide windows. Compact windows switch between
the Note List and editor through a normal navigation control. Split divides
the editor area equally and is available only in wide windows.

Keep View Mode controls in one stable editor footer. Split uses one footer
across both panes. The sidebar footer holds Backup controls. Exact dimensions
belong in `NATIVE_VISUAL_CONTRACT`, not in duplicate prose token tables.

Formatting controls sit between Note Metadata and the Markdown body. They
appear only when writing is available. Preserve GTK selection and undo history
when a formatting action changes text.

Tag chips remain compact metadata. Native Note List Tags are filter buttons
and show every matching Tag. Their FlowBox children must not add default
padding that changes chip spacing. Keep existing row and Tag widgets when
identities are unchanged so updates preserve focus and scroll position.

Search Hint appears temporarily below Search. Global Notifications provide
short-lived save, Backup, import, and error feedback without adding permanent
header chrome. Storage Recovery is an explicit app state with named actions.

## Dialogs and accessibility

About, Markdown help, deletion, Clear All, and Backup Import Preview use the
shared paper dialog. Keep a clear title, readable body, and compact action row.
Use GTK accessible roles and names. Cancel is the default focus for destructive
confirmations. Escape and closing a dialog must return the user to the app.

Use visible GTK focus states. GTK-owned titlebar nodes need styling alongside
the application widgets. Test with keyboard navigation and pointer input in
both themes. Keep Note actions discoverable without hover.

## Visual verification

Inspect the native window after changing GTK layout or CSS. Check wide and
compact sizes, Light and Dark Themes, empty and populated collections, and
Write, Preview, and Split. Include long titles, multiple Tags, Search matches,
and dialogs when those areas change.

Source-level visual contracts protect declared dimensions and relationships.
They do not prove rendered typography, focus, wrapping, or spacing. Use the
real AppImage for packaging claims. Browser automation can display GTK through
Broadway, but a web mockup does not verify native GTK styling.

See [CONTRIBUTING.md](CONTRIBUTING.md#verify-a-change) for commands and
[the brand toolkit](docs/brand-toolkit.md) for screenshot and external copy rules.
