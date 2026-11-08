# Multipage view

[[menu.view > menu.view.multipage]] lays every page of the drawing out on the
desktop at once, so you can compare pages and edit them side by side.

## The layout {#layout}

The pages sit in a grid of four columns, in page order, so a page you add goes
at the next place and the pages before it stay where they were. Each page shows its number and name below
it; the current page's are in the normal text color, the others dimmed.

## Space between pages {#page-spacing}

The pages sit 30 mm apart at first. To change it, choose
[[menu.layout > menu.layout.page_spacing]] and pick
[[menu.layout.page_spacing_0]], [[menu.layout.page_spacing_5]],
[[menu.layout.page_spacing_10]], [[menu.layout.page_spacing_20]],
[[menu.layout.page_spacing_30]] or [[menu.layout.page_spacing_50]], or
[[menu.layout.page_spacing_custom]] to type any distance in the drawing's
units. [[options.page_spacing]] on the Layout page of
[Document options](document-options.md#layout) sets it too. The spacing is
saved with the drawing, [[menu.edit.undo]] puts the previous one back, and
**Save as default** in the document options keeps it for new drawings.

## Working in multipage view {#working}

- The current page is edited as usual: rulers, guidelines, grids and the
  status bar refer to it.
- Click or drag on another page to make it current and work there at once.
- The page tabs and the navigator change the current page too.
- [[menu.view.zoom_to_fit]] (<kbd>F4</kbd>) fits every page in the window.

The button with four small pages in the page bar, right after the page
controls, turns the view on and off too; it is lit while the view is on.

Choose [[menu.view.multipage]] again to go back to one page at a time.
[[menu.view.page_sorter]] shows the pages as thumbnails instead; the two views
exclude each other.
