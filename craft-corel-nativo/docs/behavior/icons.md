# Toolbox icons

The icons of the toolbox, its flyouts, the Hints panel, the tool rows
of Options and the badge beside the cross hair are drawn with the
painter at any size, no bitmaps (code: `ui/icons.rs`, the path helpers
in `ui/icon_paint.rs`). They follow one style, "bead on a wire": ink
draws the object, the brand blue draws the action, and a small blue
square, the node, marks the live point, sitting on its line like a bead
on a wire.

## Style rules

- **Grid 24, stroke 2, round caps and joins**, a 2 unit safe margin.
  The grid is mapped onto the icon's box (16, 20, 24, 32 or 40 px in
  the toolbox and the standard bar from the Extra Small to the Extra
  Large button size, 24 px at Medium; 20 px in the Hints panel, 15 px
  on the cursor badge). Strokes are drawn at 5/8 of their design width (`WEIGHT`), so
  a stroke 2 is 1.25 px in the 22 px box, never thinner than one
  physical pixel; guides and handles are thin lines (1.4 design units),
  node and handle frames one physical pixel. A box within a twelfth of a
  whole number of pixels per unit (22 to 26 px at 1x, 44 to 52 px at
  2x) is snapped to that whole number, so every unit is on a pixel;
  any other box of about 20 px and up keeps its own size with the grid's
  side rounded to whole pixels, so a larger box always draws a larger
  icon (`icon_paint::grid_scale`, `the_grid_grows_with_its_box`; it had
  rounded every box from 21 to 35 px to the same 24 px grid, so larger
  buttons kept the same icons). The grid is centred on a pixel
  centre, so lines on whole pixels stay sharp; round cap and
  join discs are only added to strokes of 3 physical pixels or more.
  Rectangles and boxes have square 90 degree corners.
- **Three colours, by role.** *Ink* draws the object: the shape, the
  curve, the tool itself. The *accent* (the brand blue, `2F6FD0` on
  light themes, `5CAEFF` on dark ones) draws the action: what the tool
  drags, creates or changes. Never more than one blue idea per icon.
  The *background* (the panel's colour) fills opaque bodies that must
  sit in front of something else (the page in front of its ghost in
  Pan, the box in front of its shadow, the front face of the extrusion,
  the pipette's tube).
- **The node motif, from the logo.** A small blue square (3.4 units,
  soft corners) marks the live point the tool acts on. Hollow ink
  squares are plain anchors; hollow blue circles are handle ends.
  Nodes always sit on their line and the stroke runs into the square;
  never a floating square.
- **Bodies get a 14 % ink wash** so closed shapes read as objects at
  20 px; opaque bodies in the background colour only where something
  must sit in front of something else.
- **Dashes (2.4 on, 2.2 off) mean "proposed or ghost"**: the old curve
  before a drag, the piece cut off, a copy not yet made, the page's
  place before a pan, the text frame.
- **One pointer**: the pick arrow (slanted tail), reused small (0.5 to
  0.6) wherever the tool is about grabbing (Freehand Pick, Shape,
  Anchor Editing). Corners are soft (radius 1.5 to 2.5), never pill
  shaped; no 3D, no gradients.
- **Themes**: the same drawing in every theme, only the three colours
  swap. Opaque bodies take the panel's colour, so nothing is redrawn
  for a dark theme.

## Colours in the interface

- `icons::draw(painter, rect, tool, color)` draws with `color` as the
  ink on the panel; `draw_on` takes the surface's colour too (white on
  the cursor badge and in the Hints pictures).
- The accent is the brand blue at full strength only: a dimmed icon
  (the theme's disabled icon colour, or a translucent ink) stays
  monochrome, and the High Contrast theme keeps its one icon colour
  (`ui::icons::tests::the_accent_marks_the_live_point_and_dimmed_icons_stay_plain`).
- Coloured tool icons (Customization > Appearance): the ink takes the
  group's hue, the accent stays blue (see `options.md`).

## The set

One icon per tool of `Tool::ALL`, including every flyout tool. Where a
tool is drawn from, by group:

| Group | Icons |
|---|---|
| Pick | the arrow with the node at its tail; a lasso closing on a node with a small pointer; a tilted box with the rotation arc and the node on the pivot |
| Shape edit | a node with its two handles under the pointer; jagged ink becoming a smooth wave; a blob pulled into a tail; a spiral winding inside a shape; four arrows converging on a shape; a brush dragging a smear; a body with its top edge sawn into a zigzag |
| Crop | two interlocking corner brackets framing the kept square, a solid blue square on the dragged corner; a knife on the diagonal with the blue cut it leaves; a curve with the doomed segment a ghost, crossed out; a block eraser on its end with a blue rubbed-out mark |
| Zoom | a lens showing a magnified curve and node; a page shifted from its ghost with the drag arrow |
| Curves | a pencil drawing a scribble; a line, hollow start and live end; the S curve with its handles; a nib drawing a segment; a ghost control polygon with round handles; a polyline with hollow vertices; an arc with hollow ends and a live apex |
| Drawing | the calligraphic S swash (a blue ribbon on a thin ink path, the node at the live end); a round brush with three dabs; a sketched square recognised as a clean shape; sketchy strokes reading as one line |
| Rectangle, Ellipse, Polygon | the shape as a body with the node on its corner, rim or apex; the 3-point variants tilted with two hollow points and the live third; the star, the spiral ending in a node, the arrow shape, speed lines behind a shape, the grid of cells |
| Text | a bold T inside the dashed text frame; a table with a blue header row |
| Dimension | the measured edge in ink and the dimension line with ticks in blue (parallel, horizontal and vertical, the angle's arc, one segment of a path); a callout box with its tail ending in a node |
| Connector | two boxes joined by a blue link with round anchors (curved, right-angled, rounded); a box with its anchors, the live one under the pointer |
| Effects | an opaque box with a solid blue shadow offset behind it; a triangle with two offset outlines fading outward; a square morphing into a blue circle; a square with bowed sides and the corner pulled; a wavy box with four live corners; the extrusion's blue depth faces; a circle with a solid blue block behind it |
| Transparency | a circle filled with ink fading to nothing, the blue node-to-handle bar across it |
| Eyedropper | the diagonal pipette with the drop drawn into its tip, over a swatch or with a property card |
| Fill | a tilted pail holding blue paint, a drop at its lip; two rings with the lens filled; a curved mesh with four handles and the live node |
| Outline | a ring with a thick blue outline segment ending in a node (Outline Pen); the ring with a colour swatch (Outline Colour) |

## Checks

- `ui::icons::tests::every_tool_icon_draws_in_its_box`: every tool
  draws shapes that stay inside its box and fill most of it.
- `ui::icons::tests::the_accent_marks_the_live_point_and_dimmed_icons_stay_plain`.
- `ui::icons::tests::coloured_icons_follow_the_toolbox_groups_and_can_be_off`.
- `ui::icon_paint::tests`: the path syntax (numbers, relative and
  implicit commands, arcs), the triangulation of concave shapes, the
  ribbons.
- `help_shots::icon_sheet` (ignored; `ICON_SHEET=<dir> cargo test -p
  tracedraw --lib icon_sheet -- --ignored`) draws every icon on a
  light and a dark panel at 1x and 2x pixels per point into `<dir>`,
  for a look by eye.
