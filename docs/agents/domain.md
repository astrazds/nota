# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root.
- **`docs/brand-toolkit.md`** when the work touches brand, screenshots, README copy, release notes, landing pages, logo/mark direction, or external presentation.
- **`docs/adr/`**: read ADRs that touch the area you're about to work in.

If any of these files don't exist, proceed silently. The producer skill (`/grill-with-docs`) creates them lazily when terms or decisions actually get resolved.

## File structure

This is a single-context repo:

```text
/
├── CONTEXT.md
├── PRODUCT.md
├── DESIGN.md
├── docs/brand-toolkit.md
├── docs/adr/          ← including ADR-0009 native replacement, ADR-0010 AppImage, ADR-0011 Nota name
├── docs/agents/appimage-rehearsal.md  ← AppImage and migration compatibility pass
├── crates/nota-core/  ← toolkit-independent domain and compatibility formats
├── crates/nota-desktop/  ← Relm4/GTK4 product; XDG data under net.astrazds.Nota
├── build-aux/         ← Meson cargo wrapper and AppImage packager
└── data/              ← desktop entry and AppStream metadata
```

## Use the glossary's vocabulary

When your output names a domain concept, use the term as defined in `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, either reconsider the term or note it for `/grill-with-docs`.

## Flag ADR conflicts

If your output contradicts an existing ADR, surface it explicitly rather than silently overriding.
