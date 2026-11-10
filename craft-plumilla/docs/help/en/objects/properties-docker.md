# The Properties panel

The [[docker.properties]] panel (<kbd>Alt+Enter</kbd>) shows and changes the
properties of the selected objects: their outline, fill, transparency and
more, depending on what is selected. It is open at first, beside
[[docker.hints]] and [[docker.objects]]. Open it again with
[[menu.object > menu.object.properties]].

![The Properties panel](../../img/objects-properties-docker.png)

## With objects selected {#selection}

The top of the panel names the selected object (and how many more are
selected) and gives its position and size. Under it, a row of buttons opens
and closes each section the object has:

- **QR code**, for a QR code: its content and look; see
  [Barcodes and QR codes](barcodes.md#qr-codes).
- [[docker.outline]]: color, width, paint, caps, corners, dashes,
  arrowheads and more; see [Setting outlines](../lines-shapes/outlines.md).
- [[docker.fill]]: the kind of fill and its settings; see
  [Fills](../color/fills.md). <kbd>F11</kbd> opens this section.
- [[docker.transparency]]: how much of what lies below shows through; see
  [Transparency](../color/transparency.md).
- [[kind.rectangle]], for rectangles: the corner settings; see
  [Corners](../lines-shapes/rectangles-and-ellipses.md#rectangle-corners).
- [[docker.fx]]: the bitmap effects applied to the object; see
  [Bitmap effects](../effects/bitmap-effects.md).
- [[docker.character]], for text: the font, size, [[docker.bold]] and
  [[docker.italic]], and a button to [[docker.edit_with_text_tool]].
- [[docker.summary]]: a summary of the object.

The sections you closed stay closed the next time. A change applies to every
selected object and is one undo step.

## With nothing selected {#defaults}

With nothing selected, the panel says [[docker.no_objects_selected]] and shows
the [[docker.default_properties]]: the fill and outline that new shapes and
lines get. Change them here to draw with other colors from the start. See
[Object styles and defaults](../templates-styles/object-styles.md#document-defaults).
