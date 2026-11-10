# Resampling, straightening and inflating bitmaps

## Resample {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] gives the selected bitmaps a new
number of pixels; their size on the page stays.

![The Resample dialog box](../../img/bitmaps-resample-and-straighten-resample.png)

1. Type the new size in **pixels**, as a **percent**, or as the
   **resolution** it gives at its size on the page. With **Maintain aspect
   ratio** on, the height follows the width.
2. Choose the method:
   - **Smooth** (photos), at first;
   - **Sharp**, a little crisper;
   - **Hard pixels** (pixel art): every pixel stays a square of one color;
   - **Enlarge with detail**: smooth, then sharpened when the picture grows.
3. Click [[dialog.ok]].

The dialog box shows the original and new sizes and the memory the new
picture takes. With several bitmaps selected, each is scaled by the same
factor.

## Straighten Image {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] corrects photos that are
tilted, seen at an angle or bent by the lens.

![The Straighten Image dialog box](../../img/bitmaps-resample-and-straighten-straighten.png)

1. Use the **Grid** (its size and color are adjustable) to line the picture
   up.
2. Set the sliders: **Correct lens distortion** (-100 to 100), **Rotate image**
   (-15 to 15 degrees), **Vertical perspective** and **Horizontal
   perspective** (-100 to 100). The rotate buttons turn the picture a quarter
   turn first.
3. With **Crop image** on, the result is cropped to the largest rectangle with
   the original proportions; **Crop and resample to original size** brings it
   back to the original number of pixels. Without cropping, the corners are
   transparent.
4. Click [[dialog.ok]]. **Reset** puts every setting back.

Zoom and pan the preview with the tools above it. The result replaces the
pixels, so effects on the bitmap are made permanent.

## Inflating bitmaps {#inflate}

Some effects, such as blurs, spread past the edges of a bitmap.

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]] (on at
  first) grows bitmaps by what their effects need, with a transparent margin.
- [[menu.bitmaps.inflate_manual]] grows the selected bitmaps by hand: for
  the width and height, type the new size in pixels or a percentage (100 to
  1000%), with **Maintain aspect ratio**. The picture stays centered.

![The Bitmap Padding Custom dialog box](../../img/bitmaps-resample-and-straighten-inflate.png)

## Other bitmap commands {#other}

- [[menu.bitmaps.crop_bitmap]] crops a bitmap with the [[tool.crop]] tool;
  dragging a corner of a bitmap with the [[tool.shape]] tool crops it too.
- [[menu.bitmaps.edit_bitmap]] opens the bitmap in another program.
- [[menu.bitmaps.remove_background_local]] makes the background of a photo
  transparent on this computer (see
  [Removing the background](remove-background.md));
  [[menu.bitmaps.remove_background]] does it with the AI service.
- [[menu.bitmaps.break_link]] and [[menu.bitmaps.update_from_link]] work on
  bitmaps linked to a file.
- [[menu.bitmaps.save_bitmap_as]] saves a bitmap as a PNG file.
