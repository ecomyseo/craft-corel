# Rollovers

A rollover is a web button: an object that looks one way on the page, another
while the pointer is over it, and a third while it is pressed. Its three states
are **Normal**, **Over** and **Down**.

## Object > Rollover

| Item | Enabled | What it does |
|---|---|---|
| Create Rollover | one or more objects on a layer are selected, none of them a rollover, and no rollover is being edited | The objects become one group, the rollover. Its three states start as copies of them (Over and Down get objects with ids of their own); Normal is shown and the rollover is selected. One undo step. |
| Break Apart Rollover | a rollover is selected and none is being edited | Each state that has objects becomes a plain group named after it, in the rollover's place: Normal at the bottom, then Over, then Down. All three are selected. One undo step. |
| Edit Rollover | a rollover is selected and none is being edited | Starts editing its states: the objects of the state shown (Normal at first) are selected, ready to change. |
| Finish Editing Rollover | a rollover is being edited | Shows Normal again and selects the rollover as a whole. |
| Normal, Over, Down | a rollover is being edited | The state to edit: its objects are shown and selected (one undo step, Rollover State). A change to them changes that state only. |
| Live Preview of Rollovers | always (check item) | While on, a rollover under the pointer is drawn in its Over state, and in its Down state while the primary button is held. The drawing and its history do not change. Off while a rollover is edited. |

The drawing cannot be closed while one of its rollovers is being edited; the status
bar says to finish editing first.

## Links docker

With a rollover selected, or one being edited, the Links docker shows:

- **Edit Rollover** and **Finish Editing Rollover** buttons;
- **Active state** (while editing): Normal, Over or Down;
- **Target**: the window or frame a click opens the object's hyperlink in
  (`_blank` for a new window); empty for the page itself (Rollover Target, one undo
  step). The hyperlink is the object's own (the field at the top of the docker).

## Files

- The native `.tdraw` file keeps the three states, the state shown and the target.
- An HTML export writes each rollover with its three states (`class="ro"`, states
  `ro-0` to `ro-2`) and a style sheet that shows Normal, Over under the pointer and
  Down while pressed, with no script. A hyperlink opens in the rollover's target.
- An SVG export, a `.cdr` file and every other format keep the state shown, as a
  plain group.

## Not done

- Objects added to a state while editing stay in that state only.
- Reading rollovers from `.cdr` files: they open as the group of the state the file
  shows.
