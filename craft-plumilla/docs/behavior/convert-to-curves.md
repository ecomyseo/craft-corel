# Convert to Curves

Object > Convert to Curves (Ctrl+Q), the property bar's button and the
context menu turn the selected objects into curve objects, keeping
their fill, outline, transparency and effects. An envelope, perspective
or distortion becomes part of the curves: the object is a curve drawn
as before, with that effect gone. A warp after a contour, blend,
extrusion, shadow or symmetry stays live, since baking it would change
what that effect draws; the other effects stay too, switched-off ones
switched off. One undo step.

| Object | Becomes |
|---|---|
| Rectangle, ellipse, polygon, star, other shapes | a curve with the same outline (corners, arcs and points included) |
| Ellipse (whole, not a pie or arc) | one closed curve of four symmetrical nodes (equal handles along the axes), at the right, top, left and bottom, starting at the right |
| Artistic and paragraph text | a curve made of the letter outlines |
| Clip frame | still a clip frame: its frame becomes a curve and its contents are converted by these same rules (a group inside stays a group with its objects converted), keeping their ids, and stay clipped inside it |
| Table | a group of curves: one filled rectangle per cell (merged cells as one), the cell text as curves, and one curve holding every cell border with the table's outline |
| Symbol instance | a group holding the symbol's objects as curves, where the instance was, at its size and turn; a symbol inside a symbol is converted too |
| Object with an envelope, perspective or distortion | a curve with the warped outline, as it was drawn; a curve converts too |
| Group with an envelope, perspective or distortion | still a group of the same objects, each a curve warped as it was drawn, keeping its own fill and outline |
| Clip frame with an envelope, perspective or distortion | still a clip frame, its frame and contents warped as drawn; then the warps of the objects inside are baked, the group's or clip frame's first as they are drawn. The frame object's own envelope stays live, since the contents are clipped to its outline without it. Inside a group or clip frame whose contour, shadow or other drawn effect stays, the objects keep their warps, or that effect would change |
| Curve, group | stays as it is |
| Bitmap | stays as it is: a bitmap has no outline to convert, and keeps its envelope (also in a group). The status bar says so |

The command is greyed when nothing in the selection would change
(only curves, groups and bitmaps without an envelope, perspective or
distortion selected, or clip frames whose frame and contents are
already curves).

The Shape tool's double-click on a shape (to edit its nodes) converts
it but keeps an envelope live, so the frame being edited stays.

## Checks

- Given a 2 x 2 table with a border, a cell fill and text in one cell,
  when it is converted, then it is a group of six curves (four cell
  fills, the text, the border) covering the same area, and Undo brings
  the table back (`ops::curves_and_target_tests::a_table_becomes_curves_with_its_cells_borders_and_text`).
- Given an instance of a symbol made of a rectangle and an ellipse,
  moved 5 mm, when it is converted, then it is a group of two curves at
  the moved place (`ops::curves_and_target_tests::a_symbol_instance_becomes_the_curves_of_its_symbol`).
- Given a clip frame of an ellipse holding a rectangle and a bitmap,
  when it is converted, then it is still a clip frame whose frame and
  rectangle are curves with the same ids and fill, the bitmap is left as
  it is, and then the command is greyed and a second conversion changes
  nothing
  (`ops::curves_and_target_tests::a_clip_frame_keeps_its_contents_as_curves`).
- Given an ellipse, when it is converted, then it is one closed curve
  of four symmetrical nodes at its right, top, left and bottom, at any
  size
  (`ops::curves_and_target_tests::an_ellipse_becomes_a_closed_curve_of_four_nodes`,
  `geometry::tests::an_ellipse_is_closed_with_four_nodes_on_its_axes`).
- Given a bitmap, the command is greyed and the bitmap is left as it is
  (`ops::curves_and_target_tests::a_bitmap_is_not_converted`).
- Given a curve with the Arch envelope, the command is enabled, and
  converting makes it a curve reaching the top of the arch with no
  effect left; Undo brings the envelope back. A rectangle with an
  envelope becomes the warped curve at once
  (`envelope_edit::tests::convert_to_curves_bakes_the_envelope`,
  `pick_drag_tests::ctrl_q_bakes_an_envelope`).
- Given a group with an envelope, converting keeps the group and its
  object ids, each object now a warped curve
  (`envelope_edit::tests::an_enveloped_group_converts_object_by_object`).
- Given enveloped text, converting gives one curve covering what was
  drawn, in the text's fill
  (`text_effect_tests::convert_to_curves_bakes_the_envelope_of_text`).
- Given a clip frame with an Arch envelope whose frame has its own
  envelope and whose content has a perspective, converting gives the
  content drawn as before (the clip frame's warp first, then its own)
  and the frame keeping its envelope; with only the frame's envelope
  left, the command is greyed
  (`tracedraw_core::document::tests::a_clip_frame_bakes_its_warp_before_the_warps_inside`,
  `tracedraw_core::document::tests::warps_under_a_live_contour_stay`).
- A brush press on an enveloped group converts nothing
  (`envelope_edit::tests::a_brush_press_leaves_an_enveloped_group`).
- The bake itself: the warped outline equals what is drawn, the other
  effects keep their order and switched-off state, warps after a
  contour stay, perspective and distortion bake too, groups bake object
  by object, bitmaps keep their envelope
  (`tracedraw_core::live::tests::bake_outline_warps_makes_the_envelope_the_outline`,
  `tracedraw_core::live::tests::only_warps_before_a_contour_are_baked`,
  `tracedraw_core::live::tests::perspective_and_distortion_bake_too`,
  `tracedraw_core::live::tests::an_envelope_on_a_group_bakes_into_each_object`,
  `tracedraw_core::live::tests::a_bitmap_keeps_its_envelope`).
