# The Shelf

The [[docker.shelf]] panel keeps objects and colors apart from any drawing,
so you can place them in this drawing or another one, today or after
TraceDraw restarts: logos, stamps, frames, a set of colors. Open it from
[[menu.window > menu.window.dockers]], or right-click a selection and choose
[[menu.object.add_to_shelf]].

![The Shelf panel](../../img/objects-shelf.png)

## Adding to the Shelf {#add}

- Select objects and click [[shelf.add_selection]]. The item is named after
  the object (its name, or what it is), or "n objects". Symbols become plain
  objects, and the object styles they use go along.
- Click [[shelf.add_colors]] to keep the colors of the drawing's document
  palette and its color styles as one item.

The Shelf holds up to 500 items.

## Placing an item {#place}

- Drag an item onto the drawing to place a copy under the pointer.
- Double-click an item, or choose it and click [[shelf.place]], to place a
  copy in the middle of the view.

The copies go on the active layer and are selected; one undo removes them. A
color item adds its colors to the document palette and its color styles to
the drawing.

## Arranging items {#arrange}

- Click an item to choose it. Right-click it for [[shelf.place]],
  [[shelf.rename]] and [[shelf.remove]].
- Drag an item onto another to move it to that place.

### Removing an item {#remove}

Choose an item and click [[shelf.remove]]. Removing cannot be undone, so
TraceDraw asks first: click [[shelf.remove]] to confirm or [[dialog.cancel]]
to keep it. Tick [[shelf.dont_ask]] to remove items without asking; to be
asked again, turn on [[options.warn_shelf_remove]] on the [[options.warnings]]
page of the global options.

![The question before an item is removed](../../img/objects-shelf-remove.png)

## Where the Shelf is kept {#storage}

On the desktop, the Shelf is a file in your preferences folder. In the
browser, it is kept in that browser's storage, which holds only a few
megabytes, so large bitmaps may not be kept.
