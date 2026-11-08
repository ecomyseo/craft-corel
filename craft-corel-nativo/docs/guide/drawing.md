# Drawing

This page covers the tools that make new objects: shapes, curves,
brush strokes, connectors, dimensions and callouts. New objects take the
default fill and outline (to start with: no fill and a 0.2 mm black
outline); see [Colour and fills](color-and-fills.md#default-properties)
to change them.

## Shapes drawn with a box

The Rectangle, Ellipse, Polygon, Star, Spiral, Graph Paper, Common
Shapes and Action Lines tools all draw by dragging a box. While you
drag, the shape itself follows the pointer. For every one of them:

- Hold **Ctrl** to make the box square (a square, a circle, a regular
  polygon).
- Hold **Shift** to draw from the centre outwards. You can press or let
  go of it in the middle of the drag.
- Points snap as usual (see [Snapping](selecting-and-transforming.md#snapping)).

After you let go, the new object is selected.

### Rectangles

To draw a rectangle: pick the **Rectangle** tool (`F6`) and drag.

- The **3-point rectangle** tool, in the same flyout, draws a rectangle
  at any angle.
- Double-click the Rectangle tool's button to draw a frame the size of
  the page; hold Shift to frame the selection instead.

To round the corners, use the property bar:

1. Choose a corner style: **Round**, **Scalloped** (cut inwards) or
   **Chamfered** (a straight cut).
2. Type a size. With the **Edit Corners Together** lock on, one value
   goes to all four corners; turn it off to set each corner on its own.
3. **Relative Corner Scaling** (on to start with) makes the corners grow
   with the rectangle; turn it off to keep them the same size when you
   resize.

With nothing selected, these settings apply to the next rectangle. You
can also drag a rectangle's corner node with the Shape tool (see
[Shape editing](shape-editing.md#rectangles-ellipses-and-polygons)).

### Ellipses, pies and arcs

Pick the **Ellipse** tool (`F7`) and drag. The **3-point ellipse** tool
draws one at any angle. On the property bar, choose **ellipse**, **pie**
or **arc**, and set the start and end angles and the direction.

### Polygons and stars

- **Polygon** (`Y`): set the number of points (3 to 500, 5 to start
  with) on the property bar, then drag.
- **Star**: the property bar starts with the **Star** and **Complex
  Star** buttons. Set the points and the **sharpness** (from a plain
  polygon to thin spikes). A complex star joins each point to one further
  on, so its sides cross.

Polygons stay symmetric: with the Shape tool, dragging one node moves
its mirrored partners too.

### Spirals and graph paper

- **Spiral** (`A`): set the revolutions (1 to 100, 4 to start with) and
  choose a symmetric or logarithmic spiral.
- **Graph Paper** (`D`): set rows and columns (4 x 3 to start with). The
  result is a group of rectangles that fill the box exactly.

### Common shapes

The **Common Shapes** tool draws ready-made outlines: 71 shapes in five
sets (basic shapes, arrows, flowchart, banners and callouts). Click the
shape button on the property bar, pick a shape from the grid, then drag
the box.

### Action lines

**Action Lines** draws a group of parallel or radial lines (12 to start
with, up to 500) in the box you drag.

Detail for every shape tool: [the shape tool notes](../behavior/shape-tools.md).

## Curves and lines

All curve tools make curves you can edit node by node later. Pick them
from the curve flyout.

### Freehand (`F5`)

- **Drag** to draw as with a pencil. TraceDraw smooths the stroke as you
  let go; set the smoothing (0 to 100, 50 to start with) on the property
  bar.
- **Click, then click** again to draw a straight line. Hold **Ctrl** to
  keep it to steps of 15 degrees. **Esc** drops the first click.

### 2-Point Line and Polyline

- **2-Point Line**: click (or drag) from one point to the next; the
  second click finishes the line.
- **Polyline**: each click adds a straight segment; double-click or
  press **Enter** to finish.
- **Ctrl** keeps each segment to steps of the constrain angle.

### Bezier, Pen and B-Spline

These place nodes one at a time:

- **Click** to place a corner (cusp) node.
- **Click and drag** to place a smooth node and pull out its handles.
- **Double-click** or press **Enter** to finish.
- **Click the first node** (once the curve has three nodes) to close the
  curve.
- **Esc** throws away the curve you are drawing.
- **Ctrl** keeps the next node, or the handle you drag, to steps of the
  constrain angle.

### 3-Point Curve

Drag the chord (from start to end), then click the point the curve
should pass through.

### Continue a curve

With one open curve selected, start (or end) a new line with Freehand,
Bezier, Pen, B-Spline, Polyline or 2-Point Line on one of its end nodes.
The new line becomes part of that curve. If its other end lands on the
curve's other end, the curve closes. The pointer shows a small node and
line when you are over an end node that can be continued, and a ring
when a click would close the curve.

### Shape Recognition and Sketch

- **Shape Recognition** (`Shift+S`): draw a rough rectangle, ellipse,
  triangle, four-sided shape or line freehand, and it becomes a clean
  one, at the angle you drew it. Set how eagerly it recognises shapes
  and how much it smooths on the property bar.
- **Sketch** (`S`): draw in short strokes; with **Join** on, a stroke
  that starts near the end of the selected curve is added to it.

Detail: [the curve tool notes](../behavior/curve-tools.md).

## Brush strokes

The **Brush Strokes** tool (`I`) turns a freehand stroke into a filled
shape. Choose a mode on the property bar:

- **Calligraphic**: a flat nib at an angle; the stroke is thin when you
  move along the nib.
- **Preset**: a stroke with a profile such as tapered ends, a bulge or a
  wave.
- **Brush**: the preset profile with a rough, dry-brush edge.
- **Expression**: slow strokes come out wider, like pressing harder.
- **Sprayer**: small objects (circles, squares, stars, hearts, leaves)
  placed along the path.

To turn an existing curve into a brush stroke, select it and use
**Apply to curve** in the **Brush Strokes** panel. See
[the brush stroke notes](../behavior/brush-strokes.md).

## Connectors

Connectors are lines that join two objects and follow them when they
move.

1. Pick a connector tool: **Straight**, **Right angle** or **Rounded
   right angle**.
2. Drag from one object to another. The connector attaches to the
   nearest anchors (the midpoints of each object's sides, unless you add
   your own).

Move or resize either object and the connector runs again between them.
If you move the connector on its own, it lets go and becomes a plain
line.

To add your own attachment points, use the **Anchor Editing** tool:
click an object, then click inside it to add an anchor; drag an anchor
to move it; press **Delete** or double-click to remove it.

The property bar changes how selected connectors run, the corner radius
of rounded ones, and their outline. Detail:
[the connector notes](../behavior/connectors-and-anchors.md).

## Dimensions

Dimension lines measure your drawing and update when you move or scale
them.

- **Parallel** and **Horizontal or Vertical**: drag (or click) from the
  first point to the second, then click where the dimension line goes.
- **Segment**: click a segment of any object, then click where the line
  goes.
- **Angular**: click the vertex, a point on the first side, a point on
  the second side, then where the arc goes.

The property bar tells you what to click next; **Esc** starts again. Use
it to set the precision, units, whether to show the unit, a prefix and
suffix, where the text sits (above, in or below the line) and the label
size. Lengths follow the drawing scale, so at 1:100 a 10 mm line reads
1000 mm. See [the dimension notes](../behavior/dimensions.md).

## Callouts

The **3-point callout** tool draws a leader line with text:

1. Drag from the tip (what you point at) to the elbow, or click each.
2. Click where the text goes.
3. Type the text; it replaces the sample "Callout text".

On the property bar choose the outline width, line style, the head at
the tip, a shape round the text (underline, rectangle, rounded
rectangle, ellipse) and the gap. The leader and the shape follow the
text as you type or move it.

## Other things you can insert

- **Tables**: see [Text](text.md#tables).
- **QR codes, barcodes** (Code 128, EAN-13, EAN-8) and **page numbers**
  can be inserted too.
- **Coordinates** panel: build a rectangle, ellipse, polygon, star,
  line or curve from typed coordinates. See
  [the Coordinates notes](../behavior/coordinates.md).

## Where next

- Edit the nodes of what you drew: [Shape editing](shape-editing.md).
- Give it colour: [Colour and fills](color-and-fills.md).
