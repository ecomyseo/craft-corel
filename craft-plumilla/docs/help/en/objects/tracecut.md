# TraceCut

[[menu.tools > menu.tools.tracecut]] opens TraceCut, the tools for cutting
plotters. It is also on the standard toolbar's Launch button. Select the
objects to work on before opening it.

The tools are listed on the left by group. On the right are the chosen
tool's settings, a preview of the selection and what the tool finds, and
the buttons:

- **Apply** shows the result in the preview without changing the drawing;
- **OK** applies it and closes the window; anything it changes is undone in
  one step;
- **Cancel** closes the window without a change.

With nothing selected the preview asks you to select the objects to cut,
nest or measure, and Apply and OK are greyed out.

The Measure tools (Perimeter and Area, Cutting Time, Cost) show their
results beside the preview at once, and update as you change the selection
or the settings; there is no need to press Apply.

To hide TraceCut, clear [[options.show_tracecut]] in Options, General.

## Zoom in the preview {#preview-zoom}

- Turn the **mouse wheel** over the preview to zoom at the pointer (with or
  without Ctrl), or **pinch** on a touch screen or touchpad.
- **Drag** with the left or middle button (or hold Space and drag) to move
  the view.
- The buttons in the preview's corner zoom in, zoom out, fit the whole
  result, zoom to the selected objects and show them at real size (1:1).
- With the pointer over the preview, **+** and **-** zoom and **0** fits.
- **Double-click** the preview to fit the whole result again.

Each tool keeps its zoom while the window is open; selecting other objects
fits the preview again.

## Presets {#presets}

Each tool keeps its settings under names you give. Pick a preset from the
list to load it. The gear button shows the row to **Save** the settings
under a name, **Delete** a preset, or go back to the **Defaults**.

## Plotter Output {#plotter-output}

Cuts the selection on a cutting plotter. Choose the **machine** (the commands
your cutter understands), the **knife offset** of your blade (often 0.25 mm),
the **overcut** that closes each outline, the **passes** for thick material,
and where the job goes:

- **File** saves a .plt file;
- **Port** sends it to a serial or USB-serial port, such as /dev/ttyUSB0 or
  COM3 (set the port's speed to the cutter's first, with your system's tools);
- **Network** sends it to the cutter's address, port 9100 unless you give
  another.

**Plan the cut order** cuts the nearest outline next and the inner outlines
first. **Test cut** cuts a small square with a triangle inside, to check the
knife before the real job. Save your cutter's settings as a preset.

## Weeding {#weeding}

Adds weeding lines from each object to its neighbours, or a box around each
object, and a weeding frame, so the waste peels off in small pieces.

## Bridges {#bridges}

Leaves short uncut stretches in closed outlines, so the parts stay in the
sheet until you break them out.

## Cut Order {#cut-order}

Reorders the objects and their outlines for the shortest travel of the knife,
inner outlines first, and sets where each outline starts and which way it
runs.

## CutContour {#cutcontour}

Gives the selected objects an outline in the spot color CutContour, the name
print and cut workflows look for, and can remove their fill. The name is kept
when you export to PDF, EPS or SVG.

## Registration Marks {#registration-marks}

Adds the marks a cutter's sensor reads for print and cut, around the selection
or inside the page: squares, circles or L-shaped corners, with more along long
sides.

## Find Duplicates {#find-duplicates}

Objects lying exactly on top of another one would be cut twice. Set the
**tolerance**, how far apart two outlines may be and still be the same, and
choose whether OK deletes the repeats or selects them. The lowest object
of each set stays.

## Find Crossings {#find-crossings}

Outlines that cross themselves cut badly, and objects that cross each other
usually need a weld first. TraceCut finds the crossings, shows them as red
rings in the preview, and OK selects the objects that have them. Tick
**Also between objects** to find crossings between different objects too.

## Nesting {#nesting}

**Shape Nesting** places the selected objects on sheets or a roll by their
true outlines, turning them and fitting small ones into holes, to waste as
little material as possible. **Rectangle Nesting** does the same by their
boxes, faster. Set the sheet or roll size, the margin and the spacing between
objects; OK moves the objects onto the material and can draw the sheets.

## Measuring {#measuring}

**Perimeter and Area** gives the outline length and the area of the
selection, holes taken away. **Cutting Time** works out how long the cutter
takes, from its speeds. **Cost** prices the job: the material by area or by
the length of roll, cutting, weeding, pieces and machine time. These tools
change nothing in the drawing.
