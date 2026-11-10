# Files

This page covers opening and saving drawings, bringing in work from
other formats, sending it out again, and how TraceDraw protects unsaved
work.

## Open a drawing

To open a drawing: **File > Open** (`Ctrl+O`), the **Open** button on
the standard toolbar (its arrow lists recent drawings), or a recent
drawing on the Welcome Screen.

Each drawing opens in a tab of its own. You can have several open at
once; switch between them with the tabs or the **Window** menu.

TraceDraw opens:

- `.tdraw`, its own format;
- `.cdr` files, versions 7 to 22 (see [below](#cdr-files));
- SVG and SVGZ;
- PDF and AI (PDF-compatible ones and older PostScript-based ones),
  asking for the password when a protected PDF needs one;
- EPS, PostScript and PRN print files;
- DXF, EMF, WMF and HPGL (`.plt`);
- PSD and PSB, with layers as bitmaps;
- TXT, RTF and DOCX documents, as paragraph text;
- CMX (partly; see [the CMX notes](../behavior/cmx-import.md)).

Bitmaps (PNG, JPEG, BMP, GIF, TIFF and WebP) are not opened as
drawings: bring them in with **File > Import**, which places them at
the resolution their file records (96 dpi when it records none).

Opening never fails outright: if part of a file cannot be read,
TraceDraw opens the rest and tells you how many objects were left out.
For a `.cdr` file, that message has a **Report** button that starts an
issue so the reader can be improved. You can turn these messages off in
**Tools > Options > Global > Warnings**.

## Import

To place the contents of another file into the drawing you have open:
**File > Import** (`Ctrl+I`). Import takes the same formats as Open, and
bitmaps as well.

Imported objects land where the file has them. To have them centred in
the view instead: **Tools > Options > Global > File Formats**.

For bitmaps, see also [the bitmap modes notes](../behavior/bitmap-modes.md)
and [Trace Bitmap](../behavior/bitmap-tracing.md), which turns an image
into editable curves.

## Save

- **File > Save** (`Ctrl+S`) writes the drawing back to its file.
- **File > Save As** (`Ctrl+Shift+S`) always asks.

A new drawing, or one opened from a format TraceDraw does not save in
(including `.cdr`, which needs a version), opens the **Save Drawing**
dialog on the first save:

1. Choose a folder. Use **Up**, type a path and press **Enter**, use the
   shortcuts down the left, or press **New folder**.
2. Type a **File name**.
3. Choose **Save as type**:
   - **TraceDraw** (`*.tdraw`): keeps everything. Use it for your
     working files.
   - **CDR files** (`*.cdr`), with a **Version** (see below).
4. Optionally fill in **Title**, **Subject** and **Rating**. These are
   also in **File > Document Info**.
5. Press **Save**. If the file exists, TraceDraw asks before replacing
   it.

In the browser, Save downloads the file to your downloads folder; the
dialog keeps the name, type, version and properties.

To keep a copy of the old file each time you save, leave **Back up
original file before saving** on in **Tools > Options > TraceDraw >
Save**. It writes `backup_of_<name>` next to the file.

See [the saving notes](../behavior/files.md).

## `.cdr` files

TraceDraw reads and writes `.cdr` files independently:

- **Reading**: versions 7 to 22 open with their pages, layers, shapes,
  curves, text, groups, bitmaps, fills, outlines and more. Some content
  is not read yet (for example splines and most lenses and effects);
  [the feature status](../features.md) lists what.
- **Writing**: any version from 7 to 22. A program opens `.cdr` files
  of its own version and older, so pick the version of the program that
  will open the file, or an older one.
- A drawing opened from a `.cdr` file is saved in its own version unless
  you pick another, and **Save** writes that version again without
  asking.
- New drawings start with the version in **Tools > Options > Global >
  File Formats** (12 to start with, which most programs open). Every
  `.cdr` save updates it.

If a `.cdr` file does not open right and you may share it, please
[send it in an issue](https://github.com/tracedraw/tracedraw/issues/new?template=cdr_file.yml):
sample files are what improves the reader most.

## Export

To write the drawing in another format: **File > Export** (`Ctrl+E`).
The Export dialog shows a preview of the result on the left, and on the
right:

1. **Format**: SVG, PDF, EPS, AI, DXF, HTML, EMF, WMF, PLT, PostScript,
   PRN, PSD, or a bitmap (PNG, JPEG, WebP, GIF, BMP, TIFF).
2. The format's settings:
   - bitmaps: the **resolution** (36 to 2400 dpi), and for PNG, WebP and
     GIF a **transparent background**; JPEG and WebP a **quality**;
   - PDF: the standard (PDF 1.4, PDF/X-1a, PDF/X-3, PDF/X-4), bleed and
     the PDF/X output condition.
3. **Selected only** and **All pages**.
4. Check the size in pixels and the **estimated file size**, then press
   **OK** and choose where to save.

**File > Export Presets > Web** and **Documents** start the same dialog with
presets of their own. To have Export remember your last choices, use
**Tools > Options > Global > File Formats**. See
[the export notes](../behavior/export.md).

Links you give objects in the **Links and Bookmarks** panel become
clickable areas in PDF and links in SVG and HTML. See
[the links notes](../behavior/links.md).

**Send To** writes a PDF to the Desktop, Documents or a mail folder (in
the browser, it downloads the PDF).

## Templates

Save a drawing as a template, then start new drawings from it; the
Welcome Screen lists templates too.

## Auto-backup and recovery

On the desktop, TraceDraw saves a backup of every drawing with unsaved
changes every 20 minutes. If TraceDraw closes normally, those backups
are removed.

If it did not (a crash, a power cut), the next start offers the backups:

- **Recover** opens each one as an unsaved drawing. Save it to give it a
  file again.
- **Discard** leaves the backup files in the auto-backup folder and does
  not ask again.

To change how often it saves, or where (a `TraceDraw` folder in your
temporary folder to start with): **Tools > Options > TraceDraw > Save**.
Auto-backup is not available in the browser. See
[the recovery notes](../behavior/recovery.md).

## Close drawings

- **File > Close** (`Ctrl+F4`), the tab's close button, or a middle
  click on the tab closes a drawing, asking to save unsaved changes.
- **Window > Close All** closes every drawing, asking about each one with
  changes.
- **File > Exit** (`Alt+F4`) does the same and quits.

**Window > New Window** opens a second window on the same drawing, with
its own view, page and selection. **Window > Cascade**, **Tile
Horizontally** and **Tile Vertically** show several drawings side by
side. See [the drawing window notes](../behavior/drawing-window.md).

## Updates

On the desktop, **Help > Updates** checks for a newer release. If there
is one, it offers:

- **Download**: the file for your system; your system's installer does
  the installing;
- **Release notes**;
- **Skip this version**;
- **Close**.

The first time it starts, TraceDraw asks whether to check for updates
(**Always check** or **Never check**) and connects to nothing on its own
before you answer. With Always check, it checks once a day when it starts
and shows only a newer release you did not skip. To change your answer: **Tools > Options > Global >
General**, clear **Check for updates at start**. The browser version is
always the latest; there, Help > Updates opens the releases page. See
[the update notes](../behavior/updates.md).

## Where next

- Print or publish a PDF: [Printing](printing.md).
- Convert files from the command line: [Customizing](customizing.md#scripts-and-automation).
