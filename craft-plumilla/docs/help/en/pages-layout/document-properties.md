# Document info

[[menu.file > menu.file.document_properties]] opens the
[[docprops.title]] dialog box. It keeps information about the drawing, such
as its title and author, and reports everything the drawing holds.

![The Document Info dialog box](../../img/pages-layout-document-properties.png)

## Describing the drawing {#fields}

- **Language**: the language of the drawing's text, or none. PDF files written
  from the drawing carry it, which helps screen readers.
- [[dialog.title]], [[dialog.subject]], **Author**, **Copyright**,
  [[docprops.tags]] (keywords written to PDF and SVG files) and **Notes**
  (several lines).
- [[dialog.rating]]: none or one to five stars.

Click [[dialog.ok]] to keep the changes, as one undo step. [[dialog.cancel]]
leaves everything as it was. The title, subject and rating can also be set
when you save; see [Save As](../get-started/open-and-save.md#save-as).

## The report {#report}

Under the fields, a report lists what the drawing holds, in sections:

- [[docprops.sec_file]]: the name and location of the file.
- [[docprops.sec_document]]: the pages and layers, the page size (in the
  drawing units and in pixels), the orientation and the resolution.
- [[docprops.sec_color]]: the color profiles, the primary color mode and the
  rendering intent.
- [[docprops.sec_objects]]: the number of objects and of points, the most
  points and parts one curve has, and the curves, rectangles, ellipses,
  polygons, groups and tables.
- [[docprops.sec_text]]: artistic and paragraph texts, characters, words and
  the fonts used.
- [[docprops.sec_bitmaps]]: the number of bitmaps, their total pixels and the
  largest one.
- [[docprops.sec_styles]]: object and color styles.
- [[docprops.sec_effects]]: each effect, with how many objects use it.
- [[docprops.sec_fills]] and [[docprops.sec_outlines]]: the objects by kind
  of fill and outline, and how many use each color model.
- [[docprops.sec_symbols]]: symbols defined and instances placed.

The report helps before sending a file to print: for example, it shows which
fonts the drawing needs and whether it holds RGB colors. For a full check,
see [Preflight](../printing/preflight.md).
