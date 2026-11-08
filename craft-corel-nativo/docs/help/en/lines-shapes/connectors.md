# Connector lines and anchors

Connector lines join two objects and stay attached to them: when you move or
resize the objects, the connector follows. They are useful for diagrams,
flowcharts and organization charts.

## Anchors {#anchor-points}

A connector attaches to an **anchor** of each object. Every object has four
anchors, at the middle of each side of its bounding box. You can add your own
anchors with the [[tool.anchor_editing]] tool; they move and scale with the
object.

## Drawing a connector {#connector-tools}

Three tools in the Connector flyout draw connectors:

- [[tool.connector]]: a straight line;
- [[tool.right_angle_connector]]: horizontal and vertical segments with
  square corners;
- [[tool.rounded_connector]]: the same with rounded corners.

To join two objects:

1. Choose a connector tool.
2. Drag from one object to the other. While you drag, the anchors of both
   objects show, and the line starts at the anchor that will be used.
3. Release on the other object.

When you start or end the drag on an anchor, that anchor is used; otherwise
the closest pair of anchors of the two objects is used. The connector is an
open line with no fill and the default outline.

### Connector settings {#connector-settings}

The property bar, shown with the connector tools and when the selection is
all connectors, has the three kinds of connector (with connectors selected,
they change how those run), the **corner radius** of rounded connectors
(3 mm at first), and the outline width, style and arrowheads of the selected
connectors.

### Staying attached {#attached}

- When you move, scale or reshape the joined objects, the connector runs
  again between their anchors.
- When you move the connector together with its objects, it stays attached.
- When you move the connector on its own, it lets go and becomes a plain
  line. Undo attaches it again.
- When one of its objects is deleted, the connector stays where it is.

## Editing anchors {#anchors}

The [[tool.anchor_editing]] tool adds, moves and deletes anchors:

1. Choose the [[tool.anchor_editing]] tool and click an object. Its anchors
   show as diamonds.
2. Click inside the object to add an anchor there.
3. Drag an anchor you added to move it.
4. Select an anchor and press <kbd>Delete</kbd>, double-click it, or click
   **Delete anchor** on the property bar to remove it. **Remove all custom
   anchors** removes every anchor you added.

The four anchors at the middle of the sides cannot be removed.
