# Preflight

Checks a drawing for what tends to go wrong in print before it is
printed or sent to a print shop (`preflight.rs`, dialog in
`ui/preflight_dialog.rs`).

## Where

- File > Preflight... opens the list for the active drawing.
- The Print dialog shows the outcome under the prepress options: a
  warning triangle (or a note's circle) with "Warnings: n Notes: n", or
  "No issues found"; Show Issues... closes it and opens the list. The
  checks run once when the Print dialog opens.

## Checks

| Check | Kind | What it finds |
|---|---|---|
| Empty text object | note | text objects whose text is empty or only spaces |
| Text does not fit its frame | warning | paragraph text whose last frame (unlinked, or the end of a chain of linked frames) hides part of the text |
| Font not installed | warning | text in a font family that is not installed (detail: the family) |
| Font may not be embedded | note | text in a font whose licence forbids embedding (OS/2 embedding bits): PDF files get that text as curves (detail: the family) |
| Very small text | warning | text smaller than 5 pt as printed (its size times the object's scale) |
| Small text in rich black | warning | text smaller than 12 pt filled with CMYK black of 99% K or more plus other inks |
| RGB colour in a CMYK drawing | warning | RGB, HSB, HSL or YIQ colours in fills (solid, fountain stops, mesh nodes, two-colour patterns), outlines or character fills of a drawing whose primary colour mode is CMYK |
| Object off the page | warning | top-level objects whose bounds miss the page and its bleed |
| Low-resolution bitmap | warning | bitmaps below 200 dpi as placed (pixels over the placed edge length, the lower of the two directions) |
| Very high-resolution bitmap | note | bitmaps above 1200 dpi as placed |
| Very thin outline | warning | outlines thinner than 0.1 mm, hairlines included (detail: the width) |
| White set to overprint | warning | white fills or outlines with overprinting on |
| Empty page | note | pages without objects (master layers aside) |
| Objects that will not print | note | layers that are hidden or not printable and hold objects (detail: the layer and its count) |

Objects inside groups and clip frames are checked; an issue names the
top-level object, which is what gets selected. Each kind is reported once
per object and detail. Objects on hidden or non-printing layers are not
checked otherwise. Master layers are checked once, with "Master" as the
page.

## Dialog

- The outcome at the top: a green tick and "No issues found", or the icon
  of the worst kind with the counts.
- The list: a heading row per kind found, in the order of the table
  above, with its icon and count; kinds with up to three issues start
  opened. Clicking a heading opens or closes it; under it, one row per
  issue with its page, the object's name (as the Objects panel names it)
  and the detail. Clicking an issue goes to its page and selects its
  object.
- Under the list, what to do about the kind picked.
- Check Again runs the checks again; Choose Checks... shows a check box
  per check and the thresholds: bitmaps below (200 dpi) and above
  (1200 dpi), text smaller than (5 pt), black text smaller than (12 pt),
  outlines thinner than (0.1 mm), and Defaults. Changes run the checks
  again at once and are kept in the preferences (`preflight`).
- Close.

## Checks

- `preflight::tests::a_clean_drawing_has_no_issues`
- `preflight::tests::text_problems_are_found`
- `preflight::tests::colours_outlines_and_overprints_are_checked`
- `preflight::tests::pages_layers_and_placement_are_checked`
- `preflight::tests::bitmap_resolution_is_measured_as_placed`
- `preflight::tests::every_check_has_a_name_and_a_hint`
- `ui::preflight_dialog::tests::picking_an_issue_selects_its_object_and_the_dialog_draws`
