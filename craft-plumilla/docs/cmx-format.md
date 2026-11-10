# CMX exchange files

What TraceDraw knows of `.cmx` files, read by `crates/cdr/src/cmx.rs`.
Every fact below was found on public sample files and checked against the
preview picture each file carries (see `CLEANROOM.md`). "Confirmed" means
a file shows it; "assumed" means the reader relies on it without a file
that tests it.

## Container

- A RIFF form `CMX1`, little-endian (`RIFX` files, big-endian, are refused
  with a message). Confirmed.
- Chunks in file order; `LIST` chunks hold more chunks. Confirmed.

A chunk can declare a size short of its contents: pattern files written
by version 7 leave 16 bytes after `ccmm` before `page`. The reader skips
to the next bytes that read as a chunk tag (within 64 bytes) instead of
stopping there. Confirmed on ten such files.

## `cont`: the file header

- A header string, the operating system, the byte order, then at byte 52
  the coordinate size as an ASCII digit: `4` (32-bit coordinates) or `2`
  (16-bit). Only `4` is read; other files are refused. Confirmed.
- At bytes 64 to 72 the unit as an f64: 0.0001 mm (1e-7 m) in every file
  seen. Confirmed.

## Coordinates

- In the unit of `cont`, the origin at the page's centre, y up.
  Confirmed.

## Tables

Objects refer to table entries by 1-based index. A table is a u16 count,
then that many field lists (a field: u8 tag, u16 length counting the 3
header bytes, the data; 0xff after the last field). Confirmed.

| Chunk | Entry |
|---|---|
| `rclr` | colour: field 10 holds the colour as `.cdr` records do (u32 length, then parts of a u8 id, a u32 length and data; part 1: model u16, palette u16, 4 bytes, 4 value bytes); older files only fields 1 (model, palette) and 2 (values). Confirmed. |
| `rpen` | pen: width (u32), aspect in percent (u16), nib angle (i32), then a matrix. Confirmed. |
| `rott` | line type: flags (1 no outline, 2 solid, 4 dashed, 8 behind the fill, 16 scaled with the object), then caps (low 4 bits) and join (high 4 bits). Confirmed. |
| `rdot` | dash pattern: a u16 count, then the lengths in pen widths. Confirmed. |
| `rotl` | outline: line type, screen, colour, arrowhead pair, pen and dash pattern (u16 each). Confirmed. |
| `rota` | arrowhead pairs: a u16 count, then start and end arrowheads (u16 each, 1-based in `rarw`, 0 for none). Confirmed. |
| `rfnt` | fonts, field lists without a count: field 1 a u16 length and the name in 8-bit characters ending in 0, field 2 10 classification bytes, field 4 the name as a u32 count and UTF-16. Confirmed on one sample. |
| `rarw` | arrowheads, field lists one after the other without a count: field 1 is a u16 point count, the points (i32 x, y), one node byte each and a u32 (how far the line is pulled back, not used). The line ends at the origin and comes in from -x; heads keep that placement (circles centred on the end point, hooks above or below the line). 127 000 units are one outline width. Closed subpaths are filled, each on its own (an inner one is no hole); open subpaths are lines one outline width wide. At the start of a line the head is mirrored across the line, not turned round. Confirmed against the preview of the one public sample that has heads; the pull back is not used. |

## `page`: instructions

Each page chunk is a stream of instructions: a u16 size (the whole
instruction), an i16 code, then field lists. Confirmed. A size of 0xFFFF
marks the long form: a u32 size (the whole instruction, header of 8
bytes included), then the code. Confirmed on a version 2 export whose text
object styles (101) and characters (102) use it.

| Code | Instruction |
|---|---|
| 9, 10 | begin and end a page; field 4 holds the page size (u32 width, height) |
| 11, 12 | begin and end a layer; the name in field 4 (u32 count, UTF-16) or at the end of field 1 (8-bit) |
| 13, 14 | begin and end a group |
| 20, 21 | begin and end a text object (its text stream is instructions 98 to 102) |
| 101 | a character style of the text object (long form): field 1 a u16, then rendering attributes; the styles are numbered from 0 in order |
| 102 | the text object's characters: field 1 a u32 count, that many UTF-16 units, then the style number of each character (u16; 0xFFFF for none, as on spaces) |
| 65 | glyph run: field 1 a u16 count, that many UTF-16 units, then each character's place from the baseline's start (i32 x, y); fields 2 and 3 the text again and the glyph numbers. A text in several colours is drawn by one run per colour, each run leaving out the other runs' characters (the places jump over them); a run whose character style (85) names no fill takes the style (101) of its characters, found by where they sit in the text (102) |
| 66 | ellipse |
| 67 | curve or polygon: field 1 the rendering attributes, field 2 the points, a node byte each |
| 68 | rectangle: field 1 the attributes, field 2 the geometry |
| 70, 71 and 73 | bracket the drawn text (not read) |
| 72 | text placement: field 2 a matrix (u16 type; 2 is six doubles) whose translation is the baseline's start; field 3 the bounding box |
| 85 | character style: field 1 the rendering attributes, field 2 the font (u16, 1-based in `rfnt`) and size (u32, coordinate units); field 3 OpenType feature tags |
| 69 | image placement: its extent and crop (i32 each), a matrix (u16 type; type 2 is six doubles a, b, c, d, e, f with x' = a x + c y + e), a u16 and the image's number among the page's images |
| 88, 89 | add and remove a clip |
| 111 | jump to an absolute file offset; the bytes jumped over hold the page's images |
| 127 | colour context |

Codes not listed are reported once per file and skipped. Confirmed for
the codes above on the sample files.

## Rendering attributes

A mask (1 fill, 2 outline, 4 lens, 8 canvas, 16 container), then one
field list per part the mask names. Confirmed for fill and outline;
lenses, canvases and containers are skipped.

- Fill type 1, uniform: field 1 is the colour and the screen. Confirmed.
- Fill type 2, fountain: field 1 the settings (type: 0 linear, 1 radial,
  2 conical, 3 square; screen, edge pad in percent, angle i32, centre x
  and y offsets in percent, steps, blend mode, rate method and value),
  field 2 the colours (a u16 count, then colour index and position in
  percent pairs). Confirmed.
- Other fill types (bitmap, texture, pattern and the types 8 to 10 a
  sample uses) are reported and skipped.

## Embedded images

`LIST imag` chunks hold an `info` and a `data` chunk. `data`: a u8 kind
(2: a block follows), a u32 length, then a block `RI`, its length, 4
bytes, the header length (u32), the model, 1, width, height, planes, bits
per pixel and row length (u32 each), and after the header the rows,
bottom row first, blue, green, red for 24 bits. Confirmed for 24-bit
images; other depths are reported and left out.

## Not read yet

The rest of the text stream (instructions 98 to 100), lenses, the fills named above,
16-bit coordinate files and big-endian files.
