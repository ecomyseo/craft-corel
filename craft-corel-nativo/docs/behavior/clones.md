# Clones

Edit > Clone makes a copy of each selected object, moved by the
duplicate offset (Options, as for Duplicate), and selects the copies.
Unlike a duplicate, a clone stays linked to the object it was made
from, its master.

## What a clone follows

- The master's fill, outline and shape (for a rectangle its size and
  corners, for a curve its nodes, for text its characters and
  formatting). A group or a clip frame clone follows everything its
  master holds, with objects of its own (new ids).
- Not the master's place, size or turn: moving, scaling or rotating
  the master leaves the clone where it is, and the clone can be moved
  and transformed freely.
- Each property is followed until the clone's own value of it is
  changed: giving the clone a fill of its own stops it following the
  master's fill, while it still follows the outline and shape.
- An update reaches the clone in the same undo step as the master's
  change, so one Undo takes both back.
- A clone cannot be cloned again (the command is greyed when only
  clones are selected; clones in a wider selection are left out with a
  note in the status bar). Duplicating a clone gives another clone of
  the same master.
- When the master is deleted, its clones stay as ordinary objects with
  what they last had.

## Context menu

| Item | Shown when | Does |
|---|---|---|
| Select Master | a clone is selected | selects the masters of the selected clones |
| Revert to Master | a clone is selected | gives the clones the master's fill, outline and shape again and makes them follow all three again (one undo step) |
| Break Clone Link | a clone is selected | the clones become ordinary objects |
| Select Clones | a master is selected | selects every clone of the selected masters |

## Saved data

The link is kept in the clone's object data, so it survives save and
load: `clone.master` holds the master's id and `clone.state` holds, per
followed property (`fill`, `outline`, `shape`), the hashes of the
master's and the clone's value at the last update, or `own` once the
clone has a value of its own. After every recorded step the hashes are
compared: a clone whose value no longer matches its hash has been
edited and keeps its value; a master whose value changed passes the
new value on.

## Checks

- Given a rectangle and its clone, when the rectangle's fill, outline
  or shape changes, then the clone's does too, and one Undo takes back
  both; when the clone gets a fill of its own, then later fills of the
  master no longer reach it but outlines still do; Revert to Master
  makes it follow again (`object_clone::tests::a_clone_follows_its_master_until_it_is_edited`).
- Select Master, Select Clones and no clone of a clone
  (`object_clone::tests::select_master_and_clones`).
- A group's clone follows a change inside the group with objects of its
  own (`object_clone::tests::a_group_clone_follows_the_group_with_ids_of_its_own`).
