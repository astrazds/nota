# Local Notebook visual system and typography

Noter now has enough product UI surface area that visual decisions need to be explicit rather than inferred from scattered Tailwind classes.

Accepted decisions:

- Keep the product in the Local Notebook direction: calm surfaces, warm functional accents, compact controls, and a Writing Surface that stays primary.
- Add `PRODUCT.md` and `DESIGN.md` as contributor-facing product and design system context.
- Keep `.impeccable/design.json` as the machine-readable design token and component example companion to `DESIGN.md`.
- Use Source Sans 3 Variable as the app UI family and Source Code Pro Variable only for Markdown/source editing and syntax examples.
- Self-host fonts from `@fontsource-variable/*` through Trunk copy assets and Tailwind font-family tokens. Do not depend on a remote font provider.
- Keep `style/input.css` responsible for font faces and Tailwind entry directives instead of a single custom utility rule.
- Keep load-bearing visual constants in Rust helpers or `ui_recipes` when they are part of the product contract: footer height, compact controls, Tag pills, selected Note rows, editor writing measure, and notification placement.
- Keep the Write, Preview, and Split panes on one Pane Rhythm: matching content origin, shared `72ch` measure, consistent Note Title scale, and the same footer-height contract.
- Keep Preview left-aligned inside the shared reading measure instead of centring the rendered article.
- Keep Preview prose inversion on the rendered article so Light and Dark Theme readability works in both full Preview and Split.
- Treat the sidebar footer as a compact Backup utility surface with a stable label, terse health summary, and paired Export/Import actions.
- Use warm neutral hover/focus states instead of generic black overlays for sidebar Search, Note actions, menus, Tag pills, and toolbar controls.
- Make editor Tag pills compact and read-only by default, with removal deferred to the Edit tags flow.
- Keep startup Save Status quiet until a user-initiated editing or Backup action changes state.
- Keep the editor Note Title scale consistent between Write and Split so View Mode changes do not resize the active Writing Surface header.
- Keep app typography role-based: product UI in Source Sans 3, Markdown/source editing in Source Code Pro, shared Note Title recipes across Write/Preview/Split, and preview body prose matched to the editor body scale.
- Use visible desktop labels for primary creation and Markdown syntax help where space allows.
- Keep Search Hint as a temporary popup below Search so it does not push the Note List down.
- Extend browser visual contracts to check emitted CSS, computed font families, desktop Tag chip sizing and edit flow, mobile touch targets, writing measure, theme contrast, footer rhythm, startup notification quietness, labelled desktop actions, Preview/Split ordering, dark Preview/Split prose, pane content origins, and editor title scale consistency.

Implementation notes:

- `index.html` copies local font files from `node_modules/@fontsource-variable/*/files` into the Trunk output.
- `style/input.css` declares the Source font faces before Tailwind base/components/utilities.
- `tailwind.config.js` maps `font-sans` and `font-mono` to the local Source families with native fallbacks.
- `ui_recipes` owns shared type, measure, footer, search, Backup, and recovery-control recipes so editor, preview, split, sidebar, and footer styling do not drift into unrelated Tailwind strings.
- `tests/browser/visual-contracts.spec.js` protects both generated CSS contracts and rendered browser geometry.
- PRD #77 captured and validated the critique-to-polish follow-through for these visual contracts.

This keeps Noter aligned with `CONTEXT.md`: Tags remain lightweight Note Metadata, Markdown tooling stays contextual, Product Metadata and design machinery stay outside the primary workflow, and the UI remains a local-first note app rather than a developer workbench or platform clone.
