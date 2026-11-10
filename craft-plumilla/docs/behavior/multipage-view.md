# Multipage View

View > Multipage View lays every page of the drawing out on the desktop
at once, so pages can be compared and edited side by side. The page
bar's Multipage View button, after the page controls, turns it on and
off too and is lit while it is on (#1019).

## Layout

- Pages sit in a grid of four columns, whatever the page count (fewer
  pages fill one row), left to right, then down, in page order. A page
  added goes at the next place and the pages before it stay where they
  were (#1018, `multipage::tests::adding_pages_never_moves_the_pages_before`).
- Each column is as wide as its widest page and each row as tall as its
  tallest; a page sits at the top left of its cell. Cells are the
  drawing's page spacing apart: 30 mm unless the drawing sets another
  (`multipage::tests::the_cells_are_as_far_apart_as_the_spacing`).
- Every page shows its number and name below it; the current page's in
  the normal text colour, the others dimmed.
- Objects outside a page's edges are not shown on the other pages; the
  current page shows them as always.
- Every page shows its own guidelines: the current page's across the
  whole window, as always, and each other page's within that page,
  where it sits in the grid (#1448,
  `multipage::tests::the_other_pages_guidelines_show_where_their_pages_sit`).
  Guidelines belong to their page; a click on another page makes it
  current, and its guidelines are then the ones picked, moved and
  snapped to.

- With facing pages the pages sit in spreads, two spreads a row (see
  `facing-pages.md`).

## Page spacing

- The space between pages is a document property
  (`Metadata::page_spacing`, mm), saved in `.tdraw` files only once set;
  a drawing without it (any file from before) uses 30 mm
  (`tracedraw_core` `the_page_spacing_defaults_to_30_mm_and_round_trips`).
  Values are kept between 0 and 1000 mm.
- Set from Page > Page Spacing (presets 0, 5, 10, 20, 30, 50 mm, or
  Custom... in the document's units) and Document Setup > Layout > Page
  spacing. Each change is one undo step (Document Properties); Save as
  Default keeps it for new drawings.
- The current page stays where it is on screen when the spacing changes;
  the others move. The grid, hit testing of the other pages, alignment
  to their objects, their guidelines, page labels and Zoom to Fit all
  follow the spacing
  (`multipage::tests::the_drawing_s_page_spacing_moves_the_pages_apart`).
- Printing and export work on pages, not on this view, so the spacing
  does not change them.

## Working

- The current page is edited as in the normal view: rulers, guides,
  grids and the status bar refer to it.
- A press on another page makes it current before the tool acts, so a
  click selects there and a drag draws there. Nothing moves on screen.
- The page tabs and the navigator change the current page the same way.
- Zoom to Fit (F4) and turning the view on fit every page in the window.
- Page Overview and Multipage View exclude each other.

## Drawing

- Each other page keeps its own picture. While the view pans or zooms,
  the last picture is scaled and moved with its paper, and the pages are
  rendered anew once the view has rested for 0.12 s.
- A page that covers no more of the screen than the canvas is rendered
  whole; a larger one only where it shows.
- One frame spends at most 10 ms rendering the current page and the
  others; pages left over show their last picture and render in the next
  frames. One render runs every frame, so every page gets its turn.
- Renders on a thread run one at a time in the whole program, the page
  being edited first: when it had to wait, the turn stays kept for it
  for 1 s after the running render ends. A page refused its turn for
  that reason schedules a repaint for the moment the turn is free again,
  so it is rendered sharp at the current zoom without any pointer move or
  key press (#1384,
  `raster::tests::a_page_refused_a_kept_turn_asks_for_a_repaint`).

## Formula

With `cols = min(n, 4)`, page `i` is in row `i / cols`, column
`i % cols`; its left edge is the sum of the earlier column widths plus
`column * gap`, its top edge the sum of the earlier row heights plus
`row * gap` below the first row's top, `gap` being the page spacing
(30 mm by default). The grid is then moved so the
current page's bottom-left corner is the origin.
