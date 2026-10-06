# Bundled font sources

Nota bundles Gelasio for reading, Source Sans 3 for interface text and headings,
and Source Code Pro for code. The shared preview embeds the WOFF2 files as data
URLs, so GTK and WinUI use identical font bytes without network access. GTK
and WinUI native controls load the bundled TrueType files. GTK's Fontconfig
backend does not resolve the bundled WOFF2 faces, so those files are reserved
for the shared web preview.

## Gelasio

The unmodified static fonts come from
[SorkinType/Gelasio commit 7ab20e7e5c42791e603b9ee3201a0b49849cfdb2](https://github.com/SorkinType/Gelasio/tree/7ab20e7e5c42791e603b9ee3201a0b49849cfdb2).
The font family name is `Gelasio`.

| Local files | Upstream directory |
| --- | --- |
| `Gelasio-Regular.ttf`, `Gelasio-Italic.ttf`, `Gelasio-Bold.ttf`, `Gelasio-BoldItalic.ttf` | `fonts/ttf/` |
| `Gelasio-Regular.woff2`, `Gelasio-Italic.woff2`, `Gelasio-Bold.woff2`, `Gelasio-BoldItalic.woff2` | `fonts/webfonts/` |

`Gelasio-OFL.txt` is the unchanged upstream `OFL.txt`. It contains the copyright
notice and SIL Open Font License 1.1. Keep this license with redistributed fonts.
No font conversion, subsetting, or other modification is applied.

## Source Sans 3 and Source Code Pro

The unmodified TrueType fonts come from Adobe:

- `SourceSans3-Regular.ttf` and `SourceSans3-Semibold.ttf` come from
  [Source Sans commit 87b37a2](https://github.com/adobe-fonts/source-sans/tree/87b37a2daaed80fcb8e8ccb0085c4d72ddade12e/TTF).
  Their license is in `SourceSans3-LICENSE.md`.
- `SourceCodePro-Regular.ttf` comes from
  [Source Code Pro commit 803b7e2](https://github.com/adobe-fonts/source-code-pro/tree/803b7e23ec97ae58b6232ea76519a76d428ba268/TTF).
  Its license is in `SourceCodePro-LICENSE.md`.

The existing `source-sans-3-latin-wght-{normal,italic}.woff2` and
`source-code-pro-latin-wght-{normal,italic}.woff2` files provide the variable
webfont faces. Both families use the SIL Open Font License 1.1. Keep their
licenses with redistributed fonts.
