# Copy Effect and Clone Effect

Object > Copy Effect and Object > Clone Effect give the selected objects
an effect of another object: choose the entry, then click the object to
copy from (a right-click cancels).

## Copy Effect

Entries: Drop Shadow, Transparency, Envelope, Perspective, Distortion,
Extrude, Contour, Blend, Lens, Bevel, Block Shadow (`copy_effect.rs`).

- Drop Shadow and Transparency are copied as they are (the opacity and
  the transparency mask).
- Envelope fits the source's envelope to each target (see
  `envelope.md`).
- The other live effects replace the target's own effect of that kind
  where it is among its effects, or are added after them.
- What an effect pins to the source's own space is carried over by its
  place in the source's box (its outline's bounds in its own space),
  each side stretched to the target's box: a perspective's four
  corners, a distortion's centre, a vanishing point locked to the
  object. A distortion about the middle stays about the middle; a
  vanishing point locked to the page stays where it is.
- Blend From gives the selection's blends (of a selected start or end
  object) the source blend's steps, accelerations and rotation.
- A source without that effect changes nothing and says so on the
  status bar. One undo step.

## Clone Effect

Shadow and Transparency: copied as above, and the clone keeps following
the source until its own effect is edited (`clone_effect.rs`).

## Checks

- `copy_effect::tests::live_effects_copy_and_sit_the_same_way`
- `copy_effect::tests::every_kind_has_a_menu_entry`
- Given a twister centred at the top right corner of a 10 mm square,
  when it is copied to a 40 by 20 mm rectangle at (100, 0), then its
  centre is (140, 20).
