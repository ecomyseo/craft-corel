# Toolbox

- One button per flyout, top to bottom, with separator lines after the
  Shape, Zoom, Polygon, Text, Connector, Transparency and Fill groups
  (before the "+"). The Variable Outline tool has a button of its own,
  right under the brush tools' flyout (Brush Strokes, Painterly Brush,
  Shape Recognition, Sketch) and over the Rectangle flyout, inside the
  drawing block, with no line between the brushes and it; it was at the
  bottom, after the Fill flyout, before
  (`ui::toolbox::tests::the_variable_outline_button_sits_under_the_brush_tools`).
  A button shows the tool last used from its flyout (kept in the
  settings as one tool id per group, in the groups' order; each id goes
  to the group that holds its tool, so a list saved with the groups in
  an older order still applies,
  `ops::smoke_tests::toolbox_picks_saved_in_the_old_group_order_still_apply`);
  the active tool's button is tinted.
- A right click on a tool button or on the toolbox's empty part opens the
  bars' menu (Customize, button size, the bars), as on the other bars.
- Buttons with a flyout have a small arrow at the bottom-right. Clicking
  the arrow, dragging, or pressing and holding the button for 0.4 s
  opens the flyout; a click elsewhere picks the tool the button
  shows. While a flyout is open, hovering another flyout button opens that
  one instead; moving away from the toolbox or clicking elsewhere closes
  it.
- A flyout lists its tools vertically: a check mark beside the tool the
  button shows, the icon, the name and the shortcut at the right.
  Separators split Pick and Freehand Pick from Free Transform; Crop, Knife
  and Segment Delete from Eraser; Polygon, Star, Spiral and Common
  Shapes from Action Lines and Graph Paper; the dimension tools from the 3-point
  callout; the connectors from Anchor Editing; Interactive and Area Fill
  from Mesh Fill.
- Tooltips: "<Name> tool (<shortcut>)" in bold, then what the tool does.
  No tooltip shows while a flyout is open.
- Double-clicking a button: Pick selects all objects, Zoom fits all
  objects, Rectangle adds a page frame.
- The "+" at the end lists every flyout with a check box to show or hide
  it (the Outline flyout is hidden by default) and Reset toolbox.
- The toolbox starts docked on the left. Its grip (the dots above the
  first button) drags it to another edge, where it is a column or a row,
  or into a floating window, two buttons wide at first, that reflows
  when it is resized (see "Floating toolbox" below and "Moving the
  bars" in `toolbars.md`). Flyouts open away from the edge the toolbox
  is on.

## Floating toolbox

- The floating window holds the buttons in rows, left to right then
  down, in the room asked for (`ui::toolbox::flow`): a button goes on
  the row while it fits, with a 1 px gap before it and, where a group
  ends before it, a separator cell (a vertical line, 5 px) between the
  two groups; else it starts the next row. Where a group ends at a
  row's end, a horizontal line (3 px) lies between the rows. The "+"
  button is the last cell. When the rows stand taller than the height
  asked for (by more than half a button), they get wider, a button at a
  time, until they fit it or make one row. So a tall narrow window
  gives two columns (or one), a wider one more columns and fewer rows,
  a wide short one a row or two.
- The window hugs the grid: it is the grid, 4 px of margin at each side
  and 3 px above and below, the 1 px border and the title, never
  narrower than the title's least width (`bars::FLOATING_MIN_W`, 68 px,
  room for the close button and a few letters). Dragging an edge or a
  corner of the window (the pointer shows the resize arrows over them)
  asks the grid for the room between the pointer and the opposite edge,
  and the grid follows live: a side edge asks for a width and the rows
  take the height they need, the top or the bottom edge asks for a
  height and the rows widen to fit it, a corner asks for both. The
  edge opposite the one dragged stays in place. When the drag ends the
  size settles on the grid's own.
- The size is kept in the preferences (`bars.toolbox_size`), in cells
  of the grid (a button and its gap each,
  `ui::toolbox::floating_cell`), so a toolbox three buttons wide stays
  three wide when its buttons change size; none kept, two buttons
  wide. Reset Bar Positions forgets it. A workspace file carries it
  with the bars' layout (an unusable size reads as none).
- Flyouts open to the right of the button while the grid is taller than
  wide, and under it while it is wider (a row or two); moving onto
  another button of the grid switches flyouts, moving off the grid
  closes them.
- Checks: `ui::toolbox::tests::a_floating_toolbox_flows_to_the_room_asked_for`,
  `ui::toolbox::tests::the_floating_toolbox_starts_two_buttons_wide_and_keeps_its_size`,
  `ui::bars::tests::the_floating_toolbox_resizes_by_its_edges_and_by_ctrl_wheel`,
  `workspace_file::tests::odd_files_are_refused_or_tamed`.

## Button size and captions

The bars' menu (right-click a bar) > Customize sizes the buttons (Default
Button Size for every bar; Toolbox Toolbar > Button Size for the toolbox
alone) and chooses what they show: Image Only (the icon, its box 16,
20, 24, 32 or 40 px from Extra Small to Extra Large, so the icon grows
with the button; a column squeezed to fit a short window shrinks it in
proportion, `theme::icon_in`, `ui::toolbox::tests::icons_grow_with_every_button_size`),
Caption Only (the tool's name) or
Caption to Right (both). See `toolbars.md`, "Moving the bars".

Ctrl+wheel over the toolbox (docked or floating) steps its own button
size by the notches turned, up larger and down smaller, through the
five sizes from Extra Small to Extra Large (held at both ends), as
Toolbox Toolbar > Button Size does; the floating toolbox keeps its room
in cells, so it stays as many buttons wide at every size. The wheel is
taken, so the
toolbox does not slide with it and the drawing does not zoom
(`ui::toolbox::size_by_wheel`, `settings::ButtonSize::stepped`,
`ui::bars::tests::the_floating_toolbox_resizes_by_its_edges_and_by_ctrl_wheel`,
`settings::tests::the_wheel_steps_the_button_and_swatch_sizes_within_their_ranges`).

A toolbox taller (or, along the top or the bottom, longer) than its room
slides with its arrows and the wheel (`toolbars.md`, "Bars longer than
the window"), and on a touch screen with a finger dragged along it: the
column follows the finger once it has moved past a tap's distance, a tap
still picks the tool, and the button the finger passes over opens no
flyout; a finger dragged sideways off a button opens the flyout as before
(#1997, `ui::toolbox::tests::a_finger_dragged_along_the_toolbox_column_slides_it`).

Docked on a side, the toolbox fits the window's height: when its buttons
do not fit, they get shorter (their icons smaller) down to 22 px, and
only below that do the overflow arrows take over; with room they keep
their size (#494, `ui::toolbox::tests::a_column_toolbox_fits_the_window`).

## Checks

- `ui::toolbox::tests::flyout_groups_can_be_hidden_and_reset`
- `ui::toolbox::tests::separators_split_the_toolbox_subgroups`
- `ui::toolbox::tests::separator_lines_follow_the_same_groups`
- `ui::toolbox::tests::the_variable_outline_button_sits_under_the_brush_tools`
- `ui::icons::tests::coloured_icons_follow_the_toolbox_groups_and_can_be_off`
- `ops::smoke_tests::toolbox_groups_remember_the_last_tool_used`
- `ops::smoke_tests::toolbox_picks_saved_in_the_old_group_order_still_apply`

## The pointer over the drawing

Each tool's pointer says which tool is active (code: `ui/cursors.rs`):

| Tools | Pointer |
|---|---|
| Pick, Freehand Pick | the arrow, with resize, move, turning and slant arrows over handles and the selection (see `pick-tool.md`) |
| Shape | a solid arrow marking nodes, control points and segments (see `shape-tool.md`) |
| Zoom | a magnifier with a plus (a minus with Shift) |
| Pan, and any tool while the middle button pans | an open hand, closed while dragging |
| Colour and Attributes Eyedropper, palette sampling | a pipette |
| Area Fill | a paint bucket |
| Text | the text I-beam over text (a click places the caret there), elsewhere the cross hair with the tool's icon |
| Eraser | the cross hair inside the nib's outline at its size on screen (a circle, a square for the square nib) |
| Smooth, Smear, Twirl, Attract and Repel, Smudge, Roughen | the cross hair inside a circle of the brush size |
| Freehand, Bezier, Pen, B-spline, Polyline, 2-point line | the cross hair with the tool's icon; above and right of it a node with a line going on over an end node of the selected open curve (a line from there continues it), a ring over the first node of the curve being drawn once it has three nodes (a click closes it) |
| Interactive Fill | the move arrows over a handle of the selected object's fountain fill (and while one is dragged), else the cross hair with the tool's icon |
| Every other tool (shapes, dimensions, connectors, effects, fills, Crop, Knife, Table, ...) | the cross hair with the tool's icon at its lower right on a light patch |

The arrows (pointing, moving, resizing) are the system's own. The cross
hair and the text I-beam are painted by TraceDraw, black on a white
halo, with the system pointer hidden, so they show on a page of any
colour: the system's invert what is under them, and on Windows they can
show white on a white page (#655,
`cursors::tests::the_cross_hair_and_the_i_beam_are_drawn_black_on_white`).
On Windows the system pointer is not hidden but swapped for a fully
transparent pointer image: Windows hides and shows the pointer only at
the next mouse move, so a tool picked with its shortcut over a still
mouse kept the old pointer, or none at all after a tool with a drawn
pointer, until the mouse moved, while a new pointer image shows at once.
Whatever the frame last asked for decides, so a system pointer asked for
after a drawn one in the same frame (a text box's I-beam) is never
replaced by the blank image (#1124,
`cursors::tests::a_drawn_pointer_hides_the_system_one_with_a_blank_image_on_windows`).
A tool shortcut with Alt (Alt+R set by the user) changes the pointer at
once on Windows too: Alt with a letter starts no keyboard menu mode in
the window, and the system ending the pointer's leave tracking while
the pointer stands still over the window is not taken as the pointer
leaving it, so the pointer is never left as the arrow or invisible
until the mouse moves (#1922, `winit_alt_shortcuts::tests`, see
`vendor/README.md`).
The cross hair's lines cross on the middle of the pixel under the hot
spot; the icons, nibs and drawn pointers are painted beside it, above
the drawing. A tool's icon sits 4 points below and right of the cross
hair's middle, clear of its arms, so the pointer reads as one piece
(#493, `cursors::tests::the_badge_sits_close_to_the_cross_hair`). The window keeps one frame queued, not two, so what is
drawn under the pointer trails it as little as the screen allows, and a
shape being drawn has its corner where the pointer is from the first
frame of the drag (it waited at the press point for one frame,
`pick_drag_tests::drawing_a_rectangle_the_corner_keeps_up_with_the_pointer`). The icon on the light patch is painted in the pointers' own
black under every theme, not in the theme's icon colour, which is light
on the dark themes (`cursors::tests::the_badge_icon_is_dark_under_every_theme`).
Nib outlines stay between 2 and 400 points on screen.

## Clicks with drawing tools

With the drawing and shaping tools whose click has no job of its own
(Rectangle, Ellipse, Polygon, Star, Graph Paper, Action Lines and the
other box tools, Spiral, Common Shapes, Table outside a table, Knife,
Brush Strokes, the connectors, Blend, Block Shadow, Shape Recognition,
Sketch and the shape brushes Smooth, Smear, Twirl, Attract and Repel,
Smudge and Roughen), a click selects the way the Pick tool does: the
object under the pointer becomes the selection, Shift+click adds or
removes it, Ctrl+click reaches one group level further in, and a click
on empty space lets go of the selection (Shift+click there keeps it).
The tool stays active. A drag still does the tool's work, also when it
starts on top of an object (`Tool::click_selects`, #271).

Tools whose click already does something keep it: the line tools
(Freehand, Bezier, Pen, B-spline, Polyline, 2-point line, the 3-point
tools) place points, Text places the caret, and the fill tools,
eyedroppers, Zoom, Crop, Eraser, Segment Delete, Area Fill, the
dimension and callout tools and the effect tools keep their own clicks.

Checks: `pick_drag_tests::every_click_selecting_tool_selects_on_click`,
`pick_drag_tests::a_click_with_a_drawing_tool_selects_and_a_drag_draws`,
`pick_drag_tests::shift_click_with_a_drawing_tool_adds_and_takes_away`,
`pick_drag_tests::line_tools_still_place_points_on_objects`,
`pick_drag_tests::a_click_outside_a_table_selects_with_the_table_tool`.
