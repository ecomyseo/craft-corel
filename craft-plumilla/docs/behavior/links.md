# Links and bookmarks

The Links and Bookmarks panel (Window > Panels, or right-click an object >
Internet Links > Hyperlink) gives objects a link and lists the named
objects of the drawing.

## Panel

- Hyperlink: the link of the selected objects (the first one's shows);
  editing it and leaving the field sets it on all of them, an empty
  field removes it.
- Web, E-mail and Page put `https://`, `mailto:` or `#page=` in front of
  the text; Remove takes the link away.
- Bookmarks (named objects): one button per object with a name; a click
  selects the object and zooms to it.
- With nothing selected the panel says to select an object.

## In exported files

- PDF (File > Export, Export as PDF, Print, Send To): every visible
  linked object on a printable layer becomes a link area over its
  bounding box (without the outline width), with no border. Web and mail
  addresses open as addresses (characters outside 7-bit ASCII are
  percent-encoded); `#page=n` goes to the top of page n of the file, and
  is left out when there is no such page. A linked group's link covers
  its children, whose own links are ignored. PDF/X files get no links
  (the standards allow no annotations on the page). The link areas move
  with the bleed.
- SVG and HTML: a linked object is written inside `<a href="...">`, so
  the object's own shape is what reacts to a click. Links inside a
  linked group are dropped (links cannot nest); `#page=n` links have no
  target in a one-page SVG and are left out.

## Checks

- `pdf::pdfx_tests::linked_objects_become_link_areas`
- `svg::tests::linked_objects_sit_inside_links`
