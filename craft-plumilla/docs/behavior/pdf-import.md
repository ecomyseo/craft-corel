# PDF and AI import

File > Open and File > Import accept `.pdf` and `.ai` (PDF-compatible
AI files, which carry a PDF after a PostScript preamble). Open
makes one page per PDF page. Import of a file of one page places its
objects on the active layer; a file of several pages asks which pages
come in and where (below).

## Importing several pages

When the file read for File > Import has more than one page, the Import
PDF Pages dialog asks, after the Import PDF dialog (how to read the
text) and the password of a protected file, so nothing is asked twice:

- Pages to import: All (the default), or the pages typed as numbers
  and ranges from 1 ("1-3, 5"). The list below shows each page with
  its size in the document's units and its orientation, ticked when it
  is chosen; ticking a page writes the range. Pages typed out of order
  or twice come in once each, in the file's order; numbers past the
  last page are left out. OK with no page chosen says so and keeps the
  dialog open.
- Place them:
  - As new pages after the current page (the default): each chosen page
    becomes a document page with that page's size and orientation (its
    crop box and `/Rotate`, as Open reads them), in order, right after
    the current page, with one layer holding its objects where the file
    has them. The first new page is shown with its objects selected.
    The placement cursor and the view centre (Options > File Formats)
    do not move them.
  - On the current page: the objects go on the active layer, as for a
    file of one page; with several pages chosen each page's objects are
    one group, named after its page, stacked in page order, all
    selected. A place given with the import cursor, or the view centre
    when Options > File Formats says so, applies to them all.
- The file's spot inks join the drawing's as below, and everything
  (inks, pages, objects) is one undo step ("Import"). Pictures of the
  chosen pages that came in smaller are told as below.

A file of one page comes in without the dialog, as before. Taking pages
of several files into one drawing this way merges PDFs; Export or
Publish to PDF with a page range splits them. The dialog is the same on
the desktop, in the browser and on Android: the file is read once, and
the dialog keeps what was read until OK or Cancel.

## What is read

- Page geometry: MediaBox and CropBox (the crop box becomes the page),
  /Rotate 90, 180 and 270. Points become millimetres, the crop box's
  bottom-left corner is the origin.
- Paths: all construction operators, fill rules (even-odd is recorded as
  object data), strokes with width, caps, joins and dashes scaled by the
  CTM, constant alpha from ExtGStates (`ca`, `CA`).
- Colour: DeviceGray/RGB/CMYK, ICCBased (by component count), CalRGB/Lab
  (as RGB), and the special spaces of ISO 32000-1, 8.6.6 (#1703):
  - Separation: the operand is a tint, 0 no ink and 1 full ink. The
    colourant `All` is the registration colour at full tint (crop and
    registration marks, as Export as PDF writes them) and that share of
    every process ink below it; `None` paints nothing (the fill or outline
    goes); `Cyan`, `Magenta`, `Yellow` and `Black` are that process ink.
    Any other name is a spot ink: the drawing gets an ink of that name
    (Color docker, separations), one per name, whose process stand-in is
    the alternate space's colour at tint 1 from the tint transform, and
    the object a spot colour at the tint. A tint transform that cannot be
    evaluated (a PostScript calculator function) leaves black as the
    stand-in, with a warning.
  - DeviceN: one colourant reads as its Separation. Several mix as
    process inks: the process names as themselves, the others by the
    stand-in of the Separation space the attributes' `Colorants` give them
    (black, with a warning, without one).
  - Indexed: the operand picks the entry of the lookup table (clamped to
    `hival`), read in the base space, so a Separation base gives spot
    colours too.
  - `cs` and `CS` start a Separation or DeviceN space at full tint and an
    Indexed space at entry 0.
  - Open puts the file's inks in the new drawing. Import adds them to the
    drawing's inks in the same undo step: an ink of a name the drawing
    has is that ink, the others get numbers of their own.
  - Pictures in a Separation space (samples and DCT) and DeviceN space
    show their tints as ink the same way; their inks are not added to the
    drawing, the picture keeping its pixels.
- Clipping (ISO 32000-1, 8.5.4): the clipping path is set after the
  painting operator that follows `W` or `W*`, is saved and restored with
  `q`/`Q`, and each new one is intersected with the one in force (the
  exact overlap of the two areas; a rectangle round the old clip leaves
  it as it is). `W` reads the path with the nonzero rule: it is turned
  into its nonzero area, so an outline drawn twice or the middle of a
  self-crossing star stays inside; `W*` reads it even-odd. Subpaths with
  no segment (a lone `m` before `W n`) are left out. A form's `BBox`
  clips its content the same way. The objects drawn one after the other
  under the same clip go together in one clip frame whose frame is that
  clip; another clip, or an object the clip leaves alone, closes it.
  Objects are judged by the box their ink covers (the outline's box
  grown by the stroke's reach): one outside the clip is dropped, one
  inside a rectangular clip goes on the page without a frame. Text that
  is at least 60 % inside a rectangular clip is left unclipped because
  substitute fonts change its width.
- Form XObjects (recursively, 12 levels), annotation appearance streams
  (stamps, form fields; links and popups skipped).
- Images: Flate/LZW/raw samples at 1, 2, 4, 8 and 16 bits with Gray, RGB,
  CMYK, Indexed, Separation and DeviceN spaces, Decode arrays, image masks (filled with
  black), SMask soft masks (resampled nearest), DCTDecode through the
  JPEG decoder. JPX images become a grey box. Inline images (`BI`) are
  read with their abbreviated keys expanded.
- Large pictures (#1835): every picture comes in at its own size, an
  11160 x 14794 picture with a soft mask as 11160 x 14794. A Flate
  picture without a predictor is inflated a row at a time and the soft
  mask is read before the picture's pixels, and the PNG is written
  straight from them, so reading takes about the decoded picture
  (4 bytes a pixel) and its mask's 1 byte a pixel. Only past the
  memory guard (`ImportOptions::max_picture_pixels`, by default
  `FULL_PICTURE_PIXELS`: 375 megapixels, 1.5 GB of RGBA), or when the
  memory for the result cannot be had (then by 2, 4, ... until it
  can), is a picture shrunk by the smallest whole factor that brings it
  within, each pixel the mean of a block of the picture's own (its soft
  mask shrunk the same way), with the warning "image of W x H pixels
  shrunk to w x h". It keeps its place and size on the page.
- Reduce very large pictures (the Import PDF dialog and Options > File
  Formats, `pdf_reduce_pictures`, off by default; always on in the
  browser and on Android, where the box shows ticked and greyed): the
  limit is `REDUCED_PICTURE_PIXELS`, 80 megapixels and 20000 pixels on a
  side; the picture above comes in at 5580 x 7397.
- A picture that came in smaller than the file holds it is told in a
  message after the file opens or is imported ("N pictures of NAME
  came in smaller than the file holds them (W x H to w x h pixels)"),
  whatever Options > Global > Warnings says, after the missing fonts'
  dialog when that comes first (`ImportedPdf::shrunk_pictures`). Until
  #1835 such pictures were shrunk past 80 megapixels on every device,
  told only in the log.
- Only a picture of more than 400000 pixels on a side or 1.6
  gigapixels, or one whose data needs more than 1 GB inflated at once
  (filters other than Flate alone), is skipped ("too large; skipped").
  Until #1804 every picture past 20000 pixels or 80 megapixels was
  skipped, leaving only the vectors around it.
- Shadings: axial (type 2) and radial (type 3) as gradient fills, with
  type 2, 3 (stitching) and 0 (sampled) functions sampled into stops;
  `sh` paints the current clip. Shading patterns used as fills become
  gradient fills; tiling patterns become vector pattern fills built from
  the tile's content.
- Text: `BT`/`ET` blocks with Tf, Td, TD, Tm, T*, TL, Tc, Tw, Tz, Ts, Tr,
  Tj, TJ, ' and ". Consecutive strings on the same baseline merge into
  one artistic text object; a gap over 0.18 em (or a TJ adjustment under
  -180) inserts a space. Font family comes from BaseFont without the
  subset prefix (`Arial-BoldMT` becomes Arial, bold), bold and italic
  from the name, the descriptor flags and StemV. Characters decode
  through ToUnicode CMaps (bfchar, bfrange), then the font's Encoding
  (WinAnsi, MacRoman, Standard, Differences with glyph names), then
  Latin-1. Advances use Widths, W/DW for CID fonts, and Helvetica metrics
  for the standard 14. Render modes 3 and 7 (invisible) are skipped.
  Type 3 glyph procedures are not run.
- Text as curves (#386, `ImportOptions::text_as_curves`): each string
  (`Tj`, each part of `TJ`) becomes one filled curve of its glyphs'
  outlines, read from the embedded font program: `FontFile2` (TrueType)
  and `FontFile3` of subtype `OpenType` as sfnt fonts, `Type1C` and
  `CIDFontType0C` as bare CFF. A code reaches its glyph through the
  `CIDToGIDMap` (or the identity) for two-byte TrueType fonts, the CFF
  charset for CID-keyed CFF, the font's `cmap` ((3,0) at 0xF000 + code,
  then (1,0), then Unicode from the encoding) for simple TrueType fonts,
  and the CFF built-in encoding (else the glyph's standard name) for
  simple CFF fonts (`pdf_glyphs.rs`). Each glyph sits at
  `Trm * translate(advance) * scale(Tfs * Th, Tfs)` with the font's units
  per em (CFF: its FontMatrix); the fill colour and opacity are the
  text's, clipping as for text. Text whose font is not embedded (or a
  Type 3 font) stays text, and the app turns it into curves with the
  installed font that stands in (`remaining_text_to_curves`).
- The app asks how to read the text when a PDF or AI file is opened or
  imported (the Import PDF dialog: Text or Curves, Do not ask again);
  Options > File Formats keeps the choice (`pdf_text_as_curves`,
  `pdf_ask_text`).

## Not read

Soft masks on groups (`SMask` in ExtGState), blend modes, transfer
functions, optional content visibility, JPX and CCITT images (grey box
and skipped respectively), PostScript calculator functions (linear
approximation); as text, embedded font programs (text uses the installed
font of the same name, so widths can differ; as curves they are read).

## Protected files

Files protected by the standard security handler open in every revision
(2 to 6): RC4 with 40 to 128-bit keys, RC4 and AES-128 crypt filters,
and AES-256 (revisions 5 and 6), with strings and streams decrypted as
they are read, object streams and cross-reference streams included.

- A file that opens without a password (only its permissions are
  protected) opens at once.
- A file with a password to open it asks for it (Protected PDF: the
  file's name, a masked field, OK and Cancel); the user or the owner
  password opens it. A wrong one asks again and says so; Cancel leaves
  it closed and says why in the status bar. Import asks the same way.
- On the command line `TRACEDRAW_PASSWORD` gives the password.
- Other security handlers (certificates) are refused with a message.

How: the parser never decrypts. The trailer's `/Encrypt` key is renamed
(same length, so the offsets hold), a first read takes only the
dictionaries to find the encryption dictionary, the key comes from the
password (algorithms 2 and 6 for revisions 2 to 4: MD5 of the padded
password, `/O`, `/P` and the file ID, 50 more rounds from revision 3,
and the `/U` check; algorithm 2.A for 5 and 6: the owner's `/O` and
`/OE` first, then `/U` and `/UE`), and a second read decrypts each
object as it comes in (per-object keys from MD5 of the key, the object
number and generation, and `sAlT` for AES-128; the file key as it is for
AES-256). The members of an object stream are not decrypted again;
cross-reference streams, metadata that is not encrypted, streams with
their own crypt filter and signature contents are left alone.

## Checks

- `pdf_import::tests::protected_files_of_every_revision_open_without_a_password`:
  files protected by another implementation with revisions 2, 3, 4 (RC4
  and AES-128) and 6.
- `pdf_import::tests::our_protected_files_open_unless_they_need_a_password`
- `pdf_import::tests::object_streams_of_protected_files_are_decrypted`
- `pdf_import::tests::damaged_protection_is_an_error_not_a_panic`
- `pdf_crypt::tests::rc4_matches_the_published_vectors`,
  `what_is_encrypted_decrypts_again`,
  `revision_5_files_open_with_either_password`
- `ui::pdf_password::tests::protected_pdfs_ask_for_their_password`
- Several pages: `ui::pdf_pages::tests::importing_a_pdf_of_several_pages_asks_which_and_where`
  (the text question, then the pages; nothing chosen; pages 3 and 1 as
  new pages after the first of two, at 210 x 297 and 100 x 50 mm, in
  one undo step; two pages on the current page as two groups),
  `a_pdf_of_one_page_comes_in_without_asking`,
  `page_lists_read_and_write_ranges`, and in the core
  `command::tests::inserted_pages_keep_their_sizes_and_undo_as_one_step`.
- Checked by hand: files protected by an independent PDF tool in
  revisions 2, 3, 4, 5 and 6, with and without object streams, convert to
  the same SVG as the unprotected file.

- A page with `MediaBox [0 0 200 100]` gives a 70.56 x 35.28 mm page.
- `1 0 0 RG 0 0 1 rg 4 w ... B` gives a blue fill with a red 1.41 mm
  outline.
- `(Hello) Tj ( World) Tj` on one line gives one text "Hello World" at
  12 pt bold; `[(Se) -250 (cond)] TJ` gives "Se cond".
- A form XObject with a Matrix is placed by it and takes the page's
  `ca`; a rectangle painted under `W n` becomes a clip frame.
- A disc set as the clip, then twelve lines crossing it, gives one clip
  frame with the disc as frame and every line that reaches the disc
  inside it; lines that miss it are dropped (#1616).
- Two overlapping discs set as clips one inside the other give a clip
  frame shaped as their lens; after `Q` the first disc clips again.
- A type 2 axial shading pattern becomes a two-stop linear gradient.
- `/Separation /All` at `1 SCN` gives a registration-colour outline,
  `/Cyan` 100 % cyan, `/None` no outline, a spot name at `0.4 scn` a spot
  colour at 40 % on an ink of that name whose stand-in is the alternate
  colour at tint 1; an Indexed index takes its table entry; a DeviceN of
  `Cyan` and `Black` gives C and K (`pdf_import::tests::separation_devicen_and_indexed_colors`).
  A 1-bit Separation picture shows 1 as the ink and 0 as white
  (`separation_picture_tints_are_ink`). A drawing published with crop
  marks and a spot colour opens with registration-colour marks and the
  spot colour on its ink (`pdf::pdfx_tests::crop_marks_and_spot_colors_come_back_from_the_pdf`);
  imported into a drawing, the inks join the drawing's by name
  (`app::tests::pdf_spot_inks_come_into_the_drawing`) (#1703).
- A raw 2 x 1 RGB image gives a 2 x 1 bitmap at the CTM's rectangle,
  also when the file starts with a PostScript preamble (`.ai`).
- A 40000 x 2 RGB picture with a soft mask comes in whole, and past a
  small limit shrunk and listed
  (`pictures_come_in_whole_unless_past_the_limit`); with pictures
  reduced at 20000 x 1, each pixel the mean of its block and of the
  mask's (`oversized_picture_is_shrunk_with_its_soft_mask`); the factor
  for the picture of #1804 is 1, reduced 2, and without memory for the
  result the factor doubles until it fits
  (`shrink_factor_brings_pictures_within_the_limits`). The file of
  #1804 comes in at 11160 x 14794 (`the_picture_of_1804_comes_in_at_its_own_size`,
  run with `TRACEDRAW_PDF_1804` and `--ignored`; about 840 MB at most
  in a debug build). The message after opening:
  `a_shrunk_picture_is_told_after_opening`; the option:
  `pictures_are_reduced_only_when_asked_on_the_desktop`.
- Garbage and truncated input return an error, never a panic.

- Devanagari text (Hindi, Marathi, Nepali, ...) read in Text mode comes
  back in the order it is typed: a PDF draws the short i sign (ि) before
  the consonant cluster it belongs to and the reph (र्) after its
  syllable, and its text kept that drawn order, so the signs went on the
  wrong letters. A run is changed only when it shows the drawn order (an
  i sign at the start of a word or after another sign); a reph moves back
  only at the end of a word (#585,
  `pdf_import::tests::devanagari_text_from_a_pdf_comes_back_in_typed_order`).
  A reph after a consonant or a sign at the end of a word also shows the
  drawn order when the run holds a reph glyph (one glyph for र् with no
  advance); a ra drawn with its virama shown at the end of a word (Nepali
  गर्, Sanskrit पुनर्) stays as it is. In drawn order a reph before the next consonant moves
  back to its syllable too (आर्थिक, प्रार्थना); a reph in a conjunct
  (after a virama) stays (#599).
- Hindi that a PDF draws apart comes in as one word (#599): a string
  that continues the syllable before it (a vowel sign, virama, nukta or
  anusvara, the rest of a conjunct, a reph glyph, the short i sign placed
  back over its letter) stays in the same text when the producer moved it
  up to an em off the baseline or up to three ems back, and no space goes
  in before it for a gap or a `TJ` adjustment; a step back for a sign and
  on again is no gap either. A consonant after a virama joins only when
  it touches (no gap of more than 0.18 em, no big adjustment before it):
  many words end in a virama (परिषद्, अर्थात्), and the next word or table
  cell stays apart (परिषद् द्वारा). Vowels drawn as two glyphs, each mapped to
  its own character, become one character again: the pairs the Unicode
  Standard lists not to use for vowel letters (अ + ा for आ, ए + े for ऐ,
  ऊ = उ + ु, the Kashmiri vowels ॳ to ॷ, ...) and a vowel sign split
  into the aa sign and a sign on top (ा + े for ो, ा + ै for ौ). Shaped as they came, these showed dotted circles.
- Marked content with an `ActualText` (inline or named in the page's
  `Properties`): its text stands for the glyphs inside it, in place of
  what their `ToUnicode` gives; a text that takes characters from one is
  in typed order and is not reordered. The outermost `ActualText` wins.
  It applies only while all its glyphs stay in one text: when they go on
  in another (the next line), each text keeps its glyphs' own characters.
  An empty `ActualText` or one of soft hyphens leaves the glyphs (a
  hyphen drawn at a line end) their own characters
  (`pdf_import::tests::hindi_drawn_apart_or_with_actual_text_comes_in_as_typed_words`).
- Hindi written by Export as PDF with its font comes back as the typed
  words, shaped to the glyphs of the original: all 59 words of the
  Devanagari corpus (35 came back wrong before, with letters doubled,
  signs lost or a space inside the word)
  (`crates/io/tests/devanagari_pdf.rs`, a test binary of its own so the
  test font stays out of the other tests' fonts). Latin text gets no
  marked content. 30 more hard Hindi and Sanskrit words from the lists
  sent with #1806 come back the same way.
- Hindi from a word processor's PDF whose `ToUnicode` map is scrambled
  (#1806). Such a map gives the glyphs of each syllable, taken in drawn
  order, the syllable's characters in typed order: every glyph after the
  first gets one character and the first gets the rest. A glyph keeps the
  text of the first syllable it appears in, so in later words letters,
  signs and the reph swap (वार्ड came in as वार्ि, वर्ष as वषि). When the
  map contradicts the embedded font's own `cmap` on an Indic letter or
  sign, the map is put right for the whole file:
  - glyphs the `cmap` names take that character;
  - every other glyph takes, at its first appearance (all pages, in
    drawing order), the text that gives back the map's entries for the
    new glyphs of that string under that rule. The candidates are the
    pieces of those entries' text, the short i sign, the reph (only for
    a glyph with no advance) and the half form of a letter. Texts no other
    glyph stands for win, then the glyph's own entry, then texts starting
    with it, then shorter ones. A tie, or more than three such glyphs in
    one string, leaves the entries as they are.

  The text of such a font is in drawn order glyph by glyph, so every
  short i sign before a consonant moves after its cluster, also after
  another consonant (पिश्चम becomes पश्चिम).

  On the reporter's file, 56 of its 68 words now come in as typed (25
  before). The other 12 cannot be worked out from the file: their glyphs'
  characters went to a glyph seen earlier, so the map holds them nowhere.
  Those are the द्र conjunct in 8 words (पुष्पेन्द्र, देवेंद्र and so on,
  read as र), क्त in प्रतिनियुक्ति, थ् in स्वास्थ्य, the reph joined to
  the o sign in पूर्वोत्तर, and one word typed as प्रड्यूसर in the file.
  (`crates/io/tests/devanagari_pdf.rs`,
  `hindi_with_a_scrambled_tounicode_map_imports_as_the_typed_words`, and
  `pdf_import::tests::a_scrambled_tounicode_map_is_put_right`.)
