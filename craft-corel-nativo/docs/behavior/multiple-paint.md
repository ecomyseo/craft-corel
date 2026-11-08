# Several fills and outlines

An object can carry further fills and outlines over its own
(`Shape::extra_fills`, `Shape::extra_strokes`, saved in `.tdraw`).

## Properties docker

- Fill section: under the fill's editor, each further fill as "Fill 2",
  "Fill 3"... with the same editor and a cross that removes it; Add Fill
  puts a copy of the fill on top, to change. Add Fill shows only while
  the object has a fill: its first fill comes from the fill kinds above
  (#487).
- Outline section: the same for outlines ("Outline 2"..., Add Outline,
  a copy of the outline). Add Outline shows only while the object has
  an outline: the Outline box gives it its first one, so the box and
  the button never offer the same thing (#487).
- The changes apply to every selected object (they all get the same
  further fills and outlines), one undo step each ("Fills and Outlines").
- The palette, the Fill tool and the other fill and outline tools still
  set the object's own fill and outline.

## Drawing order

From the bottom: the outlines marked Behind fill, the fill, each
further fill in order, then the other outlines, the object's own first
(`Shape::layers`). The object's shadow goes under the first of them.
Screen, bitmaps, SVG (a group of the parts) and PDF draw it so; `.cdr`
files keep the object's own fill and outline only.

## Checks

- `document::tests::further_fills_and_outlines_draw_in_layers` (core)
- `tests::further_fills_and_outlines_paint_over_the_first` (render)
- `ui::dockers::tests::properties_sections_open_close_and_jump`
- `ui::dockers::tests::properties_sections_are_titled_and_add_buttons_need_a_paint`
