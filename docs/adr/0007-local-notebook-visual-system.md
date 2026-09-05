# Local Notebook visual system and typography

Nota now has enough product UI surface area that visual decisions need to be explicit rather than inferred from scattered Tailwind classes.

Accepted decisions:

- Keep the product in the Local Notebook direction: calm surfaces, warm functional accents, compact controls, and a Writing Surface that stays primary.
- Use the Frame A Quiet Notebook main app frame as the approved structural reference for the product UI: light paper root, compact left rail, warm selected Note row, thin editor toolbar, stable editor-area footer, and paper-neutral popup models.
- Add `PRODUCT.md` and `DESIGN.md` as contributor-facing product and design system context.
- Keep `.impeccable/design.json` as the machine-readable design token and component example companion to `DESIGN.md`.
- Use Source Sans 3 Variable as the app UI family and Source Code Pro Variable only for Markdown/source editing and syntax examples.
- Self-host fonts from `@fontsource-variable/*` through Trunk copy assets and Tailwind font-family tokens. Do not depend on a remote font provider.
- Keep `style/input.css` responsible for font faces and Tailwind entry directives instead of a single custom utility rule.
- Keep load-bearing visual constants in Rust helpers or `ui_recipes` when they are part of the product contract: footer height, compact controls, Tag pills, selected Note rows, editor writing measure, and notification placement.
- Keep the Write, Preview, and Split panes on one Pane Rhythm: matching content origin, shared `72ch` measure, consistent Note Title scale, and the same footer-height contract.
- Keep Preview left-aligned inside the shared reading measure instead of centring the rendered article.
- Keep Preview prose inversion on the rendered article so Light and Dark Theme readability works in both full Preview and Split.
- Tint Dark Theme Preview prose headings, links, and bold text toward the Nota frame neutral rather than pure white.
- Treat the sidebar footer as a compact Backup utility surface with a stable label, terse health summary, and paired Export/Import actions.
- Use warm neutral hover/focus states instead of generic black overlays for sidebar Search, Note actions, menus, Tag pills, and toolbar controls.
- Make editor Tag pills compact and read-only by default, with removal deferred to the Edit tags flow.
- Keep startup Save Status quiet until a user-initiated editing or Backup action changes state.
- Keep the editor Note Title scale consistent between Write and Split so View Mode changes do not resize the active Writing Surface header.
- Keep app typography role-based: product UI in Source Sans 3, Markdown/source editing in Source Code Pro, shared Note Title recipes across Write/Preview/Split, and preview body prose matched to the editor body scale.
- Use visible desktop labels for primary creation and Markdown syntax help where space allows.
- Preserve compact desktop controls while keeping compact mobile View Mode Controls and Markdown help at a 44px touch target.
- Keep Search Hint as a temporary popup below Search so it does not push the Note List down.
- Keep modal semantics on the actual popup panel rather than the full-screen overlay so assistive technology identifies the interruption surface precisely.
- Extend browser visual contracts to check emitted CSS, computed font families, desktop Tag chip sizing and edit flow, 44px mobile touch targets, writing measure, theme contrast, Frame A material surfaces, footer rhythm, popup panel dialog semantics, startup notification quietness, labelled desktop actions, Preview/Split ordering, dark Preview/Split prose, pane content origins, and editor title scale consistency.

Implementation notes:

- `index.html` copies local font files from `node_modules/@fontsource-variable/*/files` into the Trunk output.
- `style/input.css` declares the Source font faces before Tailwind base/components/utilities.
- `tailwind.config.js` maps `font-sans` and `font-mono` to the local Source families with native fallbacks.
- `ui_recipes` owns shared type, measure, footer, search, Backup, and recovery-control recipes so editor, preview, split, sidebar, and footer styling do not drift into unrelated Tailwind strings.
- The modal component keeps overlay dismissal and Escape handling on the overlay, while `role="dialog"` and ARIA labelling live on the popup panel.
- `tests/browser/visual-contracts.spec.js` protects both generated CSS contracts and rendered browser geometry.
- PRD #77 captured and validated the critique-to-polish follow-through for these visual contracts.

This keeps Nota aligned with `CONTEXT.md`: Tags remain lightweight Note Metadata, Markdown tooling stays contextual, Product Metadata and design machinery stay outside the primary workflow, and the UI remains a local-first note app rather than a developer workbench or platform clone.
