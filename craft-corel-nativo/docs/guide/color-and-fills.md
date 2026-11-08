# Colour and fills

Every object has a **fill** (what is inside) and an **outline** (the line
round it). This page shows how to choose colours, use every kind of
fill, set outlines, work with spot colours and make objects
transparent.

## The quickest way: the colour palette

The palette runs along the bottom of the window. With objects selected:

| To | Do this |
|---|---|
| Fill with a colour | click a swatch |
| Colour the outline | right-click a swatch |
| Remove the fill or outline | click or right-click the **No Color** well (white with a red line) at the start |
| Mix a little of a colour in | **Ctrl**+click a swatch (a tenth of it mixes into the fill) |
| Pick a lighter or darker shade | click and hold a swatch for half a second, then click a shade |
| See every colour at once | the button at the right end of the palette |

You can also **drag a swatch onto an object**, selected or not: drop it
inside to fill it, or on its outline to colour the outline.

If you click a colour with nothing selected, TraceDraw asks which kinds
of new objects should take it (see [Default properties](#default-properties)).

The default palette has 99 named CMYK colours; the tooltip shows each
colour's name and values. **Open Palette**, the **Palette Editor** and
the **Palette Manager** are on the palette's menu (the triangle at its
start). See [the palette notes](../behavior/color-palettes.md).

### The document palette

The second row, under the drawing window, collects every colour you
apply, so you can reuse them. You can also:

- drag a swatch, or drag objects with the Pick tool, onto it to add
  their colours;
- use its eyedropper to add a colour from the drawing;
- use its menu for **Add from Selection**, **Add from Document**,
  **Delete Color** and **Reset Palette**.

It is saved with the drawing. **Window > Color Palettes > Document
Palette** shows or hides it.

## The Colour panel

Open it with `Shift+F11` (or **Window > Panels > Color**); `Shift+F12`
opens it for the outline colour.

1. Choose a colour model for the sliders: RGB, CMYK (to start with),
   HSB, HSL, Lab, Grayscale or YIQ.
2. Pick a colour in the viewer (saturation and brightness, with a hue
   strip beside it), with the sliders, or by typing a `#rrggbb` value.
3. Press **Fill** or **Outline** to apply it to the selection, or **Add
   to Palette** to keep it in the document palette.

The panel shows the selection's current colour beside the new one, and
warns you when the new colour is outside what CMYK can print. Its
eyedropper button picks a colour from the drawing. See
[the Colour panel notes](../behavior/color-docker.md).

## Spot colours

A spot colour is a named ink printed on a plate of its own, such as a
metallic or a brand colour. To use one:

1. In the Colour panel, set the colour the ink should look like on
   screen.
2. Type its name in **Ink name** and press **New spot ink**.
3. Under **Spot inks**, set the **Tint** (100 % to start with) and press
   the ink's **Fill** or **Outline** button.

The cross beside an ink takes it off the list; objects keep its look as
a process colour. In PDF files each ink is written as a separation
colour, and when you print separations each ink gets its own plate. See
[the spot colour notes](../behavior/spot-colors.md).

## Kinds of fill

The **Properties** panel (`Alt+Enter`) shows the selection's **Fill**
section, with the fill kinds as pictures: none, uniform, gradient,
pattern, texture and mesh. `F11` opens it on the fill.

### Uniform fill

A single colour, in any model: RGB, CMYK, Gray, HSB, HSL, Lab, YIQ or
registration colour.

### Gradient fill

A gradient blends between colour stops. It can be **Linear**,
**Radial**, **Conical** or **Square**, with 2 to 64 stops, an angle, an
edge pad and (for the round kinds) a centre offset.

To draw a gradient right on the object:

1. Pick the **Interactive Fill** tool (`G`).
2. Drag across the object: the gradient runs from where you press to
   where you let go. Hold **Ctrl** to keep the angle to 15 degree steps.
3. Drag the end nodes to turn the gradient, or the centre node of a
   round one to move its middle.
4. Drag a palette colour onto a node to recolour that stop, or onto the
   line between them to add a stop.

The property bar of the Interactive Fill tool sets the fill type and,
for a gradient, the kind, colours, angle and edge pad. See
[the gradient notes](../behavior/gradient-fill.md).

### Pattern and texture fills

- **Two-colour** patterns, **bitmap** patterns and **full-colour vector**
  patterns repeat a tile across the object.
- To make a vector pattern from your own artwork, select it and choose
  **Object > Create > Vector Pattern Fill**. It becomes the default fill
  for new objects; apply it to a selection from the Properties panel
  (**Fill > Pattern**). See [the vector pattern notes](../behavior/vector-pattern-fill.md).
- **Texture** fills are generated: clouds, marble, noise and wood.

### Mesh fill

A mesh fill is a grid of coloured nodes whose colours flow into each
other.

1. Pick the **Mesh Fill** tool (`M`) and set the rows and columns on the
   property bar (2 x 2 to start with).
2. Click the object to give it a mesh.
3. Click a node, then click a palette colour to colour it. Drag nodes to
   move the colours.
4. Double-click inside the mesh to add a grid line; select a node and
   press **Delete** to remove its lines.

**Smooth colour** blends the colours more softly; **Transparency** on
the bar makes the selected node see-through; **Clear Mesh** removes the
mesh. See [the mesh notes](../behavior/mesh-fill.md).

### Area Fill

The **Area Fill** tool fills any closed area, even one made by
overlapping objects: click inside it and a new object is made from that
area. The objects round it stay as they are. Choose its fill and outline
on the property bar. See [the Area Fill notes](../behavior/area-fill.md).

## Outlines

The property bar sets the outline width (from **Hairline** and **None**
to preset widths), the line style and the start and end arrowheads. For
everything else, press `F12` to open the **Outline** section of the
Properties panel:

- colour and width (new objects: 0.2 mm black);
- caps (butt, round, square) and corners (mitre, round, bevel);
- dashes (dashed, dotted, dash dot);
- arrowheads, from presets or your own;
- a calligraphic nib, with its stretch and angle;
- **Behind fill**, to draw the outline under the fill;
- **Scale with object**, so the width follows the object's size.

The status bar shows the selection's fill and outline; double-click
either to open its settings. See [the outline notes](../behavior/outline-pen.md).

## Transparency

To make an object see-through, pick the **Transparency** tool, or open
the **Transparency** section of the Properties panel. Choose:

- the type: **Uniform** (50 % to start with), **Gradient**, **Pattern**
  or **Texture**;
- the **merge mode** (Normal to start with), such as Add, Subtract,
  Difference, Multiply, Divide, If lighter, If darker, Hue, Saturation,
  Colour or Invert;
- what it applies to: the fill, the outline, or both.

See [the transparency notes](../behavior/transparency.md).

## Copy colours and properties

- The **Colour Eyedropper** samples a colour (from an object or a bitmap
  pixel) and then applies it: click to fill, **Shift**+click to colour
  the outline.
- The **Attributes Eyedropper** copies fill, outline, text, and if you
  tick them, size, rotation, position and effects from one object to
  others.
- Dragging an object with the right mouse button offers **Copy Fill
  Here**, **Copy Outline Here** and **Copy All Properties**.

See [the eyedropper notes](../behavior/eyedroppers.md).

## Styles

The **Object Styles** panel keeps named styles with a fill, an outline
and a transparency. Objects you apply a style to stay linked: change the
style and they all follow. **New style from selection** makes one from
the first selected object; **Break Link to Style** unlinks objects. The
**Colour Styles** panel does the same for colours, with harmonies.

### Default properties

The same panel shows the fill and outline that new objects take, for
three kinds: graphics, artistic text and paragraph text. Choose a colour
with nothing selected, or use **From Selection** and **Reset** there.
**Tools > Save as Defaults** keeps them for new drawings. See
[the object style notes](../behavior/object-styles.md).

## Colour on screen and in print

- **View > Proof Colors**, or the button at the right end of the status
  bar, shows the drawing as CMYK would print it.
- Colour profiles and rendering intents are set in **Tools > Color
  Management**. See
  [the colour management notes](../behavior/colour-management.md).

## Where next

- Add live effects: [Effects](effects.md).
- Prepare colours for print: [Printing](printing.md).
