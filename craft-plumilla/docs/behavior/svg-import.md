# SVG import

File > Open, File > Import, a dropped `.svg` or `.svgz` and pasted SVG
are read by `crates/io/src/svg_import.rs`. Shapes, groups, clip paths
(as clip frames), gradients and pictures are converted from the tree the
SVG library builds (CSS, `use`, units and nested transforms resolved).

## Text

Text comes in as editable text objects (#1820). The SVG library is built
without a text engine and leaves text out, so `crates/io/src/svg_text.rs`
reads every `<text>` element itself, following SVG 1.1:

| SVG | TraceDraw |
|---|---|
| `<text>` and its `tspan`, `a` and `textPath` children | one text object, a span per run of characters in one style |
| `x`, `y` of the text and of a `tspan` with an `x` (a text chunk) | the start of the baseline; only the first value of a list is read |
| chunks with the same `x` and anchor, each the same distance below the one before (or a whole number of times it, for left out empty lines) | lines of one text, line spacing in points equal to that distance |
| other chunks | one text object each, grouped |
| `dy` and `dx` inside a chunk, `baseline-shift` | baseline shift and horizontal character offset of the characters that follow (`super` +1/3 em, `sub` -1/5 em) |
| `text-anchor` start, middle, end | alignment left, centre, right on the anchor point |
| `dominant-baseline`, `alignment-baseline` | the baseline moved below the point by a share of the font size: middle and mathematical 0.25, central 0.35, hanging 0.8, text-top 0.9, ideographic and text-bottom -0.2 |
| `font-family` (first of the list), `font-size` (px, pt, mm, em, %, keywords; 1 px = 0.75 pt), `font-weight` (bold, bolder, 600 and above), `font-style` (italic, oblique), `font` | span font, size, bold, italic; `serif`, `sans-serif`, `monospace`, `cursive`, `fantasy` become Times New Roman, Arial, Courier New, Comic Sans MS and Impact |
| `text-decoration`, `letter-spacing`, a single `rotate` value | underline, strikethrough, range kerning (% of the em), character angle |
| `fill` (colour, `currentColor`; a gradient or pattern gives its first stop colour or its fallback), `stroke`, `stroke-width` | object fill and outline; spans whose fill differs keep their own |
| `transform` of the text and its ancestors, `use`, `opacity`, `clip-path`, `mask`, `switch` | applied as for any other element; a uniform scale goes into the font size, a skew or mirror stays in the object's transform |
| `display="none"`, `visibility="hidden"` | the text is left out |

Properties come from the `style` attribute, then the most specific
`<style>` rule (later rules win ties), then the presentation attribute,
then the parent. White space collapses (newlines removed, tabs to spaces,
runs of spaces to one, none at the start or end of a chunk) unless
`xml:space="preserve"`, which keeps every space.

Not read: text inside `clipPath`, `mask`, `pattern` and `marker`
(left out, as before), per-character position lists after their first
value, text following a path (its characters are placed on a line from
the text's position), vertical writing modes and `flowRoot`.
