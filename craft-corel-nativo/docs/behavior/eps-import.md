# EPS import

File > Open and File > Import accept `.eps`, `.ps` and `.prn` (print
files: PostScript a printer driver wrote, often after a printer job
header, which is skipped up to the first `%!`), and `.ai` files that
start with `%!` (the PostScript-based versions of that format; the later
PDF-based ones go through the PDF import). A classic `.ai` file whose
header carries the format's version comments (`%AI5_FileFormat`,
`%%AI8_CreatorVersion` or an `%AI3_` comment) and whose prolog brings no
procedure set of its own, as the files of TraceDraw's Illustrator 8
export (see `export.md`, AI files), is run with the format's operators
defined first in PostScript: paths (`m`, `l`, `L`, `c`, `C`, `v`, `V`,
`y`, `Y`), painting (`f`, `F`, `s`, `S`, `b`, `B`, `n`, `N`, with the
fill and outline colours kept apart), compound paths (`*u`, `*U`),
clipping groups (`q`, `W`, `Q`), groups (`u`, `U`), process and grey
colours (`k`, `K`, `g`, `G`), outline settings and layers; operators
beyond those are reported as undefined and skipped (#1728,
`ai::tests::classic_ai_reads_back_with_its_page_and_objects`). A file that is not PostScript
(a `.prn` file in a printer's own language, for one) is refused with a
message. The file is run by
a PostScript interpreter (`crates/io/src/eps_import.rs`) that records
what the program paints; no external PostScript engine is involved.

## Interpreter

- Scanner: numbers (integers, reals, radix `16#ff`), names, strings with
  escapes and nesting, `<hex>` and `<~ascii85~>` strings, `{}`
  procedures, `[]` and `<< >>`, comments. DOS EPS binary headers are
  honoured (the PostScript section is used, previews ignored).
- Language: operand, dictionary and graphics-state stacks; `def`,
  `bind`, `exec`, `if`/`ifelse`, `for`, `repeat`, `loop`, `exit`,
  `stop`/`stopped`, `forall`, arrays, strings (`search`, `cvs`, `cvn`,
  `cvx`, `token`), dictionaries (`where`, `known`, `load`, `store`),
  arithmetic, relational and bit operators, `save`/`restore`,
  `currentfile` with `readstring`, `readhexstring`, `readline`, `token`,
  and `filter` for ASCIIHex, ASCII85, RunLength, Flate and DCT (lazy on
  the main file, so data after the consuming operator is read from the
  right place), `eexec` (the encrypted font program is skipped to its
  `cleartomark`), resource operators as stubs.
- Graphics: the path operators including `arc`, `arcn`, `arct`,
  `rect*`; `fill`, `eofill`, `stroke`; gray, RGB, HSB, CMYK, `setcolor`
  under `setcolorspace` (Device spaces, ICCBased by N, Indexed with
  string or procedure lookups, Separation and DeviceN as tints);
  line width, caps, joins, dashes scaled by the CTM; `clip`, `eoclip`,
  `rectclip`, `initclip`, `clippath`; all matrix operators;
  `gsave`/`grestore` restore the current path too, so
  `gsave fill grestore stroke` strokes the same path.
- Images: `image`, `imagemask` and `colorimage` in the Level 1 operand
  form and the Level 2 dictionary form, 1 to 16 bits, 1/3/4 components,
  single or multiple data sources (read row by row in turn), Decode
  arrays, JPEG through DCTDecode. The image matrix places the bitmap.
- Shadings: `shfill` and shading patterns (`makepattern`/`setpattern`
  with PatternType 2) become gradient fills from type 2, 3 and 0
  functions or a procedure; tiling patterns (PatternType 1) run their
  PaintProc into a vector pattern tile.
- Text: `show` and the spacing variants, `glyphshow`, `charpath`
  (shown as text) make artistic text objects with the font's name and
  size from the font matrix and the CTM; glyph outlines are not
  interpreted (Type 1/Type 3 programs), so the installed font of the
  same name draws the text and advances use 0.5 em per character.
- Clipping wraps painted objects in clip frames the way the PDF importer
  does; the page is the `%%HiResBoundingBox`/`%%BoundingBox` with its
  lower-left corner at the origin.
- Pages: each `showpage` (or `copypage`) ends a page; File > Open makes
  one page per page of the file, each at the size `setpagedevice` gave
  it (`/PageSize`), else the bounding box, else (one page) the bounds of
  its content, else A4. Objects after the last `showpage` make a page
  when there are any. File > Import places the first page's objects.

## Limits

6 million operations, 200 nested procedure levels, 50 000 operand
stack entries, 64 MB strings; past any of them the import stops with a
warning and keeps what was painted. LZW filters and font programs are
not run (warnings). Unknown operators are reported once and ignored.

## Checks

- Paths: `gsave fill grestore stroke` gives a blue fill with a red 4 pt
  outline; `rectfill` places a gray rectangle at (100, 50).
- A prolog with `def`, `bind`, `for`, `forall`, `ifelse`, nested
  dictionaries runs with no warnings and paints the expected boxes.
- `arc` under `translate`/`scale` gives a 40 pt circle; `rectclip` makes
  a clip frame.
- `findfont`/`scalefont`/`setfont`/`show` and `selectfont` give text
  objects at 12 and 10 pt; a `colorimage` from a hex string gives a 2 x 1
  bitmap; a rectangle after an `eexec` section is still painted.
- A Level 2 image dictionary with `currentfile /ASCII85Decode filter`
  decodes the pixels and leaves the program position after `~>`; the
  DOS EPS header is honoured.
- A shading pattern fill becomes a linear gradient.
- Garbage is an error; `{ } loop` and unbounded recursion stop with a
  warning; unbalanced delimiters never panic.
- Real files checked by eye: an AI logo traced with AutoTrace, two cairo
  exports, a GIMP RGB image (three `RunLengthDecode` sources).
