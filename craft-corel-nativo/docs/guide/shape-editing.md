# Shape editing

Once an object is drawn, you can reshape it: move its nodes, bend its
segments, combine it with other objects, cut it or erase parts of it.

## The Shape tool

Pick the **Shape** tool (`F10`). With the Pick tool, double-clicking a
curve switches to the Shape tool too.

A selected curve shows its outline in blue, its nodes as small markers
(squares for cusp nodes to start with) and the first node of each part
as a slightly larger triangle. A selected node shows its control
handles as dashed lines ending in arrow heads.

### Select nodes

| To | Do this |
|---|---|
| Select one node | click it |
| Add or remove a node | **Ctrl**+click it |
| Select a run of nodes | **Shift**+click the last node of the run |
| Step to the next or previous node | **Tab**, **Shift+Tab** |
| Select the first or last node | **Home**, **End** |
| Select nodes in an area | drag a marquee; choose **Rectangular** or **Freehand** in the property bar's **Selection mode** list |
| Select every node | **Ctrl+A**, the **Select All Nodes** button, or double-click the Shape tool's button |

### Move nodes and bend segments

- **Drag a node** to move the selected nodes. Hold **Ctrl** to keep the
  move horizontal or vertical. Nodes snap like a drawing tool's points.
- **Drag a handle** to change the curve's shape at that node.
- **Drag a segment** to bend it: the point you grab follows the pointer
  while both end nodes stay. A straight segment becomes a curve.
- The **arrow keys** nudge the selected nodes (with **Shift** and
  **Ctrl** for larger and smaller steps).

### Add and delete nodes

- **Double-click a segment** to add a node there; **double-click a
  node** to delete it.
- Click a segment, then press **+** (or **Add Node**) to add a node at
  the spot you clicked.
- With nodes selected, **+** adds a node in the middle of each segment
  ending at them, and **-** or **Delete** removes them.

### Node types

Each node is one of three types:

- **Cusp**: the two handles move independently, so the curve can turn a
  sharp corner.
- **Smooth**: the handles stay in a line, so the curve passes smoothly.
- **Symmetrical**: the handles stay in a line and the same length.

To change the type, use the property bar's **Cusp**, **Smooth** and
**Symmetrical** buttons, or with nodes selected press **C** (cusp and
smooth swap), **S** (smooth and symmetrical swap) or **Y**
(symmetrical). **Convert to Line** and **Convert to Curve** change the
segment ending at the selected node.

### More node commands

All of these are on the property bar, and on the menu you get by
right-clicking a node:

- **Join Nodes**: join two end nodes into one.
- **Break Curve**: split the curve at the selected node.
- **Extend Curve to Close**: join two selected end nodes with a straight
  segment.
- **Close Curve** and **Reverse Direction**.
- **Extract Subpath**: make the part with the selected node a curve of
  its own.
- **Align Nodes**: line the selected nodes up horizontally or
  vertically.
- **Reduce Nodes** and the **Curve smoothness** slider: remove nodes
  while keeping the shape. Drag the slider further to remove more.
- **Stretch and Scale Nodes** and **Rotate and Skew Nodes**: with two or
  more nodes selected, handles frame them so you can scale, turn or
  slant just those nodes.
- **Reflect Horizontally** and **Reflect Vertically**: moving a node
  moves its mirror partners on the other side the opposite way.
- **Elastic Mode**: nodes farther from the one you drag move less.

With nodes selected, **Ctrl+C**, **Ctrl+X** and **Ctrl+D** copy, cut or
duplicate the segments between them as new curves.

### Rectangles, ellipses and polygons

These keep their own kind until you convert them, and the Shape tool
edits them in their own way:

- **Rectangle**: drag a corner node to round every corner together;
  **Ctrl**+drag (or click the corner first) to change that corner only.
- **Ellipse**: drag its node inside the ellipse to make a pie, outside
  to make an arc.
- **Polygon or star**: drag a point to move all the points in or out;
  drag a node between points in or out to sharpen the star, or round the
  centre to twist it.

To edit them as free curves, use **Convert to Curves** (`Ctrl+Q`).
**Convert outline to object** (`Ctrl+Shift+Q`) turns an outline into a
filled shape.

Detail: [the Shape tool notes](../behavior/shape-tool.md).

## Shaping: weld, trim, intersect

Shaping combines overlapping objects into new outlines. Select two or
more objects and use **Object > Combine Shapes**, the buttons at the end of the
property bar, or the **Shaping** panel. The **last object you selected**
is the target: it gives the result its fill and outline.

| Operation | Result |
|---|---|
| **Weld** | one object covering all of them |
| **Trim** | the target without what the others cover (the others stay) |
| **Intersect** | only the area they all share (the originals stay) |
| **Simplify** | each object without what the objects above it cover |
| **Front Minus Back** | the front object without the ones behind it |
| **Back Minus Front** | the back object without the ones in front |
| **Boundary** | an outline around the selection (works on one object too) |

In the Combine Shapes panel you can choose, for Weld, Trim and Intersect,
whether to leave the original source and target objects. Each operation
is one undo step. Detail: [the shaping notes](../behavior/shaping.md).

## Corners and joins

- The **Corners** panel (**Window > Panels > Corners**) fillets
  (rounds), scallops or chamfers the corners of curves. Set the radius,
  or the two chamfer distances, and press **Apply**; the result is
  previewed while the panel is open. With the Shape tool and some nodes
  selected, only those corners are cut.
- **Object > Join Curves** joins nearby ends of curves. Choose **Extend**,
  **Chamfer**, **Fillet** or **Bezier Curve**, and a gap tolerance (5 mm
  to start with); ends further apart are left alone.

Detail: [the corners and joins notes](../behavior/corners-and-joins.md).

## Shape brushes

These tools reshape an outline as you brush over it. Drag over a
selected object; objects that are not curves are converted first. Each
tool has a **Nib size** on its property bar, and the effect fades from
the centre of the nib to its edge.

| Tool | What it does | Key |
|---|---|---|
| **Smooth** | evens out wobbles | |
| **Smear** | pulls the outline along with the pointer | `W` |
| **Twirl** | swirls the outline round the pointer (Alt or Shift turns the other way) | |
| **Attract and Repel** | pulls nodes towards, or pushes them away from, the pointer (Alt or Shift swaps) | |
| **Smudge** | smooths, then smears, for a pull without kinks | `V` |
| **Roughen** | adds spikes along the outline | `E` |

Each stroke is one undo step. Detail: [the shape brush notes](../behavior/brushes.md).

## Crop, Knife, Segment Delete and Eraser

### Crop

1. Pick the **Crop** tool and drag an area. It stays on the page with
   eight handles; drag inside to move it, drag a handle to resize it.
2. Double-click inside it, press **Enter** or use **Crop** on the
   property bar to apply it. **Esc** clears it.

The selected objects (or, with nothing selected, everything on the page)
are cut to the area; whatever lies wholly outside is deleted. Bitmaps,
tables and symbols are put in a clip frame instead, so you can take them
out again.

### Knife

Pick the **Knife** tool and drag a cut across objects: a straight
**Two-point line** or a **Freehand** line. Choose **Split** (the pieces
meet) or **Gap** with a width, and **Keep as one object** if the pieces
of each object should stay together.

### Segment Delete

**Segment Delete** removes the stretch of an outline between two places
where it crosses another outline (or itself). Click a stretch to remove
it, or drag a rectangle to remove every stretch it touches, whole.

### Eraser (`X`)

Drag over the selected objects (or the object under the pointer) to rub
out a band. Set the **thickness** (6.35 mm to start with) and a round or
square nib on the property bar. A click erases a dot; a double-click on
an object deletes it. A clip frame is erased through its frame, which
hides its contents along the band. Bitmaps and groups are not erased.

Detail: [the crop, knife and eraser notes](../behavior/crop-knife-eraser.md).

## Where next

- Add text to your drawing: [Text](text.md).
- Colour what you shaped: [Colour and fills](color-and-fills.md).
