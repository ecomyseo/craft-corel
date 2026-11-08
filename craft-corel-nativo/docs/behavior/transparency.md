# Transparency

| Control | Default | Range |
|---|---|---|
| Type | Uniform | None, Uniform, Gradient (linear, radial, conical, square, shaped), Pattern (two-colour, vector, bitmap), Texture |
| Transparency | 50% | 0..100 |
| Merge mode | Normal | Normal, Add, Subtract, Difference, Multiply, Divide, If lighter, If darker, Texturize, Hue, Saturation, Colour, Invert, And, Or, Xor, Red, Green, Blue |
| Apply to | Fill and outline | fill, outline, both |

Gradient transparency uses the same geometry as gradient fills with
grayscale stops where white = opaque and black = transparent. The
object is rendered to a scratch layer, multiplied by the transparency
mask, then composited with the merge mode. A shaped gradient follows
the object's outline (`tests::shaped_transparency_follows_the_object`).
A gradient mask turns, mirrors and skews with its object as a gradient
fill does (see `gradient-fill.md`, #775).
Moving nodes or cropping keeps a gradient mask where it lies on the
page, cut by the new outline, not squeezed into its new box (see
`gradient-fill.md`, #1491).

The Transparency tool shows a gradient mask's nodes as Interactive Fill
shows a fill's (`fill_tool.rs`, #448): start, end and skew handle, the
nodes between and the midpoint slider of each blend; a drag sets the
line (a gradient mask there already keeps its nodes), a double click on
the line adds a node and on a node removes it. A drag that starts on the
line itself, away from the nodes and the midpoint sliders (within the
pick distance of Options > Nodes and Handles), moves the whole mask by
the pointer's way, the line keeping its length and angle; Ctrl keeps
that move horizontal or vertical; the pointer is the move cursor over
the line (`effect_line_tests::the_transparency_line_moves_the_whole_mask`). Edits are one undo step
"Transparency" per drag; the fill is left alone
(`fill_tool::tests::the_transparency_mask_has_nodes_and_a_midpoint`).

What a drag that sets the line acts on (#1661,
`pick_drag_tests::the_transparency_tool_drags_on_the_selection_and_ctrl_constrains`):

- With a selection, pressed inside the box of a selected object (also
  where another object covers it, or where its own mask makes it
  see-through) or on no object: every selected object gets the line, as
  it lies on the page, one undo step; the selection stays.
- Pressed on an unselected object outside the selected ones: that object
  is picked and gets the line, as Interactive Fill picks the object a
  drag starts on.
- With nothing selected: the object under the press is picked and gets
  it; a drag over no object does nothing.

Ctrl (Cmd on macOS) keeps the line in steps of the constrain angle
(set in Options, 15 degrees at first) about where the drag
began, as for lines and the other angle drags; Interactive Fill's drag
does the same
(`pick_drag_tests::ctrl_keeps_an_interactive_fill_drag_to_the_constrain_angle`).

Clear Transparency, a button at the end of the Transparency tool's bar
and of the Properties docker's transparency part (greyed while no
selected object has transparency), and None in the type list take the
opacity back to 100 % and remove the mask, one "Clear Transparency" undo
step (`App::clear_transparency`, #1338,
`eyedropper::tests::clear_transparency_removes_masks_and_opacity`).

Uniform transparency on a group goes on each object it holds, as if set
on each one: members that overlap show through each other, and a
member's own transparency and that of every group around it multiply
(`tests::a_group_with_uniform_transparency_is_drawn_see_through`,
`tests::nested_group_transparencies_multiply`). The canvas, PNG and the
other pictures, SVG (an `opacity` on each member, none on the `<g>`), PDF
and saving `.cdr` all draw it so, and Ungroup multiplies it into each
object, so the objects look as the group did (#1386). The `.cdr` files
seen keep no transparency on a group's own record, only on its objects
(1587 groups in 487 files of versions 7 to 27; a version 11 file holds
groups whose objects all share one transparency, see `cdr-format.md`), so a saved
group comes back as an opaque group of see-through objects that looks
the same.

Uniform transparency with Normal is the object's opacity. With any
other merge mode it is kept as a mask of one grey (its lightness the
opacity, `mixed::uniform_mask`), drawn the same way, so Multiply and the
rest work on uniform transparency too; the
Transparency part reads it back as Uniform with its amount and merge
mode (`mockup_tests::mockup_steps`).

Freeze is not offered: an earlier version of this page listed it, but it
never existed (#1203).

## Speed (#1335)

A semi-transparent object's scratch layer covers only the pixels the
object can reach (its bounds grown by what its outline may reach, as
for culling, `Renderer::layer_area`), not the whole window: a page of
2000 half transparent objects took 2.2 s to draw at 1600 x 1000 pixels
and takes 0.11 s (`render_times_of_transparent_objects`). Pattern tiles
stay anchored to the window's corner, so the result is the same pixel
for pixel (`tests::a_transparent_object_takes_a_small_layer_and_draws_the_same`).
An object with effects or a drop shadow, a symbol and the drag preview
keep the whole window. A change from the Transparency part or the
Properties docker is one command for the whole selection
(`Command::SetEffectsAndOpacity`), which finds every object in one walk
of the drawing; a command per object searched the drawing for each. With
2000 objects selected a frame of a transparency slider drag took 39 ms
and takes 11 ms (`frame_times_of_a_transparency_drag`).

The mouse wheel steps a slider only while it has the focus or is being
dragged, as every slider does (`sliders.md`, #1240); over a slider
without the focus it scrolls the docker.

## Object opacity beside the colours (#1818)

The Properties docker's Fill and Outline sections each have an Object
opacity slider under the colour (0 to 100 %, 100 % solid). An object
has one transparency, applying to its fill, its outline or both, so the
fill and the outline cannot have opacities of their own at the same
time: the slider is the whole object's opacity, fill and outline
together, and both sections show the same value. It is the uniform
transparency the other way round (opacity 25 % is a uniform
transparency of 75 %): a change makes the transparency uniform for the
whole object (Fill and outline) on every selected object, one undo step
for a drag. With another kind of transparency (fountain, pattern,
texture), or kinds that differ across the selection, the slider is off
and the Transparency section changes it
(`pick_drag_tests::the_fill_and_outline_sections_set_the_objects_opacity`).

## Properties docker (#1439, #1203)

The Transparency section of the Properties docker
(`ui/transparency_docker.rs`) lays the settings out top to bottom; the
Transparency tool's bar keeps its one-row form.

1. **Merge mode**, a full-width list at the top.
2. **Type buttons**: No transparency, Uniform, Gradient, Vector pattern,
   Bitmap pattern, Two-colour pattern, Texture, each a picture with a
   tip. The chosen type's title follows ("Uniform Transparency",
   "Gradient Transparency"...). A type button gives the selection that
   type's first preset (a vector pattern of dots, a soft bitmap dot,
   the two-colour checker, clouds); No transparency is Clear
   Transparency.
3. **Transparency picker**: a swatch of the current transparency with a
   drop-down grid of ready-made ones. Uniform offers 0 to 85 % in steps
   of 5 (black at that opacity over a checker), Gradient twelve masks
   (linear both ways, vertical, diagonal, clear-opaque-clear, radial
   both ways, conical, square both ways, shaped, half), the patterns and
   the texture their own tiles (every two-colour tile, five vector and
   four bitmap tiles, every texture at two scales). A click applies the
   whole mask (`TransparencyField::Mask`). The swatches are rendered once
   and kept. **Edit Transparency** beside it switches to the Transparency
   tool to edit the mask on the object.
4. **Uniform**: the amount, a full-width slider (#1438).
5. **Gradient**: buttons for linear, radial, conical, square and shaped;
   the gradient bar with a marker under each stop (a click picks the
   nearest stop, a drag moves it between its neighbours); the picked
   stop's **Opacity** (100 % where the mask is white) and **Position**;
   **+** adds a stop half way to the next one with the gradient's grey
   there, **x** removes the picked stop (two stay); the angle slider for
   linear, conical and square. Stop changes patch only the stops
   (`TransparencyField::MaskStops`), so each object keeps its angle and
   type; a run of changes (a drag) is one undo step.
6. **Two-colour pattern and Texture**: **Foreground transparency** and
   **Background transparency** (0 % opaque, 100 % invisible; the mask's
   two greys, white opaque) and **Swap**
   (`TransparencyField::MaskColors`).
7. **Transparency target**: three buttons, Fill, Outline, All.
8. **Clear Transparency**, as before.

Settings that differ across the selection show as mixed; a change sends
only that setting to each object (`mixed::patch_transparency`).

Not yet: the ten missing merge modes, Free scale and skew, the mirror
tiles button, foreground and background for vector and bitmap patterns,
the texture library list, saving a new preset from the picker, and the
docker header's Scroll/Tab mode and style indicators.
