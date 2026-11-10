# Bitmap import

File > Import (Ctrl+I) of a PNG, JPEG, BMP, GIF, TIFF or WebP file
embeds the image as a bitmap object centred on the page and remembers
the file (Image > Update from Link reads it again).

File > Open and the headless MCP `open` tool also accept these raster files.
They create a new image-sized drawing; Save treats it as an imported document
and does not overwrite the source image with native JSON. A named raster file
must contain the codec its suffix describes.

## Original CMYK samples and profiles

The shared `tracedraw_io::picture` decoder keeps 8-bit CMYK and CMYKA TIFF ink
planes, unassociated alpha and the embedded ICC profile. Both interleaved and
separate-plane TIFFs are read in full, including BigTIFF containers. The source
profile is stored on `PictureInks`, per picture. Opening a tagged TIFF as a new
drawing initializes that drawing's CMYK working profile from the image; importing
into an existing drawing keeps its working profile. The display PNG is converted locally from that
profile to sRGB; no global colour engine is installed by the reader.
An untagged CMYK TIFF or CDR bitmap uses the same built-in press model, independent
of the global display calibration of any other open drawing. Previously the TIFF
reader could inherit that calibration while the CDR reader used subtractive RGB;
retaining source inks then changed the preview. Both paths now keep the same local
default. This does not implement every configurable missing-profile policy.

Native `.tdraw` save and reopen, bitmap import, linked-image reload and pixel
cropping preserve the retained profile with its matching planes. Import adds
pixels, inks, profile and link as one undo step. Save Bitmap As uses the source
profile by default. An explicit different CMYK output profile converts full ink
samples first, then flattens alpha for JPEG; it does not just relabel the inks.

This is not CMYK compositing of a whole page. Associated-alpha TIFF, CMYK16,
original CMYK JPEG/YCCK samples, RGB embedded profiles, multiple TIFF pages and
conversion of print separations between per-picture profiles remain outside
this path. The CDR writer retains process samples without an image profile in
versions 7 to 27. With an image profile, it retains them only when the document
embeds the same CMYK profile (versions 16 to 27); the reader associates that
profile with the retained image inks and converts their preview directly through
the ICC transform, without an extra approximate lookup table. A different image
profile, missing document profile, stale planes or spot channels still uses the
RGB fallback. These internal cycles do not establish external interoperability.
Invalid or mismatched ICC profiles, incomplete planes and images beyond the
decoder's budgets are reported. No external application comparison has been
performed by these tests.

The new-document initialization follows the documented default of using embedded
profiles when opening tagged files. Open and Import have separate policies in the
[2026 colour-management reference](https://help.coreldraw.com/CorelDRAW/540111192/Documentation-Windows/CorelDRAW-en/CorelDRAW-Color-management-policies.html).
The decoder alone does not choose the application import policy. The application
now stages CMYK import as a document transaction: policy 0 converts the incoming
content to the destination calibration, policy 1 assigns the destination while
retaining ink values, and policy 2 converts the existing destination to the
incoming calibration before adding the retained incoming values. A real default
profile is recorded when it supplied an untagged source's calibration; the
built-in generic model is represented explicitly without inventing ICC bytes.
Per-picture profiles, alpha, styles and shared resources participate in this
transaction. The native MCP `import` tool uses the same preparation and commit.

The native interface prepares this work off the frame and checks destination
instance, edit sequence, selection, placement and settings before commit. Cancelling
discards the result; the current worker may finish its calculation. A stale or
invalid result changes neither the destination nor its identifier allocator.
The transaction is one undo step. Combined import materializes eligible source
master content through shared editable symbols; preserving source pages and
layers as separate imported pages is not implemented by this placement mode.

Linked-image reload and RGB/grayscale profile policies do not yet use this full
CMYK transaction. Native `.tdraw` opening retains its carried calibration even
for opening policies 1 and 2; the CDR and CMYK raster opening controls are
separate. Local exact-byte and ICC tests do not establish parity with another
application's policy, clipboard or file implementation.

## The bitmap editor

Edit Bitmap writes the picture to a file in the temporary folder, links
the object to it and opens it in the bitmap editor set in Tools >
Options > Global > File Locations (a program file: `gimp`, its `.exe` on
Windows, or an application bundle on macOS, opened with `open -a`;
Browse picks it, Clear empties it). Empty, the default, opens it with the
system's program for PNG files (#593, `export::tests::the_bitmap_editor_gets_the_file`).
The browser build has no outside editor.

The file is a PNG, or a CMYK TIFF for a CMYK picture (a `.cdr` CMYK
image, or one in the CMYK colour mode), holding the inks it prints from
as Save Bitmap As writes them, so the editor works on the inks and not
on RGB pixels (#1834); a picture with see-through pixels goes with its
alpha channel beside the inks, so the transparency survives the round
trip (#1925). A CMYK TIFF read back (8 bits per ink, with or
without alpha, any compression) gives the picture those inks and pixels
that show them through the colour management, and the CMYK mode; any
other file gives pixels only, and inks the picture kept are dropped,
since they no longer fit (`bitmap_file::tests::edit_bitmap_hands_over_and_reads_back_the_inks`).

## Linked files follow their changes

A linked bitmap (an imported file, or one sent to the system image
editor by Edit Bitmap) follows its file on the desktop: once a second
the files' modification times are read, and a bitmap whose file changed
since it was last seen is read again on the spot, as Update from Link
does (`App::watch_linked_bitmaps`; one undo step per change, the status
bar says "Bitmap updated from its file"). So a save in the outside
editor shows in the drawing at once. A file seen for the first time is
only remembered; a file that cannot be read is left alone. The browser
has no files to watch
(`clipboard::linked_watch_tests::a_linked_bitmap_is_read_again_when_its_file_changes`).

## Size

The bitmap comes in at the resolution its file records, so a 300 dpi
scan 1200 pixels wide is 101.6 mm wide:

| Format | Where the resolution comes from |
|---|---|
| PNG | the `pHYs` chunk, when its unit is the metre (pixels per metre x 0.0254) |
| JPEG | the JFIF header's density in dots per inch or per centimetre; else the EXIF `XResolution`/`YResolution` with their `ResolutionUnit` |
| TIFF | `XResolution`/`YResolution` with `ResolutionUnit` (inch or centimetre) |
| BMP | the header's pixels per metre |

Width in mm = pixels x 25.4 / horizontal dpi, height likewise with the
vertical dpi. A file that records no resolution, only an aspect ratio,
or a value outside 1 to 100000 dpi comes in at 96 dpi. Pasted images
and generated pictures are placed at 96 dpi.

## On screen and in exports

A picture is decoded once and kept (`crates/render/src/image_cache.rs`,
512 MB of decoded pixels in the desktop build, 256 MB in the browser;
the least recently drawn go first). Drawn at less than half its pixels'
size, it is drawn from a reduced copy instead: level `k` halves the
sides `k` times, each pixel the average of the four beneath it, and the
level used is `floor(log2(1 / s))` for `s` screen pixels per picture
pixel (at most 12), so it is sampled at between half and the whole of
its size. The reduced copies share the same memory budget and are made
when first needed. An upright picture (not turned or slanted) covers
whole pixels: its edges are rounded to the nearest pixel boundary, so
pictures laid side by side meet without a pale seam (#853,
`band_tests::pictures_side_by_side_meet_without_a_seam`,
`band_tests::a_picture_drawn_small_shows_the_average_of_its_pixels`).

## Checks

- `tracedraw_io::picture::tests`: source CMYKA samples, including pixels with
  alpha 0 and 127, and two different source profiles survive native JSON and
  TIFF output exactly. Different profiles affect previews without changing
  original inks; planar TIFF and BigTIFF use that same path; malformed images
  and mismatched profiles are refused.
- `bitmap_file::tests`: import and Save Bitmap As preserve samples and profile,
  one undo removes the whole import, redo restores it, cropping preserves the
  profile, and opening TIFF leaves its source file unchanged.
- CLI `convert_formats`: a real TIFF-to-native invocation preserves ink planes,
  profile, alpha and physical size and leaves the source untouched.

- Each reader gives the stored value and nothing for missing, broken or
  out-of-range data (`image_dpi::tests`).
- Given a PNG of 4 pixels at 300 dpi, when imported, then it is
  4 x 25.4 / 300 mm wide; a PNG without `pHYs` is 4 x 25.4 / 96 mm wide
  (`image_dpi::tests::import_places_a_bitmap_at_its_resolution`).
