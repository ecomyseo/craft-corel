# Perspective

Perspective makes an object look as if it lay on a plane that recedes into
the distance. You can add perspective to an object you have drawn, or draw
straight onto a perspective grid.

## Adding perspective to an object {#add}

1. Select an object.
2. Choose [[menu.object > menu.object.add_perspective]].
3. Drag the four corner nodes with the [[tool.shape]] tool to tilt it.

To copy the perspective of another object, use
[[menu.object > menu.object.copy_effect > menu.object.copy_perspective_from]].

## Drawing in perspective {#draw}

[[menu.view > menu.view.draw_in_perspective]] shows a perspective grid over
the page. While it is on, every object a drawing tool makes (rectangles,
ellipses, polygons, curves, brush strokes, text frames) is laid onto the
active plane of the grid. Objects you paste, duplicate or move with the
[[tool.pick]] tool stay as they are.

The bar at the top left of the drawing window has:

- **Finish**: leaves the mode; the objects keep their perspective;
- **1-Point**, **2-Point** and **3-Point**: the kind of grid;
- the planes: **Ground**, **Left**, **Right** (and **Top** for a 1-point
  grid). The active plane is drawn strongly;
- **Grid**: shows the grid lines;
- **Depth** (5 to 98%): how far the grid reaches towards the vanishing
  points;
- **Put on plane**: lays the selected objects onto the active plane.

Drag the vanishing points (circles) and the near corners (squares) to change
the grid. An object laid on a plane gets an ordinary perspective effect,
whose four corners you can still edit with the [[tool.shape]] tool.
