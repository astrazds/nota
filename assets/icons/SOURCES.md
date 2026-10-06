# Interface icon sources

The Focus interface uses Lucide outlines from
[commit 500620a2e8123f8d1db191538886dc0c223f69a9](https://github.com/lucide-icons/lucide/tree/500620a2e8123f8d1db191538886dc0c223f69a9/icons).
The stroke width is 1.5 units in a 24-unit view box, matching the approved design.
GTK packages the SVGs as symbolic icons. WinUI draws the same outlines with native shapes.
GTK's symbolic renderer needs `transparent-fill` and `foreground-stroke` classes,
with the stroke width, caps, and joins on each shape. These attributes preserve
the Lucide geometry and let GTK apply the current foreground color.

The files are `panel-left`, `plus`, `sliders-horizontal`, `pin`, `ellipsis`,
`list`, `list-checks`, `link`, and `x`. The ISC and applicable Feather MIT licenses
are in `lucide-LICENSE.txt`. Keep the license with redistributed icons.
