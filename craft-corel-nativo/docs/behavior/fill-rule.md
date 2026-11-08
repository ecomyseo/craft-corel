# Fill rule (Fill winding)

Every object has a fill rule (#1211, #1919): how its fill covers the places
where the contours of its outline overlap.

- **Even-odd** (the default): a place inside an odd number of contours is
  filled. Two overlapping parts of one curve leave a hole where they
  overlap, as a ring made by combining two circles.
- **Nonzero** (Fill winding on): a place the contours wind round is
  filled. Overlapping parts turning the same way stay filled (traced
  lettering made of overlapping pieces); a contour turning the other way
  is still a hole.

Text always fills nonzero, whatever its object's rule.

## Where it is set

The Properties docker's Fill section has a **Fill winding** box under the
object opacity slider, for the selected curves and polygons (the shapes
whose contours can overlap). On sets nonzero, off even-odd, in one undo step
(`Command::SetFillRule`); a selection whose objects differ shows the box
mixed.

## Where it counts

- The screen, printing and bitmap export (the renderer fills by it, also
  in transparency masks, lenses and clip frames).
- PDF: `f` / `W` for nonzero, `f*` / `W*` for even-odd.
- SVG: nonzero keeps the default `fill-rule`, even-odd writes
  `fill-rule="evenodd"`.
- Shaping (Weld, Trim, Intersect and the others) reads a nonzero curve by
  the area it fills.
- `.tdraw`: saved as `fill_rule` (left out for even-odd).
- `.cdr`: read from version 16 on, from the object style string's
  `fill.winding` ("1" nonzero, "0" or left out even-odd), and written there
  for a nonzero object. Before version 16 no file seen holds a fill rule:
  a nonzero curve is written as the even-odd outline of the area it fills,
  so it looks the same. See `docs/cdr-format.md`.

## Tests

- `render::tests::a_curve_fills_its_overlaps_by_its_own_fill_rule`
- `pdf::tests::a_curve_fills_by_its_own_fill_rule`,
  `svg::tests::a_nonzero_curve_keeps_the_default_fill_rule`
- `parse::tests::version_16_plus_fill_winding_is_the_curve_fill_rule`,
  `write::tests::a_nonzero_curve_keeps_its_overlaps_filled_through_a_save`
- `pick_drag_tests::fill_winding_in_the_fill_section_sets_the_fill_rule`
