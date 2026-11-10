# Effects

Effects in TraceDraw stay **live**: the object keeps its own shape, and
the effect is drawn from it. Change the object and the effect follows;
change the effect's settings at any time from its tool's property bar or
its panel; remove it and the object is as it was.

Most effects work the same way:

- Pick the effect's tool from the effects flyout of the toolbox and drag
  on an object, or select the object and press **Apply** in the
  effect's panel or property bar.
- With an object that has the effect selected, the property bar and
  the panel show its settings and change it as you edit. A run of
  changes is one undo step.
- **Clear** removes the effect; **Break Apart** turns what it draws into
  ordinary objects.

Groups and bitmaps do not take Contour, Distort or Extrude.

## Blend

A blend fills the space between two objects with in-between steps.

1. Pick the **Blend** tool.
2. Drag from the start object to the end object.
3. On the property bar set **Steps** (5 to start with), **Rotation**,
   and the **object** and **colour acceleration** (to bunch the steps
   towards one end).

Both objects stay in your drawing: move, resize or recolour either one
and the steps follow. To run a blend along a curve, use **New path** in
the **Blend** panel (`Ctrl+F8`), with **rotate all objects** to turn the
steps along it. See [the blend notes](../behavior/blend.md).

## Contour

A contour adds evenly spaced copies of an outline, inside, outside or
all the way to the centre.

1. Pick the **Contour** tool and drag on an object: outwards for an
   outside contour, inwards for an inside one.
2. Set **To centre**, **Inside** or **Outside**, the number of **Steps**
   (3), the **Offset** between them (2 mm), the corners (mitred, round
   or bevelled), and the colours of the last step.

**Colour direction** can go straight from colour to colour or round the
colour wheel. The **Contour** panel is `Ctrl+F9`. See
[the contour notes](../behavior/contour.md).

## Envelope

An envelope is a frame of nodes round an object; the object bends to
fill it.

1. Pick the **Envelope** tool and click an object. It gets a frame with
   eight nodes.
2. Choose a mode on the property bar: **Straight Line**, **Single Arc**,
   **Double Arc** or **Unconstrained** (free handles).
3. Drag the nodes. Double-click the frame to add a node; double-click a
   node to remove it.

Or pick a **preset**: Arch, Bulge, Squeeze, Flag, Perspective or Circle.
The **mapping** (Putty, Original, Horizontal, Vertical) decides how the
object fills the frame, and **Keep Lines** keeps straight segments
straight. **Copy Envelope Properties** copies an envelope from another
object. The **Envelope** panel is `Ctrl+F7` (also **Effects >
Envelope**). See [the envelope notes](../behavior/envelope.md).

## Distort

Pick the **Distort** tool, choose a kind on the property bar and drag on
an object. The distortion is centred where you start dragging.

- **Push and Pull**: spikes with hollow sides, or rounded petals.
- **Zipper**: zigzag teeth, with **Random**, **Smooth** and **Local**
  options.
- **Twister**: a swirl, set in rotations and degrees, clockwise or
  counterclockwise.

Drag the diamond to move the centre, or use **Center Distortion**.
**Convert to Curves** makes the distorted outline permanent. See
[the distort notes](../behavior/distort.md).

## Perspective

**Add Perspective** gives an object four corner nodes; drag them to set
the object in perspective.

## Extrude

Extrude builds a 3D-looking body from an object's outline.

1. Pick the **Extrude** tool and drag on an object. For a parallel body
   the drag sets its direction; for the others it sets the vanishing
   point.
2. Drag the handle (a square with a cross) to move the vanishing point
   later.
3. On the property bar or in the **Extrude** panel (`Ctrl+F10`, or
   **Effects > Extrude**) choose:
   - the **type**: small or big, back or front, towards a vanishing
     point, or back or front parallel;
   - the **depth** (20) and whether the vanishing point is locked to the
     object or the page;
   - the **colour**: from the object's fill, one solid colour, or a shade
     between two colours;
   - the **light** angle and intensity.

### Turn, lights and bevel

Under the other controls in the Extrude panel (and in the **3D** menu of
the property bar):

- **Turn** x, y and z turn the body in space; **Reset** puts it back.
- **Elevation** raises the first light above the drawing; **Light 2**
  and **Light 3** add more lights, each with its angle, elevation and
  intensity.
- **Bevel** gives the front edge a sloping band of the width you set.

See [the extrude notes](../behavior/extrude.md).

## Bevel

The **Bevel** panel gives an object a soft edge or an emboss.

## Drop Shadow and Block Shadow

- A **drop shadow** is a soft copy of the object's silhouette drawn
  behind it. Set the offset, opacity (50 %), feathering (15 %), feather
  direction and colour, or pick a perspective preset. See
  [the drop shadow notes](../behavior/drop-shadow.md).
- A **block shadow** is a solid extrusion behind the object. Pick the
  **Block Shadow** tool and drag from an object; the property bar sets
  the depth, direction, colour (60 % black to start with) and a gap
  between object and shadow. See [the block shadow notes](../behavior/block-shadow.md).

## Lenses

A lens changes how the objects under it look, without changing them.
Draw a shape over other objects, select it and open the **Lens** panel
(`Alt+F3`). There are eleven lens types: Brighten, Colour add, Colour
limit, Custom colour map, Fish eye, Heat map, Invert, Magnify, Tinted
grayscale, Transparency and Wireframe. **Frozen** fixes what the lens
shows. See [the lens notes](../behavior/lens.md).

## Symmetry

Symmetry draws live mirrored copies across one or more mirror lines.
Choose **Object > Symmetry > Create New Symmetry** (`Alt+S`): a copy
appears beside the original, mirrored across a vertical line. **Edit
Symmetry** sets 1 to 12 lines, their angle and their centre. Edit the
original and the copies follow. See [the symmetry notes](../behavior/symmetry.md).

## Clip frames

A clip frame shows objects only inside another object's shape.

- To put objects in a frame: select them, choose **Object > Clip to
  Frame > Place Inside Frame**, then click the frame. Or drag them onto
  the frame with the right mouse button and choose **Place Inside**.
- **Edit Contents** lets you change what is inside while the rest of the
  page is dimmed; **Finish Editing Contents** leaves.
- **Extract Contents** takes the objects out again.
- **Lock Contents to Frame** (on to start with) makes the contents move
  with the frame.
- **Opacity Mask** turns the frame into a mask: the frame is no longer
  drawn, and its brightness decides how much of the contents shows,
  fully where it is white and not at all where it is black. Give the
  frame a fountain fill from white to black to fade the contents out.
  **Invert Mask** swaps that.

Right-click an object and use **Frame Type** to turn it into an empty
clip frame, or into an empty paragraph text frame of its size. See
[the clip frame notes](../behavior/clip-frame.md).

## Copy, clone and clear effects

- **Object > Copy Effect** gives the selection an effect of another
  object: choose the effect (Drop Shadow, Transparency, Envelope,
  Perspective, Distortion, Extrude, Contour, Blend, Lens, Bevel or Block
  Shadow), then click the object to copy from. A right click cancels.
- **Object > Clone Effect** copies a shadow or transparency that keeps
  following the source until you edit it.
- **Clear Effect** removes an effect.

See [the copy effect notes](../behavior/copy-effect.md).

## Effects on bitmaps

Bitmaps have their own set of 98 effects in fifteen groups, each with a
dialog that previews the result before and after. They are kept in the
object's list in the Properties panel, where you can change, hide or
remove them. See [the bitmap effect notes](../behavior/bitmap-effects.md).

## Where next

- Save and share your work: [Files](files.md).
- Print it: [Printing](printing.md).
