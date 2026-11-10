# Keyboard shortcuts

The default bindings. Every menu
item that shows a shortcut is bound to it; Tools > Options >
Customization > Commands lists them and takes shortcuts of your own
(see "Your own shortcuts" below). `Ctrl` is the Command key on macOS.

## Files and editing

| Keys | Action |
| --- | --- |
| Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Shift+S | New, Open, Save, Save As |
| Ctrl+I, Ctrl+E, Ctrl+P | Import, Export, Print |
| Ctrl+F4, Alt+F4 | Close window, Exit |
| Command+Q (macOS) | Quit TraceDraw from the application menu: asks about unsaved drawings as Exit does; File > Exit shows it there (see "On macOS" below) |
| Ctrl+Z, Ctrl+Shift+Z, Ctrl+R | Undo, Redo, Repeat |
| Ctrl+X, Ctrl+C, Ctrl+V, Ctrl+Shift+V | Cut, Copy, Paste, Paste in view |
| Ctrl+D, Ctrl+Shift+D | Duplicate, Repeat Copies panel |
| + (Pick tool, with a selection) | Duplicate in place |
| Ctrl+Space | the Pick tool, from any tool |
| Space | the Pick tool (not while a text or table cell is edited, where it types a space, nor during a drag, where it drops a copy); on the Pick tool, back to the tool used before it (`App::previous_tool`); Shift+Space and other combinations leave the tool alone |
| Ctrl+0 to Ctrl+9, 0 to 9, Alt+0 to Alt+9 | store, select (twice: zoom to) and add to selection groups |
| Ctrl+A, Delete or Backspace | Select all (with a guideline selected: every guideline of the page, #1753), Delete |
| Ctrl+F, Ctrl+J, Ctrl+W | Find and Replace panel, Options, Refresh window |


### Repeat

Edit > Repeat (Ctrl+R) does again, on the selection of the time, the
last repeatable action still in effect: a move, scale, rotation or
skew, a fill, an outline, a transparency, or a copy made with a right
click during a drag (which makes another copy, transformed as far again
from the new selection). A move, scale, rotation or skew is kept with
the box of what it was done to and repeated about the same place of the
box of the selection of the time: a rotation about the old box's centre
(or a centre of rotation moved inside it) turns about the new box's
centre, a resize from a side or corner handle resizes from the same
side or corner, and a move moves as far; the object it is repeated on
no longer moves away (#1289, `app::repeated_transform`). A copy made
with a right click during a drag is repeated the same way: a mirror
copy made by dragging a side handle across mirrors the next copy over
the same side of the selected copy, so the copies line up side by side;
it was mirrored over the original's side again and landed on the
original (#1909,
`pick_drag_tests::repeat_after_a_mirror_copy_continues_from_the_selected_copy`).

Repeat replays the last group of actions, not only the newest: the
newest action still in effect together with the actions done right
before it to the same objects, with no other undo step and no change of
selection in between (a move and then a fill change of one object, or a
copy made with a right click and then the copy's new fill), oldest
first, as one undo step named Repeat; up to 16 actions. A change of
selection between two actions starts a new group, so after moving one
object and filling another only the fill is repeated. Repeat again
replays the same group (#1566, #1547, `App::repeat_group`,
`Engine::step_below`,
`pick_drag_tests::repeat_replays_the_last_group_of_actions_on_one_selection`).

Each action is kept with the undo step it made
(`Engine::last_step_id`, `App::repeatables`): an action undone is never
repeated, the one before it is (#366); redone, it is the last again; and
an undone step replaced by another change is gone for good. A copy made
by a right click during a drag, or by its Repeat, is one undo step, the
copies and their move together (`Engine::begin_group` and
`end_group`, which also keep the undo memory limit from splitting
them; it was two, the first Undo only moving the copy back onto the
original, #471). Checks:
`pick_drag_tests::repeat_after_an_undo_takes_the_last_action_still_in_effect`,
`pick_drag_tests::repeat_after_a_right_click_copy_makes_another_copy` (#363),
`engine::tests::steps_have_ids_that_follow_undo_and_redo`.

## View

| Keys | Action |
| --- | --- |
| F2, Shift+F2, F3, F4, Shift+F4 | Zoom tool, zoom to selection, zoom out, zoom to fit, zoom to page (also while a text is being edited, #1699) |
| Ctrl++ (or Ctrl+=), Ctrl+- | Zoom in and out one step |
| Alt+Shift+R | Rulers on and off |
| Alt+Q, Alt+Shift+A, Alt+Shift+D | Snapping off, alignment guides, dynamic guides |
| Alt+Z, Ctrl+Y | Snap to objects, snap to the document grid, on and off |
| PgUp, PgDn | Previous and next page; on the first page PgUp, on the last PgDn opens Insert Page (one page, before the first or after the last; #1752, `pick_drag_tests::page_down_on_the_last_page_offers_to_insert_one`) |

## Objects

| Keys | Action |
| --- | --- |
| Ctrl+G, Ctrl+U | Group, Ungroup |
| Ctrl+L, Ctrl+K | Combine, Break apart |
| While typing text: Ctrl+L, Ctrl+E, Ctrl+R, Ctrl+J, Ctrl+H, Ctrl+N | Align left, center, right, justify, force justify, no alignment (the whole text typed at a point, which stays in place at the reference point of the X and Y fields; the paragraphs at the caret in a frame) (`pick_drag_tests::alignment_keys_while_typing`, `text_wrap_tests::a_new_alignment_keeps_the_reference_point_in_place`) |
| Ctrl+Q, Ctrl+Shift+Q | Convert to curves, Convert outline to object (a dashed outline keeps its dashes, each a closed shape, the pattern in widths as drawn; #490, `tools2::tests::a_dashed_outline_becomes_one_shape_per_dash`) |
| Ctrl+Home, Ctrl+End | To front and back of the page |
| Shift+PgUp, Shift+PgDn | To front and back of the layer |
| Ctrl+PgUp, Ctrl+PgDn | Forward one, back one |
| Alt+Enter | Properties panel |
| Ctrl+Shift+A | Paste Style From: a dialog with Outline pen, Outline color, Fill and Text properties (all ticked at first, kept), then a click on the object to copy them from to the other selected objects, one undo step; a right click or Esc cancels the wait (#1345, `pick_drag_tests::ctrl_shift_a_copies_the_ticked_properties_from_the_object_clicked`) |
| Alt+F7, Alt+F8, Alt+F9, Alt+F10 | Transformations panel on Position, Rotate, Scale and Mirror, Size |
| Ctrl+M | Merge table cells |
| Alt+Shift+F8, Alt+F11 | Run Macro, Macro Editor (Tools > Macros) |

## Text and effects

| Keys | Action |
| --- | --- |
| Ctrl+T, Ctrl+F11, Ctrl+F12 | Text, Insert Character, Fonts panels |
| Ctrl+Shift+T, Alt+F12 | Edit text, Align to baseline |
| F11, Shift+F11, F12, Shift+F12 | Gradient fill, Colour, Outline pen, Outline colour |
| Ctrl+F7, Ctrl+F8, Ctrl+F9, Ctrl+F10, Alt+F3, Ctrl+F3 | Envelope, Blend, Contour, Extrude, Lens, Symbols panels |

Panel shortcuts toggle: pressing one when its panel is already the
active tab hides the panel column.

Each key has one command. Ctrl+F12 opens the Fonts panel and Ctrl+F8
the Blend panel, so Text > Proofing > Spell Check and Text > Convert
have no built-in key (their menu rows showed Ctrl+F12 and Ctrl+F8,
which opened those panels instead, #1779); Options > Customization >
Commands gives them one. Check:
`ui::options::tests::no_two_default_commands_share_a_shortcut` (menu
commands, tools and panels; a menu row that opens a panel shows that
panel's key and counts as the panel).

## Align keys and tool keys

With a selection, the plain letters L, R, T, B, E, C and P align it
(left, right, top, bottom, centre horizontally, centre vertically,
centre of page); a single object aligns to the page. Without a
selection the same letters reach the tool keys (E Roughen, C none, B
none). Tools with letter keys: Z Zoom, H Pan, Y Polygon, A Spiral, D
Graph Paper, G Interactive Fill, M Mesh Fill, X Eraser, W Smear, V
Smudge, E Roughen, S Sketch, Shift+S Shape Recognition, I Artistic
Media; the Bezier, Pen, Crop and Knife tools have no key.

## Modal dialogs

- Esc closes any dialog without applying; while a dialog is open the
  shortcuts above are not interpreted, and typing does not reach text
  or a table cell being edited on the canvas.
- Dialogs are modal: while one is open, a transparent layer under it
  takes every click, drag and scroll aimed at the menus, toolbars,
  dockers, tabs and the drawing; only the dialog (and the lists it
  opens) can be used until it closes
  (`ui::dialogs::tests::an_open_dialog_takes_the_clicks_outside_it`).
- Tab and Shift+Tab move between the dialog's controls and stay in
  it: Tab with nothing focused (a dialog just opened) or from its last
  control goes to its first input box (its first control when it has
  none), Shift+Tab from its first control to its last input box. egui
  moves the focus in drawing order, so the menu bar, drawn first, took
  it and the File menu opened on the next keys (#1258,
  `chrome::keep_tab_in`, run for every dialog window, active only in
  the three frames after a Tab;
  `pick_drag_tests::tab_in_a_dialog_focuses_its_first_input_box`).
- The same holds for every other window and area: the floating
  panels, the Font Manager, the palette New and Rename windows, the
  effect picker, macro forms, the dropped-file question and the rest.
  Tab goes to the next control of the window that has the focus,
  Shift+Tab to the previous one; past its last control Tab goes round
  to its first input box (its first control when it has none), and
  Shift+Tab before its first control to its last input box. With
  nothing focused, Tab goes to the first input box of the window on
  top. The main window counts as one window without its menu bar: Tab
  never lands on the menu bar, and with nothing focused and no window
  open Tab stays with the drawing (it steps through the objects). A
  control that keeps Tab for itself keeps the focus, and with a menu
  or a list open Tab is left to it (#1258, `ui::tab_focus`, run around
  each frame; `pick_drag_tests::tab_stays_in_a_floating_panel`,
  `tab_stays_in_the_font_manager`, `tab_stays_in_the_palette_windows`,
  `tab_stays_in_the_effect_picker`, `tab_stays_in_the_drop_question`,
  `tab_stays_in_a_macro_form`,
  `tab_in_the_main_window_never_reaches_the_menu_bar`).
- Esc with a menu or a drop-down list open closes it and changes
  nothing else (the selection stays).

## Menus and lists from the keyboard

One navigation serves the menu bar, every menu and submenu, the
right-click menus (drawing, palette, rulers, dockers) and every
drop-down list (`ui/keynav.rs`, #1839). The highlighted row is drawn
like the row under the pointer; while the keys move it, the pointer's
row is not lit until the pointer moves.

- Alt pressed and released alone lights the menu bar's first title.
  Left and Right move along the titles (round from the last to the
  first), Home and End go to the first and the last, Down, Up, Enter or
  Space open the lit menu, a title's underlined letter opens its menu,
  Esc or Alt again lets the bar go, and a click anywhere or another key
  (a shortcut, which then runs) leaves it. Alt with a title's letter
  opens that menu at once, as before. F10 stays the Shape tool's key.
- A menu opened from the keyboard lights its first row. Down and Up
  move the highlight, skipping separators and greyed rows and going
  round from the last row to the first; Home and End go to the first
  and last rows, Page Down and Page Up by a menu's height. Enter or
  Space runs the row (a check row turns on or off), and the menu
  closes. Right on a row with a submenu opens the submenu with its first
  row lit, as do Enter and Space; Left or Esc closes it again and lights
  its row. Right on another row opens the next menu of the bar, Left in
  a menu of the bar the menu before. Esc closes one level: a submenu,
  then the menu (its title stays lit, a second Esc lets the bar go).
  The underlined letters keep picking their rows. A long menu scrolls
  to the row lit.
- A drop-down list (every list of the property bars, dockers and
  dialogs, the font size, outline width and zoom level lists): open, Down and Up
  move the highlight from the chosen row, Home and End go to the first
  and last rows, Page Down and Page Up by the rows in view; the list
  scrolls to the row lit at once, not animated. Letters typed jump to the first name starting
  with them (letters typed within a second add up, `Bo` finds Bookman;
  the same letter again goes on to the next name with it). Enter picks
  the row and closes the list, Esc closes it as it was. The list's
  button (a combo box's) keeps the keyboard afterwards.
- A list's button with the keyboard (reached with Tab from a field of
  the bar or dialog): Down, Up, Alt+Down, F4, Enter or Space open the
  list with its chosen row lit.
- The font list: its field takes the keyboard when the list opens, also
  when it is opened with its arrow, its name selected so that typing
  replaces it and filters the families. Down, Up, Page Down, Page Up,
  Home and End (while nothing is typed) move the highlight, Enter picks,
  Esc closes it as it was. Closed, Down and Up in its field still step
  to the family after or before (#1754).
- While a menu or list is open, or the bar is lit, the drawing takes no
  key: the arrows never nudge the selection, a node or the caret of the
  text being edited.
- Tests: `ui::keynav::tests` (the File menu with Alt and the arrows, a
  submenu, greyed rows skipped, the canvas's right-click menu, a list
  with the arrows, letters, Enter and Esc, a long list scrolled to the
  row lit, the zoom levels, Tab to the property bar's lists, no nudge
  while a menu is open) and
  `ui::font_list::live_tests::the_list_opened_from_its_arrow_takes_the_keys`.
- Tools keep their own keys (see `pick-tool.md`, `shape-tool.md` and
  the tool pages); the Commands page of the options can rebind them.

## Your own shortcuts

Tools > Options > Customization > Commands lists every command of the
menu bar (commands not made yet are left out) as "Menu > Submenu >
Command" with its built-in shortcut. A search field filters the list by
name.

- Clicking a command's button ("Set shortcut") makes it wait for keys:
  the next key pressed, with the Ctrl, Alt and Shift held at the time,
  becomes its shortcut (shown as `Ctrl+Alt+Shift+Key`). Esc, Tab,
  Enter, Backspace and Space alone are not taken (they edit text, move
  focus or close dialogs); Esc stops the wait without closing the
  options. Changing page or closing the options stops it too.
- A shortcut of your own runs its command wherever the command's menu
  item would: the menus are walked without being shown and the item
  reports a click, so the command does exactly what the menu does. A
  command not available at that moment (Duplicate with nothing
  selected, Edit commands with no drawing open) runs nothing and the
  status bar says so.
- Your shortcuts are looked at before the built-in ones, so a key a
  built-in command uses runs your command instead; the page names the
  commands and tools that lost it. Another command of yours with the
  same keys loses them, and a command or tool that had them built in
  is left without a shortcut (kept as an empty one, as Delete leaves
  it; Reset gives it back), so one key runs one command and no two
  commands or tools share one (#1595,
  `ui::options::tests::a_shortcut_given_to_a_command_is_taken_from_its_built_in_holder`).
  A command's built-in shortcut keeps working beside yours until
  another command takes it. A key of yours on a tool picks the tool as
  its built-in key would (once a press, not W or K during a move).
- The menus show your shortcut in place of the built-in one. Delete
  leaves the command without a shortcut, a built-in one too: it is
  kept as an empty shortcut, the menus show none and its keys do
  nothing. Reset to defaults gives back the built-in shortcuts of the
  commands in the category shown (one menu, the toolbox or the
  macros); with all commands shown it asks first, as Reset all
  shortcuts does, which gives back every built-in shortcut, tool keys
  included. Shortcuts are kept in the preferences (`shortcuts`: the
  menu item's key or tool id and the keys, empty for none).
- Tools are listed under the toolbox on the same page and take keys as
  commands do: a key alone or with Ctrl, Alt and Shift. Alt and a
  letter you give a command or tool runs it instead of opening the
  menu with that access letter, and while the page waits for keys no
  menu opens.
- Rows a menu shows only at times are listed all the same and take keys
  (Object > Clip to Frame > Finish Editing Contents beside Edit Contents,
  Text > Vertical Text), #1148.
- The bars' own buttons (listed under Toolbar Buttons) that run a menu
  command show that command's keys on their Shortcut Keys tab and set
  them: New, Open, Save, Print, Cut, Copy, Paste, Undo, Redo, Import,
  Export, Export as PDF, Full-screen Preview, Rulers, Grid,
  Guidelines, Snap Off, Zoom In, Zoom Out and Zoom to Fit
  (`bar_items::menu_command`, #1148). The buttons the menus lack (the
  zoom levels list, Snap To, the other zoom buttons, Options, Launch)
  have no Shortcut Keys tab.

## In the browser

Some of these keys stay with the browser in the browser version, which
gives no page Ctrl+N, Ctrl+T, Ctrl+W, Ctrl+Shift+T, Ctrl+F4 or Alt+F4,
and in some browsers or systems keeps Ctrl+PgUp, Ctrl+PgDn and Ctrl+1 to
Ctrl+9 (tab switching) or Ctrl+Q and Ctrl+Shift+Q (quitting, on Linux).
The page keeps from the browser every one of these it is allowed to, so
they work where the browser lets them (`web.md`, Keyboard). Each command
also has a key of the browser version, which the menus, context menus,
toolbar tips and hints show there instead of the key the browser keeps
(the desktop apps show and run their own keys only; both keys work in
the browser):

| Command | Desktop key | In the browser |
|---|---|---|
| File > New | Ctrl+N | Alt+Shift+N |
| Text panel | Ctrl+T | Alt+Shift+X |
| Edit Text | Ctrl+Shift+T | Alt+Shift+T |
| Window > Refresh Window | Ctrl+W | Ctrl+F5 |
| Window > Close Window | Ctrl+F4 | Alt+Shift+W |
| Order > Forward One, Back One | Ctrl+PgUp, Ctrl+PgDn | Alt+PgUp, Alt+PgDn |
| Convert to Curves | Ctrl+Q | Alt+Shift+C |
| Convert Outline to Object | Ctrl+Shift+Q | Alt+Shift+Q |
| Store selection group 0 to 9 | Ctrl+0 to Ctrl+9 | Alt+Shift+0 to Alt+Shift+9 (the digit keys whatever Shift types on them) |
| File > Exit | Alt+F4 | none: close the tab |

Each of these keys is bound to nothing else
(`ui::menus::tests::browser_alternatives_are_free`). Options >
Customization > Commands says which keys the browser keeps and lists
their replacements. While typing text, the alignment keys the browser
keeps (Ctrl+N) have their buttons on the property bar. Checks:
`shortcuts::tests::browser_kept_keys_map_to_alternatives`,
`ui::menus::tests::browser_kept_shortcuts_have_alternatives_in_the_menus`,
`web_page::tests::the_editors_tab_keys_and_their_alternatives_are_kept_from_the_browser`.

## On macOS

macOS keeps some Command keys for itself in every program, before the
window sees them: Quit and Log Out (Command+Q, Command+Shift+Q), Hide and
Hide Others (Command+H, Command+Option+H), Minimize (Command+M), the
application switcher (Command+Tab, Command+Shift+Tab), Spotlight and the
Finder's search (Command+Space, Command+Option+Space), Force Quit
(Command+Option+Esc), the screenshots (Command+Shift+3, 4 and 5) and
hiding the Dock (Command+Option+D) (`shortcuts::MAC_SYSTEM_KEYS`). The
commands whose key on the other systems is one of these have another key
in the macOS desktop app, which the menus, context menus, tips, hints and
Options > Customization > Commands show there (#1409; they write
Command as Ctrl and Option as Alt, so Option+Shift+C shows as
Alt+Shift+C):

| Command | Other systems | macOS desktop app |
|---|---|---|
| Convert to Curves | Ctrl+Q | Option+Shift+C |
| Convert Outline to Object | Ctrl+Shift+Q | Option+Shift+Q |
| Merge Cells (table) | Ctrl+M | Command+Shift+M |
| The Pick tool from any tool | Ctrl+Space | Command+Shift+Space |
| Force justify, while typing text | Ctrl+H | Command+Shift+H |
| File > Exit | Alt+F4 | Command+Q (the application menu's Quit; Option+F4 still works) |

The keys macOS keeps do nothing in the window should one arrive (a
Command+Q never converts to curves), and Options > Customization >
Commands does not take one as a shortcut there. The browser version on a
Mac follows "In the browser" above. Checks:
`ui::options::tests::no_default_shortcut_is_a_macos_system_key` (no
built-in key of a menu command, tool, panel, the canvas or the text
being typed is one of them in the macOS app, File > Exit's Command+Q
aside; each alternative is bound to nothing else; no two commands share
a key there),
`pick_drag_tests::macos_keeps_its_command_keys_and_the_commands_take_others`.

## Checks

- `ui::options::tests::bar_buttons_and_rows_shown_at_times_take_shortcuts`
- `ui::options::tests::no_two_default_commands_share_a_shortcut`: no two
  menu commands or tools have the same built-in keys, the documented
  split of E (Align Centers Horizontally with a selection, Roughen
  without) aside.

- Given a dialog open, when Esc is pressed, then the dialog is closed
  and the tool and selection are unchanged.
- Given a selected object, when Ctrl+Home is pressed, then it is the
  last object of the top layer; Shift+PgDn makes it the first of its
  layer.
- Given Select All given Alt+Shift+K and nothing selected, when
  Alt+Shift+K is pressed, then every object is selected and the Edit
  menu shows Alt+Shift+K beside Select All.
- Given Select All given Ctrl+D, when Ctrl+D is pressed, then the
  objects are selected and nothing is duplicated; Ctrl+A still selects
  all.
- Given Export waiting for its keys, when Ctrl+D is pressed, then Export
  has Ctrl+D and the page says it was Duplicate's; Enter alone is not
  taken and Esc stops waiting with the options still open.

## Access keys in menus

- Every menu title and row has one letter underlined (`ui/menus.rs`,
  `access`): the first letter or digit starting a word that no earlier
  row of the same menu took, else the first free one anywhere. Letters
  outside A to Z and 0 to 9 are skipped, so labels in other scripts may
  have none.
- With a menu open, pressing a row's letter (without Ctrl or Alt) works
  as a click: an item runs, a submenu opens. Only the innermost open
  menu takes the key.
- Alt and a title's letter opens that menu, except E, S, X, Q and Z,
  which commands use with Alt.
- While any menu or list is open, the drawing's shortcuts do not run.
