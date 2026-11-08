# Drawing with symmetry

Symmetry draws live mirrored copies of an object across one or more mirror
lines. The copies follow every change you make to the original, so you draw
one half (or one slice) and see the whole. The commands are in
[[menu.object > menu.object.symmetry]].

## To create a symmetry {#create}

1. Select an object.
2. Choose [[menu.object > menu.object.symmetry > menu.object.symmetry_create]]
   (<kbd>Alt+S</kbd>).

A vertical mirror line is placed at the right edge of the object, so the copy
appears next to it, and TraceDraw goes straight into editing the symmetry.

## Editing a symmetry {#edit}

[[menu.object.symmetry_edit]] (<kbd>Alt+E</kbd>) edits the symmetry of the
selected object. A bar at the top left of the drawing window sets:

- the number of mirror lines, 1 to 12, spread evenly around the center. With
  more lines you get a kaleidoscope: with *n* lines there are 2*n* - 1 copies;
- the center (X and Y);
- the angle of the first line (0 to 360 degrees);
- Show full preview (copies drawn in full, or as red outlines when off) and
  Show symmetry lines.

Edit the originals as usual, with any tool; the copies follow. A click or a
marquee picks one of the originals inside the symmetry, and objects you draw,
paste or duplicate join it. Drag a mirror line with the Pick tool to move the
lines; click a line to show round handles at its ends and drag one to turn
the lines. While an object with symmetry is selected or edited, its mirror
lines show as dashed lines.

Click Finish on the bar, or choose [[menu.object.symmetry_finish]]
(<kbd>Alt+Shift+E</kbd>), to stop editing.

## Removing or keeping the copies {#remove}

- [[menu.object.symmetry_remove]] (<kbd>Alt+Shift+S</kbd>) removes the
  symmetry; the copies go away.
- [[menu.object.symmetry_break]] (<kbd>Alt+X</kbd>) turns the copies into
  objects of their own, which no longer follow the original.
