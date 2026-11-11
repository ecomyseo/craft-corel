# Symbols

A symbol is a reusable object: the drawing keeps its definition once and
each instance draws it with its own position, size and rotation.

## Object > Symbol

| Item | Enabled | What it does |
|---|---|---|
| New Symbol | something selected | The selection becomes the definition of a new symbol (grouped when there are several objects) and is replaced by an instance in its place. |
| Edit Symbol | one symbol instance selected, no symbol being edited | The definition is put on the page where the instance is, as editable objects; the instance is hidden meanwhile and focus mode fades the rest of the drawing. |
| Finish Editing Symbol | a symbol is being edited | The edited objects become the definition again (taken back through the instance's transformation), so every instance of the symbol follows; the copy goes, the instance shows and is selected. One undo step goes back to the old definition. |
| Revert to Objects | a symbol instance selected | Each selected instance becomes a copy of its definition. |
| Insert Symbol | the drawing has symbols | An instance of the last symbol in the middle of the page. |
| Symbol Manager | always | Opens the Symbols docker. |

`symbol_edit::tests::a_symbol_is_edited_in_place_and_every_instance_follows`
checks the place of the copy, the focus, one edit at a time, the new
definition reaching both instances and the undo step.

Linked symbol libraries (Resolve Link, Update from Link, Export Library in
the reference) are not done yet.
