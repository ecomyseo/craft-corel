# Creating a drawing

Every drawing starts with a page size, a number of pages, a color mode and a
resolution. You choose them in the [[dialog.create_new_document]] dialog box.

## To create a drawing {#create}

1. Choose [[menu.file > menu.file.new]] (<kbd>Ctrl+N</kbd>), click New on
   the standard toolbar, click the **+** after the last document tab, or
   click New Document on the Welcome Screen.
2. In the [[dialog.create_new_document]] dialog box, set the values
   described below.
3. Click [[dialog.ok]] or press <kbd>Enter</kbd>.

The drawing opens in its own tab. The dialog box remembers your values for
the next drawing.

![The Create a New Document dialog box](../../img/get-started-new-document.png)

## General {#general}

- [[dialog.name]] names the drawing. It starts as the next `Untitled-N`.
- [[newdoc.preset]] sets the color mode, page size, units and resolution at
  once. The presets are [[newdoc.preset_default]] and [[newdoc.preset_cmyk]]
  (CMYK, A4, millimeters, 300 dpi), [[newdoc.preset_rgb]] (RGB, A4,
  millimeters, 300 dpi) and [[newdoc.preset_web]] (RGB, 1920 x 1080 pixels,
  72 dpi). When you change one of those values, the preset shows
  [[newdoc.preset_custom]] unless the values match a preset again.
- The button beside the preset holds [[newdoc.save_preset]], which asks for
  a [[newdoc.preset_name]] and adds the current values to the list, and
  [[newdoc.delete_preset]], which removes a preset you saved. The built-in
  presets cannot be deleted.
- [[newdoc.pages]] sets how many pages the drawing has, from 1 to 999. Each
  page is named "Page N" and has one layer.
- [[newdoc.color_mode]] is CMYK, for print, or RGB, for screens.

## Dimensions {#dimensions}

- [[newdoc.page_size]] lists the paper sizes, or Custom. Choosing a size
  keeps the current orientation. A width and height you type that match a
  paper size, in either orientation, select its name.
- [[newdoc.width]] and [[newdoc.height]] set the page size, in the units
  you choose in the list beside the width. Those units become the drawing
  units of the property bar.
- [[newdoc.orientation]]: [[dialog.portrait]] or [[dialog.landscape]].
- [[newdoc.resolution]] in [[newdoc.dpi]] (300 at first). It is used for
  the effects that are turned into pixels, and to measure sizes in pixels:
  1920 pixels are 162.56 mm at 300 dpi and 677.33 mm at 72 dpi. When the
  units are pixels, changing the resolution keeps the page's size in
  pixels.

## Color settings {#color-settings}

Open [[newdoc.color_settings]] to choose the [[newdoc.rgb_profile]],
[[newdoc.cmyk_profile]] and [[newdoc.gray_profile]] of the drawing and the
[[newdoc.intent]]. They start at sRGB, the built-in CMYK profile, a 20% dot
gain grayscale and relative colorimetric. See
[Color management](../color/color-management.md).

## Skipping the dialog box {#skip}

Tick [[newdoc.dont_show]] to create new drawings at once, with the values
you used last. To see the dialog box again, turn on the matching setting on
the [[options.general]] page of
[[menu.tools > menu.tools.options > menu.tools.options_app]]. See
[TraceDraw options](../customize/options-app.md#general).

## Other ways to start {#other-ways}

- [[menu.file > menu.file.new_from_template]] starts from a template. See
  [Templates](../templates-styles/templates.md).
- [[menu.file > menu.file.open]] opens an existing drawing. See
  [Opening and saving drawings](open-and-save.md).
