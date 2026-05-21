# Noter Brand Toolkit

This toolkit turns Noter's product, design, and domain language into practical brand guidance for contributors, agents, screenshots, project pages, README updates, release notes, and future marketing surfaces.

Source documents:

- [`PRODUCT.md`](../PRODUCT.md): product purpose, users, personality, anti-references, principles.
- [`DESIGN.md`](../DESIGN.md): visual system, tokens, component rules.
- [`CONTEXT.md`](../CONTEXT.md): domain language and product relationships.
- [`docs/adr/0007-local-notebook-visual-system.md`](adr/0007-local-notebook-visual-system.md): accepted visual-system decisions.

Concept artifact:

- [`docs/assets/brand/noter-brand-toolkit-concepts.png`](assets/brand/noter-brand-toolkit-concepts.png): exploratory brand toolkit image covering mark direction, README hero, social preview, mood board, and screenshot frames.
- [`docs/assets/brand/noter-main-app-frame-palette.png`](assets/brand/noter-main-app-frame-palette.png): exploratory palette artifact for main application frame templates.
- [`docs/assets/brand/noter-main-app-frame-mocks.png`](assets/brand/noter-main-app-frame-mocks.png): exploratory main application frame template mocks for the product UI.

Approved product frame:

- **Frame A, Quiet Notebook Frame** is the product UI reference for the main app frame: light paper default, compact restrained sidebar, warm selected Note row, thin editor toolbar, stable editor-area footer, and paper-neutral popup panels.

## Brand Promise

Noter is a calm, local-first Markdown note app for capturing, writing, finding, organising, previewing, deleting, recovering, and backing up personal Notes.

Markdown is a capability, not the brand frame. The brand frame is a dependable local notebook in the browser: private, low-friction, recoverable, and hard to disrupt.

## Audience

Noter is for people who want a fast personal note app without managing files, folders, cloud sync, command systems, or Markdown tooling first. They are usually in one of four states:

- Capturing something quickly before it disappears.
- Returning to an existing Note they vaguely remember.
- Writing or previewing Markdown without wanting an editor showcase.
- Protecting or recovering their local collection.

The product should reduce the user's need to reason about software. A good Noter surface feels like the Note was already waiting for them.

## Personality

Three physical words:

- **Calm**: quiet surfaces, predictable actions, no decorative urgency.
- **Local**: private, owned, browser-resident, not cloud-branded.
- **Practical**: clear labels, fast recovery, compact controls, direct outcomes.

The personality is expert restraint, not minimalism for its own sake. The product should feel trustworthy because it is explicit about user-owned data, destructive actions, and recovery paths.

## Positioning

Use this:

> A local-first Markdown note app for quick capture, focused writing, reliable discovery, and user-owned Backup.

Avoid these frames:

- Markdown workbench.
- Developer editor.
- File manager.
- Folder or notebook organiser.
- Command-palette productivity shell.
- Cloud sync product.
- Apple Notes clone.

## Voice

Noter copy is plain, operational, and specific. It should name the thing that will happen, especially when recovery or data movement is involved.

Use:

- "Create Note"
- "Restore Note"
- "Move to Recently Deleted"
- "Export Backup"
- "Import Backup"
- "This Backup will add 3 Notes and replace 1 Note."

Avoid:

- "New document"
- "Empty trash"
- "Sync complete"
- "Manage workspace"
- "Run command"
- "Your files are safe"
- "AI-powered note intelligence"

### Copy Rules

- Prefer product nouns from [`CONTEXT.md`](../CONTEXT.md): Note, Note Title, Writing Surface, Preview, View Mode, Tag, Flat Collection, Recently Deleted, Backup, Merge Import.
- Name destructive impact before the action happens.
- Do not imply cloud persistence. Backup is local export/import unless the product later adds another storage model.
- Keep helper copy short enough to stay out of the Writing Surface.
- Avoid generic productivity claims. Say what the user can do in Noter.
- Do not use decorative technical language for ordinary note tasks.

## Naming Language

Use these names consistently:

| Use | Avoid |
| --- | --- |
| Note | document, file |
| Note Title | derived heading |
| Markdown Note App | Markdown workbench |
| Writing Surface | source pane |
| Preview | output pane |
| View Mode | layout toggle |
| Tag | folder, primary navigation item |
| Flat Collection | folders, notebooks |
| Quick Capture | new document wizard |
| Recently Deleted | hidden undo state |
| Backup | sync, cloud backup |
| Merge Import | replace import |
| Search | command palette |

## Brand Architecture

### Product UI

Product UI uses the restrained Local Notebook system:

- Warm Capture Yellow is rare and functional.
- The Note stays primary.
- Controls are compact, stable, and discoverable.
- Shadows appear only for overlays, transient hints, menus, modals, and notifications.
- Search and the Note List lead discovery. Tags remain lightweight metadata.

### External Brand Surfaces

External surfaces may be more expressive while staying grounded:

- README hero images, app screenshots, project pages, and release graphics can use Warm Capture Yellow more prominently.
- The yellow should still read as capture, selection, focus, or preservation. It should not become generic decoration.
- Use product screenshots or carefully constructed note/paper imagery before abstract gradients or generic SaaS illustration.
- Keep claims practical and product-specific.

External brand surfaces can be warmer than the app. They should not become louder, trendier, or less trustworthy than the app.

## Color

Noter has two color strategies:

1. **Restrained product UI**: tinted neutrals plus Warm Capture Yellow at low coverage.
2. **Committed brand moments**: Warm Capture Yellow can carry larger areas in README, website, launch, or screenshot compositions.

Use OKLCH when adding new colors in CSS. Tint all near-white and near-black neutrals toward the Noter palette. Avoid pure `#fff` and `#000`.

### Core Roles

| Role | Existing Token | Use |
| --- | --- | --- |
| Warm Capture Yellow | `#FFB340` | primary actions, focus, selection, highlight, progress, brand moments |
| Paper Neutrals | `#F5F5F7`, `#E8E8ED`, `#D2D2D7` | light theme surfaces, dividers, secondary controls |
| Dim Desk Surfaces | `#1C1C1E`, `#2C2C2E`, `#3A3A3C` | dark theme root, sidebar, overlays, borders |
| Graphite Text | `#111827` | primary light-theme text |
| Muted Text | `#6B7280` | metadata, utility copy, secondary labels |
| Recovery Red | `#EF4444` | destructive actions, errors, permanent removal |
| Saved Emerald | `#10B981` | success feedback only |

### Color Rules

- Warm Capture Yellow must mean action, selection, focus, highlight, progress, or brand capture.
- Do not use yellow as a general section background inside product UI.
- Tune Light Theme and Dark Theme separately. Do not invert colors mechanically.
- Keep destructive red out of non-destructive emphasis.
- Keep success green out of decorative positive messaging.
- Avoid navy-and-gold, beige-and-slate, black-and-neon, and cloud-SaaS blue palettes.

## Typography

Noter's product typography is already decided:

- UI and reading surfaces: Source Sans 3 Variable.
- Markdown/source editing and syntax examples: Source Code Pro Variable.
- Native fallbacks only after the Source families.
- Fonts are self-hosted through project assets, not remote font providers.

### Type Character

Source Sans 3 gives Noter a humanist, open-source, practical voice. It is readable without feeling platform-native or decorative. Source Code Pro is a tool for Markdown editing, not a brand costume.

### Type Rules

- Use hierarchy through size and weight, not letter spacing.
- Keep body lines around 65 to 75 characters when possible.
- Do not introduce a display serif for brand polish.
- Do not make mono typography the identity.
- Do not use all-caps body copy.
- Do not use gradient text.
- Do not use fluid viewport-scaled body text.

## Logo And Mark Direction

Noter does not need a loud mascot, cloud, or developer symbol. The mark should make the product feel like a local note object: captured, readable, and recoverable.

### Recommended Mark Lane

Build the mark from three cues:

- **Paper**: a simple note plane or sheet.
- **Capture**: a warm focus point, corner, dot, or small fold.
- **Preservation**: a subtle containment shape or recovered-layer cue.

The mark should work as:

- 16px favicon.
- Sidebar/app icon.
- README badge or project header mark.
- Monochrome stamp in documentation.
- Warm accent version for external brand surfaces.

### Shape Guidance

Prefer:

- One folded or inset paper form with a small warm capture detail.
- A compact rounded-square app icon using paper neutral and Warm Capture Yellow.
- A simple line mark that suggests a Note without literal notebook binding.
- A mark that can be drawn in one or two colors.

Avoid:

- Cloud shapes.
- Terminal prompts.
- Markdown `#` as the whole identity.
- Folder tabs.
- Sparkles.
- Checkmarks as the main symbol.
- Apple Notes-style yellow notepad icons.
- Complex pen, quill, or document-stack metaphors.
- Generic AI or productivity symbols.

### Wordmark

Use `Noter` in Source Sans 3 with confident weight. Keep the wordmark simple enough to sit beside a product screenshot without becoming a logo showcase.

Rules:

- No gradient fill.
- No decorative ligatures.
- No tight negative tracking.
- No all-caps `NOTER` as the primary wordmark.
- Keep the mark and wordmark separable.

### Future Asset Requirements

When a final logo asset is created, include:

- SVG source.
- Monochrome version.
- Warm accent version.
- Favicon-safe simplified version.
- Minimum size and clear-space rules.
- Dark and light surface checks.

## Layout

The product layout is a working notebook, not a marketing dashboard.

### Product Layout Rules

- Keep the Writing Surface visually primary.
- Keep Write, Preview, and Split on one Pane Rhythm.
- Keep View Mode Controls in the editor-area footer, not in a persistent app header.
- Keep the sidebar dense enough to scan several Notes.
- Keep Search as the primary discovery control.
- Keep Backup Controls compact and secondary.
- Keep Product Metadata and diagnostics outside the primary workflow.

### External Layout Rules

External brand layouts may be more spacious and expressive, but they should still feel like Noter:

- Lead with a real product screenshot or a precise Note object composition.
- Let one surface, Note, or recovery action be the hero.
- Use asymmetric space when it helps the screenshot or copy breathe.
- Avoid identical icon-card grids.
- Avoid hero sections made from abstract gradients and generic claims.

## Components And UI Tone

### Buttons

Primary buttons use Warm Capture Yellow for clear action. Secondary buttons use paper neutrals. Danger buttons use Recovery Red only when a destructive action is actually being confirmed.

Labels should name outcomes:

- "Create Note"
- "Restore"
- "Clear All"
- "Export"
- "Import"

Avoid vague labels:

- "Continue"
- "Proceed"
- "Manage"
- "Apply"

### Chips

Tags are metadata. They support filtering and recognition, but they are not navigation pillars. Keep Tag chips compact, neutral, and secondary unless they are actively filtering.

### Notifications

Notifications are transient feedback for save, Backup, import, and recovery outcomes. They must not become permanent status chrome.

### Modals

Use modals only when interruption protects the user from meaningful impact:

- Delete Confirmation.
- Clear All confirmation.
- Backup Import Preview.
- Storage Recovery decisions.

The title or body must name the Note, count, or collection impact.

## Iconography And Illustration

Use icons as functional cues, not decoration. Product controls should use familiar symbols with accessible names.

Illustration should be sparse and object-led:

- A single note plane.
- A local desk surface.
- A small warm capture mark.
- Recovery or Backup as a contained copy, not a cloud.

Avoid:

- Decorative icon grids.
- Large rounded-corner icons above every heading.
- Abstract blobs or gradient orbs.
- Generic empty-state cartoons.
- Cloud upload/download imagery for Backup.

## Imagery

Brand imagery should reveal the real product or a precise product-adjacent object.

Use:

- Actual app screenshots.
- Cropped Note List and Writing Surface details.
- A clear empty collection state.
- Backup Import Preview or Recently Deleted when explaining safety.
- Simple note/paper compositions when screenshots are not enough.

Avoid:

- Dark blurred screenshots.
- Decorative gradient backgrounds.
- Stock laptops with unreadable UI.
- Generic Markdown code screenshots.
- Cloud storage imagery.

Screenshot guidance:

- Use realistic Note content.
- Show readable Note Titles and Tags.
- Do not expose private or joke data.
- Include Light and Dark Theme only when comparing theme support.
- Keep browser chrome minimal unless the browser context matters.

## Motion

Motion should confirm state, not entertain.

Use:

- Short ease-out transitions for menus, Search Hint, notifications, and mode changes.
- Reduced-motion-safe behavior.
- No layout-jarring transitions around the Writing Surface.

Avoid:

- Bounce or elastic motion.
- Animated layout properties.
- Decorative looping motion.
- Entrance choreography inside the product UI.

External brand pages may use slightly more motion, but it should feel like paper, focus, or recovery, not a generic tech reveal.

## Accessibility

Accessibility is part of the brand. Noter earns trust by making important states visible and operable.

Requirements:

- Target WCAG AA contrast for text and interactive states.
- Ensure keyboard, pointer, and touch access for primary controls.
- Do not rely on color alone for selected, destructive, or success states.
- Keep Note Actions visible enough to discover without hover.
- Preserve readable wrapping and truncation on compact viewports.
- Tune Light and Dark Theme contrast separately.
- Respect reduced motion.

## External Surface Guidance

### README

README presentation should answer:

- What is Noter?
- Why local-first?
- What can a user safely do?
- What does the app look like?

Use a product screenshot early. Keep the headline literal:

> Noter

Supporting copy can carry the value:

> Local-first Markdown notes for quick capture, focused writing, search-led discovery, and user-owned Backup.

### Release Notes

Release notes should use product language and user outcomes:

- "Search results now explain body matches with Match Snippets."
- "Backup Import Preview now shows add and replace counts before Merge Import."
- "Recently Deleted now supports explicit Restore and Clear All actions."

Avoid internal implementation labels unless the audience is contributors.

### Landing Page Or Project Page

If Noter gets an external landing page, use a Committed brand moment:

- Warm paper or yellow-led first viewport.
- Product screenshot or note-object imagery.
- One strong claim about local note-taking.
- Clear path to try, install, or inspect the project.

Do not make a SaaS hero with abstract stats, floating cards, gradient text, or cloud promises.

## Do And Do Not

### Do

- Keep the Note primary.
- Use Warm Capture Yellow as a meaningful signal.
- Use Source Sans 3 for product and brand typography.
- Use Source Code Pro only for Markdown editing or syntax examples.
- Keep Search and Note List as the discovery model.
- Treat Backup as user-owned local preservation.
- Name destructive and recovery impacts explicitly.
- Use actual product screenshots for external presentation.
- Keep Light and Dark Themes separately tuned.

### Do Not

- Do not make Noter look like Apple Notes.
- Do not make Noter feel like a developer Markdown workbench.
- Do not turn Tags into a folder replacement.
- Do not hide Note Actions behind hover-only controls.
- Do not imply cloud sync.
- Do not add permanent status chrome.
- Do not use generic destructive confirmations.
- Do not use side-stripe accent borders.
- Do not use gradient text.
- Do not use glassmorphism as a default style.
- Do not use abstract gradient blobs as brand imagery.
- Do not use identical decorative card grids.

## Brand Checks

Before shipping a UI or external brand surface, ask:

1. Is the Note still primary?
2. Does every yellow element mean action, selection, focus, highlight, progress, or capture?
3. Could a user understand local Backup and recovery behavior without guessing?
4. Are destructive actions named with their Note or collection impact?
5. Does this avoid cloud, folder, command-palette, and developer-editor framing?
6. Is Markdown supporting writing rather than becoming the product identity?
7. Are Light and Dark surfaces tuned, not inverted?
8. Would this still feel like Noter without copying Apple Notes?

If the answer to any question is no, revise before adding more visual polish.
