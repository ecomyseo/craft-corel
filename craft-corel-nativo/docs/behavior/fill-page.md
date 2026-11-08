# Fill page (Repeat Copies)

Edit > Repeat Copies (Ctrl+Shift+D) opens the Repeat Copies panel. Its
first row chooses between Copies (the number of copies, each one step
from the last, as before) and Fill page, which tiles the selection in a
grid over the page or an area of it: labels, stickers, business cards
and tags ganged on one sheet (#1255). The work is in
`apps/tracedraw/src/fill_page.rs` (the grid) and `step_repeat.rs`
(`App::fill_page_apply`, the preview).

## Fields and defaults

| Field | Default | What it does |
|---|---|---|
| Page: w x h | | The page's size in the drawing unit, for reference. |
| Fill: Page / Area | Page | The whole page, or an area set by its left, bottom, width and height (page coordinates, from the page's bottom left corner; choosing Area starts it at the page). |
| Margins, Same for all | 0, on | Room kept clear inside the page or area: one value for all sides, or Left, Right, Top and Bottom margins. |
| Gap, Same for all | 0, on | The space between copies: one value both ways, or Across and Down. |
| Shift alternate rows | 0 | How far the second, fourth, ... row from the top moves right (staggered layouts). |
| Orientation | As it stands | As it stands, Turned 90° (a quarter turn anticlockwise) or Whichever fits more (the turned grid only when it holds more copies). The first two show their count beside them. |
| Trim marks | off | Marks at every cut line, Mark length (3 mm) long, starting Distance from copies (0) outside the grid. |

Under the fields the panel shows what Apply makes: "4 x 6 = 24 copies"
(with a shifted row that holds fewer, "4 and 3 x 6 = 21 copies"), or
that the selection does not fit. The values stay as typed for the
session, as the Copies values do.

## Formulas

The selection's box (its visual bounds, outlines included) is `w` x
`h`, swapped when turned. The room is the area less its margins, `W` x
`H`; the gaps are `gx` and `gy`, the shift `s`.

- Copies across: `floor((W - w) / (w + gx)) + 1` (0 when `w > W`); in a
  shifted row the same with `W - s`.
- Rows: `floor((H - h) / (h + gy)) + 1`.
- The block (its widest row, the shift included, and its rows) is
  centred in the room; the first copy is at its top left.
- Trim marks: for every distinct left or right edge of a copy, a
  vertical mark above and below the block; for every top or bottom edge,
  a horizontal mark left and right of it. With no gap one mark serves
  two neighbours. They are one open path object named "Trim marks",
  0.2 mm in the registration colour (it prints on every plate).

## Apply

Apply moves the selection to the first place of the grid (turned about
its middle when the copies stand turned) and puts a copy of it in every
other place, on the active layer, with the trim marks: one undo step
("Fill Page"). Everything made is selected, the original first. With
nothing that fits Apply is greyed and the status bar says so; more than
10,000 copies are not made (the panel says so).

While the panel is worked in (the pointer on it, a field being typed in,
or the last click in it) every place of the grid is outlined dashed on
the page, the outlines turned once for turned copies, with the trim
marks as lines; the outlines are worked out again only when the
selection, the drawing, a value or the zoom changes.

## Tests

- `fill_page::tests` (the counts of the report's video: a 3 x 2 inch
  label on a 12 x 18 inch page, 4 x 8 = 32 as it stands and 5 x 6 = 30
  turned, 3 x 8 = 24 and 5 x 5 = 25 with 3 mm margins; gaps, margins and
  centring; shifted rows; nothing fits; trim marks)
- `step_repeat::tests::fill_page_makes_the_copies_counted_and_previewed`,
  `fill_page_turns_the_copies_to_fit_more`, `fill_page_fills_a_chosen_area`
- `pick_drag_tests::repeat_copies_fills_the_page_from_the_panel`
