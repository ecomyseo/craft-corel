# Sliders

Every slider in the bars, dockers and dialogs is the one widget
`ui::Slider` (`apps/tracedraw/src/ui/slider.rs`). Only the Color docker's
sliders (its colour channel sliders and its spot ink Tint slider) and the
Trace Bitmap dialog's sliders still use the toolkit's slider.

## Look

- The label, when the slider has one, sits above the control, left
  aligned and ending with a colon ("Detail:").
- Under it the track: a thin groove, 3 points thick, in the theme's
  `slider_rail` colour, spanning the width of a column (a docker, a
  dialog's body) up to the number box; in a row of controls (a bar) it
  takes `Spacing::slider_width`, or the width the caller gives.
- Tick marks under the groove, like a ruler: minor ticks 3 points long,
  major ticks 6 points, in the theme's `ruler_tick` colour. A round major
  value (1, 2 or 5 times a power of ten) falls about every 45 points;
  minors divide a major into fifths (fourths for a 2) and are never
  closer than the step or than 3 points. Ticks lie at multiples of their
  spacing (0, 20, 40 for a 0 to 100 range), not at offsets from the
  range's start. A logarithmic slider ticks at 1, 2 and 5 of each decade.
  `.ticks(false)` draws none.
- The thumb: an upright slip 6 by 14 points, taller than the groove,
  with the button fill and the control border; the accent border when
  hovered, focused or dragged; the panel fill and the dim border when
  disabled. Everything is rounded to whole pixels at every scaling.
- The number box at the right of the groove (`NumField`, 64 points wide
  with its spin arrows) shows the value with its suffix (`%`, `pt`,
  `°`). `.show_value(false)` leaves it out. A mixed selection shows an
  empty box.

## Behaviour

| Input | Effect |
| --- | --- |
| Press or drag on the track | The thumb jumps to the pointer and follows it |
| Left/Right arrow (focused) | One step down/up (Up/Down on an upright slider) |
| Page Down/Up (focused) | Ten steps |
| Home/End (focused) | The range's start/end |
| Wheel over the track or the box (hovered; no focus needed) | One step a notch; Ctrl a tenth (one step on an integer or stepped slider), Shift ten; the panel around does not scroll with that turn |
| Wheel beside a slider (its label, the space around it) | Scrolls the panel around it; the value stays |
| Number box | Typing then Enter or leaving applies (snapped to the step); the arrows step |

The track takes the keyboard focus only from Tab, as the toolkit's own
slider: a press or a drag on it does not take the focus (and gives up
one Tab gave), so the canvas keeps its shortcuts (Ctrl+Z, Delete, the
tool keys, the arrows' nudge) after a slider was used with the mouse.
The wheel over the track or the box steps the slider on hover, as it
does a number field: no click or focus first (#1335, a maintainer
decision; it replaces the earlier rule that a slider stepped with the
wheel only while focused or dragged, #1240). Anywhere else a docker or
the Options dialog scrolls under the wheel as before. The wheel takes
no keyboard focus. The Color docker's channel sliders and its Tint
slider and the Trace dialog's sliders, drawn differently (gradient
rails, ticks of their own), step the same way on hover: one a notch
(a channel in its shown unit, 0 to 255 or 0 to 100), Ctrl a tenth,
Shift ten. Check: `ui::color_docker::tests::the_wheel_over_a_colour_slider_steps_it`,
`pick_drag_tests::the_wheel_over_a_trace_slider_steps_it`.

The step is the one given (`step_by`), one for an integer slider, else a
round hundredth of the range (1 for 0 to 100, 0.05 for 0.1 to 5). Every
change snaps to a multiple of the step from the range's start and stays
in the range, a value typed in the box too (12.5 in a whole-number
slider is 13, which the box then shows); the value handed to the slider
is clamped to the range too. Ctrl with the wheel or the box's arrows
moves a tenth of a step on a slider without a step and one step on an
integer or stepped one, which holds nothing finer. The box and the
thumb always show the same value: the box reads the value each frame it
is not being typed in.

`Response::changed()` is set by every way of changing the value;
`dragged()` and `drag_stopped()` come from the track. An editor that
applies a value once (one undo step: the fill and outline editors of the
dockers) reads `slider::Settled::settled()`, true when a drag ends or the
value changed without a drag (a click, a key, the wheel, the box), so a
value typed or stepped is applied instead of snapping back.

## Checks

- Given a 0 to 100 percent slider in a 300 point column, when drawn,
  then the label "Detail:" is above it, the box reads "40 %", the groove
  leaves the box its room and 21 tick segments (majors every 20, minors
  every 5) lie under the groove.
- Given the slider, when the track is pressed at 75 %, dragged to 20 %,
  focused with Tab and stepped with the arrows, Page Up, End and the
  wheel, then the value follows each and the box shows it.
- Given the slider, when "250" is typed into the box and Enter pressed,
  then the value is 100 (clamped); with a step of 25, a press at 60 %
  gives 50.
- Given a disabled slider, when pressed or wheeled, then the value stays.
- Given the slider, when its track is clicked, then nothing has the
  keyboard focus and an arrow key leaves the value; while the thumb is
  held the wheel steps it.
- Given the slider with nothing focused, when the wheel turns over the
  track or the box, then the value moves one step a notch (Shift ten)
  and the panel does not scroll; over the label the value stays and the
  wheel is left to the panel
  (`ui::slider::tests::the_wheel_over_a_slider_steps_it_on_hover`).
- Given the Objects docker, when the wheel turns over its thumbnail size
  slider, then the size moves one step a notch, not two.
- Given a focused slider, when the wheel turns with Ctrl, then a slider
  without a step moves a tenth of a step and a stepped one a whole step.
- Given the slider with a step of one, when "12.5" is typed, then the
  value is 13 and the box shows "13 %".
- Given the texture fill editor, when a value is typed into its Scale
  box or its focused track is stepped with an arrow key, then the editor
  reports the change.
- Given the Export dialog at JPEG, when its Quality track is pressed,
  then the export settings take the quality.
