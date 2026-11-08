# Distort

Push and Pull, Zipper and Twister as a live effect (`Effect::Distort`):
the object keeps its own outline and draws the distorted one, so the
distortion can be changed, moved or cleared at any time. The formulas
are in `tracedraw_core::distort`; the tool in `distort_tool.rs`, its bar
in `ui/distort_bar.rs`.

## Formulas

`c` is the distortion's centre (in the object's own space; the middle of
its outline's bounds when it has none). The distortions work on the
outline's own nodes and segments: every subpath stays a subpath, an open
curve stays open.

- Push and Pull, `amount` from -200 to 200: each node `p` goes to
  `c + (p - c) * (1 + s)` and each control point `q` of a segment to
  `c + (q - c) * (1 - s)`, with `s = amount / 100`. Positive amounts
  push the nodes out and bow the segments in (spikes with hollow
  sides); negative amounts pull the nodes in and bow the segments out
  (petals). Straight segments become curves whose control points start
  on their nodes, so they move along the nodes' own lines from the
  centre: a side bows from its middle, pushed corners become spikes and
  pulled corners curl, with the side's lobe reaching past them (#1264);
  quadratic segments are raised to cubic. A Push and Pull of a `.cdr`
  file opens with the file's amount and centre as they are: the boxes
  drawn so match the boxes the files give their objects to 0.01 mm
  (#1824).

An object of a `.cdr` file distorted again over its distortion (two or
more distortions one after the other) opens with the earlier ones baked
into its outline, in order, each about its own centre, and the last one
live on the result, so it can still be edited; the object becomes a
curve. The boxes drawn so match the files' boxes to 0.01 mm in all 11
such files of #1824.
- Zipper, `amplitude` (mm, 0 to 100), `frequency` (0 to 100) and
  `tilt` (degrees, -180 to 180): each segment is cut into
  `frequency + 1` equal steps of its parameter, so it carries
  `frequency + 1` lines (#1122); the points move off the segment along
  its normal turned by `tilt` (counterclockwise when positive; the
  normal is the one on the left of the segment's direction), alternately
  `-amplitude` and `+amplitude` along the whole subpath, the nodes
  included (a node moves between the normals of its two segments), so
  every point of the zigzag is a tooth's tip, the pattern runs unbroken
  through the nodes and the ends of an open curve are tips too, not
  points left on the curve (#1108). With a tilt the teeth lean, and
  only the tips lean (#1736): the points on the side of the outline the
  handle is on (the normal's side for a tilt under 90 degrees either
  way, the other side past it) stay `amplitude` from the segment and
  move along it by `amplitude * tan(tilt)`, onto the normal turned by
  `tilt` from their point on it, while the points on the other side
  stay where the upright teeth have them, `amplitude` straight out. A
  lean so neither moves the bases nor changes the zigzag's height: it
  is `2 * amplitude` high, half to each side of the outline, whatever
  the tilt, as in the reference files of #1671 and #1736. Near 90
  degrees the move along the segment is held at ten times the
  amplitude. On a line tilted 60 degrees the zigzag is sheared toward
  a sawtooth; tilted 180 it is the same zigzag with its sides swapped
  (#1671). A closed outline whose count of tips would be
  odd takes one line more on its last segment, so the last tip
  alternates with the first. Random scales each tooth by a fixed
  pseudo-random share of its height (the same every time it is drawn);
  Local scales it by `1 - d / d_max`, `d` its distance from the centre
  and `d_max` the farthest node's; Smooth draws a wave through the tips
  instead of straight teeth: from each tip to the next one cubic that
  leaves and arrives along the outline's direction there, its handles
  0.3642 of the distance along the outline between the two tips long,
  the cubic nearest a sine's half wave. The handles never reach past
  the middle, so the wave goes on along the outline between two tips
  however high they are, crosses the outline midway and never crosses
  or joins itself; the smoothing used before (a curve through the tips
  with handles a third of the chord between them, which grows with the
  teeth) made tall teeth into loops that overlapped (#1671).
- Shifted tips (`shifted_tips`, the Zipper of a `.cdr` file, #1715):
  the teeth as `.cdr` files hold them. The points alternate from a tip
  on the normal's side, the outline's first point being a tip; every
  tip is moved by `0.2 * (centre - middle of the outline's box)`,
  whatever the amplitude, instead of leaning by the tilt, and the
  bases stay `amplitude` straight out on the other side. The handle's
  place so sets the lean: under the middle the teeth stand upright,
  toward one end they lean that way. Random, Local and Smooth work as
  above. TraceDraw's own Zipper never has it; a Zipper read from a
  `.cdr` file keeps it while its values are edited (the tilt then has
  no effect).
- `zipper_zero_direction(path, c)`: the outline's unit normal (left of
  its direction) at the point nearest to `c`, the direction the teeth
  stand in with no tilt and the one the Zipper's handle line shows; to
  the right when the outline has no segments.
- Twister, `angle` (degrees; whole turns times 360 plus degrees,
  clockwise negative): every point `p` is turned about `c` by
  `angle * (1 - |p - c| / d_max)`, `d_max` the distance of the farthest
  point of the outline from `c`, so the farthest point stays where it
  is and the turn grows toward the centre, where it is the whole angle.
  The outline keeps its place and its outer shape (a square's corners
  stay at its corners, a star's tips where they were) and swirls inside;
  turned the other way round, with the farthest point taking the whole
  angle, the object looked rotated as a whole and its orientation
  followed the pointer (#1674). Each segment is cut into steps along which
  that turn changes by at most 15 degrees (the distance from the
  centre changes by at most the length walked), and a cubic joins each
  step's ends with the true directions of the turned curve there (the
  segment's direction turned, plus the swing of the turn growing with
  the distance). The result follows the turned curve to within a few
  hundredths of a percent of its size, with no straight tails at the
  ends of an open curve and no corners between the steps (#1108); a
  segment whose points keep their distance (a circle about its middle)
  turns whole.
- A centre that is not a number or lies over 100 m from the outline is
  replaced by the middle; values out of range are brought in; a result
  that is not finite leaves the outline as it was.

## The tool

- Drag on an object, or anywhere with one object selected (off it too):
  the distortion is centred where the drag started and follows the
  pointer, one undo step for the whole drag (#1442). The drag goes in
  any direction: the handle line runs from the start to the pointer,
  wherever it is, not along an axis (#1442). Push and Pull:
  `amount = 100 * |drag| / r` (`r` half the object's larger side),
  rounded to whole percents, pushed when the pointer is right of the
  start and pulled (negative) when left of it. Zipper:
  `amplitude = |drag| / 4`, the bar's frequency and options, and the
  teeth lean the way the drag goes: `tilt` is the turn from the
  outline's normal at the point nearest the centre to the drag's
  direction, measured in the object's own space on the outline the
  zipper goes on (the object's own as the distortions it already has
  leave it), so a drag straight off a line makes upright teeth and a
  drag slanting away from it leans them that way (#1671). Twister:
  the angle the pointer turned about the start, whole turns counted.
- Each selected distortion shows its handles (#1122): the centre as a
  diamond, a line with an arrow to the end handle, and for the Zipper a
  slider on the line (a small circle). A drag that starts on one
  (within 6 pixels, or half the handle size and 2 more for larger
  handles, `App::distort_pick_px`; the centre first, then the slider, then the end) takes it: the diamond
  moves the centre; a drag on the line between them (within the pick
  distance of Options > Nodes and Handles) moves the centre by the
  pointer's way, the line with it at its length and angle, Ctrl keeping
  that move horizontal or vertical, the pointer the move cursor over the
  line (`effect_line_tests::the_distortion_line_moves_its_centre`); the end handle is a square along the line at
  `amount / 100` of `r` for Push and Pull (the other way when negative;
  dragging it sets the amount from its distance and the side from
  where it is) and at four times the amplitude for the Zipper (the
  drag's own measure; dragging it sets the amplitude from its distance
  and the tilt from its direction), or a circle for the Twister turned
  by the angle from the direction its drag began in (dragging it round
  turns the angle, whole turns counted). The line is free: every handle
  reaches the pointer during a drag (on the object or on the handle),
  in whichever direction it goes, and the line stays there when let go.
  The Push and Pull and Twister lines go to the right until the object's
  distortion of that kind has been dragged (#1442,
  `App::distort_lines`); the Zipper's line stands on the outline's
  normal at the point nearest the centre, turned by the tilt, so it
  leans with the teeth and moves with the centre, and a tilt typed on
  the bar turns it (#1671); the Twister's handle is also as far out or as
  near as the drag made it, `r` until it has been dragged, and its
  length changes only the handle, not the turn (#1263); the slider sits
  `frequency / 100` of the way from the centre to the Zipper's end and
  dragging it along the line sets the frequency. A handle drag edits the
  object's distortion in place (its centre kept) as one undo step; a
  value changed on the bar moves the handle along the same line.
- The keys steer the end handle while it is dragged, and the line a
  drag on the object sets for a new distortion (#1563, #1793, a
  maintainer decision; `distort_tool::constrain_end`):
  - Shift: the handle moves only sideways, keeping the height it had
    when the drag began (a new line stays level with the press);
  - Ctrl (Cmd on macOS): the handle keeps its length and turns freely
    toward the pointer, all the way round, with no steps (a new line,
    which has no length yet, follows the pointer);
  - Ctrl+Shift: steps of the constrain angle about the centre (Options,
    15 degrees at first, so horizontal and vertical among them), as the
    Shape tool's handles: the pointer turned onto the nearest step, as
    far out as it reaches along it (`interaction::constrain_from`).

  Push and Pull, Zipper (its tilt follows the handle) and Twister alike.
  Push and Pull turned with Ctrl keeps its amount, so only its line
  turns. The slider and the centre are not constrained, and Ctrl on the
  line still keeps its move horizontal or vertical. This replaces the
  first version of these keys, in which Shift and Ctrl alike kept the
  handle to the constrain angle steps
  (`effect_line_tests::shift_and_ctrl_steer_a_distortion_handle`,
  `effect_line_tests::ctrl_turns_a_distortion_handle_all_the_way_round_at_its_length`).
- The handles look like the Shape tool's nodes (#1574): white inside a
  2 pixel ring in the main colour of Options > Nodes and Handles, with a
  1 pixel dark edge (`Tokens::HANDLE`) outside, so they show on a white
  page (the edge), on a dark drawing (the white) and on the node colour
  itself. Squares and circles are Handle size across (6, 8 or 10 pixels,
  the size of the selection handles, #1285), the diamond a pixel
  more each way. The line is 2 pixels of the main colour on a white
  halo, its arrow head touching the end handle. Each handle stands on a
  corner of the device pixel grid and its ring and edge are whole device
  pixels at any interface size (`distort_bar::handle_marks`), so it shows
  crisp and upright wherever the view puts it.
- A drag that does not start on a handle adds a new distortion over the
  ones the object has: each is applied to the outline the ones before it
  made, so distortions build up drag after drag (#1442). The handles and
  the bar show and edit the newest; Clear Distortion takes the newest
  off, so a second Clear takes the one before it; Convert to Curves
  makes all of them, oldest first, the object's outline.
- A click selects the object under the pointer (or nothing).
- Groups and bitmaps are not distorted.

## Property bar

| Control | Default | Range |
|---|---|---|
| Push and Pull, Zipper, Twister | Push and Pull | with a distorted object selected, turn its distortion into that kind with the bar's values for it |
| Amplitude (Push and Pull) | 20 | -200 to 200 |
| Amplitude (Zipper) | 2 mm | 0 to 100 mm |
| Frequency | 6 | 0 to 100; `frequency + 1` lines per segment |
| Tilt | 0 | -90 to 90 degrees; the teeth's lean from the outline's normal, shown folded into that range (teeth leaning `t` and `t + 180` are the same zigzag with its sides swapped), while the tilt kept with the distortion, -180 to 180, also says which side of the outline the handle line is on; a value typed keeps that side (`DistortSettings::lean`, #1671) |
| Random, Smooth, Local | off | toggles |
| Clockwise, Counterclockwise | Clockwise | |
| Rotations | 0 | 0 to 9 |
| Degrees | 90 | 0 to 359 |

- The bar shows the selected object's distortion (its kind and values)
  and edits the selected distortions; each change is one undo step (a
  run of edits of the same objects, typing on, is one). A value changed
  on the bar also distorts the selected objects that have no distortion
  yet, about their middles, so the change shows at once (#1030,
  `distort_tool::tests::a_bar_edit_previews_on_an_undistorted_selection`);
  a kind button alone only turns existing distortions into that kind.
- Center Distortion: the selected distortions about their objects'
  middles again. Clear Distortion removes them. Convert to Curves
  (Ctrl+Q) makes the distorted outline the object's own (the object
  becomes a curve, also when it already was one), and Break Apart breaks
  the distorted curve, or a distorted text into one curve per letter,
  each distorted as the whole text was.
- Apply (shown when a selected object has no distortion): the bar's
  distortion on those objects, about their middles.
- The values are kept in the preferences (`tools.distort`).

## Checks

- `distort::tests::push_and_pull_move_nodes_and_bow_the_sides`
- `distort::tests::zipper_teeth_on_every_segment`
- `distort::tests::twister_keeps_the_far_points_and_turns_the_inside`
- `distort::tests::a_zipped_line_starts_and_ends_on_tips`
- `distort::tests::the_teeth_lean_by_the_tilt`
- `distort::tests::tilted_teeth_keep_their_bases_and_their_height`
- `distort::tests::tilted_teeth_still_follow_the_handle_and_smooth_ones_never_cross`
- `distort::tests::smooth_teeth_are_waves_that_never_cross`
- `distort::tests::shifted_tips_start_with_a_tip_moved_toward_the_centre`
- `parse::tests::version_16_zipper_distortions_open_live` (tracedraw-cdr)
- `parse::tests::version_16_twister_distortions_open_live` (tracedraw-cdr)
- `parse::tests::version_16_push_and_pull_distortions_open_live` (tracedraw-cdr)
- `parse::tests::version_16_stacked_distortions_draw_them_all` (tracedraw-cdr)
- `distort::tests::a_twisted_line_is_a_smooth_spiral_to_its_ends`
- `distort::tests::older_files_and_odd_values_read`
- `distort_tool::tests::a_drag_distorts_about_where_it_started`
- `distort_tool::tests::drags_start_anywhere_and_add_distortions`
- `distort_tool::tests::the_bar_edits_the_selected_distortion`
- `distort_tool::tests::settings_round_trip_and_stay_in_range`
- `distort_tool::tests::a_bar_edit_previews_on_an_undistorted_selection`
- `distort_tool::tests::the_handles_set_the_values_by_dragging`
- `distort_tool::tests::the_twister_handle_is_as_long_as_it_is_dragged`
- `distort_tool::tests::the_handles_follow_the_pointer_in_any_direction`
- `effect_line_tests::shift_and_ctrl_steer_a_distortion_handle`
- `effect_line_tests::ctrl_turns_a_distortion_handle_all_the_way_round_at_its_length`
- `distort_tool::tests::a_zipper_drag_leans_the_teeth`
- `ui::distort_bar::tests::the_bar_draws_for_every_kind_and_shows_the_selection`
- `ui::distort_bar::tests::the_handles_stand_upright_on_the_pixel_grid`
- `ui::distort_bar::tests::the_handles_take_the_handle_size`
- `ui::distort_bar::tests::the_handles_take_the_node_colour`
- `ui::distort_bar::tests::the_handles_show_on_light_and_dark`
- `ui::distort_bar::tests::the_marks_are_rounded_to_device_pixels`
- `ui::distort_bar::tests::the_handles_are_taken_to_their_edge`
- Given a 20 mm square about its middle and Push and Pull 50, then a
  corner node is 1.5 times as far from the middle and the middle of a
  side is nearer the middle than 10 mm.
- Given Twister 90 on a 20 mm square about its middle, then its four
  corners stay where they are and the middle of a side has turned by
  90 times one less its distance over the corners' (26.4 degrees);
  given a five-pointed star (tips 40 mm from its middle, inner nodes
  16 mm) and two whole turns about its middle, then its tips stay and
  its inner nodes turn by 1 - 16 / 40 of the angle.
- Given a straight 100 mm line and Twister of four turns about its
  middle, then the result is one smooth spiral from end to end: every
  point of every step lies on the turned curve within 0.03 mm, the two
  ends stay where they were and leave along the spiral's own direction,
  and no two steps meet at a corner.
- Given a straight line and Zipper with frequency 6, then it is seven
  lines between eight points, all teeth's tips, the two ends included,
  alternately on each side of the line.
- Given a 130 mm line and Zipper with amplitude 8, frequency 15 and
  tilt 60, then each of the 8 tips on the handle's side lies on the
  line's normal turned by 60 degrees from its point on the line, 8 mm
  from the line, and each of the 9 points on the other side is 8 mm
  straight out from its point on the line, where tilt 0 puts it; with
  tilt 180 the points are those of tilt 0 mirrored in the line.
- Given the same line tilted 30, -30, 60 or -75 degrees, then the
  bases stay where tilt 0 has them, the tips are 8 mm from the line
  moved `-8 tan(tilt)` mm along it, and the zigzag is 16 mm high, 8 mm
  to each side, as upright; tilted 150 the tips are on the other side;
  tilted 90 the tips stay within 80 mm of their place (#1736).
- Given the same line and a smooth Zipper tilted 25 degrees, then the
  waves go through the same points as the sharp one and each runs on
  along the line, never turning back, within the amplitude of it,
  crossing it once.
- Given a 130 mm line and a smooth Zipper with amplitude 10 and
  frequency 15 (tips every 8.1 mm, taller than they are apart), then
  each of the 16 waves runs on along the line within its own stretch
  between two tips, never turning back, crosses the line midway, stays
  within the amplitude, leaves its tips along the line and is at
  cos(pi / 4) of the amplitude a quarter of the way, within a percent.
- Given a 20 mm square and a Zipper drag from the middle of its bottom
  side 8 mm straight down, then the teeth are upright (tilt 0) and 2 mm
  high and the end handle is where the drag ended; the handle taken and
  dragged round to 30 degrees off the normal leans the teeth 30 degrees
  (every point on that side is still 2 mm off the side and the bases
  stay where the upright teeth have them, #1736), the handle under
  the pointer; tilt 0 typed on the bar puts the handle back on the
  normal.
- Given a 20 mm square with Push and Pull 50 selected, then its end
  handle sits 5 mm right of the centre, and dragged to 10 mm left of it
  the amount is -100; turned into a Zipper with amplitude 20 about the
  middle, its end handle sits 80 mm from the centre along the normal of
  the nearest side (the first of the equally near sides, the bottom,
  whose normal points up).
- Given a 20 mm square and a Push and Pull drag from its middle 6 mm
  right and 8 mm up, then the amount is 100 and the end handle sits
  where the drag ended; dragged from there to 8 mm left and 6 mm up of
  the middle the amount is -100 and the handle is there again.
