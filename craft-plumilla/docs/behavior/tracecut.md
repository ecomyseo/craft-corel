# TraceCut

TraceCut holds the tools for cutting plotters: preparing cut files,
nesting and measuring. Tools > TraceCut (and Launch > TraceCut on the
standard toolbar) opens it in a window of its own, resizable (880 x 600
at first, 660 x 460 at least). Like the other dialogs it keeps the rest
of the application waiting until it closes, and it works on the objects
selected when it opened.

Options > Global > General, **Show TraceCut in Launch and the Tools
menu** (on at first), hides both entries for those who do not cut.

The geometry is the `tracedraw-tracecut` crate (pure geometry over
page-space outlines in millimetres); the window turns its results into
commands. Contributed by LaMka21 (#170).

## Layout

- **Tool list** (left, 210 wide), grouped: Cut, Nesting, Measure. A
  group with no tools yet is not shown. The tool picked last is picked
  again the next time the window opens (kept in the settings).
- **Right side**, top to bottom: the tool's name and what it does; the
  **Preset** row; the tool's settings; the **preview**; the status line;
  **Apply**, **OK** and **Cancel**.
- **Preview**: the outlines of the selected objects fitted in a white
  box, y up, in dark grey; what the tool found in red (outlines drawn
  thicker, places as rings 6 px wide). With nothing selected the box
  shows "Select the objects to cut, nest or measure" in its middle
  instead.
- **Zoom and pan in the preview**, as on the drawing:
  - the wheel over the preview zooms at the pointer, 1.15 times a
    notch, with or without Ctrl; a pinch (touch screen or touchpad)
    zooms at the fingers;
  - buttons in its top right corner: **Zoom in** and **Zoom out** (1.25
    times about the middle), **Zoom to fit** (the whole result: the
    objects, what was found, the moves), **Zoom to selection** (the
    selected objects as large as they fit) and **Real size (1:1)** (the
    screen's calibrated resolution, Options > Display > Ruler
    calibration, 96 per inch until calibrated);
  - with the pointer over the preview: **+** (or **=**) zooms in and
    **-** out, 1.25 times at the pointer, **0** fits;
  - a drag with the left or middle button pans (Space held changes
    nothing, so Space+drag pans too); two fingers pan on a touch screen;
  - a double click fits again.
  The zoom goes from a tenth of the fit to 2000 points a millimetre (or
  four times the fit when that is more). The outlines are drawn again
  at every zoom, so they stay sharp; zoomed in, objects whose box lies
  outside the preview are skipped. Each tool keeps its own zoom and
  place while the window is open; a change of the selection fits every
  tool again.
- **Status line** (pale yellow): what was found, or "Select the objects
  to work on." with nothing selected, or "Apply shows the result here
  without changing the drawing; OK applies it." before Apply, or that
  the result is out of date.

## Apply, OK, Cancel

- **Apply** works the result out into the preview and the status line.
  It never changes the drawing.
- **OK** (and Enter) applies the result and closes the window. Anything
  that changes the drawing is one undo step. When Apply was not pressed,
  or the drawing, the selection, the tool or a setting changed since,
  OK works the result out again first.
- **Cancel** (Esc, the close button) closes without a change.
- With nothing selected Apply and OK are greyed out; their tips say to
  select the objects to work on first.

## Presets

Every tool has named presets of its settings, kept in
`tracecut_presets.json` in the settings folder (in memory only in the
browser), one list per tool. The list shows "(none)" until one is
picked; picking a preset loads it. The gear button shows a row with the
preset's name, **Save** (keeps the settings under that name, replacing a
preset of the same name, and makes it the current one), **Delete** and
**Defaults** (puts the tool's settings back). A preset saved by an
older version that lacks a setting keeps that setting's default; a
damaged file gives no presets and is logged.

## Cut tools

The Cut group lists, in this order: Plotter Output, Weeding, Bridges,
Cut Order, CutContour, Registration Marks, Find Duplicates and Find
Crossings. A new window opens on Plotter Output.

### Plotter Output

The job is the outlines of the selection (groups opened, a clip frame
by its frame), flattened within 0.02 mm, written as HPGL: the
machine's start string, `VS` speed and `FS` force when not 0, then for
each outline `PU` to its first point and `PD` with up to 64 points per
command, then the end string. Coordinates are whole plotter units, 40 a
millimetre (0.025 mm), counted from the chosen **origin** corner of the
job's box (bottom left at first) into the job.

- **Machine**: the command set. HPGL (`IN;` ... `PU0,0;`), HPGL with pen
  select (`IN;PA;SP1;` ... `PU0,0;SP0;`), HPGL with a command per line.
  Choosing one keeps the knife settings; a profile whose commands differ
  shows as Custom. Machines are named by what they send; a cutter's own
  settings are kept as a preset.
- **Knife offset** (0.25 mm at first, 0 to 2): the holder moves to
  `P + d u` to cut `P` going in direction `u`; where the direction turns
  by more than 15 degrees it draws an arc of radius `d` round the node in
  steps of at most 10 degrees, so the blade turns on the spot.
- **Overcut** (1 mm): closed outlines are cut on past their start.
- **Passes** (1 to 10): each outline is cut again, the whole job over.
- **Speed**, **force**: sent as `VS` and `FS` in the cutter's own units;
  0 sends nothing.
- **Mirror** left to right, **Turn 90°** counter-clockwise.
- **Weeding frame** with its margin (5 mm): a rectangle round the job,
  cut last.
- **Plan the cut order** (on): nearest outline next from the job's
  bottom-left corner, with **Inner outlines first**; off, the stacking
  order. The drawing does not change (Cut Order changes it).
- **Send to**: **File** (OK saves a `.plt` where you say), **Port** (a
  device path such as `/dev/ttyUSB0`, `/dev/cu.usbserial` or `COM3`;
  `COMn` is opened as `\\.\COMn`, so ports past 9 work; nothing is
  created, the device must exist) or **Network** (`host` or
  `host:port`, port 9100 at first, raw TCP, 5 s to connect). Sending
  runs on a thread of its own and the status line says how it went.
  TraceDraw does not set the port's speed: set it to the cutter's once
  with the system's tools (or the cutter's driver). In the browser and
  on Android only File is offered.
- **Test cut**: cuts, at once and with these settings, a square of the
  **test square** side (20 mm at first, 5 to 200) with a triangle inside
  (its base a quarter of the side in from the square's, its apex a
  quarter below the top), the triangle first.

Apply shows the outlines and the moves with the knife up (blue), and
the status line gives the outlines, the length cut and travelled and the
job's size. OK saves or sends; the drawing does not change.

### Weeding

Extra cuts in the waste, added on OK as one group named Weeding of
hairlines (no fill) on the active layer, in one undo step ("Weeding").

- **Weeding lines** (at first): from each object, lines run out of its
  box (**lines per side**, 1 at first, up to 20, evenly spaced) until
  they meet a neighbour or the frame; every object is grown by the
  **gap** (2 mm) and the lines stop there, each end at an object pulled
  back by **shorten ends** (1 mm). Lines shorter than 0.5 mm and lines
  lying on another are dropped.
- **Weeding boxes**: a rectangle round each object grown by the box
  margins (2 mm each way); overlapping boxes are welded into one
  outline.
- **Weeding frame** (on) the margin (5 mm) round everything.

### Bridges

Short uncut stretches in the closed outlines of the selected shapes and
curves (groups opened; text, pictures and objects with live effects
left alone). Each closed outline longer than **only outlines longer
than** (0) gets **per outline** (2) bridges, or **one every** so much
length (at least one), each **bridge length** long (2 mm), spread
evenly by arc length, the first half a spacing from the start; a bridge
moves to the nearest straight segment within a quarter spacing. An
outline the bridges would take more than nine tenths of is left whole.
OK makes each object a curve of the open pieces between the bridges, in
one undo step ("Bridges"); the preview rings the bridges.

### Cut Order

For the selection, layer by layer (locked layers and objects left
alone): the objects in the order with the least travel with the knife
up (nearest first, then 2-opt on up to 2000 objects), with **inner
outlines first** (on) so a part does not move once free; inside shapes
and curves, their subpaths ordered the same way, each closed outline
starting at the node nearest the knife (or the bottom-left one, or as
it is) and running as drawn, all clockwise or all counter-clockwise.
OK reorders the objects in their layers and rewrites the changed shapes
as curves in one undo step ("Cut Order"); the status line gives the
travel before and after.

### CutContour

Marks the selected objects' outlines as cut lines for print and cut: OK
gives them an outline in the spot colour named **CutContour** (added to
the drawing's spot colours once, its stand-in 100 % magenta), a hairline
or the **outline width** given, and with **Remove the fill** no fill, in
one undo step ("CutContour"). Its swatch joins the document palette in
that step when it is not there yet, as a colour applied from the
Properties docker does (unless the document palette's Automatically
Update is off), so the CutContour colour can be picked again from it
(#1927). The name is kept on export: PDF writes a
Separation colour space named CutContour (as for every spot colour), EPS
and PostScript a Separation colour space with the ink's name and its
process stand-in, and SVG, which has no spot colours, writes the colour
and `data-spot-stroke="CutContour"` (`data-spot-fill` for a fill).

### Registration Marks

Filled black marks a cutter's optical sensor finds, added on OK as one
group named Registration marks in one undo step ("Registration
Marks"): **around the selection** (outside each corner of its box,
**distance** 10 mm away) or **inside the page** (inside each corner,
the distance in from both edges). **Mark**: square or circle of
**size** 5 mm, or an L whose arms (**arm length** 20 mm, **arm width**
1 mm) run along the corner. On a side longer than **extra marks every**
(400 mm, 0 for none) marks also go between the corners, evenly (a square
for the L). These are generic marks; no maker's layout is built in.

### Find Duplicates

Objects lying on top of another would be cut twice. Two objects are the
same when every point of either outline lies within the **tolerance**
(0.05 mm at first, 0.001 to 10 mm) of the other's and their boxes agree
within it; the outlines are flattened within a quarter of the tolerance
and divided so that points are at most the tolerance apart. Of equal
objects the lowest in the stacking order is the original.

**On OK**: **Delete them** (at first) deletes the repeats in one undo
step ("Delete Duplicates"), locked objects left alone; **Select them**
selects them and changes nothing.

### Find Crossings

An outline crossing itself cuts badly, and objects crossing each other
usually need a weld before cutting. The outlines are flattened within a
quarter of the **tolerance** (0.05 mm at first); neighbouring segments
of one subpath and segments running along each other do not count, and
crossings closer than the tolerance for the same objects are one. With
**Also between objects** (off at first) crossings between different
objects count too. OK selects the objects that have crossings.

## Nesting tools

Both place the selected objects (locked ones left alone) on the
material and move them there on OK, in one undo step ("Shape Nesting",
"Rectangle Nesting"); with **Draw the sheets** (on) the sheets, or the
length of roll used, are added as a group of hairline rectangles named
Sheets. An object that fits nowhere stays where it is and the status
line says how many. The preview shows the objects where they will go,
and the sheets.

**Material**: **sheets** of a **width** and **height** (1220 x 2440 mm
at first), laid side by side from the page's origin to the right, 20 mm
apart, as many as it takes; or a **roll** of a **width** with no end,
its length running to the right or up the page. **Margin** (0) is kept
free along the edges, **spacing** (2 mm) between objects, and the
layout starts from the **start corner** (bottom left).

### Shape Nesting

Objects by their true outlines: each becomes a region (its closed
outlines flattened, open curves by their boxes; holes filled unless
**use holes** is on, 0.5 mm **accuracy** at first), grown by half the
spacing and drawn on a grid. Objects go largest first, each to the
lowest, then leftmost, place where it fits on the first sheet that has
one, slid down and left in exact geometry until it touches, in each
allowed **turn** (none, 0 and 180 degrees, every 90 at first, every 45);
the turn whose top ends lowest wins. Then, for up to the **search time**
(2 s at first) and 30 rounds, two objects of the order swap and the
layout is kept when it is no worse: fewer left out, fewer sheets, less
length on the last sheet, a lower sum of tops. The search is seeded, so
the same selection gives the same layout. Apply waits for the search.

### Rectangle Nesting

Objects by their bounding boxes, largest area first, each in the free
rectangle that leaves the shortest side over (on a roll, the lowest
place, then the leftmost); with **boxes may turn 90°** (on) a box may
turn when that fits better. Quick, for parts that are nearly
rectangular.

## Measure tools

They change nothing. Their results show at once in a box beside the
preview (230 wide), the first line also on the status line, without
Apply, and follow the drawing, the selection and the settings as they
change (worked out again only then). OK closes the window. Lengths and
areas are in the document's units.

### Perimeter and Area

The length of every outline (curves by arc length, the closing segment
of a closed outline included), the area by the even-odd rule within
each object (a closed outline inside an odd number of others is a hole:
the letter O counts its ring), the outer and hole areas, the box of the
selection and its area, and the counts of objects, closed and open
outlines and nodes. Objects are measured on their own: one lying on
another adds its own area.

### Cutting Time

- cutting = outline length / **cutting speed** (300 mm/s) x
  **slow-down factor** (1.2)
- moves = travel with the knife up / **travel speed** (600 mm/s) x the
  factor, the head going to the **nearest outline next** (on; else in
  stacking order) from 0,0, a closed outline entered at its nearest node
- lifts = outlines x **lift and lower** (0.2 s)
- total = (cutting + moves + lifts) x **passes** (1)

### Cost

Each object is a block, its box grown on every side by the **reserve
per side** (5 mm). The material is billed **by the roll** (at first:
the running metres of a roll of the **roll width**, 630 mm, the blocks
laid on it) or **by area** (square metres of the blocks), at the
**material price**; then, when not 0, **cutting per metre** of outline
(0.5), **weeding per m²** of the blocks (12), a price **per object**
and **machine time per hour** (the cutting time from the Cutting Time
settings). Everything is multiplied by the **copies**. Prices carry no
currency; the **currency** typed is shown after the amounts.
