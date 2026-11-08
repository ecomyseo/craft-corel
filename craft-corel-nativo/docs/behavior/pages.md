# Pages

## Duplicate Page

Page > Duplicate Page..., the page tab menu's Duplicate Page... and the
Duplicate Page button of the Pages panel open the Duplicate Page dialog
for the current page:

- Insert new page: Before selected page or After selected page (the
  default).
- Copy layer(s) only: the copy has the page's layers (names, visibility,
  printing and lock) with no objects in them.
- Copy layer(s) and their contents (the default): the objects are copied
  too, with fresh ids.
- OK duplicates, Cancel and Esc close the dialog, "?" opens the Help
  (`pages-layout/pages#duplicate-page`).

The copy keeps the page size, guides and background, is named after the
page with " (copy)", and becomes the current page. It is one command,
`Command::DuplicatePage { page, before, contents }`, so one undo step
removes it. A saved or scripted command that names only the page copies
it with its contents after it, as before (Data Merge works that way).

Tests: `command::tests::duplicate_page_goes_before_or_after_with_or_without_contents`,
`command::tests::a_saved_duplicate_page_without_options_copies_contents_after`,
`ui::window_bars::tests::the_duplicate_page_dialog_puts_the_copy_where_asked_and_undoes`,
`ui::window_bars::tests::layout_duplicate_page_opens_the_dialog`.

## Page size on the property bar

With nothing selected the property bar starts with the page size list:
the favourites (A4, Letter, A5, A3, Legal), the sizes the user saved,
then every preset under its group heading (ISO A, ISO B and JIS B,
North American, Print shop, Posters and banners, Envelopes, Cards and
postcards, Photo, Screen and social media, Web ads), a line, then
Custom... and Edit This List.... No two presets share a size in either
orientation. The list shows the
preset the page matches (either way round) or Custom when it matches
none, and Custom... is then the entry ticked. A preset keeps the page's
orientation. Custom... opens the Page Size dialog with the page's width
and height, to type any size (#650, #1092); the page width and height fields
next to the list also take any size at once. Both resize the current
page or every page as the All Pages / Current Page buttons say.
Edit This List... opens Document Setup > Page Size, where the
save button next to Size keeps a custom size under a typed name and the
delete button forgets the saved size shown; saved sizes are kept in the
preferences and listed in every page size list (the property bar, the
Page Size dialog, Document Setup and Create a New Document).
Checked by `ui::propbar::tests::the_page_size_list_offers_a_custom_size`,
`ui::dialogs::tests::page_size_presets_are_distinct_and_keep_their_names`,
`ui::dialogs::tests::a_page_size_finds_its_name`.

## Page Spacing

Page > Page Spacing, after Page Layout: a submenu of presets (0, 5, 10,
20, 30 and 50 mm, the one in use marked) and Custom..., a dialog with
one number field in the document's units (0 to 1000 mm) that OK applies.
It sets the space between the pages of the Multipage View, the same
value as Document Setup > Layout > Page spacing; one undo step, saved
with the drawing (see `multipage-view.md`,
`ui::menus::tests::the_page_menu_sets_the_page_spacing`,
`ui::dialogs::tests::the_page_spacing_dialog_sets_the_drawing_s_spacing`).

## Fit Page to Selection, Fit Page to Drawing

Page > Fit Page to Selection (pale with nothing selected) and Fit
Page to Drawing (pale on an empty page), also as buttons on the
Document Setup > Page Size page and, Fit Page to Drawing, on the
property bar with nothing selected (after the page width and height):
the current page takes the size of
the selection's bounds (effects included), or of everything on the page,
and the page's objects move so that box sits on the page from its
bottom-left corner. One undo step (`App::fit_page_to_selection`,
`fit_page_to_drawing`); a box under 0.1 mm does nothing and the status
bar says so. The property bar's button opens a popup instead of fitting
at once: a Margin distance field (in the ruler unit; kept in the
preferences, `settings.fit_page_margin`, 0 at first) and an Autofit
Page button, which fits the page to the drawing with that margin left
free on every side (`App::fit_page_to_drawing_with_margin`; a margin
that is not a number counts as none). Checked by
`ops2::fit_page_tests::the_page_fits_the_selection_or_the_drawing` and
`ui::propbar::tests::the_page_bar_draws_with_its_fit_page_button`.
Autofit Page follows the page bar's All Pages and Current Page buttons,
as the page size fields do: with All Pages on, every page is fitted to
its own drawing with the margin, in one undo step, and a page with
nothing on it keeps its size; the button is pale only when no page has
anything on it (#1393, `App::autofit_pages`,
`ops2::fit_page_tests::autofit_with_all_pages_fits_every_page`).
While the popup is open the page itself shows at the size Autofit Page
would give it, around the drawing, following the margin as it is typed;
nothing else is drawn, and closing the popup without OK puts it back
(#594, #604, `ops2::tests::the_autofit_preview_is_the_drawing_and_its_margin`).
A Live preview check box on the margin's row, right of the field (on at
first, kept in the preferences, `settings.fit_page_preview`), turns that
off: the page then keeps its size until Autofit Page is clicked (#659,
`ui::propbar::tests::the_autofit_popup_previews_only_with_live_preview_on`).
Clicks inside the popup (the box, the margin field and its arrows) keep
it open, so the preview can be switched and watched; a click outside or
Autofit Page closes it
(`pick_drag_tests::the_autofit_popup_stays_open_when_its_box_is_clicked`).

## Delete Page

Page > Delete Page... and the Delete Page... button of the Pages panel
open the Delete Page dialog (#500): Delete page (the current page to start
with) and a Through to page check box with the last page of a range, off
to start with (the range may be given in either order; numbers are kept to
the drawing's pages). OK deletes the pages in one undo step; the page after
them, else the one before, becomes the current page. A range of every page
deletes nothing and the dialog says a drawing keeps at least one page. The
page tab menu's Delete Page deletes that page at once
(`app::tests::delete_page_takes_a_range_of_pages`).

## Moving objects to another page by its tab

With the Pick tool, the selection dragged onto another page's tab goes
to that page (#1346, `page_drop`):

- While the selection is moved over a page tab, that tab lights up.
  Resting there for 0.6 seconds sends the selection to that page, which
  becomes the current page, and the move goes on there: while the
  pointer stays on the tabs the objects stay at their place on the
  page, and back on the drawing they follow the pointer from there, to
  be let go anywhere.
- Let go on a page tab (another page's at once, or the tab of the page
  the rest opened), the selection lands at its place on that page; the
  way the pointer went to the tab is not applied.
- With Ctrl (Cmd on macOS) held, or after a right click during the
  move, copies go to the page and the originals stay where they were.
- The objects go to the page's topmost visible, unlocked layer (its
  topmost unlocked one when none is visible); a page with only locked
  layers takes nothing and the status bar says so. They stay selected
  on the page they went to.
- The whole drag is one undo step, the move after the page changed
  included.

Checks: `pick_drag_tests::a_selection_dropped_on_a_page_tab_moves_to_that_page`,
`pick_drag_tests::ctrl_on_a_page_tab_sends_a_copy`,
`pick_drag_tests::resting_on_a_page_tab_opens_the_page_and_the_move_goes_on`.
