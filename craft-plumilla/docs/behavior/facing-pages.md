# Facing Pages

Facing pages show the pages of a drawing side by side in spreads while
drawing, as a book or magazine opens (#1972, #1660). Before, spreads
existed only in print imposition.

## Turning it on

- Page > Facing Pages turns it on and off; Page > Facing Pages Start On
  chooses Right Side or Left Side (choosing a side also turns facing
  pages on).
- Document Options > Layout has the same choices: Facing pages and
  Start on: Right side / Left side.
- Each change is one undo step (Document Properties). The setting is a
  document property (`Metadata::facing_pages`, `Metadata::facing_start`)
  saved in `.tdraw` files only while on, so older drawings read and
  write as before (`spreads::tests::facing_pages_round_trip_in_the_file`).

## Spreads

- Start on Right side (the default): page 1 stands alone on the right,
  then pages 2-3, 4-5 and so on pair up; the last page stands alone on
  the left when the count is even.
- Start on Left side: pages 1-2, 3-4 and so on.
- The spreads follow the page order alone, so inserting, deleting,
  duplicating or moving pages reflows them
  (`spreads::tests::adding_and_removing_pages_reflows_the_spreads`).
- The two pages of a spread touch at the spine with their tops lined up.

## Working

- The desktop shows the current page's spread: the facing page beside
  the current one at the spine. The current page is edited as always;
  rulers, grids and the status bar refer to it.
- Objects belong to one page. An object drawn across the spine belongs
  to the page it was started on and reaches over onto the facing page,
  where it shows. The facing page's objects that reach over the spine
  show on the current page too, beneath the current page's own.
- A press on the facing page makes it current before the tool acts,
  nothing moving on screen, as in the Multipage View. A press on an
  object of the current page keeps the page even where the object lies
  over the facing page, so an object across the spine is picked
  anywhere on it; a press on the current page where only an object of
  the facing page lies makes that page current
  (`multipage::tests::the_facing_page_becomes_current_where_it_is`).
- The page tabs, the navigator and Go To Page make the facing page
  current in place; a page of another spread is shown with its spread,
  fitted to the window.
- Zoom to Fit and turning facing pages on fit the spread.
- Guidelines: the facing page's show within it, the current page's
  across the window, as in the Multipage View. Snapping and alignment
  guides take the facing page and its objects too
  (`multipage::tests::the_spread_keeps_guides_snapping_and_drawing`).
- View > Bleed draws the bleed around the whole spread, not at the
  spine (#1660). Mask Outside Page leaves both pages uncovered.
- Master layers for odd and even pages (Document Options > Layout) give
  right and left pages their own master objects: with Start on Right
  side odd pages are right pages.

## Multipage View

With facing pages the Multipage View lays the spreads out two a row
(four page columns), `page spacing` apart, the pages of a spread
touching. Every slot is as wide as the widest page, so a page added
never moves the pages before it
(`multipage::tests::facing_pages_sit_in_spreads_touching_at_the_spine`).

## Export and printing

- Export works on pages as before, one page per PDF page or per file.
  With facing pages each page also carries what its facing page holds
  across the spine and within the bleed beside it, so a picture across
  the spine prints whole over the two pages.
- Export > Spreads (shown with facing pages): each spread of the chosen
  pages becomes one page as wide as its pages side by side; a page
  alone stays a page of its own. The master layers, page backgrounds
  and page number fields are drawn into each half. A PDF holds a page
  per spread; formats of one page write a file per spread
  (`export::tests::facing_pages_export_spreads_or_pages_with_the_spine_crossing`).
- Printing prints pages as before, each with what reaches over the
  spine onto it; not for the selection alone, nor with Fit to page.
  Booklet and other impositions work on these pages.

## `.cdr` files

Facing pages are TraceDraw-only: they are not read from or written to
`.cdr` files. The one note on a facing pages flag in `.cdr` files
(version 21 `mcfg` byte 26, see `docs/cdr-format.md`) is an assumption
from two templates that does not say which side the first page is on,
so it is not taken for this setting and nothing is written for it.

## Not done

- Inside and outside margins, a table of contents and an advanced
  booklet window asked for in #1660 are separate features.
- The Page Sorter shows pages one by one, not in spreads.

## Formula

With `lead = 1` for Start on Right side and `0` for Left side, page `i`
(0-based) is in spread `(i + lead) / 2`, on the left when `i + lead` is
even, else on the right. In the Multipage View spread `k` is in row
`k / 2`, column `k % 2`; with `w` the widest page, the spine of column
`c` is at `c * (2w + gap) + w`; a left page ends at the spine and a
right page starts there; rows are as tall as their tallest page, `gap`
apart, pages top aligned.
