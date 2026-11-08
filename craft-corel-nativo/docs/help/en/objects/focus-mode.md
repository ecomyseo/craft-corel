# Focus mode

Focus mode lets you edit the objects inside a group or a clip frame without
ungrouping it, with the rest of the drawing faded behind it.

## Entering and leaving focus mode {#enter}

- Double-click a group or a clip frame with the [[tool.pick]] tool.
- Or right-click an object and choose **Bring into focus**. For an object
  inside a group, the group gets the focus and the object stays selected.
- Double-click a group inside the focused group to go one level deeper.
- Press <kbd>Esc</kbd> with nothing selected to go up one level, and leave
  focus mode from the top. The cross on the breadcrumb bar leaves at once.

## While in focus mode {#working}

A bar at the top left of the drawing window shows the name of the drawing,
then each group on the way down. Click a step to go back to that group, or
the name of the drawing to leave focus mode.

Clicks and marquees select the objects inside the focused group, so you can
move, color or delete them with the [[tool.pick]] tool as usual. Everything
else is faded. Exports and printing are not affected.

## Settings {#settings}

- How faded the other objects are is set by the visibility of inactive objects
  (0 to 100%, 20% at first) on the [[options.edit]] page of
  [[menu.tools > menu.tools.options > menu.tools.options_app]].
- Whether a double-click enters focus mode is set on the same page.

See [TraceDraw options](../customize/options-app.md#edit).
