# Workspaces

A workspace is how the window is set up: which bars show and where they
sit, the toolbox, keyboard shortcuts, the colour palette options and the
interface's look. Window > Workspace picks a built-in one (Standard, Minimal,
Compact, Drawing, Publishing, Touch); the user's own setup can be
kept in a file and brought back.

## Touch

For touch screens: the interface (bars, buttons, panels, menus and text)
is drawn `TOUCH_SCALE` = 1.4 times the interface size chosen (at most
400 %), and drawings keep their real size at zoom 100 % (the points per
inch shrink by as much). The standard toolbar, property bar and toolbox
show; rulers, panels and the status bar do not. On the drawing two
fingers pan and a pinch zooms about its centre, with any tool; the
gesture drops what the first finger had begun, so it neither drags an
object nor draws a marquee. Picking another workspace brings the
interface back to the size chosen.

With a finger only (#1611, #1707, #1499): the standard toolbar's Delete deletes
the selection, its Shift, Ctrl and Alt buttons hold those keys down
until they are tapped again (a tap adds to the selection with Shift held) and its Panels button shows the dockers (Properties and
the rest); a finger held still on the drawing for 0.8 s is the right
click (egui's long press), so it opens the menu of the object under it
(picking it) or of the page, with Delete, Copy, Order, Group and the
rest; on a palette swatch a tap fills, a hold of 0.5 s let go sets the
outline and a hold of 1 s shows the shades (`color-palettes.md`).
Checks: `ui::tests::a_finger_deletes_opens_the_panels_and_the_object_menu`,
`ui::tests::the_touch_workspace_is_larger_and_keeps_real_sizes`,
`pick_drag_tests::two_fingers_pan_without_dragging_anything`.

On a phone (a screen whose smaller side is under 600 density-independent
pixels, Android's line between phone and tablet layouts, in either
orientation) the interface is not enlarged: a 360 dp phone would be
257 points wide, too narrow for any bar (#1301, #1691). The phone
layout: the bars, the property bar, the toolbox and the palette slide
with their « and » buttons, the wheel, or a finger dragged along them
(#1997, `toolbars.md`, "Bars longer than the window"); the menu titles
that do not fit go under » at the end of the menu bar; an open panel is
a sheet over the whole room beside its tab strip instead of a column
beside the drawing; the Welcome Screen's tabs wrap and Get Started puts
the recent drawings under its buttons, the page scrolling; a dialog
larger than the screen is drawn at the screen's size and its body
scrolls both ways. A tablet (600 dp and more) keeps its layout.
Checks: `ui::tests::the_touch_workspace_is_larger_and_keeps_real_sizes`,
`pick_drag_tests::two_fingers_pan_without_dragging_anything`,
`ui::phone::tests::phones_are_told_by_their_smaller_side`,
`ui::tests::phones_fit_and_tablets_keep_their_layout` (360 x 640 and
412 x 915 dp phones and an 800 x 1280 tablet, upright and turned:
nothing ends outside the screen).

## Kept across restarts

Everything below is part of the preferences (`settings.json` in the
configuration folder, or the browser's local storage under
`tracedraw.settings`, which belongs to that browser on that computer)
and comes back when TraceDraw starts again: every page of Options, where
each bar sits (`bars`), which bars show (`shown_bars`: standard toolbar,
property bar, toolbox, status bar, panels, rulers, Text, Zoom and
Transform toolbars), the tool each toolbox group shows, hidden flyouts,
shortcuts, palette options, interface size and desktop colour, the
dockers (`dockers`: the dockers in the tab column, in order, and the one
in front), the units, nudge and duplicate distances, snapping (`snap`:
what it snaps to, alignment and dynamic guides, snapping off), the tool
settings new objects take (`tools`: polygon and star points, sharpness,
rectangle corners, ellipse arcs, spiral revolutions, table and graph
paper rows and columns, blend and contour steps, contour offset, eraser
width and shape, text font, size and hyphenation), what the window
shows (`view`: grid, guidelines, page border, bleed, printable area,
pixel and baseline grids, objects that will not print, proof colours,
overprints, the view mode) and the dockers' own choices (`panels`: the
Color docker's model and views, Align To, Use Outline, Distribute To).

The preferences are saved whenever they change, written at most once a
second (so a change lasts even when TraceDraw does not close normally,
and in the browser, where closing the tab saves nothing), on Options' OK,
and when TraceDraw closes. They are looked at on a frame whose input
could change them (a key, text, a click or release, the clipboard, a
dropped file) and at most once a second otherwise (a frame that does
not look asks for one when the second is up): pointer moves, drags in
progress, the wheel and animation frames do not serialize them all,
which cost every frame on slow computers (#616). Not while Options is open, whose Cancel puts
them back. Values out of range in a file edited by hand are brought into
range, and names TraceDraw does not know fall back to the defaults.
Preferences written before the bars shown were kept have no
`shown_bars`: the built-in workspace's bars show, as before; without
`dockers` the column holds Hints, Properties and Objects; without
`tools`, `view` or `panels` they start as on a first start. Picking a
built-in workspace sets its own bars, as it always did.
Checks: `app::tests::tools_view_snapping_and_dockers_survive_a_restart`,
`app::tests::preferences_are_saved_when_they_change`,
`app::tests::preferences_are_looked_at_only_when_they_could_have_changed`,
`pick_drag_tests::an_idle_window_asks_for_no_frame`.

## Dockers

The tab column holds the open dockers, as tabs; a click on a tab brings
its docker to the front, a click on the tab in front collapses the
docker area to the column. The docker's close button (x) and its tab's
"Close Docker" take the docker out of the column and bring the next one
forward; with none left the docker area closes. Collapse (the two
triangles) only folds the area. A docker opened from a menu or its
shortcut joins the column; Help > Hints opens Hints and, ticked, closes
it. The column is kept when TraceDraw closes, so a docker closed stays
closed the next time.
Checks: `app::tests::a_closed_docker_stays_closed_the_next_time`.

## Properties docker

With objects selected: the first one's kind and box, a row of buttons,
then its sections, in this order: Outline, Fill, Transparency, then
what the kind has (Rectangle: its corners; a bitmap: its effects; text:
its character settings), then Summary. Each section has a header bar
with a triangle; the buttons (the outline pen, the fill bucket, the
transparency checker, a rectangle, "fx", "A", "i") show which sections
are open and open or close them, as a click on a header does. Sections
come open; the ones closed are kept in the preferences
(`properties_closed`). F11 and the status bar's fill and outline open
their section again. The fill kinds are pictures (none, uniform,
gradient, pattern, texture, mesh) with their names as tooltips; the
Transparency section holds the Transparency tool's settings in rows
that keep to the docker's width: the kind and what it applies to, then
the amount (or the gradient's angle and kind), then the merge mode. The
Rectangle section lists the corner kinds (round, scalloped, chamfered)
one under the other and the four radii two by two as the corners lie,
each named above its field; the two columns share the docker's width
and a name too long for its column goes on to a second line, never cut
short (#487). With nothing selected the docker shows the defaults for
new objects under the same Outline and Fill header bars, in the same
order, so each row of paint kinds is named and the docker keeps its
layout when an object is selected (#487). The docker keeps the width it has (340
points by default, or what the edge was dragged to): selecting an
object never widens it, and its contents are laid out within it and
clipped to it. Every section of every kind of object (rectangle,
ellipse, polygon, artistic and paragraph text, bitmap) with each fill
and transparency kind fits the default width, in English, German,
Russian and Brazilian Portuguese.
Checks: `ui::dockers::tests::properties_sections_open_close_and_jump`,
`ui::dockers::tests::properties_sections_fit_the_default_docker_width`,
`ui::dockers::tests::properties_sections_are_titled_and_add_buttons_need_a_paint`;
every kind's sections draw in
`ui::mod::tests::window_draws_with_every_docker_tool_and_selection`.

## Export and import

Options > Workspaces > Workspace > Export and import, or Window >
Workspace > Export Workspace... and Import Workspace...

| Part | What it holds |
|---|---|
| Keyboard shortcuts | the user's shortcuts for menu commands and tools (`Settings::shortcuts`) |
| Toolbars, toolbox and status bar | the built-in workspace it starts from, where the bars sit and whether they are locked, which bars show, the tool each toolbox group shows, hidden flyouts, the Outline flyout, what the status bar's first field shows |
| Color palette options | No Color well, right mouse button, document palette, the palette shown, colours joining the document palette |
| Appearance | interface size, desktop colour, page border |

- The options page has a check box per part (remembered in
  `workspace_parts`); Export writes the parts checked, Import applies the
  parts checked that the file has. The Window menu items use every part.
- An import from the options is part of the options: Cancel puts the
  previous setup back, OK keeps it. From the Window menu it is kept and
  saved at once.
- The status bar names the file and the parts applied, or says the file
  is not a workspace file or has none of the parts chosen.

## The file

JSON with the extension `.tdws`; parts not exported are left out.

```json
{
  "kind": "tracedraw-workspace",
  "version": 1,
  "base": "default",
  "shortcuts": [["menu.file.export", "Ctrl+Shift+X"], ["pick", "Q"]],
  "bars": {
    "layout": { "toolbox": "right", "palette": "bottom", "locked": true },
    "shown": { "status_bar": false, "zoom_toolbar": true },
    "toolbox": ["pick", "shape"],
    "toolbox_hidden": ["eraser"],
    "show_outline_flyout": false,
    "status_info": "CursorCoordinates"
  },
  "palette": { "show_no_color": true, "right_click_outline": true },
  "appearance": { "ui_size": 1.0, "desktop_rgb": [255, 255, 255], "page_border": true }
}
```

A file is never trusted:

- More than 4 MB, not JSON, a part of the wrong shape, another `kind`
  or version 0: refused, nothing changes. Later versions are read as far
  as their parts are understood; unknown fields are ignored.
- Shortcuts with an empty or overlong (over 64 characters) id or keys
  are dropped; the last keys given for an id win; at most 4096 are kept.
- Floating bar positions that are not finite or lie beyond 100000 points
  fall back to the default layout's; toolbox lists keep at most 256
  entries, and an overlong tool id counts as none.
- An interface size outside 50 % to 400 % becomes 100 %.

## Checks

- Given a workspace with custom shortcuts, bars on the right, the status
  bar hidden, palette options and an interface size of 125 %, when it is
  exported and imported into a fresh editor, then every part is the same.
- Given a file exported with shortcuts and palette only, when it is
  imported with only shortcuts checked, then the bars and palette stay as
  they were.
- Given garbage, an empty file, another kind, version 0 or a part of the
  wrong shape, when it is imported, then nothing changes and the status
  bar says it is not a workspace file.
- Given the status bar hidden and the Transform toolbar shown, when
  TraceDraw starts again, then they are still hidden and shown.
