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
- Make editor Tag pills compact and read-only by default, with removal deferred to the Edit tags flow.
- Keep startup Save Status quiet until a user-initiated editing or Backup action changes state.
- Make Split view hierarchy calmer by reducing the editor-side title scale while keeping the Preview title primary.
- Use visible desktop labels for primary creation and Markdown help where space allows.
- Keep Search Hint as a temporary inline helper so it does not cover selected Note rows.
- Extend browser visual contracts to check emitted CSS, computed font families, desktop Tag chip sizing and edit flow, mobile touch targets, writing measure, theme contrast, footer rhythm, startup notification quietness, labelled desktop actions, and Preview/Split ordering.

Implementation notes:

- `index.html` copies local font files from `node_modules/@fontsource-variable/*/files` into the Trunk output.
- `style/input.css` declares the Source font faces before Tailwind base/components/utilities.
- `tailwind.config.js` maps `font-sans` and `font-mono` to the local Source families with native fallbacks.
- `tests/browser/visual-contracts.spec.js` protects both generated CSS contracts and rendered browser geometry.
- PRD #77 captured and validated the critique-to-polish follow-through for these visual contracts.

This keeps Noter aligned with `CONTEXT.md`: Tags remain lightweight Note Metadata, Markdown tooling stays contextual, Product Metadata and design machinery stay outside the primary workflow, and the UI remains a local-first note app rather than a developer workbench or platform clone.
