# Painterly Brush tool

The Painterly Brush tool (Brush Strokes flyout) paints outlines with
brush marks instead of a solid line. A brushstroke is an ordinary curve:
the Shape tool reshapes it, the outline colour colours it and the marks
follow.

## Drawing

- Drag to paint a stroke. The points are simplified with the freehand
  smoothing (Options > Tools > Curves) and smoothed into a curve, named
  "Brushstroke", with no fill.
- The colour is the default outline's colour (black without one); the
  size is the outline width.
- A click (without dragging) selects the object under the pointer; with
  objects selected, picking a brush paints their outlines with it, and
  changing the size or transparency changes their brushstrokes.
- Clear Brushstroke gives the selected objects their plain outline back.

## Property bar

- Brush picker, in three groups: Dry media (Chalk, Charcoal, Soft
  Pastel), Wet media (Ink Wash, Watercolour, Dry Brush) and Textures
  (Bristle, Fur, Spray, Speckle, Cloud, Silk Ribbon). Default Chalk.
- Brush size: 0.1 to 200 mm, default 5 mm.
- Transparency: 0 to 100 %, default 20 %; every mark's opacity is
  multiplied by `1 - transparency`.

## How the marks are made

- The path is sampled along its length (all subpaths), every
  0.08 to 0.4 times the size depending on the brush.
- Marks come from a fixed pseudo-random sequence seeded per object, so a
  stroke looks the same each time it is drawn, exported or printed; a
  new brush on an existing stroke keeps its seed.
- Most brushes thin towards both ends (`sqrt(min(t, 1 - t) / e)`,
  clamped to 0.15..1, `e` 4 to 12 % of the length).
- Grains (dry media, spray, speckle) are small rough blobs scattered
  across the width; wet media lay overlapping soft dabs (watercolour
  wider, paler, with inner pools); bristles and the dry brush are
  parallel streaks (the dry brush skips runs); fur is short curved
  hairs; cloud is stacked soft puffs; the ribbon is nine woven threads.
- With "Scale with object", the size scales with the object.

## Export

- SVG: one filled path per opacity step (12 steps) with `fill-opacity`.
- PDF: the same, each step with its own transparency state (left opaque
  in PDF/X-1a and X-3, which do not allow transparency).
- `.tdraw` keeps the brush, transparency and seed; `.cdr` files are
  written with the plain outline.
