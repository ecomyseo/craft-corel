# Document Info

File > Document Info opens a dialog of 430 x 780 that edits the
drawing's properties and reports what it holds. The fields change
nothing until OK; Cancel, Esc and the close button leave them as they
were. "?" opens the help.

## Fields

- **Language**: None, or one of 30 languages shown by their own names
  ("English (United States)", "Português (Brasil)"), stored as a
  language tag ("en-US", "pt-BR"). PDF files written from the drawing
  carry it as the document language (`/Lang`).
- **Title**, **Subject**, **Author**, **Copyright**, **Tags** (the
  keywords written to PDF and SVG files), **Notes** (three lines; Enter
  starts a new line there).
- **Rating**: None or one to five stars.

OK stores the title and the other fields in one undo step ("Document
Properties"), and only when something changed.

## Report

Under the fields, a white box that scrolls, in sections with an accent
title and line:

- **File**: name and location (the drawing's name followed by "(not
  saved)" before it is saved).
- **Document**: pages, layers (every page's and the master layers), page
  size of the current page in the document's units and in pixels at
  the drawing's resolution, orientation, resolution.
- **Color**: the drawing's RGB, CMYK (its own when it carries one) and
  grayscale profiles, primary colour mode and rendering intent, else the
  defaults from Colour Management.
- **Graphic Objects**: number of objects (groups and clip frames are
  not counted, what they hold is), number of points (the nodes of
  curves, rectangles, ellipses and polygons), the most points and
  subpaths one of them has, then curves, rectangles, ellipses, polygons,
  groups and tables when there are any.
- **Word Count**: artistic and paragraph texts, characters, words
  and the fonts used, or "No text objects in this document."
- **Bitmap Objects**: number of bitmaps, total pixels and the largest
  one, or "No bitmaps in this document."
- **Styles**: object and colour styles, or "No styles in this document."
- **Effects**: each live or bitmap effect with the number of objects
  carrying it, or "No effects in this document."
- **Fills**: objects by fill (none, uniform, fountain, pattern,
  texture, mesh), the kinds that occur.
- **Outlines**: objects by outline (none, solid, dashed), then
  "Objects and color models": how many objects use each colour model in
  their fill or outline.
- **Symbols**: symbols defined and instances placed, or "No symbols in
  this document."

## Checks

- A group holding a two-subpath curve (5 nodes) and a rectangle, next
  to an artistic text "two words": 3 objects, 1 group, 1 curve, 9
  points, at most 5 points and 2 subpaths, 1 rectangle, 1 artistic text
  of 9 characters and 2 words; fills 1 none and 2 uniform; outlines 2
  none and 1 solid; CMYK used by 1 object and RGB by 2
  (`doc_stats::tests::counts_curves_text_fills_and_outlines_through_groups`).
- OK stores language, title, author, tags and rating in one undo step
  that Undo takes back; OK with nothing changed adds no step; the dialog
  draws and Enter is OK
  (`ui::doc_properties::tests::ok_stores_the_fields_in_one_step_and_cancel_keeps_them`).
- The language is written to PDF files as `/Lang`
  (`pdf::tests::document_properties_fill_the_information_and_the_xmp`).
