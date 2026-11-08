# Export docker

Window > Panels > Export keeps a list of assets and exports them all at
once, each in its own format, to one folder.

## Assets

- Add Pages adds every page of the drawing (a page already listed is
  not added again), named after the page.
- Add Selection adds the selected objects as one asset, cropped to their
  bounds, named after the object when it has a name ("Object" or
  "Selection" otherwise) and numbered.
- New assets take the format, resolution and, for JPEG and TIFF, colour
  mode under "New assets" (default PNG at 150 dpi, RGB).
- Each row: a tick (only ticked assets are exported), the name (the file
  name), the format, the resolution for raster formats (PNG, JPEG, BMP,
  TIFF, WebP, GIF, PSD; 72 to 2400 dpi), the colour mode for JPEG and
  TIFF (RGB, CMYK or Grayscale, as in the Export dialog; #1763) and ✕ to
  remove it. The link
  under the name goes to the page or selects the objects.
- An asset whose page is deleted goes; deleted objects leave their
  asset, and an asset with none left goes.

## Export

- Export (n) writes the ticked assets to the chosen folder (Browse...
  chooses it; the first Export asks for one). In the browser and the
  Android app no folder can be picked: the folder row says the assets go
  out as one zip file, named after the drawing with every asset in a
  folder of that name, downloaded in the browser and saved where the
  system's save picker says on Android (#1399,
  `assets_export_as_one_zip_where_no_folder_can_be_picked`).
- File names are the asset names with `/ \ : * ? " < > |` and control
  characters replaced by `_`; an empty name is "asset". Two assets that
  would write the same file (ignoring case) get " (2)", " (3)" and so on.
- Each asset is exported as File > Export would with its format,
  resolution and colour mode (raster formats on a transparent background where the
  format allows it). The status bar says how many files were written, or
  the first error.
