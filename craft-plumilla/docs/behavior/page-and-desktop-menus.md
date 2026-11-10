# Page and desktop context menus

A right-click with nothing selected opens one of two menus, after where
it falls (the reference has one for each):

- **On the page** (the Page menu): Full-Screen Preview; View; Create
  Object; Import; Paste (at the click, external clipboard data included);
  Page Setup (the page size dialog).
- **On the desktop around the page** (the Document Desktop menu):
  Full-Screen Preview; View; Create Object; Import; Paste; Paste in View
  (with something copied); AI Generate (Create Image, Remix Image, Create
  Vector, Draw with Words); Document Properties.

**View** shows and hides, with a check mark: the Toolbox; Color Palettes
(the Window > Color Palettes submenu); the Property Bar; the Status Bar;
Rulers; the document Grid; the Pixel Grid; Guidelines; the Printable
Area; the rollovers' Live Preview; Text Frames.

**Create Object** picks the tool to draw with next, the active one marked,
in the reference's order: Text, Table, Artistic Media (a submenu of its
five modes), Bezier; Ellipse, Freehand, Graph Paper, Common Shapes,
Polygon, Star, Rectangle, Spiral; then New Symbol from the selection.

The page tab's own menu (right-click on a page tab) is unchanged: Rename
Page, Insert Page After, Insert Page Before, Duplicate Page, Delete Page,
Switch Page Orientation.

Not here: the reference's Insert Object row places OLE objects, which this
program does not have; its desktop menu's New Symmetry with nothing
selected (an empty symmetry to draw into) is pending; the menu with no
drawing open is pending.

## Checks

- `ui::context::page_desktop_menu_tests::the_page_and_desktop_menus_follow_the_reference`
- `ui::context::page_desktop_menu_tests::the_page_and_desktop_rows_act`
- `ui::context::page_desktop_menu_tests::the_desktop_menu_pastes_in_view`
- `ui::context::page_desktop_menu_tests::the_empty_area_menu_depends_on_where_the_click_falls`
- `ui::context::page_paste_tests::page_context_paste_accepts_external_data_without_a_local_copy`
