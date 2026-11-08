# TraceDraw user guide

Welcome to TraceDraw, an open-source vector editor for illustration, page
layout and print. This guide shows you how to get things done in it, one
task at a time. It is written for people who draw, lay out pages and
prepare files for print; you do not need to know anything about how
TraceDraw is built.

## What TraceDraw is

TraceDraw is a vector editor: everything you draw (rectangles, curves,
text, effects) stays an object you can select, move, reshape and recolour
at any time. You can:

- draw shapes and curves, then edit their nodes;
- set headlines and paragraph text in frames, with columns,
  hyphenation and text on a path;
- fill objects with flat colours, gradients, patterns, textures and
  meshes, and give them outlines with dashes and arrowheads;
- add live effects such as blends, contours, envelopes, extrusions,
  drop shadows and lenses, and change them later;
- work on several pages and layers, with rulers, guidelines, grids and
  snapping;
- open and save `.tdraw` files (TraceDraw's own format) and `.cdr`
  files, and import and export many other formats;
- print with separations and printers' marks, and publish PDF and PDF/X
  files.

TraceDraw runs as a native program on Windows, macOS and Linux, and the
same program runs in a web browser with nothing to install.

> TraceDraw is in **alpha**. Most of what a vector editor needs is in
> place, but expect bugs and rough edges. The
> [feature status](../features.md) page lists every capability and how
> far along it is. If a file does not open right, the project welcomes a
> sample in an [issue](https://github.com/tracedraw/tracedraw/issues/new/choose).

## Who this guide is for

- **New users** who want to install TraceDraw, find their way round the
  window and make a first drawing: start with
  [Getting started](getting-started.md).
- **Artists** who want to draw and shape curves, use fills and
  effects: read [Drawing](drawing.md), [Shape editing](shape-editing.md),
  [Colour and fills](color-and-fills.md) and [Effects](effects.md).
- **Layout and print people** who set text, lay out pages and hand files
  to a print shop: read [Text](text.md), [Files](files.md) and
  [Printing](printing.md).
- **Everyone** who wants TraceDraw to fit their habits: read
  [Customizing](customizing.md).

## The pages

| Page | What it covers |
|---|---|
| [Getting started](getting-started.md) | Installing, the Welcome Screen, a new drawing, the parts of the window, interface size and saving |
| [Selecting and transforming](selecting-and-transforming.md) | The Pick tool, marquee and lasso selection, groups, handles, rotating and skewing, snapping, nudging, the Transformations panel and alignment |
| [Drawing](drawing.md) | Rectangles, ellipses, polygons and stars, other shapes, the curve tools, brush strokes, connectors, dimensions and callouts |
| [Shape editing](shape-editing.md) | The Shape tool, nodes and their types, shaping operations, corners and joins, the shape brushes, Crop, Knife, Segment Delete and Eraser |
| [Text](text.md) | Artistic and paragraph text, fonts, the Text panel, frames, text on a path, tables and proofing |
| [Colour and fills](color-and-fills.md) | The Colour panel, palettes, every kind of fill, outlines, spot colours, transparency, eyedroppers and styles |
| [Effects](effects.md) | Blend, Contour, Envelope, Distort, Extrude, Drop Shadow, Block Shadow, Lens, Symmetry, clip frames and copying effects |
| [Files](files.md) | Opening and saving `.tdraw` and `.cdr` files, importing and exporting, auto-backup and recovery, updates |
| [Printing](printing.md) | Printing, preview, imposition, marks, separations, preflight and Export as PDF |
| [Customizing](customizing.md) | Options, workspaces, keyboard shortcuts, languages, scripts and AI helpers |

## Conventions in this guide

- Menu paths are written with `>`: **File > Save As** means the Save As
  item of the File menu.
- Keyboard shortcuts are written as `Ctrl+S`. On macOS, `Ctrl` is the
  Command key.
- A "panel" is a docked window on the right of the main window (the
  menus sometimes call them dockers).
- Measurements are in millimetres unless you choose other units.
- Where a page here gives the short version, it links to the detailed
  behaviour notes in [`docs/behavior/`](../behavior/README.md), which
  list every default, range and modifier key.

## Getting help inside TraceDraw

- The **Hints** panel is open by default on the right. It follows the
  active tool and tells you what the tool does and which keys change
  it. **Help > Hints** opens and closes it. See
  [the Hints panel notes](../behavior/hints.md).
- The **status bar** at the bottom shows a hint for the active tool and
  what is selected.
- Tooltips on toolbox buttons name the tool and its shortcut.
- **Help > About TraceDraw** shows the version, the license and a link
  to the source code.

## Where to go next

Go to [Getting started](getting-started.md) to install TraceDraw and
make your first drawing, or jump straight to the page for the task in
front of you.
