# Sketching and shape recognition

Two tools turn quick hand-drawn strokes into clean objects: the
[[tool.shape_recognition]] tool recognizes the shape you meant, and the
[[tool.sketch]] tool builds a smooth line from several strokes.

## Shape Recognition {#shape-recognition}

The [[tool.shape_recognition]] tool (<kbd>Shift+S</kbd>) turns a freehand
stroke into the shape it looks like, as you drew it: a tilted rectangle stays
tilted.

1. Choose the [[tool.shape_recognition]] tool.
2. Draw the shape in one stroke. A stroke whose ends come close together is
   taken as closed.
3. Release the button.

What you get:

- An open stroke that is nearly straight becomes a straight line; otherwise
  a smooth curve.
- A closed stroke with four corners near right angles becomes a rectangle.
  When it is nearly level, it comes out level.
- A closed, rounded stroke becomes an ellipse, level when nearly level and
  never tilted when nearly round.
- A closed stroke with three corners becomes a triangle, and one with four
  other corners a four-sided shape.
- Anything else becomes a smooth closed curve.

The new object takes the default fill and outline and is selected.

On the property bar, **Shape recognition** sets how readily strokes become
shapes, and **Smoothing** how much the leftover curves are smoothed. Each is
None, Lowest, Low, Medium (at first), High or Highest. With recognition at
None, every stroke stays a curve.

## Sketch {#sketch}

The [[tool.sketch]] tool (<kbd>S</kbd>) lets you build a line from several
short strokes, as you would with a pencil.

1. Choose the [[tool.sketch]] tool.
2. Draw a stroke. It is smoothed and selected.
3. Draw the next stroke starting near the end of the first. With **Join** on,
   it is added to the same curve.

On the property bar, **Join** adds a stroke to the selected open curve when it
starts close enough to its end; the distance beside it sets how close
(5 mm at first). **Smoothing** sets how much each stroke is smoothed, from
None to Highest.
