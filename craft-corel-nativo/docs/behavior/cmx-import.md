# CMX import

File > Open and File > Import read `.cmx` exchange files; so does
`tracedraw-cli` (`info`, `convert`). The file's layout is described in
`docs/cmx-format.md`.

## What comes in

- Every page at its size, its layers with their names, groups.
- Curves, polygons, rectangles and ellipses, at their place: the file's
  origin is the page's centre.
- Uniform fills, and fountain fills (linear, radial, conical, square)
  with their colours, angle, centre and edge pad.
- Outlines: colour, width, caps, join, behind fill, scaling with the
  object, dash patterns, and arrowheads as custom heads, at the size and
  place the file gives them (127 000 head units to one outline width;
  open parts drawn as lines one outline width wide; start heads mirrored
  across the line). The lines are not pulled back under their heads.
- Embedded 24-bit images, with their placement and crop.
- Clips.
- Text, as artistic text objects at the place, font, size and colour the
  file draws it with, one object per line; the characters' spacing
  comes from the font, not from the file.

## What does not, yet

Text the file does not draw as characters (a text stream alone), lenses, bitmap, texture and pattern fills, 16-bit coordinate files
and big-endian files. Whatever is skipped is named once in the import
report; opening never fails on it.

## Checks

- `cmx::tests` in `crates/cdr`: pages, layers and shapes; outlines,
  dashes and arrowheads; arrowheads keep their size and placement, open
  parts become lines, closed parts add up and start heads are mirrored;
  fountain fills; images after a jump; text from glyph runs; what is
  not read is reported; long instructions; other files and versions;
  broken files never panic.
- Public sample files compared with the previews they carry: most render
  within an RMS difference of 0.03 to 0.07, the text sample too (0.07,
  0.16 before text was read); the outlines sample (0.23;
  its arrowheads now match the preview in size and place, 0.096 against
  0.119 before once the comparison allows the previews' one or two pixel
  offset; lines not pulled back under the heads), a brushes sample (0.16,
  unread fill types) and one more
  sample (0.22) are the known gaps.
