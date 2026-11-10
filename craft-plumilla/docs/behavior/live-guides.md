# Alignment and dynamic guides

Two kinds of temporary guides help place objects without drawing
guidelines first. Both are off on a first start, are turned on and off
in View (Alt+Shift+A, Alt+Shift+D), in the standard bar's Snap To list
and on their Options pages, and both stay quiet while snapping is off
(Alt+Q). They are drawn one pixel wide in screen space, over the
drawing and under the snap mark, and are computed again every frame.

## Alignment guides

While the Pick tool moves the selection, a scale or skew handle is
dragged, or a drawing tool places or drags a point, the selection's box
(or the pointer) lines up with the boxes of the other visible objects on
the page:

| Option (Options > Alignment Guides) | Default |
|---|---|
| Guides for: the selection as one object, or each selected object | the selection as one |
| Line up with edges | on |
| Line up with centres | on |
| The page's edges and centre | on |
| Equal spacing between objects | on |
| Equal sizes while drawing and resizing | on |
| Margins: lines at an offset outside and an inset inside the other objects' and the page's edges | off; 5 mm and 5 mm |
| Colour, line style (solid, dashed, dotted) | magenta (230, 40, 150), solid |

- A value lines up when it lies within the snapping radius (Options >
  Snapping, 10 px) of a target: the move or the point is pulled onto
  it, on each axis on its own. The nearest correction wins; on its axis
  an alignment wins over guidelines and the grids, which still snap the
  other axis.
- Each alignment shows a line through the shared value, from the moved
  box (or the point) to the box it lines up with, 6 px past both ends.
  Lines on the same value join into one.
- Equal spacing (moves only): among the objects that share the moved
  box's band (they overlap it across the axis), the box snaps to the
  middle between its nearest neighbours on both sides, or to the gap the
  next neighbour already keeps from its own neighbour. Each equal gap is
  marked by a line across it with end bars, through the middle of the
  band.
- Equal sizes: while a box is drawn (rectangle, ellipse and the other
  box tools) or the selection is resized with a handle, its width or
  height snaps to the width or height of another visible object when it
  comes within the snapping radius (from the centre, with Shift, the
  pointer moves half as far). A side handle matches only its own axis.
  On each axis the nearer of a size match and an edge or centre
  alignment wins. Each matched size is marked on both objects: a line
  with end bars and two strokes across its middle, 7 px below the width
  or left of the height.
- Hidden objects and layers, objects outside the view and the objects
  being moved or resized are left out.
- In the Multipage View the objects and the pages that are in view on
  the other pages are targets too, where they are displayed beside the
  current page, so an object lines up with what sits on the next page. The boxes are measured once per
  change of the drawing, page, view or options (sorted by value, so each
  frame is a binary search, not a pass over every object).
- Turning the selection (rotate handles, centre of rotation) shows no
  alignment guides. A rotation keeps the centre where it is, so the
  centre has nothing to line up, and its edges sweep through every
  value as the angle changes: snapping the angle to them would fight
  the steps Ctrl gives and make small turns jump. The rotated box's
  edges line up again when the result is moved or resized.

## Dynamic guides

When the pointer snaps to a point of an object (a node, intersection,
midpoint, quadrant or centre) that point is acquired: the four most
recent ones that are still in view, and the start of the line being
drawn, are the anchors. From each anchor run guides at the chosen
angles, both ways; a point near one (within the snapping radius, not on
the anchor itself) snaps onto it, and near two guides from different
anchors onto their crossing.

| Option (Options > Dynamic Guides) | Default |
|---|---|
| Angles (0 to 165 in steps of 15 as check boxes, any other angle added by value) | 0, 45, 90, 135 |
| Distance and angle beside the pointer | on |
| Tick marks along the guide, their spacing | off, 5 mm |
| Snap to tick marks | off |
| Colour, line style | green (40, 150, 90), dashed |

- A guide is drawn across the whole view through its anchor, which is
  marked with a small cross; a crossing draws both guides.
- The tip reads the distance from the anchor in the document's units
  (through the drawing scale) and the direction from the anchor, 0 to
  360 degrees counter-clockwise from the x axis: `25.00 mm, 45.0°`.
- Ticks run from the anchor to the point every tick spacing (none
  closer than 3 px on screen); with Snap to tick marks the point moves
  along the guide onto the nearest tick within the radius.
- A drawing tool's point uses the guides when no object point is near
  and no alignment guide caught it. A Pick tool move uses them for the
  grabbed point (the selection's snap point under the pointer, else the
  centre of its box), with the place the move began as one more anchor,
  when no alignment guide caught it; a move onto another object's point
  still wins.
- Angles read from a preferences file are cleaned: non-numbers are
  dropped, each is taken modulo 180 and repeats go (at most 36).

## Priorities

Object points (Options > Snapping modes) first, then alignment guides,
then dynamic guides, then guidelines, the baseline grid, the document
grid and pixels, then edges and text baselines.

## Checks

- `live_guides::tests` (each option, equal gaps, dynamic guides,
  crossings, ticks, anchors, clipping)
- `snap::tests::alignment_guides_pull_a_drawing_point_and_respect_snapping_off`
- `snap::tests::dynamic_guides_come_from_acquired_points`
- `snap::tests::equal_sizes_snap_while_drawing_and_resizing`
- `snap::tests::the_multipage_view_aligns_with_objects_of_the_other_pages`
- `pick_drag_tests::a_rectangle_drawn_with_alignment_guides_takes_another_objects_size`
- `pick_drag_tests::alignment_guides_line_up_a_moved_object_and_show_the_line`
