# Symmetry

Live mirrored copies of an object across one or more mirror lines
(Object > Symmetry).

## Model

`Effect::Symmetry { center, angle, lines }` on the object. `center` is a
page point, `angle` the direction of the first mirror line in degrees
(0 = horizontal, counter-clockwise positive), `lines` the number of
mirror lines (1 to 12), spaced evenly around `center`.

## Result

With `n` lines the copies are the dihedral orbit of the object: a
reflection across each line plus the rotations those reflections
generate, `2n - 1` copies in all. The copies are drawn above the original
(`evaluate()` puts them in `above`), share its fill, outline and other
effects, and follow every edit of the original, because they are
recomputed from it.

## Defaults

- Create (Object > Symmetry > Create New Symmetry, Alt+S, or the object
  context menu): one vertical line (90 degrees) through the right edge of
  the selection's bounds, at its vertical centre, so the copy appears next
  to the original instead of over it; then straight into editing it.
  Several selected objects are grouped first and the symmetry goes on the
  group (one "Symmetry" undo step), so they mirror together; Ctrl+click
  picks one of them inside the group (round handles) to move or colour
  it alone, the copies following.
- While a symmetry is edited, an object drawn, pasted or duplicated
  joins it: the group takes it in (an object that was alone with the
  symmetry becomes a group of it and the newcomer, the symmetry moving
  to the group), folded into the step that made the object, so one Undo
  takes both back; the newcomer stays selected inside
  (`App::join_symmetry`,
  `ops2::symmetry_tests::a_symmetry_takes_several_objects_and_those_made_while_editing`).
- Edit (Object > Symmetry > Edit Symmetry, Alt+E): the symmetry editing
  mode (`symmetry_edit.rs`). A bar floats at the top left of the drawing
  window: Finish; the mirror lines icon and their number (1 to 12); the
  centre (X, Y in the drawing's units); the angle of rotation icon and
  the angle (0 to 360 degrees), every number field with spin arrows and
  the mouse wheel; Show full preview (off at first: the copies are drawn
  as red outlines and the renderer leaves them out; on: drawn in full)
  and Show symmetry lines (on). Both toggles are kept in the
  preferences. A run of changes is one undo step "Edit Symmetry".
- While editing, the bar and the lines stay whatever is selected (a
  member picked inside, a new object, a double click): they follow the
  object being edited, not the selection. Only then: after Finish the
  bar and the mirror lines go, and the symmetry, selected whole, shows
  its selection box alone like any object (a click takes it whole, a
  drag moves it whole;
  `pick_drag_tests::finish_on_the_symmetry_bar_leaves_one_object`).
- Picking while editing: a click or a marquee takes the objects inside
  the symmetry group (its members), never the group and never a copy.
  A symmetry made of one object picks that object alone: a click on a
  copy takes nothing, and its selection box is the object's own, not the
  box of the copies
  (`pick_drag_tests::a_lone_symmetry_object_is_picked_alone_and_takes_new_drawings`).
- The mirror lines with the Pick tool: where a line runs over an object,
  a click or a drag there takes the object; elsewhere a drag on a line or
  on the centre moves the lines (the centre follows the pointer, snapping as objects
  do); a click on a line shows round handles at both ends of every line,
  90 pixels from the centre, and dragging one turns the lines about the
  centre (Ctrl: by the constrain angle); another click on a line hides
  them (`pick_drag_tests::symmetry_editing_picks_members_and_drags_lines`).
- Finish Editing Symmetry (Alt+Shift+E) and the bar's Finish end the
  mode and select the whole symmetry, not the object picked inside it
  while editing (which a drag then moved alone)
  (`pick_drag_tests::finish_selects_the_whole_symmetry_of_several_objects`);
  the mode also ends when the object is deleted or loses its symmetry,
  and as Finish does when another drawing comes to the front, a new one
  opens or the drawing's tab closes; its status bar message goes with it
  (`documents::tests::another_drawing_ends_the_symmetry_editing`, #442).
- Out of the mode a symmetry is one object: a click picks the whole of
  it and a drag moves it all; a double click on it (or on an object
  inside it) goes back into editing it, instead of into focus mode
  (`pick_drag_tests::a_finished_symmetry_is_picked_whole_and_a_double_click_edits_it`).
  Moving, scaling, turning or mirroring it (a drag, the handles, the
  bars, the Transformations docker, Align and Distribute, nudges, Repeat,
  Clear Transformations, paragraph text resized by its frame) carries its
  centre and mirror lines with the object (`Shape::transform_by`,
  `live::carry_effects`), so the whole symmetry takes the change and the
  drag's outline shows the copies too, also those of a symmetry inside a
  selected group. So do Duplicate, copies left by a right click or by
  Space during a drag, the Transformations docker's copies, Step and
  Repeat, a symmetry switched off in the Effects docker, Ungroup (a
  symmetry inside a group keeps its line in the group's space and comes
  out onto the page with it), symbols made from it, Export of the
  selection only, Convert to Bitmap and patterns made from it. The
  lines used to stay where they were: the copies mirrored the move and a
  handle rearranged them, which looked like one object still being edited
  (`pick_drag_tests::a_finished_symmetry_moves_whole_with_its_copies`,
  `a_finished_symmetry_scales_whole_with_its_copies`,
  `a_duplicate_of_a_symmetry_takes_its_line_along`,
  `a_symmetry_in_a_group_follows_the_group_and_an_ungroup`, core test
  `a_symmetry_is_carried_with_its_object`). A move, a turn, an even scale
  and a mirror carry the symmetry exactly; a stretch or a slant cannot
  keep the copies mirror images of the changed whole, so the centre goes
  with the object and the lines keep their angle (mirrored when the
  change mirrors): the copies stay mirror images of the changed object.
  Commands that only rewrite where an outline is stored, not where it is
  (Join Curves, Straighten Text, the Coordinates docker's Replace), leave
  the symmetry where it was
  (`joining_curves_leaves_a_symmetry_where_it_was`).
- While the symmetry is edited, moving, scaling or aligning its object
  leaves the lines where they are, in the same undo step
  (`editing_a_symmetry_a_drag_on_the_object_keeps_the_line`,
  `align_keeps_the_line_while_editing_and_moves_it_after`); a copy of
  the lone object made then (Duplicate, Paste, a right click during a
  drag) joins the symmetry without a symmetry of its own
  (`a_duplicate_made_while_editing_joins_without_its_own_symmetry`).
- Finish measures the symmetry again with its copies (the boxes measured
  while editing are the object's own, as picked), so its box and handles
  are the whole's at once, an object with other live effects too
  (`after_finish_the_box_is_the_whole_symmetrys_with_other_effects_too`);
  it leaves a group of the symmetry entered while editing (not a group
  entered before) and the Shape tool a double click on a member switched
  to.
- Remove Symmetry (Alt+Shift+S) drops the effect; the copies disappear.
- Break Symmetry Link (Alt+X) bakes the copies into objects of their own.

## Display

Mirror lines are drawn as dashed selection-colour lines across the whole
canvas while an object with the effect is selected, and while its
symmetry is edited (unless Show symmetry lines is off).

## Checks

- Given a rectangle at x 10..20, when Symmetry is created with the
  defaults, then the evaluated `above` contains one copy spanning x 20..30.
- Given two lines at 0 degrees through the origin, then there are three
  copies, one per other quadrant (core test `symmetry_two_lines`).
- Given a symmetry object out of edit mode, when it is moved, then the
  copies and the mirror lines move with it (the copies are recomputed
  from the carried lines, nothing else is stored).
