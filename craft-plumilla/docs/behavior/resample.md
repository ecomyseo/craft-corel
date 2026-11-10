# Resample

Image > Resample (also on the bitmap property bar) gives the selected
bitmaps a new pixel size; their place and size on the page stay.

## Fields

- Pixels: the new width and height of the first selected bitmap.
- Percent: the same as a percentage of its pixel size.
- Resolution: the dpi the new width gives at its size on the page.
- Maintain aspect ratio (on): the height follows the width, and back.
- Method:
  - Smooth (photos), the default: Lanczos 3.
  - Sharp: Catmull-Rom, a little crisper.
  - Hard pixels (pixel art): nearest pixel; every pixel stays a square of
    one colour.
  - Enlarge with detail: Lanczos 3, then, when the picture grows by more
    than 5 %, an unsharp mask with radius `0.6 x g` (0.6 to 4 pixels) and
    amount `0.35 + 0.25 ln g` (up to 1.2), `g` the enlargement, on the
    lightness only, so colours do not shift.
- The original and new pixel sizes and the memory of the new picture
  (4 bytes a pixel) are shown below.

Sizes are 1 to 20000 pixels. With several bitmaps selected, each is
scaled by the same factors as the first. One undo step per bitmap.
