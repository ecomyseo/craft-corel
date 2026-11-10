# Export

File > Export (Ctrl+E) and File > Export Presets > Web or Documents open the
Export dialog. File > Export starts with the choices of Options > Global
> File Formats, which can also keep those of the last export; the Web and
Office presets start with their own (see `options.md`).

## Layout

- On the left, a 260 x 300 point preview of what will be written: the
  page, or the selection with Selected only, rendered to fit, on a
  checkerboard when the format keeps transparency and Transparent
  background is on, else on white.
- On the right: Format, then the format's own settings, then Selected
  only and the export range, then the estimated file size.
- Export range (for every format, hidden with Selected only, which
  exports the selection on the current page; #1382): Current page, All
  pages, or Pages: with a list such as "1-3, 5" ("4-" runs to the last
  page; the pages go in the order listed; a list naming no page of the
  drawing exports every page, as Print does). All pages starts on for
  PDF, PostScript and PRN and off for the other formats; a format
  exported before comes back with All pages or Current page as it was
  exported (a page list comes back as All pages, since it belongs to
  one drawing; #1161, `prefs::tests::pdf_exports_every_page_by_default`).
- Spreads (shown when the drawing has facing pages, not with Selected
  only, #1972): each spread of the chosen pages is written as one page
  as wide as its pages side by side, numbered by spread in files of one
  page; off, each page is written alone with what its facing page holds
  across the spine and in the bleed beside it (see `facing-pages.md`).
- PDF, AI (PDF-compatible), PostScript, PRN and HTML write one file holding the pages
  chosen (a multi-page PDF). The other formats (SVG, EPS, AI of version 8, DXF, EMF, WMF,
  PLT, PSD and the bitmap formats) hold one page: with more than one
  page they write one file per page, the name given with the page's
  number after a dash ("name-1.png", "name-3.png" for pages 1 and 3);
  one page keeps the name given. Under the range a line says what will
  be written ("3 pages in one file.", or "3 pages: one file per page,
  named with its page number"), and the status bar names the first and
  last file (`export::tests::export_writes_the_pages_chosen`,
  `export::tests::export_page_choices_pick_pages`).

| Format | Settings |
|---|---|
| PNG, WebP, GIF | resolution (1 to 2400 dpi, presets 72, 96, 150, 300), transparent background; WebP also quality (the files are lossless; below 100 the colours are first rounded to fewer levels, WebP's near-lossless way: 1 low bit from 90, 2 from 75, 3 from 50, 4 below, so a lower quality makes a smaller file; #544, `export::tests::webp_quality_shrinks_the_file`) |
| JPEG | resolution, quality (1 to 100), colour mode (RGB, CMYK, Grayscale; see Colour mode below) |
| TIFF | resolution, colour mode |
| BMP, PSD | resolution |
| PDF | Export all text as curves (off at first: every text becomes filled outlines, so the file needs no fonts and cannot be searched; #1382, `export::tests::export_pdf_text_as_curves_holds_no_font`), standard (PDF 1.4, PDF/X-1a, PDF/X-3, PDF/X-4), bleed, output condition for PDF/X; the author, subject and keywords come from the drawing's properties (File > Export as PDF has bookmarks and protection, see `publish-to-pdf.md`) |
| AI | Compatibility: PDF-compatible (the default) or Illustrator 8 (EPS-based); see "AI files" below (#1728) |
| SVG, EPS, DXF, HTML, EMF, WMF, PLT | none |
| PostScript (`.ps`), PRN (`.prn`) | none; every exported page in one file (All pages, else the current page), each page setting its size with `setpagedevice` and ending with `showpage` (Document Structuring Conventions 3.0); a `.prn` print file is the same PostScript, to send to a PostScript printer |

- Bitmap formats show Size: a width and a height and their unit (px,
  mm, cm or in, kept with the other choices; #617). In pixels the size
  is the page (or selection) size over 25.4 times the resolution,
  rounded up, and typing a width or a height sets the resolution that
  gives it, so a 32 x 32 pixel icon is typed rather than worked out. In
  a length it is the printed size: typing a width or a height sets the
  scale of the artwork (the resolution stays), from 1 % to 10000 % and
  never past what 2400 dpi can draw; a button with the scale ("200 %")
  shows when it is not 100 % and puts it back. Either way the other side
  follows the page's proportions.
- Under the size: what comes out, "1240 x 1754 pixels, 150 dpi, sRGB"
  ("sRGB + alpha" with a transparent background, "CMYK" or "Grayscale"
  in those colour modes).
- PNG, JPEG and BMP files record the resolution of the dialog (PNG
  `pHYs`, the JFIF density, the BMP header's pixels per metre), so a
  scaled export opens at the printed size chosen; the scale multiplies
  the pixels, not that resolution (#617,
  `export::tests::bitmap_exports_record_their_resolution_and_scale`,
  `export::tests::the_export_size_is_typed_in_pixels_or_lengths`).
- File size: vector formats (SVG, EPS, PostScript, PDF, AI) are written in memory and
  measured; bitmap formats are estimated by encoding the preview the same
  way and scaling by the pixel count of the export (times the pages
  exported); a format of one page per file counts the first page's file
  once per page. It shows as "about 820 KB" or "about 3.4 MB".
- Text in PDF and PDF-compatible AI files is text with its fonts embedded as subsets
  (in PDF unless Export all text as curves is on),
  as Export as PDF writes it by default (see `publish-to-pdf.md`, Text
  and fonts). Vector formats write curves with their own segments:
  quadratic segments (TrueType letters, for example) become the cubic
  that draws the same curve in PDF, AI, EPS, PostScript and EMF, and
  stay quadratic in SVG; only DXF, WMF and PLT, which have no curves of
  that kind, approximate them with short lines.
- The preview and estimate are worked out again only when the drawing,
  the page, the selection or a setting changes.
- OK asks where to save; the file name is the drawing's with the format's
  extension.

## Colour mode

JPEG and TIFF have Colour mode (#1763): RGB (the default, the file as
before), CMYK or Grayscale. Other formats are always RGB and ignore it.

- CMYK writes the page's inks, made exactly as the print separations
  make them: each process ink is the page's plate for that ink (every
  colour as its coverage of that ink, black = full ink) rendered at the
  export's resolution on white, so the paper is no ink and anti-aliased
  edges, transparency and fountain fills blend as on the plates. CMYK
  colours keep their exact values (a fill of C 100 is cyan 100 % with no
  magenta, yellow or black); RGB, Lab and other colours go through the
  drawing's colour management (`Color::convert_to("CMYK")`, the CMYK
  profile when one is used, else the built-in press model, with Map gray
  to black); pictures that keep their ink channels (a `.cdr` CMYK image,
  #1659) give those inks and other pictures have their pixels converted
  the same way; spot colours print as their process mix at their tint
  and registration colour as 100 % of every ink. Spot channels a
  picture keeps are left out (they have no plate in a CMYK file).
- A CMYK JPEG is a baseline JPEG of four components, no subsampling,
  with the Adobe marker (transform 0, no colour transform) and the
  components stored inverted (255 = no ink), the convention readers of
  CMYK JPEG files expect; Quality scales the example quantization table
  of the JPEG standard by `2 ^ ((50 - q) / 20)` (every step 1 at 100).
  Its resolution goes in an Exif block (X and Y resolution in dots per
  inch).
- A CMYK TIFF is uncompressed, PhotometricInterpretation 5 (separated)
  with InkSet 1 (CMYK), 8 bits per ink, the inks as they are (255 = full
  ink), in strips of about 64 KB, recording its resolution.
- Both carry the CMYK profile the colours were converted with: the
  drawing's own (a `.cdr` file's) while Colour Management uses it, else
  the CMYK profile picked under Colour Management; none with the
  built-in conversions (in a JPEG as `ICC_PROFILE` segments, in a TIFF
  as tag 34675).
- Grayscale writes one channel: the rendered page's colours through the
  Grayscale conversion of the colour management (the luminance), as an
  8-bit grey JPEG (with its JFIF resolution) or TIFF.
- The preview shows a CMYK export as it will print (the colours through
  CMYK and back, as View > Proof Colors) and a Grayscale one in grey; the
  estimated size comes from the preview encoded in the colour mode.
- The colour mode is kept with the other choices: in presets (presets
  saved before it existed are RGB), the last export of each format and
  the preferences (`export::tests::cmyk_bitmap_exports_keep_the_inks`,
  `prefs::tests::export_presets_keep_the_colour_mode`,
  `tracedraw_io::cmyk_bitmap` tests).

## Presets

Above the format: Preset, a list of every saved preset, each with the
format it was saved for in brackets ("Web PNG (PNG)"), "(none)" first,
whole from the moment the dialog opens whatever format it opens at; "+"
to save the dialog's choices under a name (typed in a field that opens
below, OK or Enter; the same name replaces that preset of the format), a
pencil to rename the chosen preset (a name another preset of the format
has is refused) and "−" to delete it. "−" asks first, in a row that
opens below the list, as the name field does: 'Delete the preset
"name"?' with Yes and No. Yes deletes the preset and leaves "(none)"
chosen; No keeps it, still chosen. Choosing another preset, "+", the
pencil or a script filter closes the question without deleting (#1816,
`pick_drag_tests::the_export_dialog_asks_before_deleting_a_preset`).

Choosing a preset of another format switches the dialog to that format.
Changing the format leaves the chosen preset unchosen ("(none)") while
the list stays whole (#612, the reporter's second request: the list
used to shrink to the chosen format's presets). Saving, renaming and
deleting act on the presets of the chosen format only, so a preset of
another format with the same name is kept. With a script filter chosen
none is chosen and "+" is greyed. A preset of a format this version does
not know shows by its name alone; its name must be free in every format
(`prefs::tests::export_presets_save_apply_rename_and_delete`,
`pick_drag_tests::the_export_dialog_lists_every_preset_and_a_format_change_unchooses`).

Choosing a preset
puts back its format, resolution, quality, transparent background,
selection only, all pages, the size's unit and scale, the colour mode
of JPEG and TIFF and the PDF
choices (standard, bleed, output condition). Presets are kept in the preferences
(`formats.export_presets`; #547,
`prefs::tests::export_presets_save_apply_rename_and_delete`).

## WebP

Lossless (off at first) and Quality (1 to 100). Off, the file is lossy
WebP at that quality, far smaller: a picture of 450 KB lossless comes to
about 50 KB at quality 10. The desktop builds encode it with libwebp;
the browser build, where that C library is not built, with a pure-Rust
VP8 encoder (the `vaam-image-webp` crate, MIT OR Apache-2.0), whose
files are somewhat larger than libwebp's at the same quality but follow
Quality the same way and keep transparency (#592,
`export::tests::the_browsers_lossy_webp_follows_the_quality`). On, it is
lossless, and below 100 the colours are first rounded to fewer levels
(near-lossless, #544, `export::tests::webp_quality_shrinks_the_file`).
A picture larger than WebP allows (16383 pixels a side) is written the
lossless way in the browser.

## Starting choices

File > Export starts with the choices of the last export of the session
(Selected only off again, the file's title and tags empty), else those
of Options > Global > File Formats, which keep the last
export's choices for the next session while Remember the settings of the
last export is on (on; preferences from before 0.19.4 are brought up to
date once, #691; `options.md`, #617).

Each format also keeps how it was exported last: choosing a format in the
dialog brings back its resolution, quality, transparent background,
WebP lossless, all pages, size unit and scale and PDF choices (Selected
only and the preset stay as they are), from this session's exports and,
with Remember the settings of the last export on, from earlier ones
(`formats.export_by_format`). A format not exported yet keeps the
dialog's choices, and Export for Web and Office keep their own (#617,
`prefs::tests::each_export_format_comes_back_as_it_was_last_exported`).

## Checks

- `export::tests::the_export_preview_shows_the_page_and_estimates_the_size`
- `export::tests::export_writes_the_pages_chosen`, `export::tests::export_page_choices_pick_pages`, `export::tests::export_pdf_text_as_curves_holds_no_font`
- `pick_drag_tests::the_export_dialog_starts_with_the_last_exports_choices`
- `export::tests::sizes_read_in_kilobytes_and_megabytes`
- `ui::dialogs::tests::a_pixel_size_sets_the_resolution_that_gives_it`
- `pick_drag_tests::the_export_dialog_lists_every_preset_and_a_format_change_unchooses`
- `export::tests::bitmap_exports_record_their_resolution_and_scale`
- `export::tests::the_export_size_is_typed_in_pixels_or_lengths`
- `prefs::tests::each_export_format_comes_back_as_it_was_last_exported`
- `export::tests::cmyk_bitmap_exports_keep_the_inks`, `prefs::tests::export_presets_keep_the_colour_mode`

## Format list and file properties

- The Format list shows each format as "EXT - Description (*.ext)": JPG,
  PDF, AI and PNG first, a separator, then every format in the order of
  its extension, then installed script filters.
- Under the format's settings: Selected only, the export range, and "Do not
  show filter dialog": when on, File > Export (Ctrl+E) goes straight to
  the file with the last choices (Options > Global > File Formats keeps
  them); Export for Web and for Office still show their dialog.
- PDF and AI files: Title (the drawing's title when empty) and Tags,
  written as the file's title and keywords (a classic AI file keeps the
  title in its `%%Title` comment and has no place for tags).
- Checks: `ui::dialogs::tests::the_export_list_puts_the_common_formats_first`.

## AI files

The AI format comes in two kinds, chosen under Compatibility when the
format is AI (#1728). The choice is kept with the other choices of the
last AI export and in export presets.

- PDF-compatible (the default, and what every earlier version wrote): a
  PDF with the `.ai` extension, as the newer versions of that format
  are. It keeps everything the drawing holds (text with its fonts,
  fountain fills, pictures, transparency) and holds every page chosen.
  The default stays so that nobody's exports change without asking.
- Illustrator 8 (EPS-based): the classic file of version 8, PostScript
  as the public "Adobe Illustrator File Format Specification" describes
  it, for programs that read only those files (3D and engraving
  programs among them). The file starts `%!PS-Adobe-3.0` with
  `%%Creator: TraceDraw`, `%%AI8_CreatorVersion: 8.0`, `%%Title`,
  `%%BoundingBox` and `%%HiResBoundingBox` (the page, in points, from
  0 0), `%AI5_FileFormat 4.0`, `%AI3_ColorUsage`, `%AI3_TemplateBox`,
  `%AI3_TileBox`, `%AI5_ArtSize` and `%AI5_NumLayers`; its prolog and
  setup are empty (the file names no procedure set). Each printable,
  visible layer is a layer of the file (`%AI5_BeginLayer`, `Lb`, `Ln`,
  `LB`). Objects are written in the format's own operators:
  - paths with `m`, `L` and `C` (quadratic segments as the cubic that
    draws the same curve), painted with `f`/`F` (fill), `s`/`S`
    (outline), `b`/`B` (both) or `n`/`N` (neither), the lower case for
    closed paths; an object of several subpaths is a compound path
    (`*u` ... `*U`). Even-odd filled objects have their nested subpaths
    turned to run against the ones around them, so a winding fill keeps
    their holes; subpaths crossing each other may fill differently.
  - groups as `u` ... `U`; clip frames and fountain fills as clipping
    groups (`q`, the clipping path marked `W` and painted `n`, the
    contents, `Q`).
  - colours as process colours: `c m y k k` for the fill and `K` for the
    outline; RGB and other colours are converted with the built-in
    conversion (black = 1 - the largest of R, G, B), spot colours as
    their process equivalent, Registration as 100 % of every ink.
  - outlines with `w` (width in points, 0 for hairlines), `j`, `J`,
    `M` and `[...] 0 d` (dashes scaled by the width); arrowheads as
    filled paths.
  - text as filled curves (the letters' outlines), always.
  - fountain fills as 64 bands of solid colour (neighbouring bands of
    the same colour merged): strips across a linear fill's direction,
    discs for a radial fill (conical and square fills as linear), inside
    a clipping group of the object, its outline over them.
  - pattern, texture and mesh fills as their average colour; live
    effects (blends, contours, extrusions and the like) as the objects
    they draw, grouped.
  - pictures, and the bitmap effects of vector objects (shadows,
    blurs), are left out: the vector object is written without them;
    transparency is not kept. Use PDF-compatible for those.
  One file holds one page (one artboard, whose box is the page): with
  several pages each is written to its own numbered file, as for EPS.
  TraceDraw opens these files again through its PostScript reader,
  which defines the format's operators for a classic AI file that
  brings no procedure set of its own (see `eps-import.md`).
  Whether a given program accepts the file depends on what that program
  checks; the files follow the specification, but no other program was
  available to test them.
- Checks: `export::tests::export_ai_writes_pdf_compatible_or_classic_version_8`,
  `ai::tests::classic_ai_starts_with_the_header_the_format_asks_for`,
  `ai::tests::classic_ai_groups_paths_and_colours_balance`,
  `ai::tests::a_ring_s_hole_runs_the_other_way`,
  `ai::tests::fountains_become_clipped_bands_and_pictures_are_left_out`,
  `ai::tests::classic_ai_reads_back_with_its_page_and_objects`.
