# Printing

TraceDraw prints by building a print-ready PDF and handing it to your
system's PDF viewer, where you choose the printer. This page covers the
Print dialog, imposition and marks, separations, checking a drawing
before it goes out, and publishing PDF and PDF/X files.

## Print a drawing

1. Choose **File > Print** (`Ctrl+P`) or the **Print** button on the
   standard toolbar.
2. Set the pages and options (below).
3. Press **Print**. The PDF opens in your system's viewer; print from
   there.

In the browser, the PDF is downloaded instead; open it and print it from
the browser's viewer.

### Pages and copies

- **Copies**: handed on to the viewer.
- **All pages**: off prints the current page only.
- **Fit to page**: off lets you set a scale in percent.

### Layout

- **Imposition**:
  - **As in document**: one page per sheet.
  - **Several pages per sheet**: a grid of columns and rows (2 x 2 to
    start with).
  - **Booklet**: two pages per sheet side, in the order a folded,
    stapled booklet needs. Blank pages are added at the back to make a
    multiple of four.
  - **Step and repeat**: each page repeated over the grid, for cards and
    labels.
- **Sheet**: **Fit to pages**, or a paper size (A5, A4, A3, SRA3, A2,
  Letter, Legal, Tabloid), portrait or **Landscape**. Pages shrink to fit
  a fixed sheet but never grow.
- **Gap between pages** (5 mm) for the grid layouts.

A sketch shows the first sheet with its pages numbered, and how many
sheets there will be.

Hidden and non-printing layers are left out. Master layers print on
every page they show on, and a page background prints under its page.

### Prepress options

- **Bleed**: prints the document's bleed round every page.
- **Crop marks**: thin lines at every cut, outside the pages. A booklet
  also gets a dashed fold mark.
- **Registration marks**: a target on each side.
- **Color bars** and a **Densitometer scale**.
- **Page information**: the drawing's name, the pages on the sheet, the
  sheet number and the date, in small type under the pages.
- **Mirror**: flips the whole sheet, for emulsion-down film.
- **Invert**: inverts every colour, for negatives.
- **Print separations**: see below.

With any mark on, a 12 mm margin is added round the pages.

### Print preview

**Print Preview** in the Print dialog shows what will print before you
send it.

To have the Print dialog start with your usual settings, or remember the
last print, use **Tools > Options > Global > Printing**. See
[the printing notes](../behavior/printing.md).

## Separations

**Print separations** prints one grey page per ink for every sheet: C,
M, Y and K, then one page for each [spot colour](color-and-fills.md#spot-colours)
ink. A spot colour prints on its own plate only.

To check how inks overprint before printing, use the **Simulate
Overprints** view. You can set fills, outlines and bitmaps to overprint
on each object. **View > Proof Colors** shows the page as CMYK can print
it.

## Check before you print: Preflight

Preflight looks for what tends to go wrong in print. Open it with
**File > Preflight...**; a summary also shows in the Print dialog
("Warnings: n Notes: n" or "No issues found"), with **Show Issues...** to
open the list.

It checks for:

- empty text, and text that does not fit its frame;
- fonts that are not installed;
- very small text, and small text in rich black;
- RGB colours in a CMYK drawing;
- objects off the page;
- bitmaps of too low (under 200 dpi) or very high (over 1200 dpi)
  resolution;
- very thin outlines (under 0.1 mm);
- white set to overprint;
- empty pages;
- objects on layers that will not print.

Click an issue to go to its page and select the object. Under the list,
TraceDraw says what to do about the kind of issue you picked.
**Choose Checks...** turns checks on and off and sets the thresholds;
**Check Again** runs them again. See [the preflight notes](../behavior/preflight.md).

## Export as PDF

For a PDF to send to a print shop or to share, use **File > Publish to
PDF...** (also on the standard toolbar).

### General

- **Pages**: All pages, Current page, or Selected objects (a page the
  size of the selection).
- **Compatibility**: PDF 1.4, PDF/X-1a:2003, PDF/X-3:2003 or PDF/X-4.
- **Bleed**: 0 to 50 mm, starting at the drawing's bleed.
- For PDF/X: the **Output condition** (FOGRA39 to start with) and, for
  X-1a and X-3, the resolution transparency is flattened at (300 dpi).
  The output intent uses the CMYK profile set in Color Management, or
  else the CMYK profile the drawing carries.

### Document

- **Title**, **Author**, **Subject** and **Keywords**, filled in from
  the drawing's properties.
- **A bookmark for each page** (on), named after the page, and whether
  to show the bookmarks when the file opens.

### Security

- **Require a password to open the file**.
- **Restrict printing, copying and changes** with a second password,
  then choose what is allowed: printing (not allowed, low resolution
  only, allowed), copying, changing the content, comments, filling in
  forms, assembling pages and text access for screen readers.

The two passwords must differ. PDF/X files cannot be protected, because
the standards forbid it; the Security page says so when a PDF/X level is
chosen. TraceDraw never keeps your passwords; your other choices are
remembered for next time.

Press **OK** and choose where to save (in the browser the PDF is
downloaded). See [the Export as PDF notes](../behavior/publish-to-pdf.md).

You can also choose PDF in **File > Export**, with the same PDF/X levels
and bleed; Export as PDF adds the bookmarks and protection.

## Data merge

**Data Merge** fills a design from a CSV file: create or load the data,
edit it, then perform the merge.

## Where next

- Fine-tune colours for print: [Colour and fills](color-and-fills.md).
- Make the Print dialog start the way you like: [Customizing](customizing.md).
