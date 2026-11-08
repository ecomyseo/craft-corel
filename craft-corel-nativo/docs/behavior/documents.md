# Open drawings and document tabs

Several drawings can be open at once, one tab each in the strip above the
rulers:

- The strip is 26 px high over a 1 px `#00ADFE` line that runs the width
  of the window. Tabs are `#D8D8D8` with 1 px gaps of the strip colour
  `#EAEAEA`; the active tab is `#CEE3FF`; labels are 13 px black.
- The Welcome Screen tab comes first (a filled dark house and its title).
  With no drawing open it is the only tab, and the menu bar shows only
  File, Tools, Window and Help; File commands that need a drawing are
  disabled. While it is shown (with drawings open too) the toolbox and the
  property bar show pale, the colour palette keeps its colours and
  tooltips but takes no clicks (#1082), and the dockers that work on a drawing say "Open or create a
  drawing to use this docker." instead of showing the hidden drawing's
  selection; Hints still shows (#482,
  `ui::palette::tests::the_palette_rests_on_the_welcome_screen`).
- The page under the Welcome Screen's own tabs (Get Started, Workspace,
  What's New, Learn, Templates) scrolls with the wheel and a scroll bar
  when it is taller than the window, each tab keeping its own place; the
  tab row stays put, so recent documents below the window are reached
  (#1288, `ui::welcome::tests::the_recent_documents_scroll_into_view`).
- The Welcome Screen tab has a close button on hover (a middle click
  also closes it): the tab shrinks to its house alone (a 26 px square,
  the title as its tip) and the active drawing shows. With no drawing
  open the workspace stays empty and grey, the toolbox and property bar
  stay pale, and the Hints docker comes to the front at its Home topics
  (#1081). A click on the house, Window > Welcome Screen or Help >
  Welcome Screen shows the Welcome Screen again with its whole tab. The
  closed tab is remembered with the settings (#677).
- Each drawing's tab (at least 116 px wide) shows its file name, or its
  document name when it has never been saved, followed by `*` while it has
  unsaved changes. A close button shows on the hovered tab; a middle click
  also closes.
- The New tab after the last one (a grey plus on a 26 px square) runs
  File > New.
- More tabs than the strip is wide: the strip slides as an overflowing
  bar does (`toolbars.md`), with a « button at its start and a » button
  at its end (each slides it by whole tabs) and the mouse wheel over it.
  The active tab is slid into view whenever another drawing becomes
  active (a new or opened drawing, the Window menu), so the
  tab of the drawing shown is always on the strip (#1908). Check:
  `ui::tabs::tests::tabs_past_the_window_edge_can_be_reached`.

Each drawing keeps its own pages, objects, undo history, selection, view
(zoom and scroll), file, page number settings and drawing units.
Switching tabs ends the interaction in progress (a drag, text or table
cell editing, a curve being drawn, clip frame editing, symmetry editing).

New and opened drawings get a tab after the last one. A file that is
open already is not read into a second tab (#645): opening it again (the
Welcome Screen's recent drawings, File > Open and Open Recent, the
toolbar's Open, a file dropped on the Welcome Screen, the command line)
shows the tab of the drawing read from it or saved to it, its unsaved
changes kept, and the status bar says "<name> is already open". The same
file is found by another spelling of its path (links, `.` and `..`, the
letter case on Windows); a drawing read from a `.cdr`, `.svg`, `.pdf` or
other format counts as that file until it is saved under a name of its
own. Templates (`.tdt`, `.cdt`, `.cdrt`) always make a new untitled
drawing, File > Revert reads the file again, Import is not an open, and
the browser build does not look (uploads of one name are different
files). Window > New Window gives a second window on an open drawing
(`documents::tests::opening_an_open_file_again_shows_its_tab`,
`templates_always_open_a_new_drawing`).

Closing a tab
(File > Close, Ctrl+F4, the tab's close button) asks to save unsaved
changes, then shows the tab to its right, else the one to its left, else
the Welcome Screen (the empty workspace when its tab is closed). Window > Close All closes every drawing, asking about
each one with unsaved changes; Cancel stops. File > Exit, Alt+F4 and the
window's close button do the same and quit when every drawing is closed.
On macOS so do Command+Q, Quit TraceDraw in the application menu, the
Dock icon's Quit and a log-out or shut-down (#1669): AppKit asks the
application before it terminates (`applicationShouldTerminate:` in the
patched winit, `winit::platform::macos::set_quit_handler`); with an
unsaved drawing TraceDraw cancels the terminate and the next frame asks
about each one as File > Exit does (`system_quit`), with none the quit
goes ahead at once. A log-out with unsaved drawings therefore stops at
the question, as macOS expects of a document program.

The Window menu lists the open drawings, numbered, with a check on the
active one.

## Checks

- `documents::tests::each_drawing_keeps_its_objects_history_and_selection`
- `documents::tests::closing_moves_to_a_neighbour_then_to_the_welcome_screen`
- `documents::tests::exit_asks_about_each_unsaved_drawing_then_quits`
- `system_quit::tests::a_quit_with_an_unsaved_drawing_is_taken_over_and_asks_like_file_exit`,
  `a_quit_with_no_unsaved_drawing_goes_ahead`,
  `a_drawing_saved_before_the_frame_quits_without_asking`
- `winit_quit_request::tests::without_a_handler_the_quit_goes_ahead_and_with_one_the_handler_decides`
  (the patched winit's part)
- On a Mac (by hand, no headless check): with an unsaved drawing,
  Command+Q, the application menu's Quit and the Dock icon's Quit each
  show the Save / Don't Save / Cancel question; Cancel keeps TraceDraw
  running, Don't Save quits; with nothing unsaved each quits at once.
- Given a drawing open and the Welcome Screen shown, when its tab's
  close button is clicked, then the drawing shows and the tab is a house
  half as wide or less; a click on the house shows the Welcome Screen
  with its whole tab; with no drawing open it closes to the empty
  workspace with Hints at Home
  (`pick_drag_tests::the_welcome_tab_closes_to_its_house`).
- Given the Welcome Screen tab closed, when the last drawing closes, then
  the workspace stays empty
  (`pick_drag_tests::closing_the_last_drawing_keeps_the_welcome_tab_choice`).
