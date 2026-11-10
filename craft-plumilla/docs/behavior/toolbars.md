# Menu bar, standard toolbar and property bar

Interface text is 13 px for menus, labels, lists and the status bar, in
the bundled interface font. Toolbars,
panels, palettes, the status bar and dialog bodies are `#F4F4F4`; light
lines between bands are `#D8D8D8`.

## Menu bar

A 24 px `#D8D8D8` band. Titles are 13 px, the first 11 px from the edge
and about 25 px apart; the open or hovered title is a light blue box
(`#E0F0FF`) with a blue frame (`#00ADFE`). Menus are white with a 1 px
`#B2B2B2` border and square corners: a 30 px `#F4F4F4` gutter holds the
icons, the bold check marks and the dots of exclusive choices (View >
Wireframe, Normal, Smooth, Pixels); rows are 26 px with labels from
38 px and shortcuts right-aligned 28 px from the edge, all black (light
grey when unavailable); separators are 7 px with a `#D8D8D8` line from
the gutter; a submenu row ends in a grey triangle and stays highlighted
while its submenu is open; a hovered row is the same blue box as the
titles. The canvas, palette and New Document menus are drawn the same
way.

When the bar is too short for every title (a phone, a tablet upright
in the Touch workspace, a narrow window), the titles that do not fit go
under a last title, », whose menu lists them in order, each opening
its menu as a submenu (#1301, #1691). Those menus keep the access keys
they have on a wide bar (underlined in the » menu): Alt and the letter
opens » with that menu open in it and its first row lit. Check:
`ui::tests::phones_fit_and_tablets_keep_their_layout`,
`ui::keynav::tests::alt_and_a_letter_open_a_menu_under_more`.

The View menu has Zoom In, Zoom Out and Zoom
To Fit (with their pictures) and no other zoom commands.

## Panels

The open panel has a 27 px `#EAEAEA` title bar (its name at 12 px, then
collapse and close in grey at the right; the collapse button's two
triangles point to the window edge the column folds toward, right for a
column at the right and left for one at the left, mirrored in a
right-to-left language) over a white page. The tab
column to its right is 27 px wide: `#EAEAEA` under the title bar and
below the tabs, a `#B2B2B2` line along the panel, grey `#D8D8D8` tabs
with an icon and the name running downwards, the open panel's tab
`#B2B2B2`; the three-dot button under the tabs lists every panel in the
groups of Window > Panels with a box ticked at each one in the column
(ticking opens the panel in front, unticking closes it). The pointer
over a tab shows a close cross in a slot at the tab's top; a click on
it closes that panel (as the title bar's close and the tab's menu do),
a click lower on the tab brings the panel to the front
(`ui::dockers::tests::a_docker_tab_closes_with_the_cross_at_its_top`).

### Panels anywhere

- The button at the end of each tab column is three small dots in a row
  (22 px tall, an 18 px grey box behind it while the pointer is over it;
  tip `docker.add_docker`, "Show or hide panels"). Its menu lists the
  panels in the groups of Window > Panels, with separators between the
  groups and the effect panels after the text group: each row is the
  check box (ticked while the panel is open anywhere), then the panel's
  picture (the one its tab shows; a blank slot of the same width where it
  has none, so the names line up), then its name
  (`ui::menus::check_pic`). Ticking a row opens the panel in front of
  that column (`App::move_docker`), unticking closes it
  (`App::close_docker`). Taller than the window, the list scrolls, as
  every menu body does
  (`pick_drag_tests::the_three_dot_button_lists_the_panels_with_their_pictures`).
- Panels live in one of three places (`App::docker_place`): the default
  column at the right of the window (`open_dockers`, `docker_tab` in
  front, `show_dockers`; at the left in a right-to-left language), the
  column at the other side (`left_dockers`, `left_tab`,
  `show_left_dockers`; it exists only while it holds a panel), a
  second column on either side (`inner_dockers`, `left_inner_dockers`:
  `DockerPlace::RightInner` and `DockerPlace::LeftInner`, each with its
  own tab strip, panel in front, collapse and width, between the side's
  column at the window's edge and the drawing) and floating windows
  (`floating_dockers`, one panel each). Every panel starts in the default
  column. A second column never stands alone: when the column at its
  side's edge loses its last panel, the second column's panels become
  that column's (`App::settle_columns`). So a side holds at most two
  columns, and the colour palette can lie between them (see Colour
  palette below), as in drawing | Properties | palette | Color Styles |
  window edge.
- Dragging a tab, or the title bar of a panel (a column's or a floating
  one's), moves the panel: a light blue band shows where it goes, within
  56 px of a side the column there (the strip and a 340 px panel),
  elsewhere a window the size a panel floats at (340 by 480) with its
  top left corner where the grab point puts it. Releasing puts the panel
  there, in front of the column it joins (the column it leaves shows the
  tab next to it, or closes when none is left); a release outside the
  window counts at the last place the pointer was seen. Over a docked
  column (its strip or its panel) the band is that column and the panel
  joins it. Within 56 px of the drawing's side of a side's only column
  (outside it, over the drawing) the band is a column's width there and
  the panel makes the side's second column, beside it; a panel alone in
  its column has nothing to stand beside and stays. The column rects are
  recorded each frame by the window layout (`dockers::DOCKER_COLUMNS`,
  like `bars::DRAWING_ROOM`), so the targets cost nothing while
  dragging. Over a column's tab strip (the part of it on screen) the
  band gives way to a line between the two tabs the panel would go
  between, and the release puts the panel's tab there: within its own
  column the tabs change order and the column keeps the panel it has in
  front and whether it is open; from elsewhere the panel joins that
  column at that place, in front (`App::insert_docker`, #1051). Each
  strip records where its tabs lie each frame
  (`dockers::StripLayout`), so the line costs nothing while dragging.
  A panel alone in a side's edge column dropped among the second
  column's tabs makes one column of them. The tab's menu and the
  title's menu (right click) have Dock
  at Right, Dock at Right, Second Column, Dock at Left, Dock at Left,
  Second Column, Floating and Close panel (a Second Column entry is grey
  while the column at that side's edge holds no other panel).
- A floating panel is a window with a grey border and a shadow: the
  panel's title bar (its name and close; no collapse) over its page,
  resizable at its edges; where it is and how large are kept.
- A panel brought to the front from a menu, a shortcut or a tool while it
  lies in the other column or floats is put in front where it is, and
  the default column stays as it was; a panel open nowhere joins the
  default column (`App::open_docker`, `App::reconcile_dockers`). Window >
  Panels and the Help > Hints check show a panel as on when it is in
  front of an open column or floating (`App::docker_shown`); the plus
  button of a column opens a ticked panel in front of that column.
- The places are kept in the preferences (`dockers`: `open`, `front`,
  `left`, `left_front`, `left_shown`, `inner`, `inner_front`,
  `inner_shown`, `left_inner`, `left_inner_front`, `left_inner_shown`
  (the second columns, written only while they hold a panel),
  `floating` with each window's corner and size); older preferences
  with the column alone read as before.
- Checks: `app::tests::dockers_move_between_the_columns_and_float`,
  `app::tests::a_side_holds_a_second_docker_column`,
  `ui::dockers::tests::a_docker_dropped_beside_a_column_makes_a_second_column`,
  `app::tests::insert_docker_puts_a_tab_among_a_columns_tabs`,
  `ui::dockers::tests::a_docker_tab_dragged_along_its_column_changes_the_order`,
  `ui::dockers::tests::a_docker_dragged_to_a_side_docks_there_and_the_window_draws_them_all`.

## Standard toolbar

A 35 px band (and its line): 32 px buttons with 20 px icons in groups
split by thin lines:

1. New (a page with a green plus), Open with the recent drawings behind
   its arrow, Save, Print.
2. Cut, Copy, Paste, Delete (a waste bin: what the Delete key does to
   the selection: the selected characters of the text being edited, the
   chosen nodes with the Shape tool, the selected guidelines, else the
   selected objects; for a screen with no keyboard, #1611). Then, only
   with a touch screen (Android, or once a finger has touched the
   window), Shift, Ctrl and Alt, buttons that read the key's name: a tap
   holds the key down until the next tap on it, the button drawn pressed
   meanwhile, and every tool reads the held keys as the keyboard's
   (`touch_modifiers.rs`, #1499).
3. Undo and Redo, each with its list of steps behind the arrow (choosing
   a step undoes or redoes up to it).
4. Import, Export, Export as PDF.
5. The zoom box: the zoom level as text (type a value, Enter applies it)
   and, behind its arrow, Zoom to Selected, to All Objects, to Page, to
   Page Width, to Page Height and levels from 10% to 1600%. Full-screen
   preview.
6. Show Rulers, Show Grid, Show Guidelines: toggles drawn pressed (white
   with a grey frame) while on.
7. Snap Off (Alt+Q), pressed while snapping is off; Snap To with the
   snapping list.
8. Options (the TraceDraw options); Panels (a window with a side panel),
   pressed while the dockers show: shows or hides them (the Touch
   workspace starts without them, #1611).
9. Launch, an icon with the list arrow (its name in the tooltip): Font
   Manager, Welcome Screen, Color Management.

Buttons that need a drawing, and Snap To, are disabled on the Welcome
Screen, also when drawings are open behind it; the property bar and the
toolbox are pale there too, so the page fields cannot be changed
(`pick_drag_tests::the_page_bar_does_nothing_on_the_welcome_screen`).
New, Open, Options, Launch and AI Generate stay on. Hovered buttons are
light blue with a light blue frame.

## Property bar

A 52 px band (and its line); fields stack two to a column, 19 px each
and sharing their middle border, 8 px from the top; fields and lists are
white with a grey border and 13 px text, small pictures before fields
name them (hover for a tooltip), and some fields have spin arrows.

Length fields (position, size, corners and the other lengths of the
property bars, the Contour, Extrude, Table, Variable outline and
Transform bars and the paragraph lengths) show the decimals a value
needs, at least one and up to the drawing precision (Options > Edit, 3
by default), with no trailing zeros: 20.0 mm, 20.5 mm, 20.125 mm (none
with a precision of 0); a value with more decimals shows rounded
(20.12345 mm as 20.123 mm) and is kept as it is, and a value typed is
applied as typed (#1761,
`ui::propbar::tests::distance_fields_show_only_the_decimals_needed`).

Number fields show their value at the left and are typed into: a click
selects the value, Enter or leaving the field applies it, Esc puts it
back. Tab applies it and goes to the next field's box in one press: the
spin arrows and the label of a field are no stops on the way (#1537,
`ui::field::tests::tab_goes_from_box_to_box_in_one_press`). A comma or a point marks decimals; sums work (`10+5`, `2*3,5`,
brackets); a distance typed with another unit is converted (`1 in` in a
millimetre field is 25.4 mm), also inside a sum, each number in its own
unit and a number without one in the field's: the value shown with `/ 2`
added halves it (`60.352 mm / 2`), and `24 mm + 1 in`, `24mm+20pt` and
`(16mm*2)+4in+2pt` add up in the field's unit (#1495). Percent and degree
marks are read past (`50 % * 2`). The zoom field of the standard bar reads
sums too. Values outside a field's range are brought into it
(`ui::field::tests`). Every number field has a pair of spin
arrows at its right (11 px): a click on one steps the value by a unit
(or the step the field was given), as does the mouse wheel over the
field; Shift steps ten times as much, Ctrl a tenth; a whole-number field
stays whole and the range holds (`ui::field::tests::arrows_and_the_wheel_step_the_value`).
A drag on the arrows, or on the field's label ("X:", shown with a
left-right cursor), slides the value: one step every 4 pixels, right or
up higher, left or down lower, with the same Shift and Ctrl steps,
within the range (#1303,
`ui::field::tests::dragging_the_arrows_or_the_label_slides_the_value`).
The wheel steps once a notch; parts of a notch from a touchpad or a fine
wheel add up to one step. The wheel over a field (or the outline width
box, the font list, the page tabs) belongs to it: the bar or panel
around it does not slide or scroll with that notch, on the frames the
smoothed scroll of the notch lasts as well, so the field stays under the
pointer and notch after notch steps it (#783,
`ui::field::tests::the_wheel_steps_a_field_in_a_long_bar_without_sliding_the_bar`,
`ui::field::tests::the_wheel_steps_a_field_in_a_scrolling_panel_without_scrolling_it`,
`ui::propbar::tests::the_wheel_over_the_outline_width_in_a_long_bar_keeps_the_bar_still`).
Every step applies at once, whether or not the box
holds the focus: the rotation angle, the scale percentages, the arc
angles, the dimension text size and line width, the corner radii and the
Transform toolbar's fields took only the first click after the box was
clicked into (the one that took the focus from it) and then none
(`ui::propbar::tests::every_click_on_the_rotation_arrows_turns_the_selection`).
The steps one field makes in a row are one undo step: a turn of the
wheel, clicks on the arrows or a drag on them or on the label, from 1 to
100, is one step named after the change ("Transform" for the size
fields), whose Undo puts back the value from before the first step and
whose Redo gives the value where the steps stopped. A step joins the one
before when it comes from the same field, to the same selection, with
the same kind of change, at most 0.8 s after the field's last step (any
pause while one drag is held), and nothing else changed the drawing in
between; another command, an Undo or Redo, another field or another
drawing starts a new step. The same holds for the outline width box and
for sliders stepped with the wheel (`step_merge::tests`).

- Pick tool, nothing selected (page bar): page size list; page width and
  height; fit page to drawing (Layout's command, pale on an empty page);
  portrait, landscape; all pages or current page (which pages a size
  change applies to, one undo step); Units; nudge distance; duplicate
  distances; treat all objects as filled.
- A selection (object bar): the object origin selector (a 3 x 3 grid,
  the centre by default: X and Y show and move that point of the
  selection's box, counted from the ruler origin), X and Y; width and
  height (scaled about the origin point); scale percentages and the lock
  that keeps proportions; the rotation angle of a single object (read
  from its transform, changed about the origin point); mirror
  horizontally and vertically.
- Then, by the kind of the selected objects: rectangle corners (Round,
  Scalloped and Chamfered Corner buttons; the four corner sizes in two
  stacked columns, top left over bottom left, then top right over bottom
  right, with spin arrows; the Edit Corners Together lock; Relative
  Corner Scaling);
  ellipse, pie or arc with start and end angles and the change direction
  button; polygon points or sides and star sharpness (1 to 99, or -1 to -99
  for an inverted star, #1331); with
  the Star tool the Star and Complex Star buttons come first, and a
  complex star's sharpness counts steps (1 up to what its points allow).
- Then the outline: width (Hairline, None, preset widths in the ruler
  unit), line style, start and end arrowheads; then To Front, To Back and
  Convert to Curves. The mouse wheel over the closed width box steps the
  selection's outline to the next width of the list up (wheel up) or
  down, stopping at None and at the widest; a width between two of the
  list goes to the one above or below. One step per notch: a wheel event
  of a notch or more counts as one notch whatever its size (browsers send
  a notch as 100 points or 3 lines), parts of a notch from a touchpad or
  a fine wheel add up until they make one (50 points make a notch), and
  what has added up is dropped when the wheel turns the other way or the
  pointer leaves the box. The notches of one turn of the wheel are one
undo step, as in the number fields. With nothing selected
  the wheel does nothing (#663).
- Zoom and Pan tools: the zoom box, Zoom In, Zoom Out, Zoom to Selected,
  to All Objects, to Page, to Page Width, to Page Height.
- Shape tool on curves: the Selection mode list, then picture buttons in
  groups (node edits, node types, curve direction and closing, node
  transforms and Align Nodes, Reflect and Elastic toggles, Select All
  Nodes, Reduce Nodes with the Curve smoothness slider); toggles are drawn
  pressed while on. The full list is in `shape-tool.md`.

## Bars longer than the window

The standard toolbar, the property bar, the text, zoom, transform and
Time Tracker bars and the toolbox show what fits between a "«" button at
their start and a "»" button at their end (up and down arrows for the
toolbox and the panel tabs on a side, drawn as small triangles: no
bundled font has the triangle characters they were, and Windows showed
empty boxes, #1780). "»" slides the bar so the first item not wholly in
view comes first; "«" slides it back, cut between items, to the start;
the mouse wheel over the bar (away from a number field) slides it too. The items stay as they are on
the bar and work as there (the bar is drawn once). A bar that fits shows
no buttons; the slide is worked out again when the window changes size
(`ui::overflow::tests`).

On a touch screen a finger dragged along a bar longer than its room
slides the bar with the finger (#1997): once the finger has moved past a
tap's distance (6 px), more along the bar than across it, the bar follows
its movement from there and stays where the finger leaves it. A tap on a
button stays a click; a finger that slid the bar clicks nothing when it
lifts, not the button it came down on and not the one under it then. While
a finger slides a bar its items start no drag of their own: a number
field's label or arrows step no value, a tool button opens no flyout, a
docker tab does not move, a palette swatch drags no colour, a slider's
thumb stays; a finger that moves across the bar instead (a colour dragged
off the palette onto the drawing, a tool button dragged sideways for its
flyout) keeps those drags. The mouse slides nothing by dragging: its
drags on the bars stay what they were. The same finger drag slides the
toolbox on a side (up and down), a tab column of dockers, the document
tabs and the colour palette (`color-palettes.md`); a docker's contents
longer than the docker scroll under a finger too (egui's scroll areas do
this on a touch screen, as soon as the screen has been touched once).
Nothing of this costs a frame anything until the screen has been touched
(`ui::overflow::finger_slide`, `ui::overflow::finger_sliding`,
`ui::overflow::tests::a_finger_dragged_along_a_long_bar_slides_it_with_the_finger`,
`ui::overflow::tests::a_finger_dragged_along_a_column_slides_it`,
`ui::overflow::tests::a_tap_clicks_a_button_and_a_slide_clicks_nothing`,
`ui::overflow::tests::a_finger_moving_across_the_bar_does_not_slide_it`,
`ui::overflow::tests::a_mouse_drag_does_not_slide_the_bar`,
`ui::tests::a_finger_slides_the_standard_toolbar_on_a_phone`,
`ui::toolbox::tests::a_finger_dragged_along_the_toolbox_column_slides_it`,
`ui::palette::tests::a_finger_dragged_along_the_palette_scrolls_it`,
`ui::dockers::tests::a_finger_dragged_up_a_docker_scrolls_its_contents`).

Ctrl+wheel over a toolbar (docked or floating) steps the default button
size (the bars' menu > Customize > Default Button Size) by the notches
turned: up larger, down smaller, through the five sizes from Extra
Small to Extra Large (held at both ends); over the toolbox
it steps the toolbox's own size (see `toolbox.md`), over a palette the
swatch size (see `color-palettes.md`). The wheel is taken, so the bar
does not slide with it and the drawing does not zoom; the plain wheel
slides a bar as before (`ui::bars::ctrl_wheel_steps`,
`ui::bars::size_buttons_by_wheel`, `settings::ButtonSize::stepped`).

Dialogs taller (or wider) than the window keep their size inside a
window that fits the screen, and their body scrolls; dockers scroll as
before.

## Moving the bars

The toolbox, the colour palette, the standard toolbar, the property bar,
the Text, Zoom and Transform toolbars, the time tracker and the status
bar move (`ui/bars.rs`).

| Bar | Places | First place |
|---|---|---|
| Toolbox | top, bottom, left, right, floating | left |
| Colour palette | top, bottom, left, right (each beside the drawing or at the window's edge; left and right also between two docker columns), floating | bottom, beside the drawing (on a side at first: the outermost column, beyond the dockers) |
| Standard toolbar | top, bottom, floating | top, first row |
| Property bar | top, bottom, floating | top, second row |
| Text, Zoom, Transform toolbars, time tracker | top, bottom, floating | top, a row each under the property bar (hidden at first) |
| Status bar | top, bottom | bottom |

- Toolbars in rows (#837, #855): the toolbars docked along the top or
  the bottom lie in rows, top to bottom; at first each has a row of its
  own. A row is a band as high as its highest toolbar (36 px, 53 px with
  the property bar), the toolbars side by side in it, each centred in
  the band with its grip first. Each toolbar takes the length its
  buttons need while the row has room, the last one the rest; when they
  do not all fit, the first ones keep their length and the later ones
  get what is left (at least 90 px each while the row holds that), the
  part they cannot show sliding behind their arrows as a bar longer than
  its room does (`ui::bars::share_row`).
- Dragging a toolbar's grip over a row shows a blue line where it would
  go: between the toolbars whose middles lie either side of the pointer
  (or at the row's end). Releasing puts it there, beside the others.
  Over the upper or lower 7 px of a row the mark is a band across the
  window on that line, and the toolbar gets a row of its own there.
  Away from the rows a toolbar docks at the nearest edge or floats as
  below; one coming from elsewhere takes a row of its own at the edge
  of the window (the topmost row along the top, the lowest along the
  bottom). Hidden toolbars keep their places in their rows and come
  back to them. The rows are kept in the preferences (`bars.rows`: each
  toolbar's row and place) and Reset Bar Positions gives each toolbar
  its first row again
  (`ui::bars::tests::toolbars_share_rows_and_keep_their_places`,
  `ui::bars::tests::a_toolbar_dropped_on_a_row_joins_it_or_starts_one`,
  `ui::bars::tests::a_row_shares_its_length`,
  `pick_drag_tests::a_toolbar_dragged_onto_another_row_shares_it`).

- A docked bar starts with a grip: a column of grey dots 9 px wide (a
  row of dots 9 px high on a bar that runs down a side). The pointer
  shows the move cursor over it.
- Dragging a grip shows where the bar will go as a light blue band with a
  blue frame: within 56 px of an edge the bar can take, the band along
  that edge; elsewhere, the window the bar floats in, with its title
  under the pointer. Releasing puts the bar there, also when the pointer
  has left the window (the last place it was seen counts). A bar that
  cannot float, the status bar, goes to the nearer of its two edges.
- The status bar keeps the outer band: other bars dock inside it, and the
  band shown while dragging leaves it out. Along the top the bars come
  below the menu bar.
- A floating bar is a small window with a grey border and a shadow: an
  18 px title with the bar's name at 11 px and a close button, then the
  bar. A name longer than the title's room (a long translation in a
  narrow window) is cut with an ellipsis and shows whole as a tooltip
  of the title; a window is never narrower than 68 px, room for the
  close button and a few letters (`ui::bars::title_room`,
  `ui::bars::tests::floating_titles_fit_their_windows`). The title runs
  the window's whole width, the close button at its right end, also
  when the bar's controls make the window wider (the property bar of
  another tool, #1921;
  `ui::bars::tests::a_floating_bars_close_button_follows_its_width`). Dragging the
  title moves it (near an edge it docks), a double click puts the bar
  back at its first place, and the close button hides it (Window >
  Toolbars shows it again). The floating toolbox is a grid two buttons
  wide at first that reflows when its window is resized by its edges
  and corners (see "Floating toolbox" in `toolbox.md`); the floating
  palette is one row 460 px wide.
- The palette docks at each side in two places: at the window's edge
  (a row across the whole window inside the toolbars, or a column
  outside the toolbox and the docker column) or beside the drawing (a
  row or a column between the drawing and the docker column, the toolbox
  or the toolbars on that side). While it is dragged, the band shows
  either place: within 56 px of the window's edge the band along that
  edge, within 56 px of the drawing's edge a band along the drawing
  (the nearer one wins; where the drawing reaches the window's edge it
  is the window's). Its menu lists all nine places: Dock at Top and Dock
  at Bottom beside the drawing, with "Across the Window" for the edge;
  Dock at Left and Dock at Right at the window's edge, with "Beside the
  Drawing" for the inner place; and Floating. The place is kept in the
  preferences (`bars.palette_beside`; preferences without it keep the
  places they had: beside the drawing at the top and the bottom, at the
  edge at the left and the right). Further palettes shown (#1164) go
  with it
  (`ui::bars::tests::the_palette_docks_beside_the_drawing_or_at_the_window_edge`,
  `ui::bars::tests::the_palette_dropped_beside_the_drawing_lies_between_it_and_the_dockers`).
- At the left and the right the palette also docks between the side's
  two docker columns (`PaletteSpot::Between`): a column of swatches
  between the column at the window's edge and the second column, each
  keeping its own tab strip (drawing | Properties | palette | Color
  Styles | window edge). While it is dragged, the line where the two
  columns meet (or the palette between them) is a target as the
  drawing's edges are: within 56 px of it, the nearest target wins, and
  the band is the palette's width across that line (the gap itself while
  the palette lies there). The columns keep the widths they were given
  and can still be widened or narrowed by their inner edges; the drawing
  takes what is left. Its menu has "Dock at Left, Between the Panels"
  and "Dock at Right, Between the Panels" after the edge place of that
  side, grey while the side has one docker column. The place is kept in
  the preferences (`bars.palette_between`, written only while on;
  preferences without it read as before) and comes back at the next
  start. While the side has one column the palette lies after it,
  beside the drawing, and goes back between the columns once there are
  two again. Further palettes shown go with it
  (`ui::bars::tests::the_palette_docks_between_two_docker_columns`,
  `ui::bars::tests::the_palette_dropped_between_two_docker_columns_lies_between_them`).
- The palette on a side is a column: the swatches run downwards and the
  arrows sit at its ends; the expanded palette opens beside the column
  (below a palette along the top). Wherever the palette is, the document
  palette is a row under the drawing window (over it when the palette is
  along the top). The toolbox along the top or the bottom is a row with
  its flyouts opening below or above it.
- Right-clicking a grip, an empty part of a toolbar or of the status bar,
  or a floating bar's title opens the bar's menu: the places the bar can
  take (a dot at the current one); every bar with a check at each one
  shown, as Window > Toolbars lists them (`menus::toolbars_list`: the
  menu bar, always on; the standard toolbar, the property bar, the
  toolbox, the colour palette, the AI prompt bar, the status bar, the
  Text, Zoom and Transform toolbars, the time tracker); Customize; Lock
  Toolbars and Reset Bar Positions. Window > Toolbars has the same list,
  Lock Toolbars and Reset Bar Positions. A right click on the buttons and
  boxes of a toolbar, docked or floating, opens the same menu (#835). The
  menu stays open after the button comes up, until an entry is picked, a
  click lands outside it or Esc is pressed; each click opens one menu,
  shown once a frame (`pick_drag_tests::a_right_click_on_any_toolbar_keeps_the_bar_menu_open`, #994).
- Customize holds: Default Button Size (Extra Small, Small, Medium,
  Large, Extra Large, Small at first, the same five sizes Ctrl+wheel
  steps through). The icons grow with the buttons; the padding around
  them grows more slowly, and separators keep their size:

  | Size | Toolbox icon box | Toolbox and standard bar button | Standard bar icon | Property bar icon | Property bar button |
  |---|---|---|---|---|---|
  | Extra Small | 16 px | 22 px | 16 px | 14 px | 20 px |
  | Small | 20 px | 26 px | 20 px | 18 px | 24 px |
  | Medium | 24 px | 32 px | 24 px | 22 px | 28 px |
  | Large | 32 px | 42 px | 28 px | 24 px | 38 px |
  | Extra Large | 40 px | 52 px | 34 px | 28 px | 46 px |

  The icon box is the side of the icon's 24 unit grid (toolbox 16, 20,
  24, 32, 40 px: a whole pixel every 3, 6, 1, 3 and 3 units, so the
  icons stay sharp). The standard bars' command pictures use the same
  box up to Medium. At Large and Extra Large the standard and property
  bars draw their icons at about 5/6 of the toolbox's box (standard 28
  and 34 px, property bar 24 and 28 px, whole even pixels so they sit
  centred on the pixel grid), which leaves 7 px of margin each side at
  Large and 9 px at Extra Large; drawn at the full box their heavier
  pictures filled the button almost to its edges. The toolbox keeps the
  full box (`settings::tests::large_bar_icons_keep_a_margin`). The
  toolbox's flyouts list their tools with 18 px icons up to Medium, 24
  px at Large and 30 px at Extra Large. Settings saved with three sizes
  keep them (`settings.button_size`, `settings::ButtonSize::metrics`,
  `theme::set_button_sizes`, `ui::bars::size_menu`,
  `settings::tests::each_button_size_has_a_larger_icon_and_proportional_padding`,
  `settings::tests::saved_button_sizes_keep_their_meaning`,
  `ui::bars::tests::the_size_menus_list_the_five_wheel_sizes`); Toolbox
  Toolbar (Reset to Default: every flyout shown as at first, each button
  on its first tool, the default size and icons only; Button Size:
  Default, which follows the default button size, or its own size
  among the same five, `settings.toolbox_button_size`; Image Only, Caption
  Only or Caption to Right, `settings.toolbox_captions`: with captions
  every toolbox button is as wide as the widest tool name of the flyouts
  shown, plus 6 px each side, so the column keeps one width whichever
  tools the buttons show, and the name is drawn at 12 px to the right of
  the icon or in its place); Workspace (Export Workspace, Import
  Workspace, the same as Window > Workspace); and Customize..., which
  opens Options > Customization > Command Bars.
- The buttons of the Standard and Zoom toolbars are lists the user can
  change (`ui/bar_items.rs`, `settings.bar_items`, kept only when changed;
  in an exported workspace too). In customize mode, with Tools > Options >
  Customization open (its dialog leaves those two bars open to the
  pointer) or with Alt held, a press on a button no longer runs it: the
  button is outlined under the pointer, a drag carries its picture, a
  line marks the gap it would go to, and on release it moves there (along
  its bar or to the other one) or, released off both bars, goes away.
  Commands and tools dragged from the Commands page join the bar at the
  gap; a command already on that bar only moves. Added buttons show the
  command's menu picture (its name when it has none) and run it as its
  menu item would; a tool's button selects the tool and stays pushed in
  while it is the tool. Reset (Command Bars page) and Reset Bar Positions
  bring the first buttons back. The Zoom toolbar shows the zoom box and
  picture buttons, as on the Zoom tool's property bar: Zoom In, Zoom Out,
  Zoom to Selected, to All Objects, to Page, to Page Width, to Page
  Height (#563). Checked by
  `pick_drag_tests::toolbar_buttons_move_and_leave_in_customize_mode` and
  `bar_items::tests`.
- Lock Toolbars hides the grips, stops every drag and greys the places in
  the menu. Reset Bar Positions puts every bar at its first place and
  shows it; the lock stays as it was.
- The layout and the lock are kept in the preferences (`bars`; in the
  browser, its local storage) as soon as they change; older preferences
  without it start with the first places. Layouts of version 1
  (`bars.version`), whose palette stood at the right edge by default,
  put a palette found there back at the bottom once; a palette anywhere
  else stays, and wherever the user puts it after that is kept
  (`ui::bars::tests::the_palette_starts_at_the_bottom_and_stays_where_it_is_put`).

## Checks

- `ui::propbar::tests::the_object_origin_picks_corners_edges_and_centre`
- `ui::propbar::tests::page_size_changes_one_page_or_all_in_one_step`
- `ui::propbar::tests::outline_widths_read_in_the_ruler_unit`
- `ui::propbar::tests::the_wheel_steps_through_the_outline_widths`,
  `ui::propbar::tests::the_wheel_over_the_outline_width_changes_it` (#663)
- `ui::propbar::tests::corner_edits_change_the_selection_in_one_step_or_the_defaults`
- `ui::mod::tests::window_draws_with_every_docker_tool_and_selection`
- `ui::toolbox::tests::buttons_follow_the_sizes_and_the_captions`
  draws every tool's bar.
- `ui::overflow::tests::forward_brings_the_first_hidden_item_to_the_start`
- `ui::overflow::tests::back_returns_between_items_and_to_the_start`
- `ui::overflow::tests::areas_holding_items_are_not_cut_points`
- `ui::overflow::tests::a_bar_that_fits_does_not_slide`
- `theme::glyph_tests::every_interface_symbol_is_in_the_bundled_fonts`:
  every symbol in the interface's string and character literals (U+2010
  to U+2BFF and the pictographs) is in the bundled fonts (Selawik and
  egui's own), so none needs a system font (#1780). Letters of other
  scripts (language names) are left to the system fonts; the bullet
  characters Bullets puts into the drawing's text are listed apart.
- `ui::bars::tests::a_dropped_bar_docks_at_the_nearest_edge_or_floats`
- `ui::bars::tests::bars_only_go_where_they_can`
- `ui::bars::tests::the_drop_zone_leaves_the_status_bar_outside`
- `ui::bars::tests::reset_brings_every_bar_back_and_keeps_the_lock`
- `ui::bars::tests::the_layout_survives_the_preferences`
- `raster::tests::a_canvas_that_moves_renders_again`: moving a bar moves
  the drawing window; the drawing is rendered again for its new place.
