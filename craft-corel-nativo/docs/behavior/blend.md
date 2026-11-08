# Blend

Intermediate objects between a start and an end object.

| Control | Default | Range |
|---|---|---|
| Steps | 20 | 1..999 |
| Rotation | 0 degrees | -3600..3600 (the panel: -360..360) |
| Acceleration (objects, colours) | 0 (linear) | -1..1 (the bar: -100..100 %) |
| Path | none | any curve (the panel's New path), "rotate all objects" |
| Spacing (along a path) | off, 5 mm | the steps follow from the path's length: `floor(length / spacing) - 1`, 1 to 999 |
| Loop | off | with a rotation, the steps' centres leave the straight way and curve from the start to the end (a half circle at 180 degrees, a curl through the middle at 360) |
| Blend along full path | off | along a path, the first and last steps sit on the path's ends |
| Apply to sizing | off | the object acceleration changes the steps' outlines and sizes too |
| Color blend | Direct | Direct (straight through RGB), Clockwise (hue increasing), Counterclockwise (hue decreasing), through HSB |

A blend asking for more than 999 steps (from a file or a script) draws
999 (`tracedraw_core::live::tests::a_blend_draws_at_most_its_maximum_steps`, #980).

Not yet: mapping nodes, splitting and fusing.

## Panel (Effects > Blend)

Two picture buttons choose Steps or Spacing (Spacing only for a blend
on a path), then the Steps and Spacing fields (the one not chosen is
greyed), Rotation, Loop (with a rotation), Blend along full path and
Rotate all objects (with a path). Under "Acceleration" the Object and
Color sliders, Apply to sizing, Link acceleration (the colour slider
follows the object one and is greyed) and Reset (both back to even).
Under "Color Blend" three picture buttons: Direct, Clockwise and
Counterclockwise. With a blend selected every change applies live and
New path, Clear Blend and Break Apart follow; otherwise a full-width
Apply blends the two selected objects with these settings
(`tracedraw_core::live::tests::blend_options_change_the_steps`).

## Property bar (Blend tool)

Steps, Rotation, Object acceleration, Color acceleration; with nothing
blended selected they are what the next blend takes (Steps is kept in the
preferences), with a blend selected they change it live (a run of changes
is one undo step), and Clear Blend and Break Apart follow; otherwise a
line on dragging from one object to another.

Interactive: drag from the start object to the end object; the start
and end are the two selected objects' order on drop.

## On the canvas (Blend tool)

A click on a blended object (the start, the end or a step; text as any
other object) selects it and picks its blend; a click on empty space
lets go. The selected blend shows a white square on the start object's
centre, a crossed one on the end object's, the line between them, and
two sliders on that line: the object acceleration (triangle above the
line) and the colour acceleration (triangle below). Each sits where the
middle step lands (see Formulas), so at 0 both are halfway, at +1
(100 %) 10.5 % from the start and at -1 10.5 % from the end: the slider
goes as far one way as the other (#1261; the exponent `1 + acc` used
before put the middle step 25 % from the start at +1 and 3 % from the
end at -1, so the steps bunched fully at the end but only part of the
way toward the start; then, until #1192, the power `t ^ e` with the
middle step at `0.5 - 0.45 acc`, 5 % from either end at 100 %). Dragging a slider along the line sets its
acceleration to the one that puts the middle step there (two decimals,
-1..1): toward the start bunches the steps (or the colours) near the
start, toward the end near the end. With Link acceleration both
follow either slider. Each drag is one undo step and the bar shows the
new values; a press on a slider never starts a new blend
(`blend_tool.rs`, `text_effect_tests::the_blend_tool_picks_a_text_blend_and_its_sliders_tune_it`).

Dragging the start or end square moves that object (one undo step for
the drag, the steps and the line following), so the blend's direction
changes; let go on another object of the page (not the blend's own two),
the move is taken back and that object becomes the new start or end
with the blend's settings, the old start staying a plain object and the
selection moving to the new start (#965,
`text_effect_tests::the_blend_tool_s_end_squares_move_and_pick_the_end_objects`).

Dragging the line between the squares (within the pick distance of
Options > Nodes and Handles, away from the squares and the sliders, which
win where they overlap it) moves the start and the end objects together,
so the whole blend moves with its length and angle kept; Ctrl keeps the
move horizontal or vertical. One undo step for the drag, and the pointer
is the move cursor over the line. A blend whose end lives only in the
effect (no end object) has no line to drag
(`effect_line_tests::the_blend_line_moves_the_whole_blend`).
Not yet on the canvas: dropping colours on the squares.

The steps lie between the two objects in the stacking order: with the
end object below the start one (a front object blended to one behind
it, the usual way to build depth), they are drawn below the start, the
step nearest the end lowest; with the end above, above the start. The
blend follows when either object is reordered
(`blend_link::tests::the_steps_lie_between_the_two_objects`).

## Start and end objects

Both objects stay objects of the drawing. The start object holds the
Blend effect, which names the end object and keeps a copy of it in page
space (its own effects left out) for drawing the steps; the end object
draws itself (`blend_link.rs`). After every change of the drawing, in
the same undo step:

- the end object moved, resized, turned or recoloured: the copy is
  brought up to date and the steps follow;
- the start object changed: the steps follow (they are drawn from it);
- the end object deleted: the blend goes with it (Undo brings both
  back).

Selecting either object picks the blend for the bar and the Blend
panel. Selected, each object shows its own box and handles: the start
object's box is the object itself (its other live effects included),
not the steps, as the end object's is the end; the bar's position and
size, the handles, Repeat and the rotation centre all take that box,
so a side handle resizes the start object from its own opposite side.
A press on a step picks the start object, shown with the same box,
except over the end object: the steps are drawn above the end when the
start is in front, and a text step is picked anywhere in its box, so
they covered nearly all of an end text and a click on it picked the
start text, which then took the colour meant for the end. A press on
the steps where the end object lies, away from the start object
itself, picks the end object, with the Pick tool and the Text tool
alike (#1739,
`pick_drag_tests::a_click_on_a_blend_s_end_text_under_its_steps_picks_the_end`).
The steps are part of no single object's box: a group holding the
blend spans the whole blend, steps included, and a blend from an older
file, whose end lives only in the effect, is one object whose box is
the whole blend (#1540,
`pick_drag_tests::a_blend_s_start_object_shows_its_own_box_and_handles`,
`tracedraw_core::live::tests::a_blend_start_s_own_box_leaves_the_steps_out`).
Object > Group (Ctrl+G, and the context menu's Group) groups a blend
however it is selected: with its start or end object selected alone
(the blend just made, or a click on a step or on either object) the
group holds both objects and whatever else is selected; a selection
box or Ctrl+A selects both already. The blend stays live in the group,
and one undo takes the group back. Before, a blend selected by a click
was one object, so Group stayed off and Ctrl+G did nothing (#1842,
`pick_drag_tests::a_blend_groups_however_it_is_selected`).

Clear Blend removes the blend and leaves both objects; Break
Apart turns the steps into objects. Files from before this kept the end
only inside the effect; such a blend draws its end with the steps.

## Formulas

For step `k` of `n` (`t = k / (n + 1)` with acceleration `t' = ln(1 +
(K - 1) t) / ln K`, `K = 705.641 ^ (-acc)` (`705.641 = 1.3 ^ 25`), the
curve and the range the `.cdr` files' blends use, #1192;
`tracedraw_core::live::blend_place`,
`live::tests::blend_acceleration_goes_as_far_either_way`,
`live::tests::blend_steps_follow_the_file_s_acceleration_curve`):
- Geometry: the end outline is wound the way the start outline is and
  started where it best fits the start outline's first node (the two
  outlines compared as 120 points each, about their own centres and at
  their own sizes, over every place the end outline could start), so
  equal shapes blend without twisting through the middle whatever their
  relative position. Then each outline gains a node where the other has
  one at the same share of its length (nodes less than half of one
  120th of the length apart pair up instead), and the steps are
  interpolated node by node, control points included: a step's side is
  straight where both paired sides are straight. So a square into a
  circle gives steps of eight nodes that round off evenly, and two
  rectangles give rectangles with their corners kept (#1020). Open
  outlines, and closed ones with more than 512 nodes together, are
  paired as 120 points spread evenly along their length, each step a
  polygon through them. Outlines of several parts (letters, holes)
  blend part by part, the n-th into the n-th, the same way.
  A blend read from a `.cdr` file (versions 9 and 13) pairs the two
  outlines node for node instead, each run from its first node as the
  file gives it: every node of a step lies on the straight line between
  its two paired nodes (control points too), so a step between a
  rectangle and a pentagon is a pentagon with sharp corners. When one
  outline has fewer nodes, its last segment (the one closing back to its
  first node) is cut into equal parts until the counts match. Open
  outlines, and outlines of more than 4096 nodes, are paired by arc
  length.
- Transform: interpolated as translation + rotation + scale (not matrix
  entries), so blends rotate the short way unless "rotation" adds turns.
  When both ends have the same outline (a copy turned, scaled or moved)
  and no nodes are mapped, each step is that outline under the
  transformation part way: its centre on the straight line between the
  two centres, turned by an even share of the turn (the shorter way),
  scale and slant in between, so no step shrinks or turns more than the
  next (#1260,
  `live::tests::a_blend_to_a_turned_copy_turns_its_steps_evenly`). Two
  different outlines are mixed point by point as above.
- Object acceleration: each step's centre goes `t'` of the way between
  the two centres, while its outline is the even morph at `t` (moved
  there). With Apply to sizing the step is the morph at `t'` itself,
  outline and size: a square blended into a circle with the steps
  packed toward the circle has round steps by the circle and square
  ones far from it, as the previews of the `.cdr` files with that
  setting show. Before, the even morph was only scaled to the size at
  `t'`, so the steps by the circle stayed square (#1576,
  `live::tests::apply_to_sizing_morphs_each_step_at_its_accelerated_place`).
- Colour: mixed directly (each colour's values from start to end), or
  through HSB around the hue circle (Clockwise: the hue rising from the
  start; Counterclockwise: falling), `t'` of the colour acceleration of
  the way.
- Rotation: each step turns about its own centre by `rotation * t'`.
  With Loop its centre also leaves the straight way: the start's centre
  `A` turned by `rotation * t'` about the middle `M` of the two centres
  and the end's `B` turned by `rotation * (t' - 1)` about it, mixed `t'`
  of the way (`(1 - t') R(rotation t') (A - M) + t' R(rotation (t' - 1))
  (B - M) + M`), so the steps leave the start and reach the end along a
  curve to the right of the way for a positive rotation: a half circle
  at 180 degrees, a curl through the middle at 360 (#1192, the pictures
  of the `.cdr` files; turned about the middle alone, the last steps
  landed far from the end; `live::tests::a_looped_blend_curves_from_one_end_to_the_other`).
- Along a path: step `k` is placed at arc length `t' * L` on the path,
  oriented to the tangent when "rotate all objects" is on.

## Saving as `.cdr`

From version 9 a blend goes out live (#1192): the start and end objects
as objects of their own, the upper one first, with the empty linked
group files hold between them (object type 7, the step count, the
spacing and the two ends' centres; `docs/cdr-format.md`, Writing,
Blend), so the program that reads the file shows the steps and edits
the blend. From version 23 the record also holds the object and colour
accelerations, the rotation, Loop, Apply to sizing and the colour
direction, counted from the lower object (`docs/cdr-format.md`, Blend
settings; #1192): a blend whose start is the upper object goes out with
its accelerations and colour direction turned round, and its rotation,
which the record turns from the lower object, as it is. Read back in
TraceDraw the blend is on the start object with its settings from
version 14 (the package names it in `tracedraw/blends.json`); from a
RIFF file (9 to 13) it comes back the way any file's blend does: on the
lower object, even acceleration, node-for-node pairing. Versions 7 and
8 get one plain group of the start object and its steps as curves
(#946). A path is not written (no file holding one was seen); the
blend's own drop shadow is left out. A blend whose
end is elsewhere (another layer, inside a group, an object with effects
of its own) stays a drawn group.

## Checks

- `ui::effect_bars::tests::the_blend_bar_edits_the_selected_blend`
- `blend_link::tests::the_end_object_stays_and_the_steps_follow_it`
- `write::tests::a_blend_goes_out_live_from_version_9`,
  `a_blend_whose_end_is_its_own_goes_out_live_with_the_end_as_an_object`,
  `a_text_blend_opens_live_again`,
  `a_kept_blend_that_no_longer_matches_opens_as_drawn`,
  `check::tests::a_blend_group_stands_between_its_ends` and the
  conformance suite (`blends_contours_envelopes_and_extrusions`).

- Given two 10 mm circles 100 mm apart, 9 steps, then the 5th step is
  centred at 50 mm and has a 10 mm diameter.
- Given two equal ellipses, the second down and to the right, every step
  keeps the ellipse's width and height within 1 mm; reversing the second
  outline changes nothing.
- Given red to blue direct with 1 step, then the step is (128, 0, 128).
- Given two texts (or any outlines of several parts), part i of the start
  morphs into part i of the end; a part with no partner grows from or
  shrinks to a point at its centre, so the steps read as letters
  (`blends_of_several_parts_morph_part_by_part`,
  `the_blend_tool_blends_two_texts_and_a_text_with_a_shape`).

Break Apart (the docker's, or Object > Break Blend Group Apart, Ctrl+K
and the right-click menu, which take the place of Break Apart while a
selected object holds a blend) turns the steps into objects, grouped
when there are several (see `contour.md`, #424).
