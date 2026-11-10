# Page and desktop context menus

A right-click with nothing selected opens one of two menus, after where
it falls (the reference has one for each):

- **On the page** (the Page menu): Full-Screen Preview; View; Create
  Object; Import; Paste (at the click, external clipboard data included);
  Page Setup (the page size dialog).
- **On the desktop around the page** (the Document Desktop menu):
  Full-Screen Preview; View; Create Object; Create New Symmetry (an empty
  symmetry whose vertical mirror line passes through the click, straight
  into editing it); Import; Paste; Paste in View
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
program does not have.

**Create New Symmetry with nothing selected** (here and in Object >
Symmetry): what is drawn next joins the symmetry and is mirrored; finished
while still empty, the symmetry goes. One undo step.

**With no drawing open** (the Welcome Screen's tab closed), a right-click
on the empty workspace opens the reference's CorelDRAW Desktop menu: New,
New From Template, Open; View; New Window (greyed without a drawing);
About.

## Checks

- `ui::context::page_desktop_menu_tests::the_page_and_desktop_menus_follow_the_reference`
- `ui::context::page_desktop_menu_tests::the_page_and_desktop_rows_act`
- `ui::context::page_desktop_menu_tests::the_desktop_menu_pastes_in_view`
- `ui::context::page_desktop_menu_tests::the_empty_area_menu_depends_on_where_the_click_falls`
- `ui::context::page_paste_tests::page_context_paste_accepts_external_data_without_a_local_copy`
- `ui::context::page_desktop_menu_tests::the_empty_workspace_menu_follows_the_reference`
- `ops2::tests::a_new_symmetry_with_nothing_selected_takes_what_is_drawn`
- `pick_drag_tests::mesh_handle_tests::a_right_click_on_the_empty_workspace_opens_its_menu`
