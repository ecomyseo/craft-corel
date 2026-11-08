# Colour palettes

Two rows under the drawing window: the default palette, then the
document palette above the status bar (Window > Color Palettes >
Document Palette shows or hides it). The dotted grip at the start of the
default palette moves it to another edge (on a side it is a column) or
into a floating window with as many rows as the Rows setting (see "Moving the bars" in
`toolbars.md`); where it is put is kept across restarts, in the browser
too. The document palette is a row under the drawing window wherever
the default palette is (over the drawing window when the default palette
is along the top). Each row moves by its own grip: dragging the
document palette's grip takes it out into a palette bar of its own
(below) and leaves the default palette where it is, and the default
palette's grip moves only the default palette. The document palette's
bar let go at the row's side (the bottom, or the top while the default
palette is there) is the row again; anywhere else it stays a bar,
docked or floating. While the document palette has a bar of its own
its row does not show as well. Window > Toolbars > Color Palette hides or shows the
default palette. The rows described below are the same along a column,
turned downwards.

## Palette bars

Each further palette is a bar of its own (#1164 gave each a strip next
to the default palette; now each moves by itself). A palette opens in a
bar from Window > Color Palettes (ticking it), the Palette Manager (its
eye, or Open as New Palette Bar in its menu), File/palette menu > Open
Palette, or a palette menu's Open as Palette Bar (every palette not
shown yet, and the document palette, which can have a bar of its own
too, with its eyedropper and its menu). A new bar docks where the
default palette is (at the bottom while that floats), with the default
palette's swatch size and one row. A palette shows in one bar at most.

- Place: any edge (Top, Bottom: a row beside the drawing, inside the
  default palette's rows; Left, Right: a column at the window's edge,
  inside the default palette's column) or Floating. The bar's grip, or
  the title of its floating window, drags it: near an edge (56 px) it
  docks there, elsewhere it floats where it is let go (a blue band shows
  where). The palette menu's Position > Top, Bottom, Left, Right,
  Floating does the same; a double click on a floating bar's title docks
  it at the bottom. Lock Toolbars keeps the bars where they are.
- Each bar has its own Rows and Swatch Size; the default palette's are
  the ones in Options, which the document palette row under it follows.
- Close Palette in its menu (or the close button of its floating window)
  closes the bar.
- Floating: a small window with the palette's name (cut with an ellipsis
  and shown whole as a tooltip when it does not fit) and a close button.
  Its right and bottom edges and the corner between them resize it; the
  room asked for is kept (`float_size`) and the swatches flow into it
  (`ui::palette::floating_flow`): along the longer side, as many lines
  across as whole swatches fit in the shorter one, `lines = floor((across
  - side - 5) / (side + 1)) + 1`, 1 to 24, so a wide window has rows and
  a tall one columns; never shorter than the buttons and two swatches.
  The window is as thick as its whole lines. Rows N in the menu makes a
  floating palette N lines thick at its length; Fit, as many as show
  every colour. The default palette floating flows the same way.
- Kept in the preferences (`palette_bars`: the palette's id, its file's
  path, built-in key or name, or `document`; `dock`, `pos`,
  `float_size`, `rows`, `swatch_size`) and in workspace files with the
  palette part, together with the palette the default bar shows
  (`main_palette`, empty for the default palette) and the names given
  to palettes. A palette file not open yet is opened again at start (on
  the desktop); a bar whose palette is gone closes. Preferences and
  workspace files written before palette bars existed have none: the
  default palette alone at the bottom, with its rows and swatch size.
- Ticking off the default bar's palette while palette bars are open
  moves the first palette bar's palette into it (its bar closes); the
  last palette shown stays.

## Names

Every palette can be given a name of the user's (Rename... in its
palette menu or the Palette Manager, or a double click on its name in
the Palette Manager): built-in palettes and the document palette too.
The name shows wherever the palette is listed (the Palette Manager,
Window > Color Palettes, Open as Palette Bar, a floating bar's title).
Names are kept in the preferences and workspace files by the palette's
id (`palette_names`), so a built-in palette's own name and file never
change; Reset Name (or an empty name, or its own) gives back its own.
A palette of the user's kept in a `.gpl`, `.tdpal` or `.json` file that
is not locked gets the name in the file too (`Name:`, `"name"`), and
opening such a file names the palette from it.

## Look

- Each row is 25 px: a 1 px `#D8D8D8` line, a pixel of space, the
  swatches (20 x 20 px in a 1 px `#5A5A5A` frame that also separates
  them), a pixel of space. The background is the panel colour `#F4F4F4`.
- Swatch size: the palette menu's Swatch Size offers five named sizes,
  Extra Small 10 px, Small 14, Medium 20 (the first size), Large 28 and
  Extra Large 40 (`settings::SWATCH_STEPS`, all even); Options and the
  Palette Manager's slider offer the same. Ctrl+wheel over a palette row
  (or column) keeps its finer steps: larger (wheel up) or smaller (wheel
  down), 2 px a notch, from 8 to 48 px, so it reaches every named size
  and the sizes between; a size between two named ones shows in the menu
  as Custom (n px), no named size ticked. The size belongs to the bar
  under the pointer (the default palette's for its row and the document
  palette row); the shades pop-up and the window showing every colour
  follow the default palette's. The size is kept in the
  preferences (`palette.swatch_size`, whole points; preferences written
  before it existed keep 20 px). Rows are 5 px thicker than the swatch.
  Ctrl+wheel scrolls nothing and the drawing does not zoom; the plain
  wheel scrolls as before (`settings::swatch_resized`,
  `ui::palette::tests::ctrl_wheel_sizes_the_swatches_and_the_plain_wheel_scrolls`,
  `settings::tests::the_wheel_steps_the_button_and_swatch_sizes_within_their_ranges`).
- From the left: a dotted grip, the palette menu button (a dark triangle),
  the eyedropper (grey on the default palette, which cannot change; dark
  on the document palette), the left scroll arrow, then the swatches from
  77 px. At the right: the right scroll arrow and the button that shows
  every colour at once. Arrows are light grey when they cannot scroll.
- Rows (the palette menu's Rows > 1 Row to 8 Rows and Fit, and Options >
  Customization > Color Palette; 1 at first, up to 8; Fit shows as many
  rows as every colour needs at the length the bar had, at most 8,
  `palette.rows` 0): each palette bar shows that many rows of swatches (columns when docked on a
  side), each further row making it 21 px thicker. The swatches fill
  line by line: each line holds as many swatches as fit and the next
  line goes on where it ends (with 2 rows, Black and 90% Black sit side
  by side on the first line). The arrows and the wheel scroll one swatch
  (a wheel notch is one swatch in the browser too, parts of a notch
  from a touchpad adding up, #1406,
  `ui::palette::tests::a_browser_wheel_notch_steps_a_palette_row_once`),
  and the buttons stay on the first row. The document palette keeps one
  row (#610, `ui::palette::tests::the_default_palette_can_show_several_rows`,
  `ui::palette::tests::the_palette_menu_picks_the_rows`).
- The No Color well comes first: white with a red diagonal.
- The swatch under the pointer gets a white frame; the document palette's
  current swatch (the one clicked last) keeps it.
- An empty document palette shows "Drop colors or objects here to keep
  them with the document".

## Default palette

The default CMYK palette, 94 named colours, is made by a rule
(`palette::default_cmyk`), every ink a whole percentage:

1. The basics (22), first in the strip: Black, 90% Black to 10% Black,
   White, the process colours Blue, Cyan, Green, Yellow, Red, Magenta,
   then Purple (20 80 0 20), Orange (0 60 100 0), Pink (0 40 20 0), Dark
   Brown (0 20 20 60) and Powder Blue (20 20 0 0).
2. The classic mixes after them (72), in 20% steps of the inks and going
   from the blues through the greens, yellows and browns, reds and pinks
   to the violets and purples and the deep blues. Every one has a plain
   descriptive name ("Dusty Blue", "Slate Blue", "Pale Slate", "Olive
   Green", "Dark Red", "Lilac"), none of them repeated.

The list is the classic palette with five swatches taken out of its first
50: after the basics, one swatch in every fourth or fifth (4 kept, 1
out, 5 kept, 1 out and so on; the 27th, 33rd, 38th, 44th and 49th
swatches of the classic list). Swatches after the 50th are unchanged.

The palette only decides what a new window shows. A drawing keeps the
colours it stores, so changing the palette never changes a saved
drawing, and the `.cdr` writer's document palette is separate (written
empty).

In a drawing whose primary colour mode is RGB (New Document's colour
mode, Document Info) the default palette shows and applies its
colours as clean RGB values (names kept; spot inks and registration stay
as they are). The default palette's colours come from the RGB default
palette, a table of RGB values defined directly
(`palette::default_rgb`, the same 94 names in the same order, also the
Palette Manager's built-in Default RGB palette), found by name and colour
so a default palette edited in the Palette Editor keeps them, and worked
out when the palette changes rather than every frame: 10%
greys (90% Black 26 26 26, 50% Black 128 128 128) and multiples of 17
otherwise, the plain ink formula below rounded to the nearest multiple
(Black 0 0 0, Red 255 0 0, Orange 255 102 0, Sky Blue 0 204 255, Dark Green
0 51 51, Dusty Blue 102 102 204; no two swatches share a value, Deep Blue is 0 0 68). Any other CMYK colour
takes the plain RGB of its inks, R = 255 (1 - min(1, C + K)) and so on,
not the press simulation; other colours are converted as the colour
management converts. A CMYK drawing keeps them CMYK (#483, #620,
`ui::palette::tests::an_rgb_drawing_gets_an_rgb_palette`,
`ui::palette::tests::the_default_palette_of_an_rgb_drawing_is_clean_rgb`,
`palette::tests::the_default_palette_follows_its_rule`,
`ui::palette::tests::inks_with_black_become_rgb_by_adding_the_black`,
`ui::palette::tests::edited_default_palettes_keep_their_rgb_values`).

## Using a palette

| Action | Result |
|---|---|
| Click a swatch | Uniform fill of the selection (with nothing selected: the default fill for new objects); every click counts: one whose pointer slipped a few pixels but was let go on the same swatch, and the first click after a hold that opened the shades (#1336, `pick_drag_tests::every_click_on_a_palette_colour_applies_it`) |
| Right click | Outline colour (Options > Customization > Color Palette can make it open the palette menu instead); a right click that slipped a few pixels on the swatch counts too (#1336) |
| Ctrl+click | A tenth of the colour mixes into the selection's uniform fill (CMYK with CMYK, else in RGB) |
| A finger (touch screen): tap, hold, hold longer | A tap is a click (the fill). Held 0.5 s and let go: the outline colour, as the right click (when Options make the right click open the menu, a finger held 0.8 s opens it, egui's long press). Held 1 s: the shades pop-up, whose tap fills and whose long press colours the outline; the release then changes nothing. A finger that slides off the row (towards the drawing) drags the colour; one that slides along a row longer than its room scrolls the row with the finger, a swatch for every swatch's width, and fills nothing when it lifts (#1611, #1707, #1997, `ui::palette::tests::a_finger_held_on_a_swatch_sets_the_outline`, `ui::palette::tests::a_finger_dragged_along_the_palette_scrolls_it`) |
| Click and hold (0.5 s) | A 5 x 5 pop-up of the colour's shades: lighter tints upwards, more black downwards, less colourful to the left, more to the right; click one (right click for the outline) |
| Drag a swatch onto an object (left button) | Inside it (also inside a closed object with no fill): its uniform fill; within 4 points of its outline: the outline's colour (an object with no outline gets the default pen in that colour); No Color takes the fill or the outline away. The object need not be selected and the selection stays; one undo step; the colour joins the document palette as with a click |
| Drag a swatch onto the document palette | The colour joins it |
| Drag objects (Pick tool) onto the document palette | Their fill and outline colours join it (as Add from Selection); the objects stay where they were; the pointer shows a copy sign there |
| While dragging a swatch | A chip of the colour follows the pointer: filled over a fill, a ring over an outline; the pointer shows whether letting go changes anything; Esc drops nothing |
| Scroll arrows, wheel | One swatch at a time (every row moves by one); holding an arrow keeps scrolling |
| Ctrl+wheel | Larger (up) or smaller (down) swatches, 2 px a notch, 8 to 48 px, kept in the preferences; nothing scrolls or zooms |
| The show-all button | Every colour of the row in a grid above it (below it, clear of every row, when the palette is along the top; beside a column, clear of every column; #610, `ui::palette::tests::every_colour_opens_clear_of_all_the_rows`); click to apply, click elsewhere or Esc to close |
| Tooltip | The colour's name in bold, then its values one per line (`C: 60`, `M: 0`, `Y: 40`, `K: 20`; `R: 255`, `G: 0`, `B: 0`) |
| No drawing shown (Welcome Screen, or none open) | The swatches keep their colours and their tooltips; clicks and drags do nothing and the eyedropper is off (#1082) |

## Document palette

- Every colour applied to an object (fill or outline, from a palette, the
  colour panel or a dialog) joins the document palette, in the same undo
  step as the change. The palette menu's Automatically Update turns that
  off (also in Options > Customization > Color Palette).
- A colour dragged in a colour picker (Properties > Fill > Uniform, and
  Properties > Outline > Colour): the object shows each colour the
  pointer passes over, but the whole drag is one undo step and only the
  colour it ends on joins the document palette, when the button is let
  go. Other drags in the outline editor (its sliders) are one undo step
  too.
- With characters selected in a text being edited, a swatch colours
  only them (see `text.md`, Character colours).
- The eyedropper waits for a click on the drawing and adds the colour
  under the pointer; with Ctrl it keeps sampling. Esc or a right click
  stops it.
- Its menu: Automatically Update, Add from Selection, Add from Document
  (every colour used in the drawing, after the ones already there), Add
  Colors from Bitmap (see below),
  Delete Color (the current swatch), Palette > Reset Palette (keeps only
  the colours the drawing still uses).
- It is saved with the drawing; opening a `.cdr` reads its
  `color/docPalette.xml`.

## Default palette menu

Open Palette... (files of `.tdpal`, `.xml`, `.gpl`, `.ase`, `.aco` and
`.json`, the filter naming each extension; an XML palette is read as
the document palette of a `.cdr` file, its CMYK, RGB, grey, Lab and the
other colour models with their names, spot colours skipped, and a file
with no colour that can be read says so in the status bar, #882,
`palette::tests::xml_palettes_open_with_names_and_values`; the palette
opened gets a strip of its own next to the default palette, which stays
first, and the same file opened again replaces the copy already open
instead of adding another, #1343), Palette Editor..., Palette Manager, the Document
Palette switch, Swatch Size (five sizes), Rows (1 Row to 8 Rows, Fit),
Position (Top, Bottom, Left, Right, Floating), Open as Palette Bar and
Rename... (radio choices kept in the preferences, the same settings as
in Options). A palette bar's menu has the Palette Editor, the Palette
Manager, Swatch Size, Rows, Position, Open as Palette Bar, Rename... and
Close Palette; the document palette's bar has the document palette's
items instead of the first two. A right click on a row outside the
swatches, or on a floating bar's title, opens the same menu.

## Palette Manager

The Palettes docker: a filter box (the palettes whose name holds it
show) with Open Palette, the document palette from the drawing and from
the selection as picture buttons; a New Palette... button; the Swatch
Size box (the selected palette's name, a slider with a small swatch at
the left, a large one at the right and a tick at each named size, the
size in words, and "Rows of selected bar", the rows of the selected
palette's bar); then three sections, Built-in, My Palettes and Document
Palette.

- New Palette... (the Palette Manager's button, Window > Color Palettes
  and every palette bar's menu) asks for a name ("Palette 1", "Palette
  2"..., the first no palette has; a name taken gets a number) and makes
  an empty palette of the user's under My Palettes, shown at once in a
  bar of its own docked at the bottom, under the bars there, so several
  palettes stand in bars one above the other (bars docked at the bottom
  read from the top down in their order, as the Palette Manager lists
  them). An empty bar says "Drop
  colors here to add them to this palette". A swatch dragged from any
  palette onto the bar of a palette of the user's (not locked) joins it,
  and the bar's menu has Add Colors from Selection (the fill and outline
  colours of the selected objects) and Add Colors from Bitmap.

Add Colors from Bitmap (#1840), in the document palette's menu and in
the bar menu of a palette of the user's: a submenu of 8, 16, 32 or 64
colours, greyed unless a picture is selected. It takes at most that many
colours that stand for the selected picture (its pixels as shown, effects
included), chosen by the median cut of the paletted bitmap conversion
(`bitmap-modes.md`, Optimized palette; transparent pixels are left out),
as RGB colours ordered dark to light, and adds those the palette has not
got after its colours; in the document palette that is one undo step.
Check: `ui::palette::tests::colours_come_from_the_selected_picture`. Palettes made in TraceDraw (New
  Palette, Duplicate), which have no file, are kept with their colours
  in the preferences (`user_palettes`).

- Each palette is a row: an eye (open: shown in a bar; a click opens or
  closes its bar), a Show bar / Hide bar button that does the same in
  words (the last palette shown keeps its bar), and while the bar shows
  a list of where it is (Top, Bottom, Left, Right, Floating) that moves
  it; its name in bold (with a padlock when locked), a
  strip of its first colours (up to 16, as many as fit), and its colour
  count, colour model (CMYK, RGB, Grayscale, Spot, Mixed, Empty) and
  where it shows (Default bar or Bar, and the edge or Floating; Not
  shown). A click selects it, a double click on its name renames it, a
  row dragged onto another row of its section moves there, and the
  three-dot button or a right click opens its menu: Open as New Palette
  Bar or Close Palette Bar, Edit..., Rename..., Reset Name, Duplicate (a
  palette of the user's named "name copy", then "name copy 2"), Lock or
  Unlock and Delete Palette (palettes of the user's; the file stays),
  Move Up, Move Down.
- The slider sets the swatch size of the selected palette's bar as it
  moves: 8 to 48 px in the 2 px steps of Ctrl+wheel, taking a named
  size within 1.5 px of its tick (`ui::palette_manager::slider_size`).
  A palette not shown has no bar to size: the slider is greyed and says
  so. The default bar's palette is selected at first.
- The document palette's row: the eye shows or hides its row under the
  drawing; its menu opens or closes a bar of its own and renames it.
  Under it its colours (a click fills, a right click colours the
  outline) and Add Selection Colors, Add from Document and Clear (one
  undo step).

## Palette Manager lock

Each palette that is not built in has a Lock button in the Palette
Manager (Unlock while locked, shown pressed). A locked palette opens in
the Palette Editor as a built-in one does: its name, colours and values
cannot be changed, Add, Remove and Save are gone, and a line says to
unlock it in the Palette Manager. Opening its file again keeps its
colours. Locks last for the session (#1343,
`palette::tests::opening_a_palette_adds_one_and_a_locked_one_keeps_its_colours`).

## Options > Customization > Color Palette

Show "No Color" well (on); Rows of the default palette (1 to 8 or Fit;
`palette.rows`, 1 at first); Swatch size (the five named sizes, a size
set with Ctrl+wheel between two shown as it is); right mouse button: Context menu or Set
outline color (default); Show the document palette (on); Automatically
update the document palette (on).

## Checks

- `ui::palette::tests::tooltips_name_the_colour_and_its_values`
- `ui::palette::tests::the_default_palette_has_greys_process_colours_and_tints`
- `ui::palette::tests::shades_keep_the_colour_in_the_middle`
- `ui::palette::tests::mixing_moves_a_tenth_of_the_way`
- `ui::palette::tests::a_finger_held_on_a_swatch_sets_the_outline`
- `ui::palette::tests::applying_a_colour_adds_it_to_the_document_palette_in_the_same_step`
- `ui::palette::tests::rows_fit_whole_swatches`
- `ui::palette::tests::the_default_palette_of_an_rgb_drawing_is_clean_rgb`
- `ui::palette::tests::the_default_palette_can_show_several_rows`
- `ui::palette::tests::every_colour_opens_clear_of_all_the_rows`
- `pick_drag_tests::every_palette_shown_gets_a_strip`
- `pick_drag_tests::every_palette_bar_applies_its_colours_docked_or_floating`
- `ui::palette_bars::tests` (the menu's sizes, rows 1 to 8 and places;
  each bar's own rows, size and place; floating flow; resizing a
  floating palette; bars and names kept in workspace files and the
  preferences; old preferences give one palette at the bottom; names and
  Reset Name; a palette file keeps its name)
- `ui::palette_bars::rename_tests::the_rename_window_takes_the_typed_name`
- `ui::palette_manager::tests` (the slider's steps, the slider sizes the
  selected bar, the list's eye, duplicate, move, delete and filter, the
  colour model)
- `pick_drag_tests::every_click_on_a_palette_colour_applies_it`
- `ui::dockers::tests::a_fill_dragged_in_the_picker_is_one_step_and_one_palette_colour`
- `color_drop::tests` (fill inside, outline on the edge, No Color, an
  object without a fill, the document palette)
- `tracedraw_core` `engine::tests::amending_joins_the_last_step_only_right_after_it`
- `tracedraw_cdr` `container::tests::document_palette_xml_reads_names_and_tints`
