# Status bar

The bar is 41 px under a 1 px `#D8D8D8` line, `#F4F4F4`, text 13 px.
It sits at the bottom of the window, or at the top below the menu bar
(right-click it for Dock at Top; see "Moving the bars" in
`toolbars.md`). From the left:

1. The settings button (a gear with a small corner arrow). Its menu
   chooses what the first field shows:
   Tool Hints (default), Object Details, Cursor Coordinates or Document
   Color Settings. The choice is kept in the settings (`status_info`).
2. The first field (while the Pick tool moves, scales, rotates or skews
   the selection it shows that instead: "dx, dy, Distance, Angle" in the
   drawing units for a move, "Scale: x % x y %" (negative when mirrored),
   "Rotation: a°" counter-clockwise, "Skew: a°", and "(a copy, the
   original stays)" after a right click during the drag):
   - Tool Hints: the active tool's hint (`status_hint.<tool id>` in the
     language files; every tool has one). The Pick tool's hint changes
     when something is selected. A message from the last command comes
     first when there is one.
     Hints are short and share one style: the parts are joined by " · ",
     keys and modifiers are named as they are pressed (Ctrl+drag,
     Shift+click, Alt+click), and a hint says what the pointer does
     ("Double-click the tool to select all"). The Hints docker and the
     document palette's empty-state text follow the same style
     (`ui::status::tests::hints_share_one_style`).
   - Object Details: width, height and centre of the selection in the
     drawing units.
   - Cursor Coordinates: the pointer's page position in the drawing units.
   - Document Color Settings: the RGB, CMYK and grayscale profiles.
3. The object information: "Rectangle on Layer 1", "Curve on Layer 1",
   "2 Objects Selected on Layer 1" (or "on Multiple Layers"); with the
   Shape tool and a curve selected, "Curve: N Nodes"; while typing,
   "Editing text". It starts after the first field, never closer than
   156 px from it.
4. The fill: the Interactive Fill icon (20 px), a 26 x 25 px swatch and
   a description (None, the colour values, or the fill type). At 68.5 %
   of the bar.
5. The outline: the outline pen icon, a swatch and the colour values with
   the width in the drawing units (three decimals; two in pixels), or
   Hairline, or None. At 81.8 % of the bar.
6. After a short vertical line, the proof colours button at the right end
   (a monitor, red over sky blue with a lens); a click toggles soft
   proofing and its tooltip says whether it is on.

With nothing selected the fill and outline parts show the defaults for
new objects: no fill, a 0.2 mm black (C0 M0 Y0 K100) outline. A swatch
with no colour is a white box crossed by a red line. Double-clicking the
fill part opens the Edit Fill dialog box (as F11); the outline part, the
Outline Pen dialog box (as F12) (#465).

With no drawing open the bar shows the placeholders Tool Hints, Object
Information, Fill Color and Outline Color.

## Checks

- `ui::status::tests::every_tool_has_its_own_hint`
- `ui::status::tests::object_information_names_the_object_and_its_layer`
- `ui::status::tests::outline_width_is_shown_in_the_drawing_units`
