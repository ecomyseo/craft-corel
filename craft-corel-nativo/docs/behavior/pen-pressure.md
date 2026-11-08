# Graphics tablets and pen pressure

A graphics tablet's stylus moves the pointer and presses its button like
a mouse, so every tool works with it. Where the platform also reports how
hard the pen is pressed, the Freehand and Brush Strokes tools let that
pressure set the width of the stroke (#850). The mouse, and a pen whose
platform gives no pressure, draw exactly as before: an even width.

## Where the pressure comes from

The windowing layer (winit 0.30 through egui-winit 0.36) reports a pen as
a touch with a force in 0..1 (`egui::Event::Touch { force }`), and at the
same time moves the pointer and presses its primary button. TraceDraw's
raw input hook reads the force of the touch events of each frame
(`pen.rs`, `filter_raw_input`) before egui and the tools see them; a
touch that ends, or one with no force, clears it. The published winit does this on Windows
only; TraceDraw's patched copy (`vendor/winit`, see `vendor/README.md`)
adds X11, Wayland and macOS, and the patched eframe copy (`vendor/eframe`) the
browser. What each platform gives:

| Platform | Pointer from the pen | Pressure |
|---|---|---|
| Windows (8 and later) | yes (`WM_POINTER` pen events) | yes, 0..1 from the pen's 0..1024 |
| Linux, X11 (also XWayland) | yes (XInput 2 motion) | yes: the pen's valuator labelled `Abs Pressure`, as a share of its range (patched copy) |
| Linux, Wayland | yes: the tablet protocol (`zwp_tablet_manager_v2`, patched copy, #1841) | yes: the protocol's `pressure`, 0..65535, as a share of that range (patched copy) |
| macOS | yes | yes: the `pressure` of mouse events of the tablet point subtype (patched copy) |
| Browser | yes (pointer events) | yes: the `pressure` of a pen's pointer events (`pointerType` `pen`), already 0..1 (patched copy, #1410); a touch's force as before |

On X11, Wayland and macOS the patched copy sends the pressure as a touch of phase
Move only, never Start or End (those would press the button a second
time through egui-winit), with no force once the reading is at the
axis's minimum, so a lifted pen leaves no stale pressure. The hook then
takes out every moving touch whose id never started, so egui never sees
the pen as a touch: egui switches to touch screen mode for the whole
session on any touch (lists scroll by dragging, windows move from
anywhere, text in labels cannot be selected by dragging). A real touch
screen's touches start, move and end, and reach egui as before.

## On Wayland

A Wayland compositor sends a pen's input only over the tablet protocol
(`zwp_tablet_manager_v2` and the tablet seat, tablet and tool of each
seat, the public tablet-v2 protocol of wayland-protocols); GNOME's sends
a window that does not bind it nothing at all, so before #1841 the pen
neither moved the pointer nor drew on a Wayland desktop (Ubuntu's
default). The patched winit copy now binds the protocol when the
compositor offers it (`platform_impl/linux/wayland/tablet.rs`) and turns
each tool's events into what the other platforms send
(`vendor/winit/src/tablet_tool.rs`):

- The pen coming near the tablet over the window enters it, and moving
  away leaves it, as the mouse pointer does; its motion moves the
  pointer. Over the window the pen shows the window's own cursor and
  follows its changes, as the mouse does: none where TraceDraw draws
  the tool's cursor itself (the cross hair with the tool's badge, the
  Shape tool's arrow, the hand), the system's resize, move or text
  cursor through the cursor shape protocol (`wp_cursor_shape_manager_v1`,
  when offered), a custom image on the pen's own cursor surface. Before
  (#1841 follow-up) the compositor always showed its arrow for the pen.
  Over the window's frame the pen shows the default arrow.
- The pen's tilt is read and carried with its pressure as an altitude
  (`Force::Calibrated`, whose share egui reads unchanged, so the
  pressure is exactly as before); egui's touch events have no place for
  it and no tool uses it yet.
- The tip touching the tablet presses the primary button, lifting it
  releases it. The barrel buttons press the middle (`BTN_STYLUS`) and
  the secondary (`BTN_STYLUS2`) button, a third one (`BTN_STYLUS3`)
  the back button. The eraser end is a tool of its own and draws like
  the tip.
- While the tip touches, its pressure (0 to 65535) goes out as a moving
  touch with the reading as a share of that range, at every frame that
  moves the pen or changes the reading; a reading of 0, lifting the
  tip, or the pen leaving gives no force. Leaving the window also
  releases any button the pen still held.
- The protocol groups a tool's events into frames; a frame's place,
  pressure, presses and releases are sent together when it ends.
- The pen's distance, rotation, slider and wheel, and the tablet's own
  buttons, rings and strips (the pad) are not read: the windowing
  layer's events have no place for them and no tool uses them.
- Over the window's frame (drawn by the windowing layer on GNOME) the
  pen does nothing; move or resize the window with the mouse.
- A compositor without the tablet protocol (rare: GNOME, KDE and the
  wlroots compositors offer it) leaves the pen as before.

`tracedraw --x11` still opens the window through X11 instead: it asks
the windowing layer for its X11 backend, so the window opens on
XWayland, where the pen moves the pointer, clicks and reports its
pressure (XWayland lists the pen as an XInput 2 device, `xwayland-tablet
stylus`, with the same `Abs Pressure` valuator). The environment is left
as it is, so programs TraceDraw starts (a browser for a link) still run
on Wayland. Without `DISPLAY` (no XWayland, no X server) `--x11` stops at
once with a message saying it needs XWayland or an X11 session, and
exits with status 2. `env -u WAYLAND_DISPLAY tracedraw` opens the window
on XWayland too. `GDK_BACKEND=x11` does not: it is read by GTK programs
only, and the window opens on Wayland whenever `WAYLAND_DISPLAY` is set.

An X11 touchpad driver that exposes
an `Abs Pressure` valuator for a finger (the old synaptics driver) would
drive the width too; the option below turns it off.

## In the browser

The browser sends a pen as pointer events of `pointerType` `pen`, with
their `pressure` (0 to 1), beside the mouse events the page already
moved the pointer and pressed the buttons with; Chromium, Firefox and
Safari all fill `pressure` in for a pen whose system reports it. The
published web layer of eframe read a force only from touch events, so
a pen drew with an even width (#1410). The patched copy
(`vendor/eframe/src/web/pen_pressure.rs`, `events.rs`) sends a pen's
pressure the way the X11 and macOS copy does: a moving touch with the
reading as its force, never a start or an end, which the raw input hook
reads and takes out as above. It is sent at the pen's `pointerdown`,
at every `pointermove` over the canvas (anywhere during a drag) and at
`pointerup` and `pointercancel`, which give no pressure.

The pen's place comes from its own pointer events as well (#1655):
Safari on an iPad sends an Apple Pencil no mouse events, only pen
pointer events and touches of `touchType` `stylus`, so the Pencil did
nothing at all while a finger worked. A pen's `pointerdown` and every
`pointermove` now also move the pointer, before the press they may
carry; the browser is kept from handling a pen pressed on the canvas
itself (no text selection, no handwriting), and the canvas takes every
touch and pen gesture (`touch-action: none` in `web/index.html`, so the
page neither scrolls nor zooms under it); a `pointercancel` releases the
tip and the pointer, so no drag is left pressed; and the Pencil's touches
are left out of the touch events, so it presses once and egui never takes
it for a finger. On a desktop browser a pen's mouse events still come
too and move the pointer to the same place.

- Only a pen gives pressure. The mouse sends nothing: browsers report
  0.5 for any pressed button that cannot measure pressure, which is not
  a reading. A touch screen's finger (or a stylus the browser reports as
  a touch) keeps its force from the touch events, as before.
- A pen gives pressure only while its tip or its eraser touches
  (`buttons` bit 1 or 32) and the reading is above 0 and a number; a
  hovering pen, its barrel button alone, a lift or a cancel clear the
  pressure. Readings above 1 count as 1.
- A pen that cannot measure pressure also reports 0.5 while pressed, so
  0.5 counts as no pressure, and such a pen draws like the mouse, until
  a pen on the page has given any other reading; from then on 0.5 is a
  reading like any other. (A real pen whose first readings of a page
  are exactly 0.5 thus draws its first stroke without pressure.)
- One reading per event. Freehand and Brush Strokes take one point of
  the drag per frame, so the readings `getCoalescedEvents()` would add
  between two frames would be dropped; the event's own reading is the
  latest. The pen's tilt (`tiltX`, `tiltY`) is not read: no tool uses
  it.

## What the pressure does

- Options > Tools > Curve tools > Graphics tablet > "Pen pressure sets the
  width of Freehand and Brush Strokes drags", on by default. Off, a pen
  draws like the mouse.
- Along a Freehand or Brush Strokes drag, the pressure at every sampled
  point is recorded with it. A drag that starts without pressure records
  nothing, so the stroke gets its usual even width. Once a drag has
  pressure, a point that comes without a reading (a move with no force,
  the pen lifting off) takes the last reading again.
- The recorded pressures are read by distance along the drawn polyline
  (before smoothing), and each width node averages the samples within
  half a node spacing of it, so the jitter of the pen's readings does
  not reach the outline; a node with no sample that close takes the
  nearest one.
- Freehand: the new line's outline gets 25 width nodes (`t` = 0, 1/24,
  ..., 1), each `w x (0.05 + 0.95 x pressure)` wide and never less than
  a hairline (0.0762 mm, or `w` when the outline is thinner still), `w`
  the default outline width: a feather-light touch leaves a hairline,
  full pressure the full width. A line that continues a selected curve keeps that
  curve's outline. The width nodes are the outline's variable width:
  the Shape tool edits the curve, and the Properties panel the outline,
  as for any variable-width line.
- Brush Strokes: the pen scales the width of the Preset, Brush and
  Expression modes by `(1 - p) + p x pressure`, `p` the Brush Strokes
  panel's Pressure setting (0.5 by default: half the width at no
  pressure, the full width at full pressure; 1: zero to full). Preset
  and Brush keep their profile along the stroke (the taper, the wave,
  the jittered edge) and multiply it by the pen's scale; Expression
  takes the pen's pressure in place of the drawing speed. The
  calligraphic nib and the sprayer's objects keep their size.
- Apply to curve (Brush Strokes panel) has no pen: it builds the stroke
  as before.
- Nothing is done per frame beyond reading the frame's touch events (a
  frame without any costs one scan of its events) and pushing one number
  per sampled point: the stroke's preview stays the
  thin line, and the widths are built once, on release.

## Checks

- Given a Freehand drag with a pen pressed at 0.9 over the first half
  and 0.2 over the second, then the new line's outline has width nodes
  more than twice as wide in the first third as in the last, never
  wider than the outline width; the same drag with the mouse, or with
  the option off, gives an outline without width nodes (test
  `pen_pressure_sets_the_width_of_a_freehand_line`).
- Given Brush Strokes in Expression mode at width 6 mm and Pressure 1,
  when a pen presses 1.0 then 0.25 along an even drag, then the stroke
  is 6 mm wide at the start and less than 2/5 of that at the end; the
  mouse's stroke takes its widths from the speed, as before (test
  `pen_pressure_drives_an_expression_stroke`).
- Given a pen that sends only moving touches (X11, macOS), pressed at
  0.9 then 0.2 with one reading without a force in the middle and none
  as it lifts, then the Freehand line is still wide then thin, no width
  node is below a hairline, and egui reports no touch screen (test
  `a_pen_that_only_sends_moving_touches_sets_the_width`).
- Given a pen on a Wayland window, the tablet protocol's events of a
  stroke pressed at 0.9 then 0.2 of the range (proximity, down, motion
  and pressure, up, proximity out, in frames) turned into egui events
  as the patched winit copy and egui-winit do, then the Freehand line
  is wide then thin and egui reports no touch screen; a pen whose
  pressure stays at 0 gives an outline without width nodes (test
  `a_pen_on_wayland_sets_the_width`).
- The Wayland translation of a tool's events: nothing before a frame
  ends; a stroke enters, moves, sends its pressure, presses and
  releases in that order; a lift sends no stale pressure; a tap in one
  frame presses and releases; barrel buttons press the middle,
  secondary and back buttons once each; leaving proximity, or moving to
  another surface, releases what is held and leaves; events out of
  proximity send nothing (tests in `vendor/winit/src/tablet_tool.rs`,
  run as `winit_tablet_tool::tests`).
- Given a pen in the browser, its pointer events pressed at 0.9 then
  0.2 and lifted, turned into egui events as the web layer does, then
  the Freehand line is wide then thin and egui reports no touch screen;
  the mouse's pointer events (0.5 while pressed) and a pen without
  pressure (a constant 0.5) give an outline without width nodes (test
  `a_pen_in_the_browser_sets_the_width`).
- Given an Apple Pencil in Safari, whose pointer events alone (no mouse
  events) carry its place, press and pressure, then a Freehand line is
  drawn wide then thin and egui reports no touch screen; a cancel in the
  middle of a stroke leaves nothing pressed
  (`an_apple_pencil_in_safari_draws_with_its_pressure`); the web layer's
  helpers leave out a stylus's touches only, move the pointer for a pen
  only and release the tip on a cancel (`eframe_web_pen_pressure::tests`).
- The web layer's mapping of a pointer event's type, pressure and
  buttons: nothing for a mouse or a touch, the reading for a pen pressed
  on its tip or eraser, no pressure for a hovering pen, a reading of 0
  or not a number, and 0.5 before any other reading; the event is a
  moving touch of the pen's own device (tests in
  `vendor/eframe/src/web/pen_pressure.rs`, run as
  `eframe_web_pen_pressure::tests`).
- The pressure follows the touch events (a start or move with a force
  sets it, a move without one, an end or a cancel clears it, readings
  are clamped to 0..1); moving touches that never started give their
  pressure and never reach egui, whose `has_touch_screen` stays off,
  while a touch screen's start, moves and end reach it; a stroke repeats
  its last pressure at a point without one; a Freehand line is never
  thinner than a hairline; and the profile averages the samples within
  the window and scales the width between the floor and the full width
  (tests in `pen.rs`).
