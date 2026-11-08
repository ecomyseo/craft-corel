# Focus mode

Focus mode edits the objects inside a group or a clip frame without
ungrouping it, with the rest of the drawing faded behind it.

## Entering and leaving

- Double-click a group or a clip frame with the Pick tool (Tools > Options >
  Workspace > Edit: "Double-click to activate focus mode", on by
  default).
- Right-click an object > Bring into Focus. For a plain object inside a
  group, the group that holds it gets the focus and the object stays
  selected.
- Focus can go deeper: double-click a group inside the focused group.
- Esc with nothing selected goes up one step; once at the top it leaves
  focus mode. The ✕ on the breadcrumb leaves at once.

## Breadcrumb

A bar at the top left of the drawing window shows the document title,
then each group on the way down to the focused one. A click on the
title leaves focus mode; a click on a step goes back to that group.

## While focused

- Clicks and marquees pick the members of the focused object, not the
  object itself, so every Pick tool action (move, scale, colour, delete)
  works on its members.
- Everything else is drawn at the "Visibility of inactive objects"
  (Tools > Options > TraceDraw > Edit, 0 to 100 %, default 20 %; Reset restores
  it). The focused object is drawn again at full strength on top.
- Exports and printing are not affected.

## Formula

Every pixel of the page is multiplied by `visibility / 100`, then the
focused object is rendered over it with its parents' transformation.
