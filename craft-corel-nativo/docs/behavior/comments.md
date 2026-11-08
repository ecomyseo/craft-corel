# Comments docker

Window > Panels > Comments collects review notes on the drawing. They
are saved with the drawing (`.tdraw`) and never printed or exported.

## Adding

- Name: who comments and replies are signed with (kept in the
  settings); empty, the drawing's author (File > Document Info),
  else "Me".
- Add: Pin, Box or Arrow, then on the page: a click places a pin; a drag
  draws a box around something or an arrow pointing at it (a click with
  Box or Arrow places a pin). The new comment is picked and its text
  field takes the keyboard.
- Each comment shows its number in an orange bubble on the page: at the
  pin, the box's top right corner or the arrow's tail.

## The list

- "n open of m", then a card per comment in the order they were added:
  number, author, date and time (local), page, text, replies.
- A click on a card picks the comment and goes to its page. The picked
  card edits its text (recorded when the field is left, one undo step)
  and has a reply field (Enter or Reply adds the reply, signed and
  dated).
- Resolve marks a comment dealt with (Reopen undoes it). Resolved
  comments are hidden in the list and on the page unless "Show
  resolved" is ticked; shown, they are faded.
- Hide marks hides the bubbles, boxes and arrows on the page.
- Delete removes the comment and its replies.
- With the docker shown, a click on a bubble or inside a box on the
  page picks its comment.

## Rules

- Every change is one step of Undo (`SetComments`).
- Ids are kept; a repeated or missing id gets the next free number.
- Deleting a page deletes its comments.
