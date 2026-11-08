# Object styles and default properties

The Object Styles panel (Window > Panels, or right-click an object >
Object Styles > Object Styles Panel) has two parts: the default
properties of new objects and the drawing's styles. Both are saved with
the drawing (`Document::defaults`, `Document::object_styles`); the
engine applies them (`SetObjectDefaults`, `SetObjectStyles`,
`LinkObjectStyle`), each change one undo step.

## Default properties

New objects take the fill and outline of their kind:

| Kind | A new drawing's defaults |
|---|---|
| Graphic (shapes, curves, tables, everything but text) | no fill, a 0.2 mm black outline |
| Artistic text (text without a frame) | black fill, no outline |
| Paragraph text (text in a frame) | black fill, no outline |

The black is the one of the drawing's primary colour mode: 100 % K in a
CMYK drawing, RGB black (0, 0, 0) in an RGB drawing. Only this untouched
default black follows the colour mode; a colour the user chose, a black
included, stays exactly as chosen:

- Each kind's defaults mark which of their fill and outline colours are
  the default black (`DefaultProperties::follow`, saved with the
  drawing and with Tools > Save as Defaults). A new drawing's defaults
  and Reset mark the black outline of graphics and the black fill of
  text.
- Picking a fill or outline colour for the defaults (a palette colour,
  the colour docker, the Properties panel, From Selection) makes it the
  user's own, even when it is the same black; changing only the outline
  width keeps the mark (`ObjectDefaults::settled`, run by the
  `SetObjectDefaults` command).
- Saved defaults (Tools > Save as Defaults) give a new drawing the
  drawing's black where they still hold the default black, and every
  other colour as saved, so a CMYK 100 % K outline chosen on purpose
  stays CMYK in an RGB drawing.
- Changing the primary colour mode (Document Color Settings) takes the
  default blacks along, in the same undo step, and leaves chosen colours
  alone.
- Drawings and settings saved before the mark existed carry none: there
  a black (100 % K or RGB black) where the built-in defaults put one,
  the outline of graphics and the fill of artistic and paragraph text,
  counts as the untouched default and follows; a black anywhere else
  (a graphic's fill, a text outline) and every other colour count as
  chosen (`DefaultProperties::follows`).

- A fill or outline chosen with nothing selected (a palette colour, the
  Fill tool, the outline colour or width, None for the outline on the
  property bar) changes defaults. A question asks which kinds of new
  objects take it: Graphic, Artistic text, Paragraph text (the kinds
  ticked last time are ticked; Graphic at first), with "Use these kinds
  from now on without asking". OK changes the defaults of the kinds
  ticked and says so in the status bar; Cancel changes nothing. An
  outline width only changes kinds that have an outline.
- In the panel each kind shows its fill and outline swatches (a red
  stroke across means none) with From Selection (the fill and outline of
  the first selected object, its colours the user's own) and Reset (a
  new drawing's defaults, black in the drawing's colour mode and
  following it again). The
  check box under them turns the question on or off.
- The Properties panel with nothing selected edits the Graphic defaults.
- Object > Create > Pattern Fill and Vector Pattern Fill make the
  pattern the Graphic default fill.
- Tools > Save as Defaults keeps the defaults for new drawings.
  Drawings saved before default properties existed open with a new
  drawing's.
- The text defaults' black is the drawing's own: in a CMYK drawing new
  text is CMYK black, in an RGB drawing RGB black, as long as the text
  black is the untouched default (`settings::defaults_for`,
  `settings::tests::stored_black_text_defaults_follow_the_drawings_colour_mode`,
  `settings::tests::saved_defaults_keep_chosen_blacks`,
  `settings::tests::old_settings_with_the_stock_blacks_follow`,
  `command::tests::chosen_blacks_stay_as_chosen_when_the_mode_changes`,
  `command::tests::older_drawings_work_out_which_blacks_follow`).

## Styles

A style is a named set of properties with the parts it sets: Fill,
Outline (all of the outline: colour, width, line style, arrowheads) and
Transparency (the object's opacity). Objects a style is applied to stay
linked to it.

- New style from selection (panel, context menu): a style named Style n
  with the fill, outline and opacity of the first selected object (parts
  Fill and Outline, and Transparency when the object is not opaque),
  linked to every selected object.
- Each style shows its swatches, its name (edited in place), how many
  objects are linked to it (in groups and clip frames too) and its parts
  as check boxes. Apply gives the selection the style's parts and links
  it; Update from selection takes the first selected object's fill,
  outline and opacity; Delete removes the style.
- Changing a style (its parts, Update, a name) gives every linked object
  the style's parts again; parts the style does not set stay as each
  object has them. A linked object can still be changed directly; the
  next change of its style sets the style's parts again.
- Break Link to Style (panel, context menu) unlinks the selected objects;
  they keep their properties. Deleting a style unlinks its objects the
  same way.
- The context menu's Object Styles lists the styles to apply.
- Pasted objects stay linked when the drawing they go into has the same
  style (same number and settings), as within one drawing; otherwise
  they are unlinked and keep their properties.
- Styles from drawings saved before links existed set fill and outline,
  and get a number the first time one is applied.

## Checks

- `command::tests::styled_objects_follow_their_style`
- `command::tests::new_objects_take_the_defaults_of_their_kind`
- `defaults::tests::changes_reach_the_kinds_picked_and_new_objects`
- `styles::tests::a_style_from_the_selection_links_it_and_follows_edits`
- `ui::defaults_dialog::tests::the_question_draws_and_keeps_the_choice`
