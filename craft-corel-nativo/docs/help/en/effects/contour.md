# Contours

A contour draws evenly spaced copies of an object's outline inside it,
outside it, or to its center, like the lines of a map. It is a live effect:
change the object and the contour follows.

## To add a contour {#create}

With the tool:

1. Choose the [[tool.contour]] tool from the Effects flyout.
2. Drag from an object: away from its middle for an outside contour, towards
   it for an inside one. The further you drag, the larger the spacing.

With the panel:

1. Select an object.
2. Open the [[docker.contour]] panel with [[menu.effects > menu.effects.contour]]
   (<kbd>Ctrl+F9</kbd>).
3. Set the options and click [[docker.apply]].

![The Contour panel](../../img/effects-contour.png)

Groups and bitmaps cannot take a contour.

## Contour settings {#settings}

The property bar and the panel show the settings of the selected contour and
change it at once.

- **To center**, **Inside** or **Outside** (at first).
- **Steps**: 3 at first, up to 999. To center makes as many as fit.
- **Offset**: the distance between steps (2 mm at first).
- **Corners**: mitered (sharp, at first), round or beveled.
- **Color direction**: direct, clockwise or counterclockwise around the color
  wheel, from the object's colors to the last step's.
- The **fill and outline colors of the last step** (white and black at
  first). An object without a fill gives unfilled steps.
- **Object acceleration** and **color acceleration** (-100 to 100%) crowd the
  steps or the colors towards the object or towards the last step.

Outside steps are drawn behind the object, inside steps in front of it.

**Clear contour** removes the contour; **Break apart** turns the steps into
objects, grouped beside the object. [[menu.object > menu.object.break_contour_apart]]
(<kbd>Ctrl+K</kbd>, also on the right-click menu) does the same.
