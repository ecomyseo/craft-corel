# Drawing polygons, stars, spirals and other shapes

These tools draw ready-made shapes by dragging a box. As with rectangles,
hold <kbd>Ctrl</kbd> to keep the shape regular and <kbd>Shift</kbd> to draw
from the center; see [Drawing with a box](rectangles-and-ellipses.md#drawing).

## Polygon {#polygon}

The [[tool.polygon]] tool (<kbd>Y</kbd>) draws a regular polygon. Set the
number of **points** (3 to 500, 5 at first) on the property bar before or
after drawing. The first corner is at the top center.

Polygons are symmetric: when you move one node with the [[tool.shape]]
tool, the matching nodes of every side move with it, so you can turn a
pentagon into a flower in one drag.

## Star {#star}

The [[tool.star]] tool draws a star. Its property bar starts with two
buttons that choose what it draws:

- **Star**: a star with the given number of points (5 at first) and
  **sharpness** (0.5 at first; 0 makes a polygon, 1 the thinnest spikes).
  A negative sharpness inverts the star: its inner points come out on the
  other side of the centre. Dragging an inner point through the centre
  with the Shape tool does the same.
- **Complex star**: a star whose sides cross, made by joining each point to
  one further on (9 points and sharpness 2 at first). Its middle stays
  empty. With some numbers of points it is made of several closed shapes, as
  six points give two triangles.

## Spiral {#spiral}

The [[tool.spiral]] tool (<kbd>A</kbd>) draws a spiral, an open curve. On the
property bar set the **revolutions** (1 to 100, 4 at first) and choose a
**symmetrical** spiral, whose turns are evenly spaced, or a **logarithmic**
one, whose turns grow further apart. Hold <kbd>Ctrl</kbd> to draw a round
spiral.

## Common Shapes {#common-shapes}

The [[tool.common_shapes]] tool draws ready-made shapes. Click the shape
button on the property bar to choose one from five sets:

- **Basic shapes**: parallelogram, trapezoid, diamond, triangles, hexagon,
  octagon, plus, heart, lightning, crescent, ring, cylinder, cube, frame,
  teardrop, smiley face and no symbol.
- **Arrows**: in every direction, chevron, pentagon, notched, striped and
  bent arrows.
- **Flowchart**: process, decision, data, document, terminator, connector,
  storage and the other usual symbols.
- **Banners**: ribbons, waves, scroll, explosion, burst and pennant.
- **Callouts**: rectangle, rounded, oval, cloud and line callouts.

Then drag a box. The shape becomes one curve with the default fill and
outline. Parts that should look separate, such as the faces of a cube, are
separate parts of the curve, and holes such as the middle of a ring stay
holes whatever the fill.

## Action Lines {#action-lines}

The [[tool.action_lines]] tool draws speed lines in a comic-book style: a
group of **lines** (2 to 500, 12 at first) that are **parallel** (at first)
or **radial**. Their lengths vary, but the same box always gives the same
lines.

## Graph Paper {#graph-paper}

The [[tool.graph_paper]] tool (<kbd>D</kbd>) draws a grid of cells: a group of
rectangles that fill the box exactly. Set the **rows** and **columns** (4 by
3 at first) on the property bar. Hold <kbd>Ctrl</kbd> for square cells.

The default points, revolutions, rows and columns are also on the tool pages
of the options; see [Tool options](../customize/tool-options.md).
