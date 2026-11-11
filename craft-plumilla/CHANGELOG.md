# Changes

Versions follow the tags of this repository; every push to `main` also
publishes a nightly build. [`docs/features.md`](docs/features.md) has the
full, row-by-row status.

## 0.42.0 (2026-11-11)

- 22 new bitmap effects where the reference's Effects menu has them: Zigzag; Shadows/Midtones/Highlights, Tint/Temperature, Grayscale and Vibrance; Dabble; Tune Blur and Feather; Lens Flare, Lighting Effects and Spot Filter; Tune Sharpness; Band Pass and User Defined; Shear; Tune Noise and 3D Stereo Noise; Brick Wall, Bubbles, Canvas, Plaster Wall, Screen Door and Underpainting; Threshold.
- New menu rows: PowerClip > Create Empty Clip Frame and the four ways to fit the contents; Align > Center to Page Horizontally/Vertically and Align with Pixel Grid; Hide > Show Objects; Insert Formatting Code > Quarter Em Space, Tab and Nonbreaking Hyphen; Paragraph Text Frame > Create Empty Text Frame; Color Palettes > Add from Document and Close All Palettes. Show All is greyed when nothing is hidden.
- The submenus follow the reference's order and separators too: Export For, Insert Page Number, Insert, PowerClip, Copy Effect, Align and Distribute, Order, Adjust, Insert Formatting Code, Paragraph Text Frame, Workspace and Color Palettes; Local Equalization moves to Contour, and Object > Perspective holds Add Perspective and Draw in Perspective.
- Edit > Bring into Focus and Exit Focus Mode, and Bitmaps > Image Adjustment Lab and Remove JPEG Artifacts (a new effect that takes away the 8 by 8 blocks of a strongly compressed JPEG and keeps the real edges), where the reference's menus have them.
- The order and separators of the menus follow the reference's menu configuration: Edit keeps Cut to Delete together, Object lists Create and Insert before AI Generate, and Bitmaps has the trace commands (Quick Trace, Repeat Last Trace, Last Trace Settings, Centerline and Outline Trace) in the menu itself.
- File > Acquire Image > TWAIN works where the TWAIN source manager is installed: the scanner's own dialog, every picture it gives imported as with WIA; with several sources the Select Source dialog chooses.

## 0.41.1 (2026-11-10)

- The editor is now called Craft Plumilla (it was Craft Corel), so that its name does not use a trademark of another company. Programs, installer, settings folder and links take the new name; preferences of a copy with the old name carry over the first time.
- With no drawing open (the Welcome Screen's tab closed), a right-click on the workspace offers New, New From Template, Open, View, New Window and About.
- Create New Symmetry works with nothing selected: an empty symmetry whose mirror line passes through the click (or the middle of the view), into which what is drawn next goes and is mirrored; finished while still empty, it goes.
- Menu rows that looked enabled and did nothing in some cases are greyed there: Ungroup and Ungroup All without a group, Clear Effect and Flatten without an effect, the clip frame rows without a clip frame. Window > Dockers also closes a docker shown, and Window > Toolbars > Menu Bar hides and shows the menu bar.

## Unreleased

- Saving .cdr (version 16 and later) with Embed fonts: a drawing whose text uses only the bold face of a font (bold "Khand" text, or regular Khand text drawn by the only Khand file installed, the bold one) no longer lists a regular face of that font too. The file carried a plain record of the regular face and named it in its style table, two references to a face it neither used nor embedded, so the program that reads `.cdr` files could ask for "Khand (Normal)" on a computer where Khand is not installed, although the bold face is in the file. Files that program saves hold a record per face used and name the first of them in the style table, and TraceDraw now writes the same; regular text keeps its record, and files before version 16 are unchanged (#1733, #1009, #1055).

- File > Export with "Do not show filter dialog" on offers every format in the file dialog, the one set in Options > File Formats first, and exports in the format of the extension chosen; it offered that one format only.

- Weld, Trim and the other shaping operations keep a mesh fill: the result showed no colour when the object with the mesh had been moved or resized.

- Objects docker: an object's row dragged up or down the list moves the object in the stacking order, in front of the row it is let go on (its upper half) or behind it (its lower half), with a line showing where it goes; before, only the Order commands and their shortcuts changed the order.

- Holding N opens the Navigator, as holding its button in the corner between the scrollbars does: move the pointer over the picture to go there.

- Twirl and Attract and Repel start at the press and keep working with the button held still; they waited for the pointer to move before doing anything. Roughen adds its teeth once per spacing along the stroke instead of at every frame, so going over a stretch no longer piles nodes into a tangle.

- Area Fill fills the area an open line closes off when its ends lie exactly on another outline, as Virtual Segment Delete leaves them: on a large circle the ends sat a hair outside the outline as it is measured, and the area was taken for open (#2076).

- Sliders in dockers and dialogs take the whole width of their row, as Object opacity does, also when a box or a unit list shares the row (the outline width, tolerances, the font sample size); before they kept a short fixed length there.

- File managers show a .tdraw drawing as a small picture of its first page. On Windows the installer adds a thumbnail provider for the Explorer; on Linux the .deb installs the drawing type and a thumbnailer, and `tracedraw-cli thumbnail <file> <out.png> [size]` makes the picture for any file TraceDraw opens.

- Ellipse arcs stay unfilled while Fill open curves is off; uniform transparency set on the fill alone or on the outline alone fades only that part; the dimension tools snap to the ends, nodes and edges of objects even with Snap To Objects off.

- Page tabs: a tab dragged along the row and let go over another tab moves its page there, so pages can be put in another order without the Page Sorter (#2064).

- Polyline tool: a drag draws a freehand line, as a click places a straight segment, so one curve can mix straight and freehand parts (#2074).

- Multipage View: a press on an object that lies off its page, on the desktop, or over the current page while it belongs to another page, makes that object's page current and picks the object in the same click; such objects could not be selected (#2055).

- Guidelines: a click on a guideline of the master page (every page shows them; opening a `.cdr` file puts its guidelines there) picks it, moves it or deletes it, and a right click opens its menu, also while the page's own guidelines were the ones being edited: the click switches to the set the guideline is in. Such a guideline could be seen but not touched (#2029).

- Ctrl+Tab brings the next document tab to the front and Ctrl+Shift+Tab the previous one, round the end, so open drawings can be switched from the keyboard (#2068, #2058).

- Symbols: Create New Symbol keeps the selection handles around the new symbol. They jumped to its lower left corner as a 10 x 10 mm box, and a symbol inserted from the Symbols docker was picked the same way (#1939, #1980).

- Text: Shift+F3 opens Text > Change Case while a text is being typed as well; the shortcut did nothing until the text was selected with the Pick tool (#2059).

- Page tabs: the current page's tab is highlighted in the same colour as the active document tab above the rulers, so the page being worked on stands out (#2049).

- Print and Print Preview: the printer printed to last is offered again for the rest of the session, instead of the system's default every time; a new run starts on the default again unless Options > Printing remembers the last choices (#2056).

- Touch screens: a finger dragged along a toolbar, the property bar, the toolbox, the colour palette or a column of docker tabs that is longer than the screen slides it with the finger, so the hidden buttons, tools, options and colours of a phone's narrow screen can be reached; a tap on a button still works, a finger that slid the bar presses nothing when it lifts, and a finger dragged off the palette onto the drawing still drags the colour. The mouse keeps its drags on the bars. A docker's contents longer than the docker scroll under a finger too (#1997).

- Update card: when a newer release is known, a card opens in the middle of the window at start with the icon, an UPDATE badge, the two versions and Update now, so TraceDraw can be updated from there without opening Help > Updates; it shows the download and then offers Restart now, What's new opens the release notes, and Dismiss closes it until the next start (which brings it back until the update is installed or skipped).
- Help > Updates: Install now works for every kind of copy that has a release file, not only for a Linux AppImage: the Windows installer and portable zip, the macOS app, the Linux tar.gz and the Linux `.deb`, so a new version no longer needs the old one to be uninstalled first (the `.deb` asks for the administrator's password through the system's dialog; the portable zip keeps its settings folder). The download is checked against the release's digest; on Windows TraceDraw then closes (asking about unsaved drawings), the installer runs over the existing copy with your settings and file associations kept, and the new version starts by itself; on a Mac the app is replaced in place. After an in-place install a Restart now button opens the new version. A failed check leaves the installed copy untouched and offers Download.
- Linked text content: Text > Link Text Content (also on the right-click menu) links the characters of two texts or more, so whatever is typed in any of them appears in the others at once, both ways, while each keeps its own font, size, colours and effects. Change the word of a 3D blend, an extrusion or a shadow text once. The Blend panel can link the two texts of a blend; a linked text shows the others of its link with dashed outlines and a chain mark, Undo takes a change back in all of them in one step, copies stay linked, and Unlink Text Content lets a text go its own way. This is not the frame flow of Paragraph Text Frame > Link; `.tdraw` files keep the link, `.cdr` files get plain texts (#1568, #1364, #1194).

- File > Import of a PDF (or PDF-compatible `.ai`) file of several pages asks which pages to bring in and where, in a new Import PDF Pages dialog: All or a list such as "1-3, 5", with each page's size and orientation shown and ticked. As new pages after the current page (the default), each chosen page becomes a page of the drawing with its own size and orientation, in order, its spot inks joining the drawing's; On the current page puts each page's objects on the active layer as a group. The whole import is one undo step, and it follows the text question and the password of a protected file as before, on the desktop, in the browser and on Android. This merges PDFs into one drawing; before, Import took only the first page. A file of one page comes in as before, without asking.

- Facing pages: Page > Facing Pages (and Document Options > Layout) shows the pages side by side in spreads while drawing, page 1 alone on the right and then 2-3, 4-5..., or 1-2, 3-4... with Page > Facing Pages Start On > Left Side. Objects can be drawn across the spine and show on both pages; a press, the page tabs or the navigator make the facing page current without moving the view; guidelines, snapping, the bleed (around the whole spread) and the Multipage View (spreads two a row) follow. Adding or removing pages reflows the spreads. Export has a Spreads option that writes each spread as one page; pages exported or printed alone carry what reaches over the spine from the facing page. The setting is saved in `.tdraw` files; `.cdr` files do not carry it (#1972, #1660).

- Tools > TraceCut... has a picture of its own in the menus: a dashed cut line with the knife on it.

- TraceCut: the preview zooms and pans as the drawing does. The wheel zooms at the pointer (Ctrl+wheel too, and a pinch on touch screens and touchpads), a drag with the left or middle button pans, a double click fits again, and buttons in its corner zoom in and out, fit the whole result, zoom to the selected objects and show real size (1:1); + and - zoom and 0 fits with the pointer over it. The outlines are drawn again at each zoom, so they stay sharp, and what is outside the box is skipped. Each tool keeps its zoom while the window is open. With nothing selected the preview says "Select the objects to cut, nest or measure" instead of showing an empty box, and Apply and OK, greyed out, say why. The Measure tools show perimeter, area, cutting time and cost beside the preview at once and as the selection or settings change, without Apply.

- Windows with two graphics cards: a start whose window never shows now goes on with OpenGL by itself. The start counted as working once the window was made, before the graphics card had drawn anything, so a driver that hung on the first frame (Vulkan on a GeForce GT 750M beside an Intel HD 4600) left the window hidden for good and the next start tried the same card again. A start now works only once its first frame is on the screen: after 20 seconds without one TraceDraw starts again with OpenGL, a start after one closed by hand goes on with OpenGL too, and the status bar says so once, with a click to Options > General to choose the graphics card again (#1999).

- Repeat Copies: Fill page tiles the selection over the page, or an area of it, as many copies as fit: margins (one for all sides or each side apart), a gap across and down, every second row shifted if asked, the copies as they stand, turned 90 degrees or whichever fits more, with the count of each way shown ("4 x 6 = 24 copies") and trim marks at every cut line. The places are outlined while the panel is worked in, and Apply is one undo step (#1255).

- Standard toolbar: the Launch list no longer holds Color Management. Launch is for the tools that come with TraceDraw (Font Manager, TraceCut, the Welcome Screen); Color Management is a settings dialog and stays in the Tools menu.

- Print and Print Preview: the gap between pages is set across and down apart, for several pages per sheet and repeat on the sheet (#1795).

- Linked paragraph text frames: deleting a frame of a chain (Delete, Cut, a group holding it) removes only the frame. Its text stays with the chain and flows on into the frames left, the frame after it showing what it showed and the last frame taking the rest with the overflow mark when it does not fit; a chain of two left with one frame becomes an ordinary paragraph frame holding all the text. One Undo brings the frame and the flow back. The text was deleted with the frame and the chain broke in two (#1986, item 2).

- Print and Print Preview: several pages per sheet can run down the columns, either on each sheet (1 2 3 down the first column, 4 5 6 down the second) or through all sheets, so the columns cut from the stack run in one series, for serial numbers on ID cards (#1658).

- Shape tool: Edit > Delete removes the chosen nodes, as the Delete key and the right-click menu do, and the curve stays; it deleted the whole object while nodes were chosen. With no node chosen, Edit > Delete and the right-click menu's Delete remove the selected object as the Delete key does (the menu row was greyed then). One undo step brings the nodes back (#1982).

- Print and Print Preview: the layout (imposition, sheet, gaps, margins, booklet settings, bleed and marks) can be kept as a named preset and chosen again, and saved to a `.tdlayout` file and loaded from one (#1796).

## 0.41.0 (2026-11-06)

- Text in older fonts made for the symbol encoding (Tamil fonts among them): a letter the font keeps at codes 0x80 to 0x9F now draws when it is typed on Windows, which delivers it as the character code page 1252 has there ("„" for 0x84), and when pasted text holds it at U+F080 to U+F09F. Such letters went to another font and showed as a box or not at all (#1937).

- Master guidelines: guidelines can be set once for all pages. The Objects docker shows "Guides (all pages)" under the Master Page and each page's own "Guides"; a click on a row chooses which guidelines new ones go to and which are picked and moved, and its padlock locks all of them. Opening a `.cdr` file puts the master page's guidelines there instead of a copy on every page, and saving writes them back to the master page (#1928).

- Text tool: the flow tabs of a paragraph frame link frames as with the Pick tool. A click on the bottom tab of the frame being edited, then a click on another frame (or a drag from the tab onto it), flows the text on. Before, only the Pick tool could do this (#1914).

- Windows: a tool shortcut with Alt (Alt+R set for the Rectangle tool) changes the pointer at once with the mouse standing still, as the F-key shortcuts do. The pointer stayed the system arrow, or vanished after a tool with its own pointer, until the mouse moved: Alt with a letter started the system's keyboard menu mode, which took the mouse, and the window then took the pointer for gone (#1922).

- Duplicate: Undo right after the copies of a Duplicate were deselected by accident selects them again, and the next Duplicate repeats what was done to them again. No change of the drawing is undone (#1494).

- Android: the camera cutout and the rounded corners of a phone's screen no longer hide the menu bar, the toolbox, the palette or the status bar. The interface keeps clear of the cutout, the system bars and the corners the system reports, and Options > Display > Screen margins adds margins of your own on each side, in dp, for phones that do not report their corners (with Same on all sides and Reset). The margins are filled with the theme's frame colour, Corner radius rounds the interface's corners to follow the glass, every change shows at once and the values stay when the screen turns (#1917).

- Touch screens: Shift, Ctrl and Alt buttons on the Standard toolbar, beside Delete. A tap holds the key down until the next tap on it (the button stays pressed meanwhile), so a finger can add objects or nodes to the selection or take them out, keep a drag's angle or proportions, pick inside a group or the object behind, as with the keys held on a keyboard. They show on Android and wherever a finger has touched the window (#1499).

- Browser on an iPad: the Apple Pencil draws, picks and taps the menus. Safari sends the Pencil no mouse events, only pen pointer events, which moved nothing, so the Pencil did nothing while a finger worked; a pen's pointer events now move the pointer and press and release it, with its pressure as before, and the page no longer scrolls or selects text under it (#1655).

- Linux, Wayland: a graphics tablet's pen shows the tool's cursor over the drawing (the cross hair with the tool's badge, the Shape tool's arrow, the hand) instead of the system arrow on top of it, and follows the cursor as it changes, as the mouse does. The pen's tilt is now read too and goes with its pressure, which stays the same (#1841).

- Browser: WebP export with Lossless off writes real lossy WebP, so a low Quality gives a far smaller file, as on the desktop. The browser had no lossy encoder and always wrote the near-lossless file whatever the checkbox said; it now uses a pure-Rust encoder (#592).

- Android: Options no longer shows settings that do nothing there: Show the splash screen, Use hardware acceleration with the graphics card choice, and the AI agents section are hidden on Android (as the splash and the agents already were in the browser), and an agent link set on a desktop opens no port on the phone (#1415).

- Android: Print Merge's pictures folder can be chosen. The button opened the picker for single files, so a folder could not be picked; it now opens the system's folder picker, and the pictures in the folder chosen are used for the merge (#1926).

- Opening `.cdr` version 13: graded transparency fades as drawn. Shapes whose colour fades to see-through (pages 1 and 2 of the reporter's file) opened as one flat colour; version 13 holds the same transparency record as version 15, fixed in 0.40.0, and is now read the same way (#1257).

- Opening `.cdr` version 7: the guidelines every page shows come in; that version keeps them in a chunk of their own, which was not read, so the file opened without guidelines (#1775).

- Opening `.cdr`: centre and decimal tab stops come in as centre and decimal stops (they opened as left stops), and saving writes them back as such (versions 16 and later). Paragraph text frames of two or more columns in files of versions 7 to 15 open with their text and their columns; they were dropped with "2 objects could not be read" (#1852).

- Saving to `.cdr`: objects on a master layer shown behind every page are written once, on the master page, as the program that reads these files keeps them; the file reopens with them on the master layer. They were copied onto every page, so the reopened file had a copy on each page that had to be edited page by page. Master layers in front of the pages or on odd or even pages only are still written as copies on each page (#1775).

- Fill winding: every object now has a fill rule. Opened `.cdr` files (versions 16 and later) keep each curve's own rule, so traced lettering made of overlapping parts no longer shows white gaps where the parts meet, on screen, in print, PDF and SVG; saving to `.cdr` writes it back (before version 16, as the outline of the filled area). The Properties docker's Fill section has a Fill winding box to set it: on, overlaps stay filled; off (the default), they are holes (#1919, #1211).

- Shape tools: panning with the middle button while a rectangle, ellipse or other shape is being dragged works. The preview froze, no shape was made, and the frozen preview stayed on the page after a tool change; now the shape follows the pointer over the moving page and is made when the left button goes up (#1923).

- File > Import places the file with the import cursor. After the file is chosen the pointer becomes a corner bracket: a click puts the file's top left corner there at its own size, a drag draws the box it is fitted in (proportions kept), Space or Enter centres it on the page, Esc or a right click calls it off. It came in at once where the file had it. Options > Global > File Formats, "Place imported files with the import cursor", turns it off. The drawing's right-click menu has Import... too (#1936, #1931).

- Edit > Repeat (Ctrl+R) replays the last group of actions done to the same objects, not only the newest one: after moving an object and changing its fill, Repeat moves and fills the selection, as one undo step. A change of selection between two actions starts a new group (#1566, #1547).

- B-splines: a control point can be fixed or floating. With the Shape tool, C (Cusp) or Fixed Control Point on the right-click menu makes the chosen control points fixed, so the curve passes through them; C again, S or Floating Control Point lets them float again, so the curve only comes near them. A fixed point is drawn with a second frame, and the choice is kept in `.tdraw` files. There was no way to do this before (#1758).

- Shape tool: control handle arrow heads grow with Options > Nodes and Handles > Node size (7, 9 or 11 pixels), and a press anywhere on the head takes the handle. They stayed 7 pixels whatever the node size (#1643).

- Options > Nodes and Handles: Object click distance, how far from an object's outline a click still selects it (3 pixels by default as before, 0 to 20). Thin lines are easier to hit with a larger value (#1679).

- Edit Fill and Outline Pen: the mouse wheel over the drawing zooms it (or scrolls it, as Options say) while the dialog is open. Clicks on the drawing still do nothing until the dialog closes (#1035).

- Menu bar on a narrow window, a phone or the Touch workspace: Alt and a letter now open a menu whose title moved under the » title at the end of the bar. It opens » with that menu open in it and its first row lit, so the arrows and Enter go on from there; the menus under » keep the letters they have on a wide bar, underlined in the » menu. Alt and those letters did nothing, so these menus needed the mouse or a finger (#1301, #1839).

- Document tabs: with more drawings open than the tab strip has room for, the tabs past the window edge can be reached. The strip now slides like an overflowing bar, with « and » buttons at its ends and the mouse wheel, and the tab of the drawing that becomes active (a new or opened drawing, the Window menu) slides into view. The tabs past the edge could not be reached at all, worst on a phone (#1908).

- The mouse wheel over the Color docker's colour sliders (RGB, CMYK, HSB and the others) and its Tint slider, and over the Trace dialog's sliders, now steps them on hover, one a notch, as over every other slider. These sliders kept ignoring the wheel after #1335 (#1335).

- Palettes: Add Colors from Bitmap. The document palette's menu and the bar menu of a palette of your own have a new submenu that adds 8, 16, 32 or 64 colours that stand for the selected picture (chosen as the Optimized palette of the bitmap colour conversion picks them), after the colours already there; in the document palette it is one undo step (#1840).

- TraceCut > CutContour: the CutContour spot colour's swatch now joins the document palette, in the same undo step, so it can be picked again from there. The outlines got the spot colour, but the document palette stayed without it (#1927).

- Floating bars: the close button stays at the right end of the title when the bar grows, for example the floating property bar of the Rectangle tool. It stayed where the bar's usual width put it, in the middle of the title, and the title strip ended there too (#1921).

- Windows: the title bar follows the theme. With a dark theme the window's title bar and its buttons are now dark, with a light theme light, whatever Windows itself is set to, and they change at once when the theme is switched. The title bar stayed white with the dark theme (#1920).

- Save Bitmap As TIFF and Edit Bitmap keep the transparency of a CMYK picture (one converted to CMYK with transparency, or a `.cdr` CMYK image with see-through parts): the TIFF carries an alpha channel beside the inks. The file was written flat, and Edit Bitmap then read the flat file back, so the transparency was lost from the drawing (#1925).

- Printing: Grayscale (Print > Color, under Composite, and in Print Preview) prints every colour and picture as its grey, and Reverse order (Print > General, under Collate, and in Print Preview) prints the last sheet first, for a stack that comes out face up or for printing on the backs of printed sheets. Both are kept with the other print choices (#1657, #1394, #1571).

- Cropping pictures for real: the Crop tool's bar has Crop picture pixels, which cuts an upright picture's pixels to the area so it stays a free picture (its transparency and CMYK inks cut alike) instead of going into a clip frame, and Bitmaps > Crop to Frame cuts a picture already in a clip frame to the frame and, for a rectangular frame, sets the picture free (#1913).

- Tab ruler: the paragraph text flows to a tab stop while it is dragged, so the columns move with the marker; before, the text changed only when the button was let go. The text is laid out again at most 30 times a second, so long texts keep the drag smooth, and the drag stays one undo step (#1852, #1912).

- View > Scrollbars: the drawing window's scrollbars can be hidden, on every platform, to give the drawing their room; the page tabs and the Navigator button stay. The choice is kept with the preferences (#1344).

- Text bar: a Strikethrough button beside Bold, Italic and Underline (also beside Underline in the Text docker's Character section and on a table's cell bar), so a line through the text no longer needs the docker's extra options (#1022).

- View > Page > Page Shadow: the page's shadow is an option of its own beside Page Border, so the shadow can go while the border stays (a white page on a white desktop keeps its edge) or the other way round. Page Border hid both (#1906).

- View > Page > Printable Area uses the printer's own hard margins (Windows: the printer's printable area; Linux and macOS: the CUPS PPD's imageable area for the paper set), side by side as the printer reports them, instead of a fixed 5 mm, which stays only when the system does not say. Print Preview shades the edge the chosen printer cannot print and warns when the artwork reaches into it (#1924).

- Edit > Repeat (Ctrl+R) after a mirror copy: with the copy selected, Repeat now mirrors a new copy over the same side of that copy, so the copies line up side by side. It mirrored over the original's side again and put the new copy on top of the original (#1909).

- Coordinates panel: selecting a curve drawn with the Freehand or Bezier tool reads its nodes into the multipoint curve, with their real X and Y, and keeps its bends; choosing a node with the Shape tool chooses its row, the list scrolls to show it, and an edit of the curve on the drawing shows at once. Curves with bends were not read, so the panel kept showing the values of a rectangle and the multipoint curve's own two default points (#1889).

## 0.40.0 (2026-11-05)

- Release note: the CMYK and Grayscale JPEG and TIFF export (#1763) was announced before 0.39.0 but did not reach that release; it is in 0.40.0 ("Export: JPEG and TIFF can be written in CMYK" below). The sharp zooming announced for 0.39.0 is also really in 0.40.0 ("Zooming" below).

- Opening `.cdr`: paragraph tab stops come in. A menu whose prices sit at a right tab stop opened with every price just after its name, at the default stops; the stops of each paragraph are now read (versions 7 to 21), so the prices end at the stop as in the other program. Saving: paragraph text goes out as one artistic text per line as before, now cut at its tabs with each part where the stops put it, so the columns keep their place in the other program (#1852).

- Properties docker: an Object opacity slider under the colour in the Fill and in the Outline section. It sets how solid the whole object is (fill and outline together, as an object has one transparency) without going to the Transparency section; it is off while the object has a fountain, pattern or texture transparency, which the Transparency section changes (#1818).

- Menus and lists work from the keyboard. The arrow keys did nothing in the File menu or in drop-down lists such as the font list, so a mouse was needed to pick a row, and the arrows could nudge the selected objects behind an open menu. Now Alt pressed alone lights the menu bar (Left and Right move along it, Down or Enter opens a menu); in any menu, submenu or right-click menu Down and Up move a highlight drawn like the hover box, skipping separators and greyed rows, Home and End go to the ends, Enter or Space runs the row, Right opens a submenu and Left closes it (on other rows they go to the next or previous menu), Esc closes one level. In every drop-down list Down, Up, Home, End and the page keys move the highlight from the chosen row and scroll to it, typed letters jump to the first name starting with them, Enter picks and Esc closes it unchanged; a list's button reached with Tab opens with Down, Alt+Down, F4, Enter or Space. The font list opened with its arrow now takes the keys and the typing as one opened from its field. While a menu or list is open the drawing takes no key, so the arrows never nudge the selection (#1839).

- Android phones: the interface fits the screen. On a screen whose smaller side is under 600 dp the Touch workspace no longer draws everything 1.4 times larger (a 360 dp phone was left 257 points wide); the menu titles that do not fit go under a » title at the end of the menu bar (also on a tablet upright or a narrow window, where they were cut off); an open panel is a sheet over the whole room instead of a column beside the drawing; the Welcome Screen's tabs wrap and Get Started puts the recent drawings under its buttons in a page that scrolls. The bars, property bar, toolbox and palette slide as before, and dialogs larger than the screen scroll. Tablets keep their layout (#1301, #1497, #1691).

- Android: palm rejection. While a pen touches the screen or hovers over it, the hand resting on the screen no longer draws, picks or pans; a hand that landed before the pen is let go of as the pen comes down. With no pen near, fingers draw, pan and pinch as before. Options > Tools > Freehand/Bezier, "Ignore the hand while drawing with a pen", on by default, turns it off (#1832).

- Guidelines: the rotation centre of a guideline in rotate mode can be dragged, and it snaps to objects' nodes and the other snap points, the page, the grid and guidelines, so an angled guideline can be turned about a chosen point of the drawing. It stayed where the guideline was clicked. A guideline turned about a centre off it keeps its distance from the centre (#1850).

- Shape tool: the node marquee, rectangular or freehand, is drawn as the Pick tool's marquee, blue dashes on a white line, so it shows on blue artwork. It was a plain thin blue line that was lost on blue (#1849).

- Property bars: length fields (position, size, corners and the other lengths of the property bars, also the Contour, Extrude, Table, Variable outline and Transform bars and the paragraph lengths) show only the decimals a value needs, at least one and up to the drawing precision (Options > Edit, 3 by default), with no trailing zeros: 80.0 mm, 140.5 mm, 140.625 mm instead of 80.000 mm and 140.500 mm. They always showed every decimal of the precision, which crowded the bar. A value with more decimals shows rounded and is kept as it is; a value typed is applied as typed (#1761).

- The mouse wheel over a slider (transparency, opacity and the other sliders of the bars, dockers and dialogs) changes its value on hover, one step a notch, as it does over a number field. Before, a slider stepped with the wheel only after it was focused with Tab or while its thumb was held, so the wheel over it did nothing. Beside a slider the wheel scrolls the panel as before (#1335).

- Save Drawing dialog: the column of shortcuts lists the mounted drives, memory cards and network shares (on Linux those in /media, /run/media and /mnt and the desktop's shares; on macOS /Volumes), the folders bookmarked in the desktop's file manager and here (right click on a folder: Add to bookmarks), and the folders used lately. Before, it had only the home folders and /, so saving to a USB stick or a share meant typing its path (#1391).

- Pages: objects can be moved to another page by dragging them onto its page tab. Resting on the tab for a moment opens that page with the objects on it and the move goes on there; letting go on the tab drops them at their place on that page. Ctrl (Cmd on macOS) sends copies and leaves the originals. The whole drag is one undo step. Before, only cut and paste moved objects between pages (#1346).

- Pen and Bezier tools: Redo of a close ends the curve. After a closed curve was reopened with Undo and closed again by a click on its first node, undoing that close and redoing it closed the curve but left the Pen drawing on from the first node, so the next click turned the closed curve into another shape. The redone close now ends the line as a close by hand does: the curve stays closed and selected, and the next click starts a new curve (#1846).

- QR codes: every kind of content can be picked in Properties. Only URL and Contact (vCard) could be chosen: Email, Phone number, SMS, Text and Location share no field with a URL, so picking one made empty content, which cannot be encoded, and the kind went back. The kind picked now stays with its fields ready to fill in, while the object keeps its code until they make content (#1845).

- Shape tool: a right click while dragging a selection box over nodes is ignored, as with the Pick tool. It opened the node menu in the middle of the drag, and the box then stayed frozen on the canvas, also after choosing another tool, until the next box replaced it. The box now goes on with the pointer and chooses the nodes inside when the left button goes up (#1848).

- Blend: a blend can be grouped. Selected by a click (or just made), a blend is its start or its end object alone, so Object > Group stayed off and Ctrl+G did nothing; only a selection box or Ctrl+A, which select both objects, let it be grouped. Group now takes a selected blend's start and end objects together, and the blend stays live in the group (#1842).

- Shape tool: a rectangle corner rounded to the most leaves no stacked nodes. Rounding one corner of a square as far as it goes (in the property bar or by dragging its node) put the two ends of its arc exactly on the neighbouring corners, whose nodes stayed under them: 5 nodes, two pairs stacked, that could not be told apart. A node that falls on another is now shown once, so that square shows 3 nodes; the arc's ends stay and drag the rounding back (#1851).

- Linux on Wayland: a graphics tablet's pen works in a native Wayland window. It did nothing at all there (no pointer, no clicks, no pressure), because the windowing layer did not speak Wayland's tablet protocol, and the compositor sends a pen over nothing else. The patched windowing layer now reads the protocol: the pen moves the pointer, its tip clicks and drags, its barrel buttons give the middle and right buttons, and its pressure sets the width of Freehand lines and brush strokes as on X11. `tracedraw --x11` (XWayland) still works as before; `GDK_BACKEND=x11` never changed TraceDraw's window, which is why the pen seemed to be ignored under XWayland too (#1841).

- Backspace or Delete while drawing with the Bezier, Pen, B-spline or Polyline tool takes back the last node placed with its handles, one node a press, so a misplaced point can be fixed without starting over. The keys used to delete the selected objects instead, the curve the new line went on from included. Ctrl+Z puts the node back; with no curve being drawn the keys delete the selection as before (#1527).

- Android and other touch screens: the standard toolbar has Delete (what the Delete key does to the selection) and Panels (shows or hides the dockers, which the Touch workspace starts without, so Properties and the other panels can be opened with a finger). On a palette swatch a finger held half a second and let go sets the outline colour, which needed a right click; a tap still fills, and a hold of a second shows the shades. A finger held on the drawing opens the menu of the object under it or of the page, as before (#1611, #1707).

- Snapping to objects is off on a first start, so objects, pictures and guidelines move where the pointer puts them instead of jumping to nearby objects. Guidelines and the page still snap, within the same 10 screen pixels. Alt+Z, the Snap To list on the standard toolbar and Options > Snapping turn it on, and the choice is kept; preferences saved before keep the setting they have (#1814).

- macOS: Convert to Curves has a key that works. Its Ctrl+Q is Command+Q on a Mac, the application menu's Quit, so the key asked about unsaved drawings or quit TraceDraw instead of converting. In the macOS desktop app the commands whose keys macOS keeps for itself now have keys of their own, shown in the menus, tips and Options > Customization > Commands: Convert to Curves Option+Shift+C, Convert Outline to Object Option+Shift+Q (Command+Shift+Q logs out), Merge Cells Command+Shift+M (Command+M minimizes), the Pick tool from any tool Command+Shift+Space (Command+Space is Spotlight) and Force justify while typing Command+Shift+H (Command+H hides the program). File > Exit shows Command+Q, which stays Quit. Windows, Linux and the browser version keep their keys (#1409).

- Guidelines: Ctrl+A with a guideline selected selects every guideline of the page, as Edit > Select All > Guidelines does. It selected all objects instead and the guideline lost its selection. With no guideline selected Ctrl+A selects the objects as before (#1753).

- PDF import: pictures come in at their own size. Since #1804 a picture larger than 80 megapixels or 20000 pixels on a side was shrunk without a word outside the log (the 11160 x 14794 picture of #1804 came in at 5580 x 7397). On the desktop every picture now comes in whole, read a row at a time with its soft mask before it and its PNG written straight from its pixels (about 840 MB at most for that file); only a picture past 1.5 GB of pixels, or one there is no memory for, is shrunk. A new choice, Reduce very large pictures (Import PDF dialog and Options > File Formats, off by default), keeps the 80 megapixel limit; the browser and Android always keep it. A picture that came in smaller is now told in a message after the file opens or is imported (#1835).

- Text tool: a tab ruler along the top of the paragraph frame being edited. It shows the tab stops of the paragraphs at the caret (the default grid, every 12.7 mm unless Options says otherwise, when they have none); a click adds a left stop, a drag moves a stop, a drag away from the ruler removes it, and a right click picks a stop's kind (left, centre, right or decimal). Each change is one undo step (#1852, refs #1838).

- Tab and Shift+Tab move between the controls of the window that has the focus and stay in it, in every window: the floating panels, the Font Manager, the palette New and Rename windows, the effect picker, macro forms and the rest, as dialogs already did. Before, Tab from a window's last control (or with nothing focused) went on to another window or to the menu bar, whose File menu then took the keys. Tab never goes to the menu bar (#1258).

- Export: JPEG and TIFF can be written in CMYK for print work, or in Grayscale. They were RGB only, so the CMYK values of a drawing were lost: the files opened in a bitmap editor as RGB, and a pure yellow fill read back there as about C 5 M 1 Y 93, converted to RGB and back. The Export dialog (and the Export docker's rows and its New assets row) now has Colour mode for JPEG and TIFF: RGB (the default, unchanged), CMYK or Grayscale. CMYK writes each pixel's inks the way the print separations make them: CMYK colours keep their exact values (C 100 stays cyan 100 % with no other ink), other colours and pictures go through the drawing's colour management, pictures that keep their ink channels give those, spot colours print as their process mix. The JPEG is a four-component file with the Adobe marker, the TIFF a separated (CMYK) image, both recording the resolution and carrying the CMYK profile the colours were converted with. The preview shows a CMYK export as it will print. Presets, the last export of each format and the preferences keep the colour mode; presets saved before are RGB (#1763). This was meant for 0.39.0 but a later change of that release took it out again before it was released, so no release had it yet (#1843).

- TraceCut: nesting and measuring. Shape Nesting places the selection on sheets or a roll by the objects' true outlines, turned and fitted into holes, with a time-limited search for less waste; Rectangle Nesting does it by their boxes. OK moves the objects in one undo step and can draw the sheets. Perimeter and Area, Cutting Time (from the cutter's speeds, lifts and passes) and Cost (material by area or roll length, cutting, weeding, pieces and machine time) measure the selection without changing it.

- TraceCut: the cutting tools. Plotter Output writes the selection as HPGL with a machine (generic HPGL, with pen select, a command per line), knife offset, overcut, passes, speed, force, origin corner, mirror, turn, weed frame and the planned cut order (nearest next, inner outlines first), and saves a .plt file or, on the desktop, sends it to a port path (`/dev/ttyUSB0`, `COM3`) or to a cutter on the network (raw TCP, port 9100); Test cut cuts a small square with a triangle inside. Weeding adds weeding lines, boxes and a frame; Bridges leaves uncut stretches in closed outlines; Cut Order reorders the objects and their outlines for the shortest travel; CutContour gives the selection an outline in a spot colour named CutContour; Registration Marks adds print and cut marks around the selection or inside the page. Each is one undo step. EPS, PostScript and SVG export now keep the names of spot colours (Separation colour spaces in EPS, `data-spot-stroke` and `data-spot-fill` in SVG), as PDF export already did.

- TraceCut: the tools for cutting plotters, in a window of their own (Tools > TraceCut, and Launch on the standard toolbar). The tools are listed by group (Cut, Nesting, Measure); the right side holds the chosen tool's presets and settings, a preview of the selection with what the tool finds, and Apply (shows the result, changes nothing), OK (applies it in one undo step) and Cancel. Each tool keeps named presets in `tracecut_presets.json`. The first tools are Find Duplicates (objects lying on top of another, deleted or selected) and Find Crossings (outlines crossing themselves or each other, shown and selected); the cutting, nesting and measuring tools follow. Options > General, Show TraceCut, hides it. Based on the work of LaMka21 (#170).

- Opening .cdr: a file whose master page holds objects opens with its own number of pages. The objects left beside the pages on the master page's desktop layer made the master page open as an extra first page (four pages for a three-page drawing, in every version from 7 to 21), and the guidelines every page shares showed on that extra page only. The master page's objects now open on a master layer, shown from every page, and its guidelines on every page next to the page's own. Guidelines and a desktop shared by the whole document as TraceDraw features of their own are not part of this change (#1775).

- Text > Tabs: tab stops have a kind (left, centre, right or decimal) and an optional leader, and Add tab works. Clicking Add tab on a paragraph frame just drawn did nothing: the stop was stored on the empty text where nothing read it back. The dialog now lists each stop with its position, kind and leader and a button to remove it; right stops end the text after the tab at the stop and decimal stops line up decimal points, as in a menu's prices. A paragraph without stops of its own keeps a left stop every 12.7 mm (half an inch), now set in Options > Text > Paragraph Text > Default tab spacing; a stop past the frame's edge sends the text after the tab to the next line (#1852, refs #1838).

- Saving as `.cdr`: a drawing with paragraph text opens in the other program again. Since 0.33 paragraph text went out as paragraph text with its frame, and the 2026 release of the program that wrote the samples (27.2) hung or closed on opening such files (versions 20 and 27), while the same drawing opened with its text converted to curves, and TraceDraw's files with artistic text open there. Paragraph text goes out again as one artistic text per line, in its place and with its looks, as before 0.33, so the frame is not kept in the other program; TraceDraw's paragraph frame records are kept behind `TRACEDRAW_CDR_PARAGRAPH_FRAMES=1` in `tracedraw-cli` until a layout of them is confirmed to open there (#1698).

- Saving as `.cdr` with paragraph frames (`TRACEDRAW_CDR_PARAGRAPH_FRAMES=1`, off by default): paragraph text has the record header and paragraph style of the files the other program saves. TraceDraw wrote paragraph text with the record header and paragraph style of artistic text, which no paragraph text in that program's files has: the layout 1700 header, and no hyphenation, bullet, drop cap, indent or tab settings. Paragraph text written as a frame now has the header (layout 1800, and from version 22 the 4 bytes after it) and the paragraph settings every paragraph text of that program's files holds, and `tracedraw-cli check` reports files that lack them, such as the files of the report. The default, one artistic text per line, is unchanged until the frames are confirmed to open there (#1698).

- Saving as `.cdr` (version 19 on): text has the record header of the files the other program's recent releases save. Every artistic text in their files of versions 19 to 27 starts its text record with 1, 1, 0, 1 and layout 1800 (from version 22 with 14 and 4 more bytes after the paragraphs), and its styles carry `variableAttributes` (from 19) and `font_axes` (from 22); TraceDraw wrote the header of the program's 2019 release (-1 and layout 1700) and neither member, and the reporter of #1698 says one-line text also hangs that program. Text is now written as those files hold it; versions 16 to 18 are unchanged. Paragraph text of layout 1500, as the 2019 release writes it, is no longer reported by `tracedraw-cli check` (#1698).


- Page Down on the last page opens Insert Page with one page after it, and Page Up on the first page opens it with one page before it. The keys did nothing there. Between the first and the last page they still go to the next and the previous page (#1752).

- Welcome Screen, What's New: a history of the last ten releases instead of three highlights that no longer matched the version shown. Each row shows a version, its date and a short title; a click opens a plain paragraph about what changed and a "Want to know more?" link to that release's page with the full list. The newest release is open at first. Help > What's New opens it.

- Shaping a clip frame (Weld, Trim, Intersect, Simplify, Front Minus Back, Back Minus Front) keeps it a clip frame: its frame takes the new shape with its own fill and outline, and what it held stays inside; before, the result was a curve with no fill and the contents were gone (#1745 to #1750).

- Opening .cdr: paragraph text saved with the older text layout comes in. A version 21 file opened with every paragraph text missing (84 frames of a two-page manual), and the report gave the warning "implausible style record count". Its paragraphs hold one style each, where TraceDraw expected two, so the rest of each text record was misread. Both forms are now read (#1833).

- Opening .cdr version 15: an object with a graded transparency fades as drawn. Such objects, like a sky blue to white or a pink to white page background, opened as one flat colour, because their transparency record was read as a plain opacity of 0. TraceDraw now reads the record's fountain of greys as the object's transparency, placed between the record's own start and end points, so the reporter's backgrounds open as their page previews show them (#1257).

- Opening `.cdr`: an object distorted two or more times (a Push and Pull, Zipper or Twister applied again over the first) opens with all its distortions. It opened with the first one only, and saved that wrong shape again. The earlier distortions are now part of the outline and the last one stays live and editable; the shapes match the pictures the files carry (Refs #1824).

- Rulers: a right click on a ruler or on the corner where they meet opens a menu with Ruler Setup, Grid Setup and Guidelines Setup (Document Setup at that page) and Rulers to hide them. A right click on a ruler did nothing (#1831).

- Save Bitmap As and Edit Bitmap keep a picture's kind and its CMYK inks. Both always wrote an RGB PNG, so a JPEG, TIFF or CMYK picture could not be saved as it is, and the ink channels a CMYK picture keeps for printing were lost. Save Bitmap As now writes PNG, JPEG or TIFF (by the extension; the dialog offers the kind of the picture's own file first, else TIFF for a CMYK picture, else PNG). A CMYK picture saved as JPEG or TIFF is a CMYK file with its inks as the drawing keeps them, its resolution and the CMYK profile in use. Edit Bitmap hands a CMYK picture to the other program as a CMYK TIFF and reads the inks back when it saves. The CMYK JPEG and TIFF writers of #1763 are back in `tracedraw-io` for this (#1834).

- Cropping a picture that has a transparency: what is left looks as it did. Dragging a picture's corner nodes with the Shape tool, or cutting it with the Crop tool, put the picture in a clip frame that kept a copy of its transparency and opacity, so they were laid on twice and the picture turned much paler; with a gradient transparency the frame's copy also started again across the cut-down picture at each node moved. The picture inside now keeps its transparency in its place on the page, and the frame keeps none. A clip frame's own gradient transparency also stays in place when the frame's nodes are moved (#1491).

- Blend, Apply to sizing: each step takes the outline and the size of its accelerated place. A square blended into a circle with the steps packed toward the circle kept square steps right up to the circle, because only their size followed the acceleration; the steps by the circle are now round, as the pictures of the `.cdr` files saved with that setting show (#1576).

- SVG import: text comes in as editable text. Every `<text>` element was left out, so a file with a rectangle and a caption imported as the rectangle alone. Text now keeps its font, size, weight, style, colour, outline, anchor and position, with styles from attributes, `style` and `<style>` sheets, `tspan` runs in their own style, lines written as one `tspan` each as one text with its line spacing, and transforms, `use` and clips as for shapes (#1820).

- Opening a PDF: Hindi from a word processor's PDF comes in as typed in many more words. Such files give the glyphs of each syllable the wrong characters in their `ToUnicode` map: the characters are handed out in typed order to glyphs in drawn order, and a glyph keeps the characters of the first word it appears in. Matras, the reph, half letters and conjuncts then swapped in later words, so वार्ड came in as वार्ि, वर्ष as वषि and निर्गत as र्रि्ति. TraceDraw now checks the map against the embedded font's own character map and works out the other glyphs from where they first appear. On the reporter's file 56 of 68 words come in right (25 before). The file holds the characters of the other 12 nowhere, such as the द्र conjunct in पुष्पेन्द्र, so those still come in wrong (#1806).

- Browser: the mouse wheel scrolls dockers, lists and dialogs as far a notch as on the desktop. egui's own browser units made a notch scroll 100 points in Chromium and 24 in Firefox, where the desktop scrolls 40. A touchpad scrolls the distance it reports, as before (Refs #1406).

- Export dialog: deleting a preset asks first. The "−" beside the preset list deleted the chosen preset with one click, for good. It now asks 'Delete the preset "name"?' in a row under the list, with Yes and No (#1816).

- Text tool: paragraph text frames dragged one after the other all stay. Dragging a second frame (or clicking to start an artistic text) deleted the first one while it was still empty, so only the last empty frame was left on the page. An empty frame now stays as a frame to type or flow text into later; an artistic text left with no characters still leaves nothing (#1822).

- Opening .cdr: Push and Pull distortions made in the program that writes .cdr files open distorted and live, as Zipper and Twister already did. A pulled square opened as the plain square with the note "a distortion of kind 1 is not read"; it now opens as TraceDraw's live Push and Pull with the file's amount and centre, drawn as the previews inside the files show, and the distort tool adjusts it. A distortion applied again over another one opens with the first only, with a note; saving to .cdr still writes distortions as the curves they draw (#1824, refs #1715).

- Opening a PDF: a very large picture (more than 20000 pixels on a side or more than 80 megapixels) comes in shrunk to fit those limits instead of being left out. A PDF holding an 11160 x 14794 pixel picture opened with only the outline around it; the picture now comes in at 5580 x 7397 pixels, in its place, with its soft mask (#1804).

- Several monitors: TraceDraw opens on the primary monitor, centred on it. The splash screen and the main window were centred by the monitor's size from the top-left corner of the whole desktop, so with the primary monitor below or right of others (X11, four monitors) the window opened on the top-left monitor (#1800).

- .cdr page names: opening a .cdr keeps the names given to its pages, and saving a .cdr (version 14 and later) writes them. Every page opened as "Page 1", "Page 2" and so on, and a save wrote no page name, so a name was lost for good after a round trip (#1807).

- Envelope: dragging a segment of the frame bends it, with the Envelope tool and with the Shape tool, as the Shape tool bends a curve. Only the nodes and handles could be dragged, and a drag on the frame between two nodes did nothing (#1339).

- Distort: Shift and Ctrl steer the end handle of a Push and Pull, Zipper or Twister distortion while it is dragged (and the line of a new one while it is dragged out). Shift moves the handle only sideways, at the height it had; Ctrl keeps its length and turns it freely all the way round; Ctrl+Shift keeps it to steps of the constrain angle about the centre, as the Shape tool's handles do. Both keys did nothing there (#1563, #1793).

- Transparency: the Properties docker's transparency slider and the Transparency tool's bar keep up with a large selection. Every semi-transparent object was drawn on a scratch layer the size of the whole window, so a page of 2000 half transparent objects took more than 2 s to draw again after each change; the layer now covers only what the object can reach, with the same result, and that page draws in about a tenth of a second. A change to the selection's transparency was also two commands for each object, each searching the whole drawing; it is now one command that finds all of them in one pass. With 2000 objects selected a frame of a slider drag went from 39 ms to 11 ms. The mouse wheel still steps a slider only once it has the focus, as for every slider (#1335).

- Saving .cdr with Embed fonts: italic text in a font that has no italic face no longer makes the program that reads .cdr files report the font missing. TraceDraw slants such text itself (RAJ 100 or Khand, whose files are all upright), but saved it as an italic face beside the embedded upright one, so that program asked for "RAJ 100-Bold-Italic", which neither the file nor the system has, and drew "RAJ 100-Bold" instead. The text is now saved as the upright face that draws it and is embedded, for every weight (Light, Medium, SemiBold, Bold) and with or without underline; that program, which offers no italic for such fonts, shows it upright. TraceDraw opens its own file with the text italic again (#1733).

- Opening .cdr: Zipper and Twister distortions made in the program that writes .cdr files open distorted and live. A Zipper on a line opened as a straight line and a Twister on a square as a plain square, although the previews inside the files showed the effect, and saving again lost it. They now open as TraceDraw's live distortion with the file's settings (amplitude, frequency, Smooth, the centre and the lean it gives the teeth; the Twister's angle), drawn as the previews show, and the distort tool adjusts them. A Push and Pull distortion still opens undistorted, with a note, until a sample file shows how it is stored; saving to .cdr still writes distortions as the curves they draw (#1715, #1031).

- Linux on X11: a large picture copied by a program that holds only the picture (`xclip -selection clipboard -t image/png` with a picture of about 1 MB) pastes with the first Ctrl+V, and the window no longer freezes. The windowing layer read the clipboard as text on every Ctrl+V; such a program answers with its picture sent in parts, the text read stopped after the first part, and the program then waited for the rest and answered nothing else, so the picture never pasted and the window froze for about 24 s while TraceDraw's own reads waited. On X11 the clipboard's text and files are now asked for only when its list of types offers them (#1488).

- Zooming: the drawing now really stays sharp while zooming in quickly, as 0.39.0 announced. The change was taken out again by a later change before 0.39.0 was released, so 0.39.0 still showed the old picture blown up until the view had rested for 0.12 s. A zoom is now rendered at once on a thread, the latest zoom winning, with one shared copy of the drawing for its renders (#1773).

## 0.39.0 (2026-11-04)

- Convert to Bitmap: the Grayscale and CMYK colour modes give a bitmap in that mode. Every choice gave an RGB bitmap: Grayscale made the pixels gray but kept an RGB picture, and CMYK did not even change the pixels. Grayscale now stores a grayscale picture, and the status line says "Bitmap (Grayscale)"; CMYK marks the bitmap as a CMYK picture and shows its pixels as the inks print them, as Image > Mode > CMYK Color does (#1777).

- Shortcuts: Ctrl+F12 and Ctrl+F8 each run one command. Text > Proofing > Spell Check showed Ctrl+F12 and Text > Convert showed Ctrl+F8, but those keys open the Fonts and Blend panels, as the shortcut tables say, so neither menu command had a working key. The two menu rows no longer show a key (Options > Customization > Commands can give them one), the Spelling options page no longer names Ctrl+F12, and a test checks that no two menu commands, tools or panels share a built-in key (#1779).

- Windows: the scroll arrows of a toolbox or a panel column too long for the window show again. They were empty boxes because none of the fonts TraceDraw brings has the small triangle characters they were written with; Linux usually found them in a system font. The arrows are now drawn as triangles. The other interface symbols no bundled font had (the drop-down marks, the up and down buttons of the effect lists and macros, the line and arrowhead lists of the callout and outline bars, the portrait and landscape buttons, the AI panel's marks and a few more) are drawn with characters the bundled fonts have, or as shapes, and a test checks that every symbol the interface writes is in the bundled fonts (#1780).

- Print Preview: the bleed limit can be set beside the Bleed box. Print Preview had only the box, and the limit (how far past the page edges objects print, 4 mm at first) could be changed only in the Print dialog. The field is the same setting as the Print dialog's Bleed limit (0 to 50 mm), greyed while Bleed is off, and the sheets follow it at once (#1653).

- Blend: a click on the end object picks the end object where the steps lie over it. The steps are drawn by the start object, above the end when the start is in front, and a text step is picked anywhere in its box, so the steps of a text blend covered nearly all of the end text: a click on it picked the start text, sometimes far away, and a colour from the palette went to that other end. Only a click at the end text's edge, beyond the steps, picked it. Now a press on the steps where the end object lies, away from the start object itself, picks the end object, with the Pick tool and the Text tool; elsewhere on the steps the start object is picked as before. A text that lies inside the box of the text in front of it (a copy offset by a millimetre or two) is still picked by that front text's box, as for any two texts (#1739).

- Linux on Wayland: files dragged from a file manager onto the window open or import again. On a native Wayland window (GNOME, KDE and the other Wayland desktops, when TraceDraw is not started with `--x11`) a drop did nothing: the windowing layer did not take drags at all, so the file manager showed the drag as refused. It now takes a drag of files like X11, Windows and macOS: the frame and the label say "Drop to import into the drawing" or "Drop to open" while the files hover, and where they are let go decides between importing them into the drawing and opening them in a tab of their own. `--x11` is no longer needed for drag and drop (#1142).

- Browser version: a graphics tablet's pen now sets the width of Freehand lines and of Preset, Brush and Expression brush strokes with its pressure, as on Windows, X11 and macOS. The browser's web layer read a force only from touch events and dropped the pressure of the pen's pointer events, so the pen drew an even width. A pen's pointer events (Chromium, Firefox and Safari report their pressure where the system does) now give the pressure through the same path as the desktop, so the tools behave alike, and the Options switch under Graphics tablet still turns it off. The mouse gives no pressure (its pointer events report 0.5 while pressed, which means none), a pen that reports a constant 0.5 is taken as a pen without pressure and draws like the mouse, and touch screens keep their force as before. Tilt is not read (#1410).

- Stars: the inner points can be dragged through the centre into an inverted star. With the Shape tool, a star's inner node stopped just short of the centre (the sharpness was held between 0.001 and 0.999), and a drag on past it turned the points half a turn instead. Now the inner points come out on the far side of the centre, as a negative sharpness (down to -99 %), and go back to a plain star when dragged back through; Ctrl still keeps their turn. The sharpness boxes of the property bar, the Coordinates docker and Options take -1 to -99. `.tdraw` files keep the negative sharpness and older files open as before; SVG, PDF, EPS and `.cdr` exports get the outline as drawn (`.cdr` writes it as a curve, as it does every polygon) (#1331).

- Corners panel: the preview can be switched off. While the panel was open every selected object showed the dashed outline of the result over it, hiding its own outline, snap points and nearby lines, and the only way to get rid of it was to close the panel. The panel now has a Preview box under the values: on at first, as before; off, nothing is drawn over the objects and Apply still works. The choice is kept in the settings (#1759).

- Font list: Down and Up in the property bar's font field step the selected text through the fonts. With the Pick tool and a text selected, a click in the field gave it the keyboard but left the list closed, and the arrows then did nothing; after Enter they nudged the text. Now, while the field has the keyboard and the list is closed, Down takes the next family and Up the one before, in the list's order, applied to the text at once as the mouse wheel over the field does, and the field keeps the keyboard for the next press. Enter after stepping keeps the family stepped to; it put back the one the field showed first. With no text to change the arrows open the list (#1754).

- Tab in text being edited: the text typed after the tab goes into the text. Tab typed its tab character, but the window also passed the keyboard focus on to its first field (the property bar's), so everything typed afterwards went there and the paragraph text seemed to ignore the key. With nothing of the window focused and no dialog open, Tab now belongs to the drawing only: the tab moves the following text to the next tab stop and typing goes on in the text. The same theft stopped Tab and Shift+Tab with the Pick tool after the first press; they now go from object to object press after press. Tab in dialogs and between the fields of the bars works as before (#1667).

- Export: AI files can now be written as the classic PostScript-based file of version 8, for programs that read only those and refused TraceDraw's `.ai` files as "File format is incorrect". Choose it under Compatibility when the format is AI: "Illustrator 8 (EPS-based)". It follows the public file format specification: the `%!PS-Adobe-3.0` header with the bounding box (the page), the `%%AI8_CreatorVersion` and `%AI5_FileFormat` comments, layers, groups, compound paths, clipping groups and process colours in the format's own operators. Text is written as curves, fountain fills as bands of colour, pattern, texture and mesh fills as their average colour, RGB colours as CMYK; pictures and transparency are left out, and each page goes to a file of its own. PDF-compatible stays the default, so existing exports do not change. TraceDraw opens these files again (#1728).

- Language at first start: TraceDraw starts in the language chosen in the Windows installer. The installer only set the language of its own wizard, and the app, which looked at the `LC_ALL` and `LANG` variables alone, started in English on Windows and then saved that English as if it had been picked. The installer now has a "TraceDraw's language" page with System language first and selected (it names the language Windows gives) and every interface language, and records the choice in the registry (`Software\TraceDraw`, `InstallerLanguage`); `/APPLANGUAGE=<code>` sets it for silent installs. Until a language is picked in Options, TraceDraw starts in the installer's choice, else in the system's preferred language (Windows display languages, macOS preferred languages, `LANGUAGE`, `LC_ALL`, `LC_MESSAGES` and `LANG` on Linux, the browser's languages), matched to the closest supported language (Portuguese of Portugal gives Portuguese (Brazil), `zh-Hant` Traditional Chinese), else English. A language picked in Options always wins, and Options > Global > General now offers System language. Settings of earlier versions keep their language. See `docs/behavior/language.md`.

- macOS: TraceDraw can be made the default program for `.cdr` files. The system types `.cdr` files as disk images, and TraceDraw offered itself for them by extension only, so Open With worked but Get Info > Change All did not keep TraceDraw and a double click still went to the disk image mounter ("The disk image couldn't be opened"). The package now declares its document types by content type: `.cdr` as the system's disk image type and a type of its own, as an alternate program (it never takes the files over by itself), and `.tdraw` as TraceDraw's own type. Change All then sticks and a double click opens the drawing in TraceDraw (#1504).

- Snapping: a move snaps to where the selection itself was when the move began. An object moved by one of its nodes, midpoints or its centre snapped only to other objects, so a square could not be put with a node exactly where another of its nodes was, nor brought back exactly to its place. Now, once the pointer has gone further than the snapping radius from where the move began, the grabbed point also snaps to the selection's own nodes, midpoints, centre and edges where they were at the start, by the snap modes on, and to its own former place. A short move inside that radius still follows the pointer, so nothing sticks to where it was; the moving selection itself is never a target (#1765).

- Rectangles: one corner can be larger than half the shorter side when the corners beside it leave room. Each corner's size was cut to half the shorter side, so on a 100 x 80 mm rectangle with a 10 mm top left corner and square bottom corners the top right corner stopped at 40 mm instead of the 80 mm quarter circle down the whole right side asked for (lock tabs, dust flaps). The two corners on a side may now together be as long as the side; only when they are longer is every corner drawn smaller by the same factor, so equal corners still stop at half the shorter side. The property bar and the Properties docker keep the size typed and show the size drawn; Ctrl+drag of one corner node with the Shape tool goes as far as its neighbours leave it room. `.cdr`, SVG and PDF files get the corners as drawn (#1757).

- Distort, Zipper: only the tips of leaning teeth lean. A tilt turned every point of the zigzag about its place on the outline, so the bases (the points on the side away from the handle) slid the other way along the outline and the teeth got lower the more they leaned. Now the bases stay where the upright teeth have them and the tips, still the amplitude away from the outline, move along it onto the leaned line of the handle, as in the reporter's reference files: the zigzag keeps its height, half to each side of the outline, at any tilt. Smooth teeth stay sine waves and the Twister is unchanged. Drawings with a tilted Zipper show the new shape (#1736).

- Zooming: the drawing stays sharp while zooming in quickly. A zoom showed the last picture of the page blown up until the view had rested for 0.12 s, and every frame of a wheel notch's glide started that wait again, so a burst of notches stayed blurry all along and for a render after it. A zoom on the drawing window now renders the new zoom at once on a thread of its own; a notch that comes while it runs does not restart it, and when it ends its picture shows and the next render starts at once for the latest zoom. On a page of 1000 curves the picture is sharp about 35 ms after the last notch instead of about 145 ms, blown up 1.08 times on average during a burst instead of 1.47, and the slowest frame of the burst takes 3 ms instead of 20 to 28 ms (the render no longer runs on the window's thread); on a page of 5000 curves, which takes over 100 ms to draw, it is sharp after about 150 ms instead of 230 ms and a sharper picture shows every render during the burst. Renders during a zoom share one copy of the drawing (a copy of a large drawing takes a frame's time). Panning is unchanged; the other pages of the Multipage View and the browser version still wait for the view to rest.

- `.cdr` envelopes and linear fills of newer files (version 16 on) look as in the program that saved them. An enveloped object kept the outline it had before the envelope: the "2.0" of the reporter's version 27 logo came out straight, with its outlines doubled and its dot 50 mm away under another picture. The envelope (its box, its outline and the matrices around it) is now read and the object bent into it, for objects and for groups; text and pictures with an envelope are only placed, with a warning. Linear fountain fills were placed by their angle and edge pad only, so the white of the letters turned yellow in their middle; they now follow the fill's own centre and run stored in the file, which put the turn along the letters' lower edge as the picture the file carries shows. The envelopes are read as bent curves, not live envelopes; radial, conical and square fills are read as before (#1668).

- Interactive effects: an effect's control line can be grabbed anywhere along it to move the whole effect. With Interactive Fill and Transparency the line between the start and the end could only be moved by its start node; now a drag that starts on the line (within the pick distance of Options > Nodes and Handles), away from the nodes and the midpoint sliders, moves the start, the end and the skew handle together, so the line keeps its length and angle, for linear, radial, conical and square fills and masks. The same goes for the Drop Shadow's line (the shadow moves, its start staying on the object), the Extrude line to the vanishing point (the vanishing point moves by the drag), the Blend line (both objects move together) and the Distort handle line (the centre moves, the line with it). End handles still turn and stretch the line, and the sliders on it still win where they are pressed; Ctrl keeps the move horizontal or vertical; the pointer shows the move cursor over the line; each drag is one undo step. A drag of a drop shadow's handle no longer merges into the shadow's previous undo step.

- Print separations: pictures are split into their inks. Every plate printed each picture whole, in its colours, so a CMYK picture came out on the cyan, magenta, yellow and black plates alike. Now a CMYK picture of a .cdr file prints each ink it holds on its own plate (cyan ink alone only on the cyan plate, black ink alone only on the black plate), and RGB and greyscale pictures are converted to CMYK with the drawing's colour management, as vector colours are: red prints magenta and yellow, grey prints black. A picture's transparency and clipping stay on every plate. Print Preview shows the same split for one plate and for any set of ticked inks. Each picture is split once per print job (#1659).

- Saving .cdr with Embed fonts: text drawn with a different face of its font than the one it asks for is now saved as the face that draws it. Khand text with only the Khand Bold file installed is drawn bold in TraceDraw, but was saved as regular Khand beside the embedded bold face, so the program that reads .cdr files asked for "Khand-Normal", reported the font missing although Khand was installed, showed a substitute and dropped the embedded font when saving again. Such text is now saved as Khand Bold, as that program saves the same drawing, and opens with the installed or embedded face. Bold or italic that TraceDraw makes by thickening or slanting a face is still saved as bold or italic (#1733, #1484, #1455).

- `.cdr` blends: the object and colour acceleration, the rotation, Loop, Apply to sizing and the colour direction (Direct, Clockwise, Counterclockwise) are now read from files of version 23 on and written back, where the program that saves such files keeps them. They were neither read nor written: a blend with an angle opened without it, an accelerated one with even steps, and a blend saved by TraceDraw opened in that program with every setting at its default. The layout was learned from the reporter's 134 version 27 files, each changing one setting, and the renders were compared with the pictures they carry: the steps' places, the loop's curve, the turns and the colours now match (#1192, #1576). Along with it: the acceleration curve is the files' one (the middle step at most 10.5 % from an end instead of 5 %), so existing drawings with an acceleration look a little different; Loop now curves the steps from one object to the other instead of swinging them about the middle, where the last steps ended far from the end object; a blend into an ellipse read from a file no longer twists and pinches in the middle; and a blend read from a file sits on its lower object, the one the record counts from. Blends along a path are still not read or written (no file with one was available).

- Toolbars: Large and Extra Large buttons are no longer cut off at the bottom. A docked toolbar's row kept the height of Medium buttons (36 px, 53 px for the property bar), so the standard toolbar's 42 and 52 px buttons lost their lower edge. The rows now grow with the button size, with the same margins; Extra Small to Medium keep today's heights.

- Default properties: a black the user chose stays as chosen. Since 0.38.0 every black fill or outline of the defaults turned into the drawing's black, so a CMYK 100 % K outline saved on purpose with Tools > Save as Defaults became RGB black in an RGB drawing, and changing a drawing's colour mode converted it too. Now only the untouched default black (the outline of graphics and the fill of text, as a new drawing or Reset gives them) follows the colour mode; a colour picked for the defaults, even the same black, stays exactly as picked, and changing only the outline width keeps the default black following. Drawings and settings saved before count a black where the built-in defaults put one as untouched and any other as chosen (#1665).

- Toolbars: at the Large and Extra Large button sizes the standard bar's and the property bar's icons keep a margin. They were drawn at the toolbox's icon size and filled their buttons almost to the edges, which looked heavy. They are now about 5/6 of that size (standard bar 28 and 34 px in 42 and 52 px buttons, property bar 24 and 28 px in 38 and 46 px buttons), whole pixels so they stay sharp, with 7 and 9 px of margin each side. The smaller sizes and the toolbox are unchanged.

- Color palettes: dragging the document palette out by its grip floats the document palette, not the default palette. Both rows' grips moved the default palette, so the default palette came up in a floating window and the document palette stayed docked. Now each grip moves its own bar: the document palette becomes a bar of its own that floats or docks by itself, and let go at the bottom again (the top while the default palette is there) it is the row under the drawing once more; the default palette and every further palette bar float and dock on their own as before.

- Palettes: New Palette, and every palette shown in a bar of its own in plain words. Several palettes could already stand in bars of their own, but only the Palette Manager's bare eye opened one, and there was no way to start a palette from nothing. Now New Palette... (a button in the Palette Manager, Window > Color Palettes and each palette bar's menu) asks for a name ("Palette 1", "Palette 2"...), makes an empty palette under My Palettes and shows it at once in a bar at the bottom, under the others; colours dropped on that bar join the palette, and its menu adds the selection's colours. Each palette's row in the Palette Manager has a Show bar / Hide bar button and, while its bar shows, a list that moves it (Top, Bottom, Left, Right, Floating); the rows setting there reads "Rows of selected bar". Palettes made in TraceDraw (New Palette, Duplicate) are kept in the preferences with their colours.

## 0.38.0 (2026-11-03)

- Web and Android: every folder button does something, the same way in every dialog. A browser and the Android app cannot pick a folder, and the buttons behaved differently in each place: hidden in the Export docker, greyed without a reason in the Font Manager and Options, a note in Picture Mosaic, and on Android enabled but silent everywhere. Now no button there shows a folder dialog: the Export docker writes its assets into one zip file named after the drawing (downloaded in the browser, placed by the system's save picker on Android); Font Manager > Add Folder, Print Merge's pictures folder, Picture Mosaic's Browse and, on Android, Options > Spelling's More dictionaries pick several files at once instead (the fonts are added, the pictures serve as the folder); the backup folders in Options > Save are greyed with a tip saying why, as Package's Browse is in the browser. Each button's tip says what it does there. The browser's picker for several files now takes several at once. The desktop is unchanged (#1399).

- Android: tone curve and duotone presets, masks, macro scripts and exports and palettes ask where to save them. They went to the app's own folder under the suggested name without asking; now the system's save picker asks, with that name, as Save As does (a form exported with its `.frx` asks for each file) (#1402).

- Paragraph text: resizing a frame with the Pick tool's handles flows the text anew while the handle is dragged. The drag outline stretched the old lines with the frame and the words only flowed into the new width on release; now the outline shows the line breaks the release will make, and the frame turns red as soon as the text no longer fits. Only the line breaking is done again as the pointer moves (the text engine keeps the shaped glyphs), once per frame at most; a very long text whose layout would slow the drag down shows its frame alone and flows on release, as before. The release is still one undo step (#1327).

- Web: every command whose shortcut the browser keeps has a key of its own in the browser version, shown in its place. Ctrl+N, Ctrl+T, Ctrl+W, Ctrl+Shift+T, Ctrl+F4 and Alt+F4 never reach a page, and some browsers also keep Ctrl+PgUp/PgDn and Ctrl+1 to Ctrl+9 for switching tabs or Ctrl+Q and Ctrl+Shift+Q for quitting, so New, the Text panel, Edit Text, Refresh Window, Close Window, Forward One, Back One, Convert to Curves, Convert Outline to Object and storing selection groups could not be run from the keyboard there, and nothing said so. In the browser they now also run with Alt+Shift+N, Alt+Shift+X, Alt+Shift+T, Ctrl+F5, Alt+Shift+W, Alt+PgUp, Alt+PgDn, Alt+Shift+C, Alt+Shift+Q and Alt+Shift+0 to 9; the menus, context menus, toolbar tips and hints show these keys there, and Options > Customization > Commands lists which keys the browser keeps. The desktop apps are unchanged (#1409).

- Distort: the handles are easy to see and stand straight. They were thin blue outlines that faded into a white page, drawn wherever the view put them between pixels, so they came out smeared and lopsided, and the Zipper's frequency slider was a narrow upright bar that looked crooked on a slanted line. Now every handle (centre, end, Zipper slider, Twister circle) is white inside a ring of the node colour of Options > Nodes and Handles with a dark edge, as large as the Handle size there, and stands on the pixel grid; the slider is a small circle, and the line is thicker with a white halo. Larger handles are taken anywhere on them (#1574).

- Start-up: the splash screen turns into the main window without a broken frame. While the window grew from the splash's size to its own, the editor was already drawn at the old size, so for half a second to a second the window showed the splash in one corner, the editor squeezed below it and torn, leftover pixels over the rest (seen on X11). The window is now drawn plain in the interface's background colour until it has its size, or is maximized when it starts so (at most one second), and the editor appears at the window's final size (#1673).

- Dimensions stay on the object they measure when they are moved by themselves: dragging, nudging, scaling or turning a dimension whose points are held to an object's nodes or edges silently let go of the object, so it no longer followed it. Now the held points stay on the object and only the dimension line (or an angle's arc) takes the move, along its normal; moving the object still takes the dimension along. Break Apart is what frees a dimension from its object (#1492).

- `.cdr` files of version 16 and later: objects given a uniform transparency in the other program now open see-through when the file keeps the amount outside the object's style string, as a group made half transparent in a version 24 file does for each of its objects. Such a group opened fully opaque, hiding what lay under it. Saving keeps the transparency on each object (#1706).

- Web: a mouse wheel notch is one step outside the canvas too. The browser reports a notch as three lines or 100 pixels, which the Objects docker's thumbnail size read as three or two steps, and a palette scrolled a swatch for every half notch from a touchpad. Every widget that steps with the wheel (number fields, sliders, the Objects docker, palette scrolling and swatch size, Ctrl+wheel over the toolbars and the toolbox) now counts notches as the canvas zoom does, one step a notch on the desktop and in the browser alike (#1406).

- Color palettes: several palettes can be open at once, each in a bar of its own (the document palette too), docked at any edge of the window (columns of swatches at the left and right) or floating in a small window. A floating palette is resized at its right and bottom edges and its swatches flow into rows in a wide window and into columns in a tall one. Each bar has its own rows and swatch size; the palette menu now offers Swatch Size (Extra Small, Small, Medium, Large, Extra Large; Ctrl+wheel keeps its finer 2 px steps), Rows from 1 to 8 and Fit, Position (Top, Bottom, Left, Right, Floating), Open as Palette Bar and Close Palette. The palettes open, their places, rows and sizes are kept in the preferences and in workspace files; preferences from earlier versions give the one palette at the bottom as before. Every palette, built-in ones and the document palette included, can be given a name of its own (Rename..., Reset Name), kept by the preferences without changing built-in palettes; palettes saved as `.gpl` or `.tdpal` keep it in the file. The Palette Manager is laid out anew: Built-in, My Palettes and Document Palette sections, each palette with a strip of its first colors, its color count, color model and where it shows, an eye to open or close its bar, a filter box, a swatch size slider with ticks at the named sizes for the selected palette's bar, its Rows, and a menu with Open as New Palette Bar, Edit, Rename, Duplicate, Lock, Delete, Move Up and Move Down (rows can also be dragged); the document palette's section has Add Selection Colors, Add from Document and Clear.

- B-spline: the curve placed is the curve previewed. While drawing, the preview took the pointer as one more control point even right after a click, when it lay on the point just placed: the end point was doubled, so the curve's end bent differently from the curve made, and finishing dropped the pointer's point. Now the pointer is only a tentative next control point, adding nothing while it rests on the last point (a second click there places no second point either), so the preview after a click is the curve so far and finishing makes the last preview without the tentative point (#1506).

- Blend: the start object shows its own selection box and handles. Selecting it put the box around the whole blend, out to the last step, so its handles stood far from it (on the end object) and a side handle stretched it from the wrong place, while the end object got a box of its own. The steps lie between the two objects and belong to neither box: the start object's box, the property bar's position and size, its handles and its rotation centre are now those of the object itself (its other live effects included). A group holding the blend still shows the whole blend, and so does a blend from an older file whose end lives only in the effect (#1540).

- PDF import: crop marks and spot colours come back. A colour in a Separation colour space at full tint opened as white, so the crop and registration marks of a PDF published with printers' marks were invisible when it was opened again, and every spot colour came in lighter the stronger it was. Separation tints are now ink: the `All` colourant is the registration colour, `None` paints nothing, the process colourant names are their process ink and any other name is a spot ink, added to the drawing's inks (by name, once) with the alternate colour as its process stand-in. DeviceN colours mix their colourants as process inks, Indexed colours take their table entry in the base space, and pictures in Separation and DeviceN spaces show their tints as ink. Import adds the file's inks to the drawing's, an ink of the same name being the same ink (#1703).

- Default properties: new graphics in an RGB drawing get an RGB black outline (0, 0, 0); it was CMYK black (100 % K) in every drawing, while new text already followed the drawing's colour mode. A black in the saved defaults, CMYK or RGB, becomes the drawing's black, other saved colours stay as chosen; changing the primary colour mode in Document Color Settings takes the black defaults along, and Reset in the Object Styles panel gives the defaults of the drawing's mode (#1665).

- `.cdr` gradients of turned objects run the way they were drawn: a file measures a gradient's angle from its object, so a rectangle turned a quarter turn opened with its gradient running across it instead of along it (green from the top left in place of the lower left, blue on the left in place of the bottom). Saving takes the object's turn off again, so the other program shows TraceDraw's turned gradients as they look here. A two-colour linear gradient also keeps its mid point, where the two colours mix half and half: a gradient whose colours change within a few percent of its start opened as an even blend (#1257).

- Standard bar: the Launch button is an icon with its list arrow, like Options; its name moved to the tooltip, so the bar is narrower.

- Multipage View: the space between pages can be changed. Page > Page Spacing, next to Page Layout, offers 0, 5, 10, 20, 30 and 50 mm and Custom... for any distance in the drawing's units, and Document Setup > Layout has a Page spacing field. The spacing is saved with the drawing (drawings saved before keep 30 mm), each change is one undo step, Save as Default keeps it for new drawings, and the page grid, clicks on other pages, snapping to their objects, page labels and Zoom to Fit follow it. It was a fixed 30 mm with no setting.

- Workspace: the icons of the toolbox, the standard and property bars and the toolbox's flyouts now grow with the button size. Ctrl+wheel over a bar and the bars' menu > Customize (Default Button Size, Toolbox Toolbar > Button Size) made the buttons and the room between them larger while the icons kept one size. There are now five sizes, Extra Small, Small, Medium, Large and Extra Large, the same in both menus and for Ctrl+wheel, with icons of 16, 20, 24, 32 and 40 px in buttons of 22, 26, 32, 42 and 52 px (property bar: 14 to 34 px icons in 20 to 46 px buttons); the padding grows more slowly than the icons, and the floating toolbox keeps its width in buttons. Sizes saved before keep their meaning.

- PDF import: clipped objects open clipped. Objects drawn under a clipping path that is not a rectangle (a disc holding a pattern of lines, say) kept the parts outside it whenever they lay inside the clip's bounding box, and every other object got a clip frame of its own, so a drawing came in with dozens of frames. Now the objects drawn under one clip go together in one clip frame shaped as that clip. Nested clips are their exact overlap, a clip read with the nonzero rule keeps an outline drawn twice (as some producers write it) or the middle of a star, and a thick outline is judged by where its ink reaches (#1616).

## 0.37.0 (2026-11-02)

- Eraser: a selected clip frame (or the one under the pointer) is erased. The band is taken out of its frame, which becomes a curve keeping its fill and outline, and the contents stay inside, clipped by what is left of the frame, so the frame and its contents disappear along the band and nowhere else. A clip frame erased away is deleted. One undo step. The Eraser did nothing on a clip frame (#1672).

- Transparency tool: a drag puts the gradient mask on the selected objects, all of them in one undo step. It went on the object under the press, so with an object selected a drag started where another object covered it, or over a part of the selected object made see-through by its mask, picked and changed that other object. A press on the selected object (also where another one covers it) or on no object now takes the selection; one on an unselected object outside the selection still picks it, and with nothing selected the drag takes the object under it as before. Ctrl (Cmd on macOS) keeps the line in steps of the constrain angle (Options, 15 degrees at first), with the Transparency tool and with Interactive Fill, whose page already said so (#1661).

- Zoom keys while editing text: F2 (the Zoom tool for one zoom), Shift+F2 (zoom to the selection), F3 (zoom out), F4 (zoom to fit) and Shift+F4 (zoom to the page) now work while a text is being edited, and the editing goes on: the caret and the selected characters stay and typing goes on into the text, also after F2's one zoom. The text took every key, so seeing the whole page meant leaving the text first (#1699).

- Distort, Twister: the turn now grows toward the centre and the farthest point of the outline stays where it is, so a twisted object keeps its place and its outer shape (a square's corners stay at its corners, a star's tips where they were) and swirls inside, as a drag round the centre expects. The farthest point took the whole angle before, so the object looked rotated as a whole and its orientation followed the pointer round. Twisters in saved drawings draw the new way (#1674).

- Distort, Zipper: the teeth lean the way the handle is dragged. A drag slanting away from the outline tilts the teeth by the angle between the drag and the outline's normal (a drag straight off a line keeps them upright), the handle line leans with them and a Tilt field on the bar shows and sets the lean; the teeth always stood upright on the outline before. Smooth teeth are now waves the shape of a sine through the tips, which run on along the outline between two tips however tall the teeth are; tall rounded teeth became loops that overlapped and joined each other (#1671).

- Workspace modernization: sharper, lighter tool icons. Strokes are thinner (1.25 px at the medium size) and fall on the pixel grid, so the icons are crisp at normal size. Crop, Knife, Eraser, Drop Shadow, Transparency and Interactive Fill are simpler symbols of the tool; the Rectangle is a true square with 90 degree corners; the Page menu pictures use the one blue accent. The status bar's Fill and Outline pictures follow (#1682).

- Number fields: the steps one field makes in a row are one undo step. Turning the mouse wheel over a field of the property bar, a docker or a dialog (size, scale, position, rotation, outline width, corner sizes and the others), clicking its spin arrows or dragging them made one undo step per notch or click, so going from 1 to 100 filled the Undo list with about a hundred "Transform" steps. Now the steps join while they come from the same field to the same selection with less than 0.8 s between them; Undo puts back the value from before the first step and Redo the value where they stopped. A pause, another field or any other command starts a new step.

- macOS: Command+Q, Quit TraceDraw in the application menu, the Dock icon's Quit and a log-out ask about unsaved drawings (Save, Don't Save, Cancel) as File > Exit and the window's close button do. They quit at once, so the changes since the last save were lost unless an auto-backup kept them; with nothing unsaved they still quit at once (#1669).

- Pick and Freehand Pick tools: Alt+drag draws the touching frame or lasso also when it starts over the selected object, so a small shape ordered behind a larger one that is selected (as right after drawing it) can be taken by an Alt+drag over it. That drag moved the front shape, so the hidden one could not be selected by touching. A drag without Alt still moves the selection (#1350).

- Shape tool: the corner rounding rings inside the corners of a curve being edited are now shown by default. They were off at first, so they seemed to be missing; turn them off in Options > Nodes and Handles > Show corner rounding rings on curve nodes. A choice already saved in the preferences is kept. Dragging a ring now rounds that corner alone (or the chosen nodes' corners when it is one of them); hold Shift to round every corner of the curve together, and Ctrl+click rings to choose several corners that round together (a chosen ring is filled). Options > Nodes and Handles > Round one corner at a time switches back to the earlier behaviour, where a drag rounds every corner.

- Rectangle corners: the four size fields on the property bar and in the Properties docker are the corners as they lie on the page. On a rectangle turned by 180 degrees, flipped, or stretched across its own side with a handle (as a flap is made from a panel), a size typed in the top left field rounded the rectangle's own top left corner, which lay at the bottom right; each field now shows and edits the corner seen at its position, and two rectangles whose corners match as seen are not marked mixed (#1670).

- Shape tool on text: the characters chosen with it take the format. Text Properties (Ctrl+T) changed the whole text, whichever characters were chosen; now the font, style, size, underline, kerning, background, strikethrough, OpenType features and font axes, a fill from the docker or a palette and an outline colour go to the chosen characters only, the docker's Character section shows the first chosen character, and the Paragraph settings go to the paragraphs holding them, as for characters selected while editing. With no character chosen a change goes to the whole text as before (#1662).

- Table: the Rows and Columns fields of the property bar change the selected table (rows at the bottom, columns at the right, the other cells as they were, one undo step each), and a table selected with the Pick tool shows the table fields after the object fields. The fields only set the counts of the next table, so an existing table's rows and columns could not be changed from the bar, and the Pick tool showed no table settings at all (#1680).

- Brush Strokes: the property bar has the mode list (Preset, Brush, Sprayer, Calligraphic, Expression) the hint speaks of, and after the width the mode's own fields: the stroke shape, the nib angle, the sprayer's spacing, the pressure. The bar showed only the calligraphic width and nib angle, so another mode could be chosen from the Brush Strokes panel alone (#1681).

- Options > Nodes and Handles > Handle size: Small (6 points), Medium (8) or Large (10, the size so far and the default) for the Pick tool's selection handles; the handles' gap to the selection box, their reach for a press and the text flow tabs follow the size (#1285).

- Floating toolbox: its window can be resized by its edges and corners, and the buttons reflow to fit it, live: a tall narrow window holds them two to a row (or one), a wider one more to a row in fewer rows, a wide short one in a row or two, with the separators between the tool groups kept (a vertical line between two groups in a row, a line between two rows) and the "+" button at the end. The window hugs the grid and the size is kept with the bar layout (in buttons, so it holds when the button size changes); before, the floating toolbox was always two buttons wide.

- Floating bars: a bar's name too long for its title (a long translation in a narrow window) is now cut with an ellipsis and shows whole as a tooltip of the title, and a window is never narrower than its close button and a few letters; the name overflowed the window before.

- Ctrl+wheel sizes: over the color palette, Ctrl+wheel makes the swatches larger (wheel up) or smaller (wheel down), 2 px a notch from 8 to 48 px, every palette row following; over a toolbar it steps the default button size, over the toolbox the toolbox's own button size, Small to Large. The sizes are kept in the preferences (older preferences keep 20 px swatches). The plain wheel still scrolls a palette or a bar that does not fit, and Ctrl+wheel over these panels does not zoom the drawing.

- Multipage View with several drawings open: each drawing now keeps its own pictures of its other pages. They were kept by page number for all drawings together, so after switching drawings the other pages first showed another drawing's pages until clicked (#1663).

- Browser version: a drag with the mouse button held, or a finger held down, on the drawing no longer selects the page's text in the browser (Safari on iPad selected the whole window while dragging a marquee or an object, and offered its long-press callout); the page and the canvas turn the browser's text selection off, and the loading text stays selectable so an error message can be copied (#1654).

- Pick tool: a click anywhere on a thick outline picks the object, out to 3 px beyond its visible edge, at the width the outline is drawn (scaled with the object when it scales with it). Only the 3 px about the outline's centre line counted, so an unfilled path with a 4 or 5 pt outline was missed by a click on the visible outline away from its middle and had to be marquee-selected; a thin outline is still picked within 3 px (#1678).

- Guidelines snap to the nodes of curves. A guideline dragged (or dragged out of a ruler) near a node, a segment's midpoint, an ellipse's quadrant or an object's centre now lands on it, by the snapping modes ticked in Options > Snapping, as the pointer does; only the edges and centres of the objects' boxes caught a guideline before, so with Nodes ticked a guideline still could not be put on a curve's node. The other way round, an object moved by a node under the pointer puts that node on a guideline (or a grid line) it comes near, where only the object's box edges and centre caught guidelines before (#1645).

- Tracing: the Low quality image and High quality image presets (and any trace asking for many colours) give clean curves on compressed clip art, as Clipart does: the fringes of a compressed edge, whose colours drift off the blend of the two colours either side and overshoot them, go to those colours instead of staying as slivers of their own along every edge (the doubled lines in Wireframe), shades of one colour no longer chain into a mean colour none of them had (a pink cheek came out as a yellowish orange, or vanished into the yellow round it), a small feature a dozen pixels across keeps its colour, and the glint in a dark eye, a few white pixels blurred into several shades, stays at the low detail that drops every other speck. On a 300 pixel JPEG of a cartoon bee, High quality image went from 1309 nodes and 14 colours to 646 and 14 with every line single, Low quality image from 313 nodes with bitten eyes and no cheeks to 155 with round eyes, glints and cheeks; a 1200 pixel photo traces as before (#611).

- Pick tool: a right click on a selected object keeps the selection and opens the menu for it, also where another object covers it (a small shape ordered behind a larger one, picked with Alt+click or a marquee); a right-button drag from it moves the selection too. The right click selected the object in front and opened its menu instead, so the covered object could not be cut, copied, ordered or given its properties from the menu (#1684).

## 0.36.0 (2026-11-02)

- Variable Outline on a closed path (an ellipse, a rectangle, a closed curve): the outline's width now runs on across the path's start, easing from the last width node round to the first as between any two nodes, and a node dragged along the path slides on past the start and comes round, the nodes keeping their path order. The path's start counted as two ends, where the width fell back to the plain outline's, and a node stopped there, so the outline could not be shaped all the way round (#1322).

- Shape tool: a marquee (rectangular or freehand) keeps the selected curve, also when the drag starts over another object, such as the one behind the curve or the group holding it, and chooses the curve's nodes inside the box. A drag that started over another object selected that object instead and the curve's nodes went away, so a box drawn round nodes on a background object left node editing; only with nothing selected does a drag select the object under the pointer first (#1594).

- Shape tool: Ctrl+drag on a node that is not chosen drags that node alone, kept to one axis, from the first drag, as a plain drag does; the same for a text's character nodes. Ctrl at the press added the node to the chosen ones instead, as Ctrl+click does, so the nodes chosen before moved along with it and Ctrl seemed to work only after the node had been clicked first. Shift+drag still adds the node (or the run up to it) and moves the chosen nodes together, and Ctrl+click still adds or takes out a node (#1325).

- Color docker: the eyedropper button shows the Color Eyedropper tool's own icon, as in the toolbox, instead of the outline pen's picture, and Fill and Outline under the swatch are framed buttons, not bare words (#1485).

- Banner Hems and Eyelets: the preview tells mirrored edges from stretched ones (diagonal hatching for the design flipped over the edge, streaks for the edge drawn outwards; the same grey band stood for both), and when the grommets have no room on the page (the margin and the grommet size reach past its middle) the preview says so and OK reports it in the status bar, where nothing was placed with the word "0 grommet marks" and no reason (#1332).

- Opening a damaged `.cdr` or `.cmx` file whose lists nest without end no longer takes TraceDraw down. The chunk walker reads a list's chunks by calling itself once per level, and a file can nest a list at every 12 bytes, so a few hundred kilobytes of lists inside lists ran the stack out, which cannot be caught, and closed the program before the file could even be refused. Lists now nest at most 256 deep (real files nest a few dozen levels: the document, a page, a layer, then one per group inside a group); a list deeper than that is read as a leaf, with a warning, and the rest of the file opens.

- A `.tdraw` file with groups inside groups some 30 deep opens again. The JSON reader stops at 128 nested brackets, and a group inside a group takes four, so a drawing with about 30 levels of groups (imported drawings and ones grouped again and again reach that) saved fine and then refused to open with "recursion limit exceeded": the drawing was lost. The reader now counts the file's nesting itself and reads up to 1024 levels (some 250 groups inside groups, far beyond any drawing and well within what the stack of every platform holds); a file nested deeper is refused with a clear message instead of a crash. Files saved before are rescued the same way.

- Opening a damaged PDF or AI file whose pattern tile paints with the pattern itself no longer takes TraceDraw down. A tiling pattern's tile is content of its own, and it was drawn at a fixed nesting depth whatever content asked for it, so a tile that filled a shape with its own pattern drew itself without end until the stack ran out, which cannot be caught. Patterns now count toward the same nesting bound as forms inside forms (12 levels); past it the pattern is left out with a warning and the rest of the page opens.

- Opening a damaged EMF file whose header bounds or frame span the whole range of their numbers no longer takes TraceDraw down. Their width and height were taken as 32-bit differences, which overflowed and panicked on such a header; they are taken in 64 bits now, and the file opens with the page size the header gives.

- Opening a damaged or hostile EPS, PostScript or AI file whose procedures nest without end no longer takes TraceDraw down. The reader of a `{ ... }` body called itself once per brace, so a file of a hundred thousand opening braces (100 KB) ran the stack out, which cannot be caught, and closed the program; procedures now nest at most 512 deep in the text (real programs nest a handful), the ones inside left out with a warning. A program running procedures inside procedures was stopped after 200 levels, but each level takes far more stack than that allowed for on some platforms (a 150-level program closed TraceDraw too); the program now runs on a thread with a stack of its own and is stopped, with the same warning, when its recursion has used the room it has, whatever its frames weigh.

- Opening or importing a damaged SVG file whose elements nest without end no longer takes TraceDraw down. The XML reader walks the elements by calling itself once per level, with no bound of its own, so a file of fifty thousand opening `<g>` tags (350 KB) ran the stack out, which cannot be caught, and closed the program. The nesting is now counted from the text first and a file nested deeper than 256 elements (drawings nest a few dozen at most) is refused with a clear message.

- Font list on Linux: the folders the filter sorts fonts into are read once. They come from the fontconfig configuration, which was parsed in full (over a millisecond, more with many configuration files) each time the list looked at a family it had not seen, so scrolling a long list or filtering it spent whole seconds reading the same files again.

- Shape tool: To Curve, To Line and Break act on the closing segment of a closed curve through its start node, the node that segment ends at, and on a clicked closing line. The commands took the element at the node's own index, and the start node is a move with no segment of its own, so the segment where a closed curve's last node meets its first could not be made a curve, straightened or broken, on every rectangle, ellipse and polygon converted to curves (#1618).

- Options > Customization > Commands: a shortcut given to a command or tool is taken from the command or tool that had it built in, which is left without one (Reset gives it back), so one key runs one command. The built-in holder kept the key: a tool key given to another tool picked both tools, and a tool given a menu command's key never got it, the command running instead (#1595).

- Clip to Frame: Ungroup, Break Apart and the Knife take a clip frame apart through its frame, each piece a clip frame holding the contents, so the picture stays where it shows. Ungroup on a clip frame whose frame is a group did nothing (#1605), the Knife left a clip frame alone unless it held one bitmap (#1606), and Break Apart on a clip frame whose frame is a combined curve did nothing (#1607).

- Exporting a drawing to EMF or WMF no longer takes TraceDraw down when a text size is not a number (a division by zero somewhere, or an import with a number too large to hold, can leave one). The font record of both formats holds the em negated, and a size of minus infinity clamped to the smallest value the record holds overflowed when negated; the size is now clamped to the positive half of the range first, and the rest of such a drawing (places, outline widths, sizes) exports as before.

- Drawing a page whose size is not a number (a division by zero somewhere, or a damaged file, can leave one) no longer takes TraceDraw down. Such a page asks for an image as tall as a number holds; splitting it into bands for the machine's cores, and cutting a strip of it for printing, added past the end of the range, and asking the allocator for a picture that large closed the program, which cannot be caught. The bands and strips now end at the image's last row, and an image of more than a thousand million pixels (4 GB, more than any export or print asks for) is refused as a render that failed, so the drawing stays open to be saved.

- A `.tdraw` file is saved with every number a number. A value that is not one (a division by zero somewhere, or an import with a number too large to hold, can leave such a value in a coordinate, an outline width, a transparency, a colour component or a font size) was written as `null`, which no TraceDraw before 0.35 takes for a number, so a drawing saved by this version with one such value anywhere refused to open in those versions and was lost there; it is now written as 0, as 0.35 reads it, so the file opens everywhere, and settings that are not there are still written as `null`.

- Contour with round corners: each rounded corner now runs on from the straight sides it joins, leaving one side along it and arriving at the next along it, at the contour's distance all the way round. The corner curves set off at an angle to the sides (about 14 degrees on a 1 mm contour) and pinched toward the corner, so the corners of a contoured text looked bent at the wrong angle and, on a small contour, crossed the curve opposite (#1542).

- Print Preview, Transform: dragging a corner handle scales the page keeping its proportions, both percentages alike, with the opposite corner staying put; the side handles still stretch one axis alone. A corner stretched the width and the height each by its own move, so a drawing fitted to the sheet by its corners came out squeezed or stretched (#1543).

- Print Preview: ticking Fit to page brings back a page that was dragged or stretched on the sheet. The page's move and stretch (Artwork position, Transform) stayed as they were, so a page dragged off its place kept sitting there with Fit to page on; choosing Fit to page in the Print dialog's Layout tab puts the page back the same way (#1577).

- Dependencies: rqrr 0.11 (lru 0.16, soundness advisory)

## 0.35.0 (2026-11-01)

- Visual modernization of the workspace: the default palette follows the classic order with plain colour names, and five swatches fewer among the first 50.

- Clip to Frame: Edit Contents (also by Ctrl+click) and Extract Contents take a clip frame apart where it shows. A clip frame moved, turned or resized after it was made keeps that in its own transform, which the commands dropped, so the frame and its contents jumped back to where the clip frame had been created and Finish Editing left them there (#1562).

- Space held down no longer leaves a trail of copies or flips between tools. A key held sends its press again and again, and every one of them counted: during a move each repeat dropped another copy of the selection, and on a tool Space went back and forth between the Pick tool and the tool before it as long as it was held. Space now drops one copy a press, and the tool keys pick their tool once; the arrow keys, Delete and the menu shortcuts still repeat while held (#1502).

- 3-Point Ellipse: the dragged line is the ellipse's centre line, an axis from end to end, and the third click sets how far the ellipse grows from it on both sides, as the tool's hint says. The ellipse was built like a 3-point rectangle, with the line as one of its edges, so it stood to one side of the line and the click gave its whole height (#1511).

- Number fields: Tab goes from one field's box to the next field's box in one press, in every bar, panel and dialog. The spin arrows beside each box, and the label before it, took the keyboard focus on the way, so Tab had to be pressed twice (three times with a labelled field) to reach the next value (#1537).

- Multipage View: every page shows its own guidelines, each other page's within that page where it sits in the grid. Only the current page's guidelines were drawn, so a guideline added on page 1 vanished as soon as page 2 was clicked and came back with a click on page 1, which read as the guideline being lost (#1448).

- Dependencies: zip 9 (security). The `zip` crate, which reads and writes the ZIP packages of `.cdr` files from version 14, `.tdraw` files, DOCX and XLSX imports and the packaged copies of a drawing, goes from 4.6.1 to 9.2.1 for RUSTSEC-2026-0369 (critical, GHSA-4684-h2q9-2vcp), RUSTSEC-2026-0367 (high, GHSA-vv5m-jxxh-4627) and RUSTSEC-2026-0368 (GHSA-4v9h-p7m9-rh26), all in the crate's extraction to disk, which TraceDraw does not use (every member is read into memory). The files written are the same (members, order, compression); a member whose name cannot be decoded is skipped when a package is searched for its RIFF stream, colour profiles or embedded fonts.

- Printing a blend: the steps print between the two objects as they are printed, in their printed colours. The sheets are made from a copy of the drawing whose objects are moved, shrunk and recoloured outside the editor (Fit to page, the selection printed as the page, a negative, text in black, spot inks as CMYK), and the blend was still drawn from the copy of its end object it keeps, left where (and as) it was on the page, so its steps ran off to the end's old place and the blend came apart on paper, in Print Preview and in a PDF or PostScript file printed to. Exporting the start object alone (Selected only) keeps the steps beside it too (#1541).

- Opening a damaged PDF or AI file no longer takes TraceDraw down. A gradient whose function gives no number for a colour (a negative exponent at 0 makes 0 times infinity) left a colour with a component that is not a number, and the screen colour conversion of such a colour called itself until the stack ran out, which cannot be caught: the program closed when the file was drawn, exported or printed. Such components are read as no ink, and the conversion of any colour now ends. A path segment before any start point in a content stream (a cut-off or overwritten stream) begins its path where it starts instead of stopping the import.

- A `.tdraw` file holding a value that is not a number opens again. A division by zero somewhere, or an import with a number too large to hold, can leave such a value in a drawing (a coordinate, an outline width, a transparency, a colour component, a font size); the native format writes it as `null`, which the reader refused as a number, so one such value anywhere made the whole file refuse to open and the drawing was lost. The reader now reads each such value as 0 and opens the file, with everything else as it was; files saved before are rescued the same way.

- Dependencies: security updates for rustls 0.23.46, rustls-webpki 0.103.16, webpki-roots 1.0.10 and webpki-root-certs 1.0.10.

- Visual modernization of the workspace: a new default colour palette. The 99 swatches come from a rule instead of a list: the blacks and greys, the six process colours, then twelve hue families (Rose, Red Orange, Orange, Amber, Lime, Leaf, Emerald, Teal, Azure, Indigo, Violet, Purple) each at 20%, 40%, 60% and 80% of its strength, at full strength and darkened with 30% black ("Orange 40%", "Dark Teal"), two browns and eight warm and cool greys, all with plain names and whole-percentage inks. The strip is still a full, smooth run of colour in the same hue order, and the RGB default palette follows with the same names. Drawings keep the colours they store, so nothing changes in a saved file; only the swatch names a new window shows are new. See `docs/behavior/color-palettes.md`.

- Visual modernization of the workspace: new status bar hints. Every tool hint in the status bar, the Hints docker's intro and topic names, and the empty document palette's text are written again in shorter, plainer words with one style: the parts are joined by " · " and each says what a click, a drag or a modifier key does ("Click again to rotate or skew · Double-click the tool to select all · Shift+click adds to the selection · Alt+click picks the object behind"). The same texts are translated in all 17 other languages (marked `# mt` for native review); the keys are unchanged, so nothing else in the program or the language files moves.

- Visual modernization of the workspace: new toolbox icons. Every tool, the flyout tools included, has a new icon in one style, "bead on a wire": ink draws the object, the brand blue draws the action (what the tool drags, creates or changes), and a small blue square, the node of the logo, marks the live point on its line. Bodies carry a light ink wash, dashes mean a ghost (the old curve, the piece cut off), and the same drawing serves every theme, the blue lighter on dark panels. The icons are drawn at any size, so they stay crisp at every button size and screen scale; the Hints panel, the tool rows of Options and the badge beside the cross hair use them too. Coloured tool icons keep their group's hue as the ink. The Variable Outline icon is new as well (an ink ribbon swelling in the middle with its width handle). See `docs/behavior/icons.md`.

- Visual modernization of the workspace: the Variable Outline button now sits under the brush tools (Brush Strokes, Painterly Brush, Shape Recognition, Sketch) in the toolbox, over the Rectangle flyout, instead of at the bottom after the Fill flyout. Its shortcut and behaviour are the same. The tool each button shows, kept in the settings, still comes back right from preferences saved with the old order.

- File > Import reads `.tdraw` files, so a `.tdraw` let go over the drawing window is imported into the open drawing like a `.cdr`: the objects of all its pages go into the active layer, selected, as one undo step, with the fonts the file carries. TraceDraw's own format was the one drawing format a drop on the drawing could not import: the file always opened in a tab of its own and the hint said "Drop to open" (#1142).

- Pick tool: one Alt+click on the object in front takes the object hidden behind it. With nothing selected under the pointer, Alt+click took the topmost object, so a shape ordered behind a larger one needed a second Alt+click, which read as Alt+click not working; the dig now starts from the topmost object (an object alone under the pointer is still taken) and goes round to the top again after the bottom one (#1350).

- Workspace menus and terminology modernized. The menu bar now reads File, Edit, View, Object, Text, Image, Effects, Page, Tools, Window, Help: Bitmaps is Image, Layout is Page and sits after Effects, and Table is a submenu of Text. The tracing commands sit under Image > Trace (Automatic, Repeat Last Trace, the Centerline and Outline presets, now named Line Art, Logo, Logo (Detailed), Flat Colors, Photo (Low Detail), Photo (High Detail), Technical Drawing and Sketch). Items go by plainer names: Export as PDF, Export Presets (Documents, Web), Scan or Camera, Data Merge, Document Info, Repeat Copies, Paste Style From, Smooth (view), Page Overview, Show Overprints, Preview Effects as Bitmap, Document Setup, Snap Outlines to Pixels, Combine Shapes, Bitmap Padding (Automatic, Custom), Color Mask, Indexed (8-bit), Use Web-safe Text, Insert Special Character, Text Frame, Proofing, Word Count, Text Encoding, Save as Defaults, Banner Hems and Eyelets, Merge Windows, Context Help, Getting Started, Mark New Features and Notifications; the workspaces are Standard, Minimal, Compact, Drawing, Publishing and Touch. Dialog titles, panel names and toolbar tips that echo a menu item follow it. Every command, its key and its shortcuts are unchanged, so saved shortcuts and workspace files keep working; the one command key renamed (the grammar checker's) is migrated when the preferences or a workspace file are read.

- Area Fill fills the curved side of a circle extruded with a Back type (Small back, Big back or Back parallel). The outlines of the side and the far face, cut where the filled circle hides them, came back a few hundred millionths of a millimetre off the circle's outline, below the grid the areas are cut on, so the side leaked to the outside and a click in it made nothing; the short lines that close such gaps now cross the outline they meet whichever way the ends round (#1281).

- Distort: the handle follows the pointer in any direction. A Push and Pull or Zipper drag put its handle on a line to the right of the centre whichever way the pointer went, and a Twister handle turned from the right, not from where the drag began; the handle now ends where the drag ends, in every mode, for a new distortion as for a handle taken and dragged. Push and Pull takes the drag's length (half the object's larger side being 100), pushed right of the centre and pulled left of it; the Zipper's slider runs along the line; a value changed on the bar keeps the line's direction (#1442).

- Print and Print Preview, Layout: the sketch, its sheet count and the booklet summary under it (pages, blank pages added, sheets, signatures) count the pages that print, the page range, the page list, odd or even pages or the selection, not the whole drawing. A booklet of pages 1 to 20 of a hundred-page drawing reported 100 pages and 25 sheets while 5 sheets printed (#788).

- Font list filter: the Folders section counts the system's font folders the way the font database reads them, so each is listed once. On macOS the fonts downloaded on demand, one folder each under `/System/Library/AssetsV2`, gave one entry per font; on Linux a library registered with fontconfig and sorted into subfolders gave one entry per subfolder; on Windows the user's roaming fonts folder and the system folder as `SYSTEMROOT` names it are now known (#867).

## 0.34.0 (2026-11-01)

- Opening a PDF or AI file nested far deeper than any real one (thousands of arrays, dictionaries or parentheses inside one another) no longer closes TraceDraw: the PDF reader (`lopdf`, now 0.46) stops at a fixed depth and reports the file as damaged. A file of about 20 KB could take the program down with a stack overflow, which cannot be caught (RUSTSEC-2026-0187, GHSA-6fj4-rcr3-r7rr).

- Roles in the `tracedraw` organization (contributor, reviewer, maintainer, owner) are set out in `CONTRIBUTING.md`, and `MAINTAINING.md` describes how reviewers and maintainers triage issues by priority, read attached files safely, keep feature requests for later, and review and merge pull requests. Maintainers commit under their own names; only the project's own account commits as TraceDraw Maintainers.

- The project is credited to TraceDraw members and contributors everywhere (README, About box, licenses, installers, packages, the browser page and the store listings); commits by the maintainers are made as TraceDraw Maintainers.

- The project lives in the `tracedraw` GitHub organization: every link to the repository, the issue tracker, the releases and the browser version (`tracedraw.github.io/tracedraw`) points there. The old addresses under `igtoth` redirect, except the browser version's page, which does not.

- A trademark notice in the README, `PRIVACY.md`, the About box, the browser page and the store listings says that TraceDraw is an independent project, not affiliated with the owner of the `.cdr` format's trademark, and that `.cdr` support is implemented independently for interoperability. VBA macros reach the application object as `Application` only; the alias named after that program is gone.

- `tracedraw-core`: `shaping::overlay_with` combines outlines read with the nonzero rule as well as the even-odd one (`FillRule`), so text-like outlines need no clean-up before shaping; `overlay` stays even-odd.

## 0.33.0 (2026-10-31)

- Saving .cdr with Embed fonts: each embedded font is described as the face its file holds (its own weight and slant in the font table, the embedding and the font header), not as the style the text asks for, and the kind of font (TrueType, signed TrueType, PostScript outlines) and its version are named as the program that reads .cdr files names them. Text set bold italic in Khand Light went out as a "bold italic" Khand Light that was really the upright light font, and that program drew it light and upright; regular Khand text drawn with the bold Khand file was listed as a regular face, and that program reported Khand missing although it was installed. Bold text in a light static face such as Khand Light is now drawn and saved with the family's bold face (Khand Bold) when it is installed, and underlined faces carry the underline in their font records, as that program's own files have them (#1455, #1484).

- Weld, Trim, Intersect and the other shaping operations give one node where two outlines cross at a shallow angle, instead of a node followed by one or two lines of no length.

- Linux (X11): a picture copied in a screenshot tool pastes on the first Ctrl+V. The paste gave the program 10 ms for each part of a large picture and asked for PNG only, so it often came out empty and took two or three tries; it now reads the clipboard's list of types, asks for the best picture type offered (PNG, BMP, TIFF, WebP, GIF, JPEG), gives each part 2 s and tries again briefly when nothing came. One clipboard connection is kept for the session (#1488).

- Saving as `.cdr`: guidelines are saved and come back when the file is opened again. The guidelines every page has go once on the master page's guides layer, as the other program's files keep theirs, and each page's others on that page. They were not written at all (the status line said so); their colour, style and lock are still not written (#1161).

- Android: online features work (AI Drawing, AI Generate, GitHub, dictionary downloads). Every HTTPS request failed its certificate check, because the system verifier the desktop builds use needs a Java part the app does not carry; the app now checks certificates against Android's own certificate store, with bundled roots when it cannot be read. The CI emulator check sends a request and wants it answered (#1396).

- Reduce Nodes and the Curve smoothness slider keep the curve within the tolerance all along it: a node only goes when the whole changed stretch stays close to the original, so a wobbly or traced curve is no longer flattened to a single node, and a smooth curve comes down to the fewest nodes (an ellipse of 12 nodes to 4). The start of a closed curve can go too, and long traced curves reduce much faster.

- Android and touch screens: a tap with the Text tool starts a text where it lands and the on-screen keyboard comes up, so Chinese and every other language type straight onto the page, not only into a text box drawn first. A finger's tap reached no tool: the release comes with the pointer leaving, so the frame of the click had no pointer position (Windows and Linux touch screens and browsers on tablets had the same problem). The keyboard also comes back when you tap into the text after hiding it, and no longer hides for a single frame without the caret, which ended what the input method was composing. The CI emulator check now taps with the Text tool and types Chinese through a keyboard and Latin letters as key presses (#1489, #1496).

- Text > Convert: artistic text turned into paragraph text, or back, keeps its letters where they were. A text with a position, size or turn of its own (every text of an opened `.cdr` file) jumped away, because its new frame was placed from its box on the page as if that were its own space (#1238).

- Saving as `.cdr`: paragraph text is saved as paragraph text, its frame and the text flowing in it, as the other program's own files hold it (versions 7 and 9 to 27), and opens again as paragraph text in TraceDraw. It went out as one artistic text per line, so the frame was lost in the other program and when TraceDraw opened the file again. Frames those files cannot hold as TraceDraw draws them (scaled or slanted, vertical text, fit to frame, placed from the middle or bottom, linked frames, several columns before version 16, version 8) still go out as lines. Columns are kept from version 16 (#1226).

- Tracing: where two colours meet, both shapes use one and the same curve, so the Wireframe view shows each border once and no gap or overlap is left between neighbours. Each shape was fitted on its own and reached 2 px under its lighter neighbour, so two curves ran side by side along every border, a pixel or two apart, with twice the nodes (#1250, #611). A colour inside another's holes (the eyes and cheeks in a head) is drawn over it, and the colour around is one ring with no holes under them; colours side by side meet along their shared curve. Remove object overlap now keeps every shape to its own pixels, holes and all. On the bee of #1250 Quick Trace gives 6 curves of 342 nodes where it gave 9 of 750, and the borders between its colours are single curves, as in the trace of the same picture made by the program that wrote the sample.

## 0.32.0 (2026-10-31)

- Tracing: traces look like the picture with far fewer objects. On a sugar skull with sunflowers (halftone dots in the flowers' hearts, fine white hatching, a black outline drawing, a picture with wide transparent margins) the Clipart preset now gives 16 curves where it gave 31, and draws 7.3 % of the picture's pixels visibly wrong where it drew 10.6 %; High Quality Image gives 46 curves where it gave 94. Small dots and hatch ends that stand out from what is around them stay (every patch under the detail's speck size went to the colour around it, so halftones came out flat); the shades the colour reduction made of one colour's noise and of its edges join that colour, which keeps the art's exact value (the black came out in three shades and #0B0703, the white in seven); in art drawn in flat colours the palette comes from the flat colours, not from the anti-aliased pixels of the edges; and each colour is drawn whole under the colours over it, its holes filled, so a black outline drawing is one silhouette of 1 ring where it was 217, with no page showing between neighbours.
- Tracing: a new Optimized preset, which Quick Trace now uses, chooses the detail for each picture: it traces samples of the picture's finest parts at several details, compares each trace with the picture and keeps the one that draws it best for the nodes it takes (16 curves and 5.8 % drawn wrong on the skull, in under a second on the desktop). The Trace dialog has a Best optimization button that runs the same search on the picture in the dialog and puts the detail found in the Detail slider, as a step undo takes back.
- Trace dialog: OK and Cancel sit in the bottom right corner; they followed Options... at the left.
- Trace dialog in the browser: the preview traces the art without its transparent margins, at up to 360 pixels, so a drawing in a wide picture shows its detail instead of blobs, and a note in red says the preview is reduced, that OK traces the whole picture and that the desktop app shows it exactly.
- Browser version: on Windows, the message shown at start (and the one at a macro command) has a third button, Get it from the Microsoft Store, and says that the Store version may be one or more releases behind the latest download while the Store certifies each version. The close button of these messages now only closes them; it opened the downloads page.
- Opening a file from the system: on macOS a drawing opened from Finder (double click, Open With, the Dock icon) now opens, in a tab of the running window; before, TraceDraw came to the front but the file did not open. The Android app is offered for opening and sharing drawings, pictures, SVG and PDF from other apps, and they open in tabs of the running app; the browser version installed as an app is offered for .tdraw, .cdr, .svg and .pdf files and opens them in the window already open. Linux and Windows worked already (#646).

- Docker tabs: a tab dragged along its column and dropped between two others goes there, a line showing where it will go, so the tabs of a column can be put in any order; dropped on the tabs of another column it joins that column at that place. The drop did nothing before (#1051).

- Extrude: an object without a fill whose body takes its colour is drawn as a wire frame, every side and the far or near face as outlines with no fill, so a box shows all twelve edges and a cylinder its two ends and the two lines along its side. The far face of a back body is drawn again behind a filled object (0.29.0 left it out) and is a part of its own when the body breaks apart. Area Fill takes only the lines that show: an outline under an opaque fill no longer cuts the area, so a click in a side of a Small front, Big front or Front parallel body fills the whole side instead of the pieces the hidden object's outline cut it into; through an object without a fill every line still counts (#1281).

- Distort: a drag can start anywhere with an object selected, off the object too, and the distortion is centred where it starts; it had to start on the object. A drag that does not start on a handle now adds a new distortion over the ones the object has, so distortions build up drag after drag, instead of replacing the last one. The handles and the bar edit the newest, Clear Distortion takes them off newest first, and Convert to Curves makes all of them the outline (#1442).

- Master page: the New Master Layer menu of the Objects docker and Options > Page have a choice per kind of master layer (all pages, odd pages, even pages): "In front of the page's layers" puts the drawing's master layers of that kind over the page's own objects in one undo step and makes new ones in front too; the choice is kept in the preferences, and each master layer has its own In front tick in Options > Page. The Objects docker lists every page with its layers (a button in its bottom bar shows only the current page again); a click on a layer or an object of another page goes to that page. An object's context menu moves it, or the selection, to a layer of any page or of the master page, so an object of a master layer can be put on one page; a layer's menu adds New Layer, Select the Layer's Objects, Show, Print and Lock ticks, and Move to Master Page or Move to the Current Page (#1132, #1195, #1477).

- Moving nodes with the Shape tool (two or more at once, or any change that moves the object's box), editing a B-spline or cropping with the Crop tool keeps a gradient transparency and a gradient fill where they lie on the page: what is left of the object keeps the part of the ramp that was over it, and the rest is cut off. The transparency and the fill started again across the new box, so a cut-down object was squeezed back to fully transparent at one edge (#1491).

- Android: Print, Send To, Edit Bitmap and opening files go through the system. Print hands the print PDF to the Android print service, whose dialog picks the printer, copies and paper; Send To saves to Documents through the save picker and to a mail recipient through the share sheet (Desktop is greyed, Android having none); Edit Bitmap passes the picture to an editing app that can save it back, which is then read again; Help, links and picked documents open in their apps. They all ran the desktop's commands before and failed (#1398).
- Font Manager: a font added on Android or in the browser is kept for the next start. On Android the picked font is copied into the app's own storage and installed from there; in the browser it is stored in the page's database and installed again when the page opens, and can be uninstalled. Both were lost when the app closed (#1400).

## 0.31.0 (2026-10-31)

- Welcome Screen: on Get Started only the recent documents list scrolls; New and Open stay in place (#1383).
- Glyphs panel: the Recent glyphs box is two rows tall instead of filling the panel, so the font list and the grid show at once; a glyph size slider and a Copy button sit under the grid (#1320).
- Hints panel: every tool choice shows that tool's hints, also choosing the same tool again; effect, fill and drawing tools keep their hints when an object is clicked (#1321).
- Step and Repeat: the copies Apply would make are outlined on the page while you work in the panel and follow every value; Apply stays one undo step (#1197).
- Guidelines: a locked guideline shows no rotation handles, and Clear and colour or style changes in the Guidelines panel leave locked guidelines alone (#1385).
- Browser version: Alt tapped alone no longer moves the keyboard focus to the browser's menu (#1407); pictures and files copied in other apps paste, also on Android (#1403); the shelf is kept in the browser's database and a full or refused storage is reported instead of losing items silently (#1414).
- Spelling: dictionaries are found in more system folders, Windows' own spell checker is used, and dictionaries can be downloaded (42 languages) or added from files on every platform, the browser and Android included (#1405).
- Browser version: the build compiles again (the web build of main had failed since the Windows printing change).
- Android: the app no longer freezes for good when the system relaunches it. A change of the system's resource overlays (new wallpaper colours, another icon shape, a display cutout switched on) or of the screen density relaunches the activity, and the app then stayed on the system's starting window, never drawing; an Android 14 emulator does that right after booting, which failed about one CI start in four (and both runs of the 0.30.0 release). The app now backs its drawings up, ends its process and is started again by the system, offering them; the other configuration changes it follows by itself (font size and weight, colour mode, touch screen, mobile network) no longer relaunch it at all. The CI emulator check relaunches the app on purpose and wants it back.
- Knife: bitmaps are cut too. A cut across a picture (or a cropped one) splits it into pieces that each show the whole picture, its pixels untouched, behind an outline of their own along the cut, as a cropped picture is kept, so the Shape tool edits each outline; with Split the outlines meet exactly and the picture shows no seam, with Gap they are that far apart, and Keep as one object leaves one picture with an outline of several parts. A turned picture is cut where it is drawn and the pieces keep its turn and effects. The Knife cut only vector objects. One undo step (#1333).
- File > Export: an export range for every format: the current page, all pages, or a list of pages such as 1-3, 5. PDF, AI, PostScript, PRN and HTML write one file with the pages chosen, so Export makes a multi-page PDF of any pages; SVG, EPS, DXF, EMF, WMF, PLT, PSD and the bitmap formats write one file per page, named with the page number (name-1.png, name-3.png). SVG and the other formats of one page wrote only the first page even with All pages ticked, and PDF had no page choice at all. Export to PDF also has Export all text as curves, for a file that needs no fonts; Publish to PDF stays as it was (#1382).
- Printing on Windows: the sheets go to the printer at its own resolution (the one set in its preferences, between 300 and 1200 dpi), so fine lines and small text print as sharp as on macOS and Linux. Every Windows print was a 300 dpi picture of each sheet. Each sheet now goes in strips with its blank parts left out, so a sheet never needs one huge picture in memory and empty paper adds nothing to the job; General > Print as bitmap still sends its own resolution (#1404).
- Properties docker, Transparency: the merge mode is a full-width list at the top, then a row of type buttons (No transparency, Uniform, Gradient, Vector pattern, Bitmap pattern, Two-colour pattern, Texture) with the chosen type's title, and a transparency picker: a swatch with a drop-down grid of ready-made transparencies (uniform levels, gradients, pattern tiles, textures) and Edit Transparency, which switches to the Transparency tool. A gradient has type buttons, its stops on a bar (click to pick, drag to move) with the picked stop's opacity and position and add and remove; a two-colour pattern or texture has foreground and background transparency with a swap button. What it applies to is three buttons, Fill, Outline and All, under "Transparency target". Vector and bitmap pattern transparency can now be chosen. Shared code: desktop, web and Android (#1439, #1203).
- Options > Tools > Zoom/Pan: Zoom relative to 1:1 (on: zoom 100% follows the ruler calibration; off: 96 pixels per inch), Center mouse when zooming with the mouse wheel (the spot under the pointer comes to the middle of the view, the pointer with it), and a Pan tool section whose right mouse button opens the context menu or zooms out.
- Options > Tools: the pages come in a new order (Pick, Eraser, Zoom/Pan, Freehand/Bézier, Shape Recognition, Rectangle, Ellipse, Polygon, Graph Paper, Spiral, Table, Dimension, Connector, Mesh Fill); Curve tools is now Freehand/Bézier, and new Shape Recognition (recognition and smoothing levels), Dimension (precision, units, prefix and suffix of new dimensions), Connector (corner radius) and Mesh Fill (columns and rows) pages edit the same defaults as the property bars, kept across restarts and put back by Cancel.

## 0.30.0 (2026-10-31)

- Saving .cdr in version 8: the style table no longer holds the table that only the bidirectional variant of version 8 and later versions have, so the file has the layout of the files version 8 itself saved. The version 8 file of the last test bundle crashed a 2010 release of the program that reads `.cdr` files on opening, while the same drawing's files of versions 7 and 9 to 15 opened (#1139; not yet confirmed in that program).
- Saving .cdr (versions 16 to 27): text with several colours in one line is written as the files of the program that reads `.cdr` files hold such a line: first one plain record for the letters in the line's own look, then one record per other colour, its style complete (fill, angle, outline, the three fonts, shifts) and marked as applying, each letter naming its record. 0.28.0 marked every run as taking the line's style, and that program drew the line in one colour; 0.29.0 marked every run as applying but kept one record per run, with no plain record first and styles lacking members that program's records always hold, and a line of more than 127 runs (each letter its own colour) wrapped the letters' record numbers (#1297, #1347, #1361, #1379, #1238, #1226).
- Saving .cdr (versions 7 to 15): a line of more than 128 colours keeps the colours of its first 128 and gives the letters of the others the line's first colour; every further colour got a record of its own that no letter could name (a letter names its record in 7 bits), and those letters took the 128th colour.
- Opening .cdr (versions 16 to 27): a text whose run record carries no marks at all (a line the program that reads `.cdr` files saved as one run) reads its letters again; the style string after such a record was not read, and the text came out empty or garbled.
- Tracing (Quick Trace, Outline Trace and the Trace dialog) of flat art from a compressed image, a screenshot or a video frame: each colour comes out as one clean region with smooth curves and few nodes. Noise split one colour into several shades that each became regions of their own, the soft and anti-aliased pixels along each edge were handed out unevenly, and one-pixel bumps were fitted node by node, so edges came out jagged, notched and speckled, with slivers of in-between colours along them and hundreds of tiny pieces in the wireframe. On the cartoon bee of the reports the Clipart preset now gives 342 nodes in 36 subpaths where it gave 5874 in 786, and 664 nodes where it gave 8403 on the other one, with its small letters still readable. Art on a transparent background also keeps all its colours: with nothing on the picture's edge to take for the background, Remove entire color took out the last colour of the palette, which could be the yellow of the bee's head and stripes, leaving them white with only their fringe as outlines (#611, #1250).
- Tracing (Quick Trace and Outline Trace) of flat art saved as a compressed image or a screenshot: each colour comes out as one region with smooth edges. Noise split one colour into several shades that each became regions of their own, the anti-aliased edge pixels were handed out unevenly, so edges came out notched and stepped with pale flecks along them, and small letters lost their thin strokes. On a cartoon bee the Clipart preset now gives 109 curves where it gave 1353. Photos trace as before (#611).
- Browser version on Windows: AltGr types its characters (€, @, [ ], { }, \ and the letters of Polish, Czech, German and other keyboards) in text and in fields. Windows browsers report AltGr as Ctrl and Alt held together, so the page kept these keys from the browser as Ctrl shortcuts and nothing was typed, and AltGr+A selected all or AltGr+E exported instead (#1408).
- Browser version: closing or reloading the tab, going back or leaving the page with unsaved drawings asks first with the browser's own question, and auto-backup and session recovery now work in the browser too. Backups are kept in the browser's storage for the site (a folder of their own per tab), written every few minutes, when the question shows and when the page is hidden, and the next visit offers what a closed, reloaded or crashed tab left. Before, the page closed at once and the work was lost (#1395).
- Auto-backup: when the system quits TraceDraw without asking about unsaved drawings (the Quit of the macOS menu, logging out) or the Android app goes to the background, the drawings are backed up at once and the next start offers them; before, a quit removed the backups and the Android app wrote none until its next auto-backup minute. An idle window now keeps its session marker fresh, so a second window never offers its drawings (#1395).
- Start-up: the splash screen shows at once and says "Loading fonts..." while the list of system fonts is read, its bar moving, and the editor opens when the list is in. The editor waited for that list on the window's own thread, so on a computer with many fonts or a slow disk the window stayed blank, or did not show at all, for as long as the read took (minutes on some Linux systems), and then the splash flashed up broken before the Welcome Screen (#1388). Without the splash screen the window shows at once, plain, until the editor is ready. The log now says how long the fonts took (`system fonts read in N ms`). The Welcome Screen also asks the disk whether each recent drawing is still there on a worker thread: it asked about every one every frame on the window's thread, so a drawing on a network share that did not answer held the window.
- Windows: a tool picked with its shortcut over a still mouse shows its pointer at once, and the pointer never vanishes. Picking the Pen tool kept the arrow, and leaving the Pen tool (or any tool with a drawn pointer) left no pointer at all, until the mouse moved: Windows hides and shows the pointer only at the next mouse move. A drawn pointer now swaps the system pointer for a fully transparent pointer image, which Windows applies at once, instead of hiding it (#1124).
- Colour palette between panels: a side of the window can hold two columns of panels, each with its own tab strip (drag a panel's tab just inside its column, over the drawing, or pick Dock at Right, Second Column from its menu), and the colour palette, with the further palettes shown, docks as a column between them: drawing, Properties, palette, Color Styles, window edge. Dragging the palette shows the blue band where the two columns meet; its menu has Dock at Left, Between the Panels and Dock at Right, Between the Panels. The panels in each column and the place of the palette are kept in the preferences and come back at the next start.
- Android: the on-screen keyboard types Chinese, Japanese, Korean, Tamil and every other script its input methods compose, into text, table cells and fields; only the Latin letters of its key presses arrived before (#1309).
- Typing in older Tamil fonts that set the letters at the places of Latin ones (such as SR-Tamil fonts): Autocorrect no longer turns the first letter of a sentence into a capital or a quote into a curly one, each of which is another letter in such a font; the characters at code page places 0x80 to 0x9F type, show in the Glyphs docker and draw; and a letter one character map of the font lacks is drawn from the other instead of as an empty box (#1358, #1271).
- Input methods: their candidate window opens beside the caret instead of at the corner of the drawing, and on Windows the letters that Tamil and other Indian input methods change while typing fast no longer delete the letter before them (#1271).
- Properties docker, Transparency: the amount is a full-width slider with its label above, like the Fill section's angle and edge pad, and a fountain transparency's angle is the same angle slider. The property bar keeps its compact controls.
- Options > Tools: each page in the list shows its tool's icon. The Zoom/Pan page has a Zoom tool section and a Calibrate Zoom... button that opens the ruler calibration (Display page).
- Saving `.cdr`: the objects of master layers are kept. Each page a master layer shows on gets them on a layer of the same name, behind or in front of the page's own objects as drawn; they were left out of the file. The Objects docker says "In front" after a master layer drawn over the page's layers (#1132).
- Colour palette: a right click whose pointer slipped a few pixels on a colour sets the outline; it applied the fill. Only a drag with the left button carries a colour away to drop it (#1336).
- Free Transform: a drag let go off the canvas (over a docker) is still one undo step, so one Ctrl+Z puts the object back; it left an extra step that did nothing. Free Scale (Alt) no longer blows the object up on the first move of a drag, and a long drag no longer keeps a copy of the drawing for every pointer move until the button is up (#1342).
- Uniform transparency on a group: it was stored but never drawn, so a grouped object stayed opaque and a `.cdr` saved from it lost the transparency. It now goes on each object of the group, as if set on each one (where they overlap they show through each other, as when the transparency is set before grouping), on the canvas, in pictures, SVG, PDF and saved `.cdr`, and Ungroup keeps it on each object. SVG export also keeps the uniform transparency of a single object, which it dropped (#1386).
- Saving .cdr: a clip frame (Place Inside Frame) is linked to its contents as the program that reads `.cdr` files links its own: the group of the contents holds its own number plus one, as every such group in that program's files does. TraceDraw wrote 2 for every group, so a drawing with text before the frame, or with two frames, held links that point at the wrong number. Files of version 11 now hold clip frames as clip containers too, as that program's version 11 files do, instead of a plain group of the frame and its contents. `tracedraw-cli check` reports a group with the wrong number (#1226).
- Saving .cdr: a second difference between TraceDraw's pictures and those of the program that reads `.cdr` files, besides the node marks of 0.29.0, that can make that program close when a picture is clicked (#1226, #1381, #1268, #1222): every object without an outline (pictures, and shapes and text drawn without one) named in its style string an outline type that no file of that program holds; it now holds the outline that program's own files give such objects (every setting, width 0, no type). `tracedraw-cli check` reports the old outline type, and checks a picture's outline node by node for the node marks instead of its first node alone.
- Text being edited: Edit > Cut, Copy, Paste and Delete, the toolbar's buttons and the right-click menu act on the selected characters, as Ctrl+C, Ctrl+X and Ctrl+V do. Edit > Copy used to copy the whole object as SVG, and a Ctrl+V in the text then typed that SVG code into it. Copied characters keep their font, size, colours and outline when pasted into a text or with the Pick tool, where they all took one colour before; other programs get their plain text (#1377, #1378).

## 0.29.0 (2026-10-30)

- Drop Shadow: a change on the property bar (Flat or an edge, offset, opacity, color...) applies at once to the selected objects, also to an object with no shadow yet, so a flat shadow can be given directly; it only set the default for new shadows before. The shadow handles are larger and can be grabbed from a little outside them. Native ellipses, polygons and stars are checked to cast and pick their shadow by their own outline, the same as after Convert to Curves. Shared code: desktop (Linux, Windows, macOS), web and Android (#1323).
- Number fields: dragging left or right on a field's arrows, or on its label, slides the value down or up, a step every few pixels (Shift for bigger steps, Ctrl for finer ones) (#1303).

- Extrude: each side of the body is one object, filled with the side's colours (a curved side with a fountain that follows the light) and outlined once, so a square's body is its two sides and a circle's its one curved side, to colour, paint with Smart Fill or break apart as whole parts; the far face, hidden behind a filled object and its sides, is no longer drawn. Before, a curved side was a row of dozens of slats (#1281).
- Dialog boxes: Tab goes to the first box you can type in and on through the dialog's controls, and Shift+Tab back, without leaving the dialog. Tab with nothing focused (a dialog just opened) or from a dialog's last control went to the menu bar, so the File menu took the next keys (#1258).
- Linked paragraph text frames: a picked frame now has a tab over its top edge too; click it, then click another frame to make that frame's text flow into this one, or click (or drag) on empty space for a new frame that the text starts in. A drag from the bottom tab onto a frame or onto empty space links as a click there does, and while a link waits for its target a dashed line runs from the tab to the pointer.
- Area Fill: the Fill and Outline lists on the property bar now show Specify first, then Use default, then None. The choice kept in the preferences is unchanged.
- Saving .cdr: text with several colours in one line keeps them in the program that reads `.cdr` files. Each run of another colour was written with a style of its own but marked as a run that takes the paragraph's style, so that program left the style out and drew the whole line in one colour, embedded fonts or not; such runs are now marked as their files mark them (#1297, #1347, #1238).
- Browser version on an iPhone or iPad: Open and Import offer every file, so `.cdr`, `.cmx` and `.tdraw` files can be picked. The system greyed out the files whose type it does not know as soon as the page listed the extensions it takes, so only files such as SVG and EPS could be chosen (#1311).
- Font list filter: the Folders section lists the folders the Font Manager has, each folder added to the libraries and the system's font folders, once each, whatever subfolder a font file lies in; every subfolder was listed on its own (#867).
- Trace Bitmap dialog laid out anew: the Preview list offers Before and After (stacked or side by side), Before, Large Preview (the trace alone, as large as the preview area) and Wireframe Overlay, which shows an Overlay slider beside the list for how strongly the bitmap shows under the curves (80 at first). The settings sit in a white panel at the right, each label above its slider and the sliders stretched to the panel's width, with Reset, undo and redo under the panel; controls that do not apply are greyed instead of hidden (a centerline trace's corner smoothness, Remove and Objects; Group by color while Merge adjacent is on). The other dialogs with a Before and After preview (Remove Background, Black and White, Duotone, Paletted, every bitmap effect, Image Adjustments) take the same panel, and all of them get Reset with undo and redo of their settings (a slider's drag is one step). The Help pages on tracing describe the new layout in every language.
- Colour palette: it now also docks beside the drawing at the left and the right (between the drawing and the docker column), and across the whole window at the top and the bottom, besides the places it had. Dragging it shows a band at the place it would take, along the window's edge or along the drawing's, and its menu lists every place. The place is kept in the preferences.
- Free Transform: one drag is one undo step, so Undo puts the object back as it was (it went back one pointer step only), and a quick diagonal skew no longer mirrors the object, text included (#1342).
- Colour palette: every click on a colour applies it to the selection. A click whose pointer slipped a few pixels on the swatch was taken for a drag and did nothing, and after a hold that opened the shades the next click on any colour was lost (#1336).
- Browser version: a right click during a move, scale, rotation or skew with the left button let go first ends the drag with its copy; the browser sent no release for the left button then, so the object stayed on the pointer until Esc (#1334).
- Attributes eyedropper: copies transparency (uniform, or a gradient, pattern or texture mask with its merge mode), on by default in the Properties menu; it copied the colour only. The Transparency tool's bar and the Properties docker have a Clear Transparency button (#1338).
- Ctrl+Shift+A opens Copy Properties From: a dialog to tick what to copy (outline pen, outline colour, fill, text properties), then a click on the object to copy from; the shortcut did nothing in the browser. The Align and Distribute panel keeps its menu entry and button and no longer has Ctrl+Shift+A (#1345).
- Clip frames: editing the contents ends with a Ctrl+click off them, besides the bar's Finish; Alt held while objects are dropped on a frame puts them inside, as W does (#1337).
- Selecting with Alt: Alt+drag with the Pick and Freehand Pick tools starts the touching frame or lasso also over an object that is not selected (it moved that object, so a selection could not start over a background); with the Shape tool Alt+drag draws a freehand node marquee, Alt+Shift adds to the chosen nodes (#1341, #1350).
- Saving .cdr: clicking a picture in the saved file no longer makes the program that reads `.cdr` files close at once. The outline of every bitmap object was written without the node marks curves carry, which that program's own files always have; `tracedraw-cli check` now reports such outlines (#1226).
- Browser version on an iPhone, iPad or Android device: Open and Import offer every file, so `.cdr`, `.cmx` and `.tdraw` files can be picked. The system greyed out the files whose type it does not know as soon as the page listed the extensions it takes, so only files such as SVG and EPS could be chosen (#1311).
- Distort, Push and Pull: a straight side bends from its middle (its control points start on its nodes and move along the lines from the centre), so pushed corners become spikes and pulled corners curl with the side's lobe reaching past them; the bend started at the corners (#1264). Twister: the handle is free, it reaches the pointer during a drag and stays as long as it was let go, instead of a fixed length (#1263).
- Typing on Windows with AltGr (Polish ą ć ę ł ń ś ź ż, the extra letters of Czech, Croatian, Hungarian and other layouts, German @ € { } [ ]): the letters type into artistic and paragraph text and into fields. Windows reports AltGr as Ctrl and Alt held, so its keys selected all (and the letter replaced the text), aligned or resized the text, or became copy, cut and paste, and the windowing layer dropped their characters (#1271).
- Convert Colors to RGB or CMYK: the Color Eyedropper's readout and the Color panel show a converted object's colour in the model it went to. The readout always showed the drawing's colour mode (CMYK by default) and the panel kept its sliders' model and did not read the fill again, so an object converted to RGB still looked CMYK (#1254).
- Blend between an object and a turned copy of it: every step turns by the same share and keeps its size. The steps were mixed point by point, so they shrank toward the middle and turned unevenly, the first and last most of all (#1260).
- Text: a font picked for several text objects changes only their font. Each kept its own size before only when the bar showed it, so texts of 9 and 11 pt all became 11 pt, and bold or italic went to all of them too; now only the setting changed goes to every selected text (#1304).
- Break Apart (Ctrl+K) on vertical text: its pieces stay in their columns, one word or letter under the other and the columns right to left. They came out side by side in a row, as if the text had turned horizontal (#1303).
- Palettes: opening a palette file adds that palette in a row of its own next to the default palette, and opening the same file again does not add a second copy. The opened palette took the default palette's place, which moved to a row of its own, so two palettes seemed to come at once. The Palette Manager has a Lock button for each palette that is not built in: a locked palette cannot be changed in the Palette Editor until it is unlocked (#1343).
- Shape tool on rectangles, ellipses and polygons: Ctrl counts while a node is dragged, also when pressed after the button. On an ellipse's pie or arc node it now steps the angle by the constrain angle (15 degrees at first), and on a star's inner node it keeps the points from turning; before, Ctrl was read only at the press and only for rectangle corners (#1325, #1331, #1340).
- Master layers: an object on a master layer can come over the page's own objects. To Front of Page (Ctrl+Home) on it puts its master layer in front of the page's layers, To Back of Page (Ctrl+End) behind them again, and a master layer's menu in the Objects panel has In front of the page's layers. Before, master objects always stayed under the page's objects and Order could not change it (#1132).
- Export: PDF, PostScript and PRN export every page by default (All pages starts ticked), so a multi-page drawing no longer comes out with only the current page; a format last exported with the current page alone keeps that choice (#1161).
- Save to `.cdr`: a drawing with guidelines adds "Guidelines are not saved in .cdr files; save as .tdraw to keep them." to the status line, since the `.cdr` writer cannot write them yet; `.tdraw` files keep every page's guidelines (#1161).

## 0.28.0 (2026-10-30)

- Variable fonts with many axes (Roboto Flex) draw their text: the font was listed and its axis sliders showed, but every letter came out empty because the glyph reader refused glyphs with more than 32 variation tuples (#870).
- Fit Objects to Path: seven more orders, Top to bottom, Bottom to top, Tall to short, Short to tall, Wide to narrow, Narrow to wide and Random (shuffled the same way each time for the same objects, so the preview and Apply agree) (#1256).
- Opening .cdr: Chinese, Japanese and Korean font names in the style strings of version 17 and later files that a system in that language wrote (GBK, Big5, Shift JIS or Wansung bytes rather than UTF-8) read as written and find their installed fonts, instead of showing garbled letters and a missing-font notice (#898).
- Step and Repeat: the horizontal and vertical distances, the number of copies and the modes stay as typed. They went back to the defaults (6.35 mm across, 0 down, one copy) whenever the selection changed, so after every Apply (its copies are selected) and every click on another object a distance set to 0 showed 6.35 mm again and the next copies took it (#1198). With nothing selected the fields are greyed and keep their values.
- Welcome Screen: the page under its tabs scrolls with the mouse wheel and a scroll bar when it is taller than the window, so recent documents below the visible area can be reached and opened; the row of tabs stays in place and each tab keeps its own place (#1288).
- Edit > Repeat (Ctrl+R) after a rotation, resize or skew repeats it about the same place of the selection it is used on (its centre, or the side or corner the handle kept), so the object no longer moves to another position; a move still moves as far again (#1289).
- Table > Convert Text to Table opens a dialog that asks what separates the cells: commas, tabs, paragraphs (a cell each, in one column) or a separator of your own, and tells how many rows and columns the table will have; every paragraph is a row, the spaces around each cell's text are left out and the columns are as wide as their text. Three names separated by commas now make three cells; before, only tabs separated cells (#1276).

- Space: pressed on the Pick tool, it now goes back to the tool used before, so Space switches between a tool and the Pick tool. It still picks the Pick tool from other tools, types a space while text is edited and drops a copy during a drag (where it no longer changes the tool); Shift+Space and Ctrl+Space are unchanged.
- Panels docked at the left: the collapse button in the title bar now points left, toward the edge the panels fold to; it pointed right on both sides.
- Blend: the object and colour accelerations go as far one way as the other. At -100 % the steps bunched right at the end object, at +100 % they only went a quarter of the way toward the start, and the slider on the canvas went to the end but stopped short of the start; now the middle step lands 5 % from either object at the full setting, the same distance both ways (#1261).

- Help: pages brought up to date with releases 0.19 to 0.27 (opening and saving, fonts, text, printing, export, drawing tools, objects, colours, effects, pages, options, menus and shortcuts), and new pages for Package, Missing Fonts, Convert Colors, finding by properties, PDF printers' marks, booklets, Fit Objects to Path, the Before and After preview, graphics tablets and tablets with the Android app; Brazilian Portuguese follows, other languages where the pages changed most. Docs only.
- Saving `.cdr` (version 12 and later): a clip frame (Place Inside Frame) is written as a clip container, the way files saved by the program that reads them hold one: the frame as an object of its own fill and outline naming, in its record, a group of the document's clip contents list that holds the objects placed inside it. A picture inside a frame went out as a bitmap with a crop path and any other frame as a plain group of the frame and its contents, which that program showed without the picture and without the frame (#1222). Read back in TraceDraw the frame is a clip frame again, its contents inside it. Versions 7 to 11 keep the group. `tracedraw-cli check` reads clip containers (the number must name a group of the list).
- Saving `.cdr` (version 16 and later) with Embed fonts: a text of two or more colours in one line kept its font only in the program that opened it when the whole text was one colour; with a second colour that program asked for the embedded font and drew the text in one colour (#1297). Each run that carries a colour of its own now names its font in its style the way the paragraph's style does (character set, technology, script and version included) and its fill with the same members, as files saved by that program hold their runs.
- Save Drawing dialog, `.cdr`: when a font the text uses cannot go into the file, a note under Embed fonts names it and says why ("Not embedded: Raj 50 (its licence forbids embedding)"): a licence that forbids embedding (the font's own embedding bits), or no font file to embed (a face drawn by a stand-in, or in a collection file). The box showed no size and nothing said that the font was left out on purpose (#1219). The fonts that may be embedded go in as before.
- Saving `.cdr` (version 9 and later): a blend is written live, the way files of the program that reads them hold one: the start and end objects as objects of their own with the empty linked group between them that holds the step count, the spacing and the two ends' centres. That program can then show the steps and edit the blend (steps, acceleration, colours); a blend went out as one plain group of the start object and its steps drawn as curves, which closed that program when a blend of two texts was clicked (#1192, #946). Read back in TraceDraw the blend keeps its settings from version 14 (the package names the start object); from an older file it opens the way any file's blend does, on the upper object. Versions 7 and 8 keep the drawn group. `tracedraw-cli check` checks the linked group (between its two ends, a step count of 1 to 999).
- Objects docker: each object now shows a thumbnail of itself, its shape with its fill, outline and effects drawn small, beside its name, instead of a plain colour swatch. The size goes from 16 to 96 points with the new slider in the docker's bottom bar, the wheel over it, or Ctrl+wheel over the list, and is remembered. The thumbnails are cached and drawn again only when their object changes, only for the rows in view and for a few milliseconds per frame, and the cache is capped at 24 MB, and rows scrolled out of sight are no longer drawn at all, so the docker over a drawing of 3,000 objects takes about 2 ms a frame where it took 55 ms (#896).
- Contour, Extrude and Smart Fill bars: Other in a colour button's list opens the colour picker at the first click, beside the button, and the list closes. The picker sat inside the list, so the first click on Other closed both and nothing opened, and on the next try its Model list took the list and the picker away with it, so neither the model nor the eyedropper could be used (#1262).

- Hints panel: selecting an object now shows a page for what was picked (a curve, rectangle, ellipse, polygon or star, artistic or paragraph text, bitmap, group, clip frame, blend, dimension, connector, callout, table, symbol, an object with an effect, or several objects): what you can do with it, the tools that work on it and what the property bar offers. Letting go of the selection goes back to the tool's page. Before, the page changed only when a tool was chosen.

- Browser version: its notice now opens with the warning that a browser is slower, marked in red and underlined, then says it is the same program compiled to WebAssembly, and last points to the desktop app, in every language.
- Shape tool: while a node is dragged (a segment bent, the chosen nodes stretched or turned), the page keeps the picture it had before the drag and the curve follows the pointer as the thin line the tool draws over it; the outline with its width nodes, brush, arrowheads and effects is made again once, on release, not every frame (#1259, #509). On a page with much text the outline was made again at every pointer move and the drag lagged.
- Tools > Border and Grommet with Source: Selection mirrors (or stretches) the selection's own edges on the first run, also when the selection reaches past the page, as a large imported photo does; the strips were cut from what the page showed of it, so they came from inside the picture and only a second run was right (#1283).
- Remove Background: removes several colours at once. The colours to remove are a list: the first is Automatic or picked, "+" adds another picked on the preview or the drawing, each row has its own tolerance and a remove button; with the eyedropper on, Shift+click adds a colour and Alt+click removes the nearest one. A pixel within the tolerance of any listed colour is background, and with Contiguous the fill also starts where colours were picked. The Tolerance shows whole numbers, and the settings are kept for the session.

## 0.27.0 (2026-10-30)

- Browser version: the page can be installed as an app on an Android tablet, an iPad or a desktop (the browser's "Install app" or "Add to Home Screen"): it opens full screen with the TraceDraw icon, in any orientation, keeps clear of a notch or rounded corners, and starts without a network connection once it has been opened, at the version last used; a new deploy is taken at the next start after its pages are closed, never a stale module against a new script. It is the same WebAssembly build served from the same address, not a native Android program (#893).
- Android app (a native port): the editor built as a shared library for 64-bit ARM tablets and phones and for x86_64 Chromebooks, loaded by the app's activity, drawn with wgpu through OpenGL ES 3, with touch and stylus input, packaged by `scripts/android/build.sh` and by CI as debug-signed APKs and as an App Bundle for Google Play (signed with the upload key from the repository's secrets). A fresh install starts in the Touch workspace. Open, Import, Save, Save As, Export and Publish to PDF go through the system's document picker, and text through the system clipboard; the system fonts are used, Google Play updates the app (no update check of its own). CI starts the app on Android 11 and 14 emulators on every push and fails unless it stays up, draws its first frame and shows the editor. Not available: printing and external programs; see `docs/behavior/android.md` and `docs/android-play.md`. The APKs are attached to the nightly and to the releases with the installers. The desktop and browser builds are unchanged; the app crate is now a library plus a thin program (#893).
- Colour eyedropper readout: a colour picker closed while its eyedropper button waited for a click on the drawing (Edit Fill or Outline Pen closed with OK, the Properties picker closed by a click beside it) left the crosshair pointer and the colour box following the pointer everywhere, and the next click on the drawing went to the closed picker instead of the tool (#1246). The request now lives only while its picker is open: the box and the pointer go with the picker.
- Convert Colors and Bitmaps > Mode: bitmaps now take the colour mode too. Tools > Convert Colors to CMYK gives every selected (or every) bitmap the CMYK mode and shows its pixels as the inks print them, through the colour management profiles, in the same undo step; to RGB the mode alone changes and the pixels stay. Bitmaps > Mode > RGB Color and CMYK Color do the same for the selected bitmaps and tick the mode they are in, and the status line names it ("Bitmap 600 x 400 px (CMYK)"). A `.cdr` CMYK picture opens in the CMYK mode and `.tdraw` files keep it. Before, a bitmap was left out of the conversion ("Every color is RGB already") and the mode items changed nothing (#1254). Saving `.cdr` still writes RGB pixels for every picture.
- Missing Fonts dialog: it has a choice of its own in Options > Global > Warnings ("Ask which installed font draws each missing one"), on by default, so it asks even where "Do not show this message again" was ticked on the plain missing-fonts message before the dialog existed; that tick had turned the dialog off as well, and a drawing with missing fonts then opened straight away with only the status line naming them. The dialog's own "Do not show" box now turns off the dialog alone, and with the dialog off the status line's "Missing fonts substituted" note opens the Warnings page when clicked (#1243).
- Saving `.cdr`: objects with a distortion, a contour, an envelope, a perspective, an extrusion, a bevel, a block shadow or a symmetry open again in TraceDraw as that object with its effect, adjustable as before saving (files of version 14 and later); they came back as the curves and steps the effect drew. Drop shadows are saved too, as a picture of the shadow beneath the object, which any program shows (drawn from the object's outline where no rendered picture is passed; none for hidden objects, one for a whole paragraph text; the shadow of a blended object is not written); before, they were left out of `.cdr` files. In TraceDraw the object comes back with its shadow. Other programs still see what the effect drew; reading their own distortions, contours and shadows from `.cdr` files needs sample files (#1031).
- Saving `.cdr` files: groups keep their names from version 14 (argument 1000 of the group's own record, as a version 14 file names a group; TraceDraw also reads them now); a page of another size than the first keeps its size from version 16 (argument 19130 in the page's own record), and in older versions, which have no place for it, its objects are placed from the document page's centre so they stay where they were drawn instead of moving by half the difference. Found by the new round-trip conformance suite (#98, #1139).
- Internal: `crates/cdr/tests/conformance.rs` writes every object kind the `.cdr` writer handles (shapes, curves, polygons, text, groups, clip frames, bitmaps with and without see-through pixels, every fill kind, outlines, transparency, blends, contours, envelopes, extrusions, pages, layers, hidden objects and backgrounds) in every version from 7 to 27, reads each file back and checks that it survives as `docs/cdr-format.md` promises for that version, and that every file holds together (chunk sizes, list nesting, required chunks, references, package members and the data file list of version 16 and later). It runs with `cargo test --workspace`, so every writer change is checked (#98, #1139).
- Saving `.cdr` files in version 14 and later: bitmap objects are written as every file of those versions the program that saved the samples wrote holds them (versions 7 to 13 are written as before, no file of theirs with a bitmap being at hand). The picture's box now runs from the origin to its size and the object's transform places it on the page (it was written at its page position with the identity transform), the image's number after the pixel size is 1, 2, ... instead of 0, and from version 14 the record ends with the four zero bytes files have after the crop outline. A drawing with a picture saved by TraceDraw showed no picture in that program and closed it as soon as the picture was clicked; whether these three differences were all it objected to still needs the reporters' check. `tracedraw-cli check` now names each of them, and the files the writer's tests produce are held to them (#1226, #1268).
- Graphics tablets: a pen's pressure sets the width of Freehand lines and of Preset, Brush and Expression brush strokes, on by default in Options > Tools > Curve tools; the mouse draws as before. The pressure reaches the app on Windows, and now on X11 and macOS too through a patched copy of the windowing layer (`vendor/winit`). On a Wayland desktop the pen reached nothing, because the windowing layer does not speak the tablet protocol yet: `tracedraw --x11` opens the window on XWayland, where the pen moves the pointer and reports its pressure (#850).

## 0.26.0 (2026-10-30)

- Trace Bitmap: the preview rests once the trace of the settings on show is in. In the browser, where a large picture's whole trace waits for OK, the preview waited for that trace and traced the picture again every frame: the spinner beside the pan button never stopped, the preview flickered and the window lagged. The quick look is now the request's last result there, the counts show it without the dots sign, and an idle dialog asks for no frames; a trace whose thread died is not started again until a setting changes.
- Before and After preview: one preview for every dialog that changes a picture (Trace Bitmap, Remove Background, Black and White, Duotone, Paletted, every bitmap effect and Image Adjustments). Each dialog is now a window the user sizes, the preview filling its left and growing with it: a Preview list (Before and After stacked, side by side, Before only, After only; the wireframe over the picture in Trace Bitmap, a split view with a draggable divider in Image Adjustments), zoom to fit, zoom out, zoom in and a pan button, the two panes with one zoom and place (a drag pans, the wheel zooms about the pointer), a checkerboard under transparent parts, and the dialog's settings in a column at the right. The previews are drawn at the size shown (a copy as large as the panes, worked out once per change and kept; a small copy while a slider is dragged) and nothing is worked out again while the settings stand.

- Sliders: every slider in the bars, dockers and dialogs is drawn the same way: its label above the control, ending with a colon; a thin groove with tick marks along it like a ruler (round major values, minors between) and a slim upright thumb; the value in a number box at the right of the groove, typed into (a value typed takes the slider's step) or stepped with its arrows. Clicking the groove puts the thumb there without taking the keyboard, so the canvas shortcuts keep working; once Tab gives it the focus, the arrow keys step it (Page Up and Down by ten, Home and End to the ends) and the wheel over it steps it (Ctrl a tenth, or one step on a whole-number or stepped slider; Shift ten). The wheel over a slider without the focus scrolls the panel as before. The fill and outline editors apply a value typed, stepped or wheeled, not only the end of a drag. The Color docker's sliders (the colour channels and the spot ink Tint) and the Trace Bitmap dialog keep the plain slider for now.

- Color panel (Shift+F11), laid out anew: one top row with the view buttons, the model list across the width and a gear (the views, the lock, Reset panel); each slider's rail is the gradient of its component (no cyan to full cyan, the spectrum for a hue), in the colour picker popup and the fill editors too; a small Hex box under the sliders; the palettes view has the spot ink tint under its swatches (the panel shows one Tint control), and a spot colour swatch clicked is taken at that tint, any other swatch as it is. At the bottom a strip: a lock, the new colour as a fill swatch in front of the selection's outline colour, the eyedropper and a "..." button that drops the other actions: No Fill, No Outline, Swap Colors (each selected object's fill and outline colours change places, in one undo step that walks the drawing once, so thousands of objects swap without a pause), Add to Document Palette (the Add to palette button moved here), Add to Custom Spot Colors (named as typed in Ink name, else by the colour's code) and Add to Color Styles Palette; under the swatch, Fill and Outline apply the new colour. While the lock is on, either gives the selection the new colour as fill and as outline in one step.

- Bitmap effect dialogs (Blur, Adjust, Color Transform, Tone Curve, Contrast Enhancement and the rest): the effect shows on the drawing while the dialog is open and follows every value as it changes, with a Preview box to switch it off; the preview is made off the interface thread with a busy mark, so the canvas keeps answering on large pictures. Cancel or Esc put the drawing back exactly as it was with no undo step; OK keeps what the drawing shows as one step without making it again. Before, only the small before and after pictures in the dialog followed the values and the drawing changed on OK.
- Trace Bitmap dialog (Bitmaps > Outline Trace, Centerline Trace, Open Last Trace Settings): the dialog is a window you size (with a maximize button), the picture and its trace large at the left, stacked, with Before, After and a wireframe view, zoom and pan, and the counts of curves, nodes and colours under them; the trace follows every change by itself, made off the interface thread (a quick look at once, then the whole picture, which OK places as is), so there is no Preview button and sliders never wait. The settings sit in Trace type and Image type lists (the presets), Remove (background colour, the entire colour or nothing; found by itself or picked from the preview with an eyedropper), Merge adjacent, Remove overlap and Group by color, with Reset and undo and redo arrows for the settings; a Colors tab lists the trace's colours with their curve counts, merges the ticked ones and edits a colour, beside the colour mode (RGB, grays, black and white) and the number of colours. Removing the background now leaves the patches of that colour inside the picture (the white of an eye) as curves; Entire color from image takes them all out as before. Options... opens the tracing options. Before, the dialog had two small previews, the presets as links and a Preview button that only counted curves.

- TraceDraw opens faster. The splash screen no longer stays up for at least 1.6 s whether the start needs it or not: it goes as soon as the editor can be built (the editor opens in 0.1 s here instead of 1.75 s). The list of system fonts is read while the window and the graphics driver start up instead of after them, the system fonts for Chinese, Japanese, Arabic, Hindi and Bengali no longer hold the editor back unless the interface language needs one, and the file a start leaves for later starts to find the window by is no longer flushed to the disk. The browser version starts compiling the module while it downloads, and downloads it together with its script. Each start logs `the editor was built in N ms` and `first frame N s after the start`.
- Panels: the button at the end of each tab column is now three small dots instead of a plus (tip: Show or hide panels), and each row of the list it opens shows the panel's picture between its check box and its name, as the tab does, with a blank slot where a panel has no picture so the names line up. What the list holds and what ticking does are unchanged.
- Saving as .cdr with Embed fonts: each embedded font member names its face in its header (family, style flags, PANOSE, licence) and the drawing's font records of the embedded faces carry its PANOSE bytes, script and PostScript name and mark it as embedded, as files that embed fonts have them; a drawing saved without Embed fonts is written as before. The header stopped at the version string, so the other program could not pair the font with the text and asked for it to be installed even though it was in the file (#1009, #1055, #352).

- Windows installer: the setup now asks which language to run its wizard in and ships English, Brazilian Portuguese, Czech, Dutch, French, German, Italian, Japanese, Polish, Russian, Spanish and Turkish. It offered only English and Brazilian Portuguese. The command line option and the Start menu uninstall entry are in the chosen language too. It has always created an uninstaller (Start menu "Uninstall TraceDraw" and Windows "Apps & features").
- Bitmap effect dialogs (Blur, Adjust, Color Transform, Tone Curve, Contrast Enhancement and the rest): the effect shows on the drawing while the dialog is open and follows every value as it changes, with a Preview box to switch it off; the preview is made off the interface thread with a busy mark, so the canvas keeps answering on large pictures. Cancel or Esc put the drawing back exactly as it was with no undo step; OK keeps what the drawing shows as one step without making it again. Before, only the small before and after pictures in the dialog followed the values and the drawing changed on OK.

## 0.25.0 (2026-10-29)

- `.cdr` mesh fills: the node colours of a version 11 mesh are read column by column, as the file stores them, so the skin, face and clothes of the drawing in #896 show their shading where the program that saved it puts it, instead of dark streaks across them; the grid lines keep their curves (every node keeps its handles, `MeshNode::handles`) and the cells are drawn as curved (Coons) patches with the colour mixed pixel by pixel, no longer as flat quads with straight sides. The mesh's picture is worked out once per zoom and kept (the cache is shared by the render threads), so panning or dragging another object does not compute it again, and its work is bounded by what shows on the canvas (cells and pieces off it are skipped, triangles painted in bounded batches); dragging a node moves its handles with it; a column or row added through a curved cell cuts the curve where the new node is, and removing it restores the curve (#896).
- `.cdr` version 11 transparencies: the 120-byte transparency records of such files are read, uniform ones as the object's opacity (they were read as opaque) and graded ones as a transparency mask from the fountain fill of greys the record names; the drawing's shadow fades out, the eye highlights and the white highlights on the jacket show through (#896).
- Saving `.cdr` in versions 7 to 13: the files now hold, record by record, what the program that reads them saves in those versions (the same drawing saved by it in each version, which a tester sent). The master page lists the drawing's layers and every page lists them in the master's order (the pairing files of those versions rely on); objects and groups carry the zero 12000 in every version and the zero rotation argument (12030) in 7 to 12, shapes of 7 and 8 the drawing flags (19170), texts the 16015 argument; the version 8 style table holds its set11 table, style names before 12 end with the NUL their length counts, and font entries before 10 hold one size and three style flags; a text record before 14 lists the distinct looks of a line with every letter numbering its record (the 4-byte descriptions of 7 to 11 too), the version 8 header word is 4, every text is followed by its line breaking tables and flagged 8 like the shapes; version 13 files lose the version 14 traits they had (12000 on layers, the master page's 0x40 flag and object box, a 24-bit picture); the summary is 60 bytes with the save time and a build number in every version, transforms 184 bytes in 7 to 12, and the configuration records of 7 to 11 carry the paper preset and the orientation. Every file of versions 7 to 13 crashed that program on load before; versions 14 and 15 are written as before. Save As, Options and the command line now offer and save versions 7 to 13 like every other version, without the experimental mark or `TRACEDRAW_CDR_EXPERIMENTAL`. Reading, the version 8 style table is told apart from its bidirectional variant by where its records end, and font entries before 10 are read with their one size (#1139).

- Microsoft Store: the copy installed from the Store no longer checks GitHub for new releases or offers their download, which the Store's policy refuses (10.2.10.1); the Store updates it. Help > Updates opens TraceDraw's Store page there, and Options > Global > General has no update check option. The installer, the portable zip and the macOS and Linux packages are unchanged.

## 0.24.0 (2026-10-29)

- Fit Objects to Path panel: Keep originals places copies and leaves the selected objects where they are; Group all objects groups what was placed; Treat subpaths as contiguous walks every subpath of the path one after another (on at first; off, only the first subpath, as before); Reference and Origin choose the point of each object that goes on the path (its center of rotation, or one of nine points of its bounding box, with or without the outline); Ignore initial rotation takes the objects' turn off them first; Modify rotation adds a uniform, progressive, jittered or progressive jittered turn, with a start angle, spin angle, revolutions, range and direction. The panel shows how many objects are selected and what the path is (#1091, #1196).
- Internal: while a `.cdr` file opens, the reader now keeps every chunk and `loda` argument it does not interpret in a side table keyed by object (bytes shared with the file's streams, so a large file costs no more memory), the first piece of the standalone `.cdr` library's lossless round trip; the writer does not use it yet. `tracedraw-cli info file.cdr --unknown` lists the unknown tags with their counts.
- Internal: the `.cdr` container, chunk tree, embedded font and compound file readers moved to a crate of their own, `tracedraw-cdr-format`, which depends on nothing else in the workspace; `tracedraw-cdr` re-exports them, so nothing changes for users. First step of splitting the `.cdr` support into a standalone library with a neutral document model.

- Fit Objects to Path panel: copies along the path (a number of objects, the selected ones and then copies of them in turn, or as many as fit at a fixed distance), spacing measured between centers or between edges, an end offset, a row at a fixed distance placed at the start, middle or end, the objects on, above or below the path at a set distance, turned with the path or all by a fixed angle, and the order (as selected, reversed, left to right, right to left, largest or smallest first, large and small in turn). Copies go into their original's layer and Apply stays one undo step; the preview is worked out only when a value, the selection or the drawing changes (#1091, #1196).
- Welcome Screen, Recent documents: one drawing comes off the list by the x that shows at the right of its row while the pointer is on it, or by Remove from list in its right-click menu, a drawing whose file is gone too; the others stay. Clear list still empties the whole list (#1152).
- Options > Display: View when a drawing opens sets how every drawing opened or made is first shown: the whole page (as before), its objects, the page's width or height across the window, 100% (real size), or as saved, the page, place and zoom it had when it was last saved. TraceDraw files now keep that view, with no undo step and no change mark (#1134).
- Page bar: a Multipage View button (four small pages) after the page controls, before the page tabs, turns the Multipage View on and off in one click, and is lit while it is on, however it was turned on (#1019).
- Saving `.cdr` files in versions 9 to 15: every object carries the drawing flags argument (19170) files of those versions have on each shape; versions 14 and 15 write the fill and outline records as the newest releases save them for those versions, with the palette id after each colour (the CMYK or RGB constant such files hold), so version 14 solid fills, gradient stops and outlines have the sizes of version 15 instead of the shorter 2018 ones. Files of versions 14 and 15 saved by TraceDraw showed their shapes without fills in a user's test (#1139, #98); whether this is what was missing is not confirmed yet. Opening: a version 14 gradient whose stops carry the palette id (files saved for version 14 by newer releases) reads its colours, instead of one colour.
- Opening `.cdr` files of versions 7 to 15: text whose style sets another font or size for East Asian letters than for Latin ones (the style table's second script slot) now shows its Chinese, Japanese and Korean letters in that font and size. The Latin slot was used for every letter, so the titles of a version 14 certificate came out as 12 pt Arial instead of a 54 pt script font and a label at 21 pt instead of 26 pt. Version 16 and later style strings read their `farEast` section the same way (#955).
- `tracedraw-cli check file.cdr` reports what a reader would stumble on in a `.cdr` file: curves that end back at their start without closing on their last node, segments and subpaths of no length, an object box that does not hold its geometry, outline box corners outside it, object numbers used twice, text objects without letters, and fills, outlines or styles that name no record. Every file the writer's tests produce is run through it, so such faults (the ones that closed the program opening a file in #1130 and #986) cannot come back.

## 0.23.0 (2026-10-29)

- Font list filter: the choices now open beside the list, in sections: fonts shown, style (serif, sans serif, script, decorative, monospaced, symbol), weight (light, regular, bold), width (condensed, normal, expanded), technology (TrueType, OpenType, variable), scripts (Latin, Greek, Cyrillic, Arabic, Devanagari, CJK and more), each OpenType feature on its own, embedding rights and the folders the fonts are in. Every value is read from the font files once (OS/2 weight and width classes, embedding bits, Unicode ranges, variation axes, GSUB and GPOS feature lists). Monospaced and Symbol now widen the style choices like the other styles instead of narrowing them (#867).
- Segment Delete: besides the click and the box, a freehand stroke deletes every stretch it crosses, up to the crossings with other lines, in one step. Choose Freehand on the property bar, or hold Alt as the drag starts to take the other way for that drag (#1150).
- Tools > Convert Colors: every colour of the selected objects or of the whole drawing (fills, gradient stops, mesh colours, outlines, text, the colours of effects and shadows, the pages' backgrounds) goes to CMYK or to RGB through the colour management profiles, in one undo step. Spot colours stay unless asked; registration colour and bitmaps' pixels never change (#972).
- Color Eyedropper and the eyedropper of the color pickers: while the pointer hovers to take a color, a box beside it shows that color with a swatch, its name when it is a palette color or a spot ink, its values in the drawing's color mode (CMYK or RGB) and its hex value, so a color can be checked before it is taken. Before, its values only showed after the click (#990).

## 0.22.0 (2026-10-29)

- Microsoft Store package: `tracedraw-cli` is now a command-line alias of the TraceDraw app instead of a second app hidden from the Start menu, which the Store refuses ("headless app"). The command works in a terminal as before and the Start menu still shows only TraceDraw; the `tracedraw` alias that opened the window from a terminal is not in the Store package (an app can declare only one alias). The installer and the portable zip are unchanged.

## 0.21.0 (2026-10-29)

- Publish to PDF, Prepress tab: crop marks, registration marks, colour bars and file information can be added around every page, drawn by the same code as the Print dialog's marks. The page grows by a 12 mm margin outside the bleed to hold them; the TrimBox stays the page, the BleedBox the page and its bleed and the MediaBox takes in the marks, and the drawing is clipped to the bleed. Marks in registration colour print on every plate (the All separation), in printed files too. A page background now covers the bleed in PDF files (#789).
- Find and Replace: a Properties mode finds objects by type, fill colour (or no fill), outline colour (or no outline), outline width and font, on the current page or in the whole drawing. From Selection takes these from the selected object; Previous and Next step through the objects found, going to their page; Find All selects them. Replace All puts a new fill colour (in gradients too), outline colour, outline width or font in every object found, as one undo step. Only text and names could be searched (#1071).
- Package: File > Package, and a Package choice in Save As for TraceDraw files, put the drawing, the pictures it links to (in Pictures, with the copy's links pointing to them) and the font files its text uses (in Fonts, only fonts whose licence allows them to go with a document) into one folder, with a report of what went in and what could not and, if asked, a PDF. Links relative to the drawing are now read from its folder, so a package can move, and a drawing opened from a package uses the fonts of its Fonts folder for the session (#852).
- Opening a drawing with fonts that are not installed shows a Missing Fonts dialog: each missing font with the installed font that draws it now (the automatic choice), and a list to choose another, either for this session only (the drawing keeps its font names) or replaced in the drawing for good (one undo step). Opens from tests, scripts and the MCP server are never stopped by it and keep the note (#936).
- Print and Print Preview, Booklet: the binding is chosen by two pictures, Saddle stitch (each sheet folded inside the one before) or Perfect bound (signatures of a chosen number of sheets, each folded on its own and stacked). Saddle stitch takes a creep compensation that moves the pages of the inner sheets toward the spine, and either binding a gap between the two pages of a spread. Under the sketch the Layout fields show the pages, the blank pages added to reach a multiple of four, the sheets of paper and the signatures. Booklets could only be saddle-stitched, with no gap (#788, #947).

## 0.20.0 (2026-10-28)

- Variable fonts: the text follows the axis sliders (Text docker, and the button beside the font size) while they are dragged, so the weight, width or any other axis can be judged on the page. The text changed only when the slider was released. Releasing still keeps the value as one undo step (#870).
- Fit Objects to Path panel: it no longer starts with the values and the preview left from earlier work (on other objects or in another drawing); for other objects it starts from the defaults and shows no preview until a value is entered (#1136).
- Print and Print Preview, Selection: only the selected objects print (objects on master layers and the page background printed too), and the area they cover is the page printed: with the sheet sized to the page the paper is that area, on a fixed sheet they are centred at their size, the layouts (several per sheet, step and repeat) repeat that area and Keep page size keeps them at their size. Before, they printed where they stood on the whole page (#1001).

- Dimensions: a dimension placed between two edges of an object (the sides of a rectangle, say, not its corners) now holds to them like one placed on nodes: moving, resizing, turning or reshaping the object takes the dimension along, each end keeping its place along its edge, and the value follows. Only ends on nodes were held, so a dimension between two sides stayed where it was (#1178).
- Color palettes: every palette ticked in Window > Color Palettes or the Palette Manager shows, each in a strip of its own next to the default palette, with Close Palette in its menu. Only the first palette ticked was shown, so a second or third one stayed in the menu with a tick but never appeared (#1164).
- Options > Customization > Commands: the bars' buttons that run a menu command (New, Open, Save, Undo, Zoom In, Rulers and others) take shortcuts on their Shortcut Keys tab, the same keys as their menu command; they had no Shortcut Keys tab. Rows the menus show only at times (Finish Editing Contents, Vertical Text) are listed and take shortcuts too (#1148).
- Text tool on Windows: input methods that type by sending characters (Keyman, NHM Writer and other keyboards for Tamil and Indic scripts) now type into artistic and paragraph text and into fields; the windowing layer dropped characters sent with no key press before them and the characters of a key the input method took, so nothing was typed (#1151).

## 0.19.12 (2026-10-28)

- Blend tool: the squares at the ends of a picked blend's line (text blends too) now work: dragging one moves its object, so the blend changes direction, and dropping it on another object makes that object the new start or end with the blend's settings. They were only shown (#965).
- Opening .cdr: blends saved by version 25 read their real step counts (the count is the low half of the field; every one read as 20 with a warning), a record the file counts as an object no longer makes a 150-step blend that floods the drawing, and blends between two open arcs draw open arcs end to end instead of closed shapes crossed through the middle, so the reporter's file opens as the program that saved it shows it (#980).
- Export: the Preset list holds every saved preset from the moment the dialog opens, each with its format in brackets; choosing one of another format switches to that format, and changing the format leaves the preset unchosen while the list stays whole. Since 0.18.0 the list shrank to the chosen format's presets, so a dialog opening at a format without presets showed none (#612).
- Export: Remember the settings of the last export (Options > Global > File Formats) is switched on once for preferences saved by versions before 0.19.4, which stored it off, so File > Export starts with the last export's format, resolution and quality in the next session too; since 0.19.4 only new preferences had it on, and anyone who had used an earlier version still saw the dialog start at SVG each session (#691, #617).
- Pick tool: Ctrl+click on a clip frame starts editing its contents (as Edit Contents does), with the object under the pointer selected; Finish puts them back. Ctrl+click picked the object inside the frame and left it there, so it was moved and changed while still clipped (#982).
- Multipage View: the pages always sit in four columns, so a page added goes at the next place and the pages before it stay where they were. The number of columns followed the page count (2 columns for 2 and 3 pages, 3 for 4 to 9, 4 from 10), so adding the 4th or the 10th page moved every page (#1018).
- Perspective: two corners dragged onto one point (the frame a triangle) now bend the object to a tip there, so an ellipse becomes a drop. The object stayed flat when the corners met exactly, and when they ended a little past each other it came out as a thin sliver running off past the frame. The outline's nodes and handles are now mapped, so each segment stays one segment and the outline is drawn faster (#879).

## 0.19.11 (2026-10-28)

- Undo keeps to a memory limit as well as to its number of steps: past 1 GB of a drawing's own undo data (256 MB in the browser) the oldest steps go, and the newest always stays; a copy made with a right click during a drag stays one step. An undo-heavy session on a drawing of 40,000 curves and six 40 MB pictures held 6.9 GB after 72 steps and now stays at 1.35 GB. Slow pages are drawn in the background one at a time, the page being edited first: Multipage View could run a render for every other page at once, each with its own band threads and picture buffers, and a render made stale kept running beside the next one.
- Document palette: a colour picked with its eyedropper that the palette has already gets the white frame of the last colour used, so it shows where it is; a new colour takes the frame as it joins, and applying a colour of the palette moves the frame to it (#1125).
- Status bar: a tool hint or message cut off at the right shows whole in a tip on hover, so a long path before the hint no longer hides it for good (#1126).
- Pages added to a drawing (Layout > Insert Page, the + beside the page tabs, the navigator's insert buttons) are named in the interface's language, as the first page is, and so is their first layer. They were named "Page 2", "Layer 1" and so on in English whatever the language (#1127).
- Step and Repeat: the number of copies, the distances and the modes start from the defaults for each new selection, and again after an Apply (the copies it made are selected). The values of the last object stayed, so the next object got the old count and distances and came out with the wrong number of copies; with nothing selected the fields are greyed out (#1032).
- Shape tool with several objects selected: the object picked last shows its outline in the node colour (blue) and the other objects in the secondary node colour (red), also while a node is dragged, so the objects can be told apart. All of them showed the same blue outline (#1007).
- Convert Outline to Object on a wide outline with round ends: the tip of each end always gets its node, so the end is two curves meeting there. On outlines of about 10 mm and wider the tip fell between two points of the end's fan and got no node of its own. Outlines with many corners convert faster (#623).
- Color palette with 2 or 3 rows: the window showing every colour opens clear of all the rows. Docked along the top or at a side it was placed as for one row and covered rows 2 and 3 (#610).
- Pen: the preview segment from the last node to the pointer is a solid blue line on a white line twice as wide instead of dashes, so it shows over any colour; the handle lines are dashed in blue and white in turn, as thin as the preview's blue line (#670).
- Save As of a drawing opened from a `.cdr` file (or any other format TraceDraw reads) starts in that file's folder with its name, so saving over it replaces it. It started in the folder any dialog used last, so with another file opened from Downloads the save went there and the original stayed as it was. A folder typed in the path bar now takes the file without pressing Enter first (#900).
- Objects panel: selecting an object on the page scrolls the list to its row, so an object far down a long list comes into view; a selection that stays the same leaves the scroll alone (#1044).
- Envelope: Copy Envelope Properties (and Object > Copy Effect > Envelope From) picking an object with no envelope, such as a circle, a star or a drawn curve, now fits the selected text or objects into that object's shape. It said the object had no envelope, so text could not be fitted into a drawn shape (#993).
- Bevel: Emboss now does something, and the panel has more to set. The style was stored but never read, so Emboss drew the same band as Soft edge; it now draws the band around the outline, so the object stands out of the page. New Level (1 to 8) fades the light and the shade across the band instead of two flat tones, and Emboss color tones the band from a color of your own instead of the object's fill. The panel is laid out like the other effect panels (Depth, Level, Light angle, Intensity, the colors), and it shows the bevel of the selected object (it showed the last settings used, and the first slider moved overwrote the object's distance, angle, intensity and colors with them). Bevels saved before open as they were (#1135).
- Hints panel: every tool's page now walks through the tool in steps, each with a picture of the object, its handles or nodes, the drag and the tool's cursor (181 steps for 64 tools, in all 18 languages). It showed only the one-line status bar hint.
- Pen: while a curve is drawn, new or going on from an end, every click places the next node, also on a segment or a node of that curve or of another one. Automatic adding and deleting of nodes works only when no curve is being drawn, so drawing no longer changes other curves by accident (#1143).
- Pen: going on from the end of a curve, a second click on that end node makes it a cusp, so the new segment leaves it straight; the curved, mirrored start stays the default (#1138).
- Dropping files: a `.cdr` (or any drawing File > Open reads) let go over the title, menus, toolbars or document tabs opens in a tab of its own, and only one let go over the drawing window is imported into it; the frame and label while dragging show which. Before, a `.cdr` dropped anywhere on an open drawing was imported. Where the system does not say where the pointer is (Linux, macOS), drawings open in a tab like a `.tdraw` and pictures are imported (#1142).
- Status bar: it no longer keeps showing the path of a drawing that was closed (or of another tab) after making a new drawing or switching tabs; the message about a file now goes when another drawing is shown (#1128).
- Saving `.cdr`: blends, text blends too, open again in TraceDraw as live blends to their end object, with their steps, accelerations and options, so they can be changed and follow the end object as before saving (files of version 14 and later). They came back as loose steps that could not be edited; other programs still see the blend as a group of its start object and its steps (#986).
- Saving as .cdr: curves whose outline ends back at its start, such as text fitted to a path (written as its letters' outlines), close on their last node instead of adding a closing line of no length after it. The other program that opened such a file closed when the text was clicked; lines of no length and empty subpaths are also left out (#1130).
- Opening .cdr: objects embedded from another program that keep their picture as a Windows metafile, such as barcodes, are drawn, also inside clip containers. Version 26 files name such an object's data in a link record instead of a name, so the objects were skipped; the metafile is now played and stretched over the object's rectangle, and a metafile that sets no viewport (as these pictures do) is drawn at its window's size instead of shrinking to a speck (#1131).
- Smudge and Twirl no longer slow the app down until it hangs or crashes on a long drag, and dragging a segment with the Shape tool keeps its cusp nodes cusp instead of turning them smooth (#1133, #1140).
- Distort: the Zipper draws frequency + 1 lines, and the amount, amplitude, frequency and angle have handles on the page to drag (#1122).
- Tips: every control of the bars, dockers, dialogs and Options pages shows a title and a description on hover (#1106).

## 0.19.10 (2026-10-28)

- New Document: the RGB, CMYK and grayscale profile lists of Color Settings offer the ICC profiles installed on this computer, as Color Management does; they held only sRGB and Generic CMYK. A CMYK profile picked there goes with the new drawing, so its colours show through it, and the grayscale profile chosen is kept in the drawing (#1033).
- Snapping: with only the baseline grid turned on in Snap To, nodes dragged with the Shape tool, drawing tools and moved objects snap to its lines. Snapping counted as off unless another target was on, so the baseline grid alone did nothing; a node that lands on a baseline is now marked like a grid line (#3).
- Paragraph text wrapped around an object keeps the first-line indent: when the object pushes the first line in (an object at the left, or text flowing on its right), the line now starts the indent past the object's edge. The indent was lost there, so the first line started flush with the lines under it (#1110).
- Drawing tools: the tool's icon beside the cross hair (the small rectangle of the Rectangle tool, the ellipse of the Ellipse tool, and so on) sits close to the cross hair's middle. It was 9 points off each way, so it looked like a shape trailing the pointer (#493).
- Blend of two open lines (two arcs, one mirrored): the steps are open lines that run end to end, top to top and bottom to bottom, with as few nodes as the lines. Each step was closed back to its start and, when the second line was drawn the other way, crossed through the middle, so the blend came out as a dark tangle instead of a fan of curves (#1024).
- Page sizes: the page size lists offer about 130 sizes in groups (ISO A, ISO B and JIS B, North American, Print shop, Posters and banners, Envelopes, Cards and postcards, Photo, Screen and social media, Web ads), with A4, Letter, A5, A3 and Legal first. Edit This List... at the foot of the property bar's list opens Document Options > Page Size, where a custom size can be saved under a name and deleted again; saved sizes show in every page size list (#1092).
- Windows: a tool picked with its shortcut shows its pointer at once, with the mouse still. Windows changed or hid the pointer only once the mouse moved, so the old pointer stayed until then (#1026).
- Shape tool: on a stretched curve, the corner rounding rings now round corners on the page, so a new rounding comes out round, also after other corners were rounded. A corner rounded before the stretch keeps its stretched arc and its ring: dragging it changes that corner alone, and once it is dragged back to sharp it rounds round again (#1000).
- Shape tool: end nodes of two different objects are no longer joined, by Join or by dropping one on the other; the status bar says to combine the objects first (Object > Combine, Ctrl+L), after which their ends join as before (#1007).
- Text panel: the Fit Text to Frame button in the Text Box section shows its icon, now drawn: it was an arrow character the bundled interface font lacks, so on Windows the button was blank (#1041).
- Text on a path: Top and Bottom in the placement list centre text that is set off its circle (a Distance from path), as a stamp's bottom line. The text landed an eighth of the way round, because it was centred on the path's length rather than on the longer or shorter line the letters stand on (#193).
- Saving as .cdr in version 9, 10 or 11: the file carries the whole document settings record files of those versions hold (ruler origin, grid, units, nudge, drawing scale). It held only the page size followed by zeros, a zero grid and scale (#98).
- Corners panel: on a stretched object a fillet or scallop comes out round, with the radius in page millimetres. The corners were cut in the object's own space, so the stretch made them oval (#1002).
- Zoom: F2 picks the Zoom tool for one zoom. After a click, a right click or a marquee the tool used before comes back (the Pick tool, most of the time); it stayed on the Zoom tool. Z and the toolbox button still pick the Zoom tool until another one is chosen (#1039).
- Blend: the steps between two objects are clean shapes with few nodes. Each step has the nodes of the two objects paired by their place along the outlines, with straight sides where both objects are straight and smooth curves elsewhere: a square blended into a circle gives steps of eight nodes that round off evenly. Each step was a polygon of 120 points, with corners cut off and lumpy outlines, heavy to edit after Convert to Curves (#1020).
- Saving as .cdr: text in a SemiBold face (or a Light, Medium, Black ...) is saved as its family at that weight ("Titillium Web" at SemiBold), as the file it came from names it. It was saved under the name "Titillium Web SemiBold", which no installed font goes by, so the saved file opened elsewhere in a substituted font (#892, #624).
- Saving as .cdr: a paragraph written in one colour keeps it. In a text of several lines, a line in one colour of its own (one red line among black ones) came back black when the file was opened again, here and in other programs (#1057, #1055).
- Saving as .cdr with Embed fonts: a static face of a large family ("Khand SemiBold") is saved as its family at its weight ("Khand", SemiBold), the names the font gives itself, also when its old-style family name is the whole name. It was saved as "Khand SemiBold" Regular, which another program reported as a missing font on a computer where the face was only embedded in the file (#1055, #1009).
- Start: when the first renderer failed at once and TraceDraw opened with the next one, a second, empty TraceDraw window no longer opens 20 seconds later (the start that handed over still counted the time).
- Dimensions without arrowheads (or with open, tick or no heads): the selection box and its handles wrap the dimension's lines and label. Their empty arrowhead curve counted as a point at the page's lower left corner, so the box stretched from the dimension down to that corner, and a moved dimension seemed to leave its box behind (#996).
- Status bar in a narrow window: the fill and outline parts keep room for their swatches and values and move left as the window narrows, the tool hints and object information giving way first and ending in an ellipsis. The outline's colour and width were cut off at the right, and the parts could not be read or double-clicked (#1004).
- Area Fill: areas filled side by side now meet exactly, with no hairline gap between their colours where curves cross or shapes overlap. A corner where two outlines cross was placed where their flattened outlines cross, up to 0.02 mm off the curves, so neighbouring areas could leave a thin gap or a missed corner when zoomed in; the same holds for the results of Weld, Trim and Intersect (#1065).
- Distort: the Twister spirals smoothly all the way to the ends of a shape, and the Zipper teeth run through the nodes, also on closed shapes; the first node no longer stays locked (#1108).
- Smudge and Twirl: a long stroke stays quick and no longer freezes or closes TraceDraw. The curve under the nib is rebuilt with TraceDraw's own curve fitter; the one used before could take ever longer, hang or crash partway through a stroke on a simple shape (#1133).
- Shape tool: dragging a curve segment between two nodes made cusp keeps them cusp. Their other handles stay where they are; when a node's handles happened to line up, they turned along with the dragged segment, as if the node were smooth or symmetrical (#1140).

## 0.19.9 (2026-10-28)

- Docker tabs: when more dockers are open than the column of tabs has room for, the column slides, with arrows at its top and foot and the mouse wheel over it, so every tab and the add button can be reached. The tabs past the window's bottom edge could not be clicked (#950).
- Updates on Linux x86_64: the release's x86_64 AppImage is now named `tracedraw-<version>-linux-amd64.AppImage`, so that updating from 0.19.8 or earlier downloads it instead of the arm64 one, which those versions took because it came first in the list. TraceDraw looks for the new name and still finds the old one in earlier releases; the tar.gz names are unchanged (#1038).
- Distort: a value changed on the property bar (Amplitude, Frequency, Random, Smooth, Local, direction, Rotations, Degrees) shows on the selected object at once, also when it has no distortion yet. It changed only objects already distorted, so with a plain object selected the bar seemed to do nothing until Apply (#1030).
- Opening `.cdr`: the steps of blends from versions 9 and X3 keep the sharp corners of the objects they blend, as where the files were made: between a rectangle and a pentagon each step is a pentagon. The corners were cut off, so the steps came out rounder and larger than in the original (#597).
- Print Merge: Place Inside Frame, picking the shape with a click, takes a shape that already holds a picture, not the picture inside it. The merge then put each record's photo in that picture's box, with the old picture still showing next to the shape (#596).
- Web version: a notch of the mouse wheel zooms 15 % about the pointer, as on the desktop. The browser sends a notch as 100 pixels or three lines, which the canvas read as several notches, so every notch zoomed far more than that at once (#1016).
- Saving as .cdr in version 7 or 8: the file carries the whole document settings record files of those versions hold (ruler origin, grid, units, nudge, page colour). It held only the page size followed by zeros, and version 7 files crashed the program that wrote the samples, even with only shapes in them (#98).
- Pick and Freehand Pick: a right click while dragging a selection box no longer leaves the box frozen on the canvas. The right button is ignored: the box goes on with the pointer and selects when the left button goes up, whichever button is released first (#1015).
- Pen and Bezier: a handle pulled out of the drawing window (over a docker or a toolbar) and let go there ends the pull. The preview stayed stuck where the pointer left the drawing, and the first click back on it only brought the preview back instead of placing a node (#1014).
- Saving as .cdr: text in a variable font keeps the weight set on its weight axis (600 reopens at 600, as the family's SemiBold face); every such text was saved at the regular or bold weight and came back at 400 or 700 (#1013).
- Updates on Linux x86_64 download and install the x86_64 AppImage, so TraceDraw starts again after updating. The update took the first AppImage of the release, the arm64 one; now every system takes the file named for its processor and for how it was installed (AppImage, .deb, tar.gz, Windows setup or portable zip, macOS disk image), and when the release has none, Download opens the release page instead of a file for another processor (#1038).
- Main window: with the splash screen turned off, a window left maximized opens at the size of the screen straight away; it showed at its normal size first and then grew to fill the screen (#787).
- Menus taller than the window (such as Object on a short screen) fit inside it and scroll with the mouse wheel or the scroll bar at their right edge; a menu of the menu bar stays under its title. The bottom items ran past the screen edge and could not be reached (#1008).
- Drop shadows show in exported and printed PDFs, beneath their objects, in their colour, opacity, feathering and merge mode; they were left out, so a shadow showed only once broken apart into a bitmap (#1029).
- Convert Outline to Object (Ctrl+Shift+Q) on a combined curve whose lines cross (such as an "f" made of a stem and a cross bar) gives one shape with a merged outline, filled where the lines cross, so it looks as it did before. Each line came out as a shape of its own and the crossing turned into a hole (#997).
- Save As with Embed fonts: the fonts of all the text now go into the `.cdr` and `.tdraw` file, so the drawing keeps its letters when opened again where they are not installed. Only text whose font had a face of exactly that name, weight and slant was embedded before, so text in a family named with its weight ("Montserrat SemiBold"), bold or italic in a family without that face, or a variable font at another weight saved no font at all. A font file two faces share is embedded once (#1009).
- Toolbars: a right click on the empty part of a docked toolbar opens its menu and keeps it open until an entry is picked or you click elsewhere; the menu showed and closed at once. A right click on the buttons of a floating toolbar opens the menu too (#994).
- Convert Outline to Object (Ctrl+Shift+Q) on an object without a fill, or on an open line or curve, replaces the object by its outline: one object, where the original was in the stacking order, in one undo step. The original stayed behind as an invisible object with no outline and no fill, which could still be selected and clicked. The outline of a filled object now lies right above it instead of on top of everything else in the layer (#999).
- Opening `.cdr`: a blend whose step count the file does not hold where expected opens with 20 steps and a warning naming the value read, instead of 999 steps, and the blends of one file take at most 100000 steps together, so files with many blends open without filling the memory. A blend never draws more than 999 steps, whatever asks for them (#980).
- Shaping: Trim with an open curve (a line, a polyline or a curve) crossing a closed object cuts the object along it into separate pieces, each keeping the object's fill and outline; several curves make more pieces, and a curve that ends inside cuts nothing and the status bar says so. The object came back whole. Front Minus Back and Back Minus Front split along open curves the same way (#1005).
- Tracing: the grain of photos and compressed images is taken out before the colours are picked (more at lower detail, none at the highest), so flat colours no longer come out full of white holes, scratches and specks, and the trace has far fewer curves; edges and corners keep their place (#611).
- Text fitted to a path: with the Pick tool the property bar shows the path's own fields (placement with Top centre and Bottom centre, distance from the path, the two mirrors, Straighten Text) before the text fields, so text on the bottom of a circle can be turned to read left to right, as on a stamp. The text fields came first and hid them (#193).
- Blend: a bitmap effect (such as Add Noise) on the blend's start object keeps the blend drawn. The start object shows with its effect and the steps and the end object stay as they were; the effect's picture held the start object together with its steps, placed wrongly and cut off, so the blend fell apart (#1043).
- `.cdr` mesh fills: objects filled with a mesh open with their colour grid instead of being left out. In a version 11 drawing of 412 objects, 62 mesh fill objects (the skin, the jacket and skirt shading) were skipped and now show; each takes its outline from the mesh's edge, which is where the program that saved the file draws it. The grid's cells are drawn with straight sides between the nodes (#896).

## 0.19.8 (2026-10-27)

- Graphics cards with OpenGL 2.1 only (the Intel HD Graphics of 2010 PCs under Windows, driver 8.15.10.2900): the window no longer comes out black. Such drivers took egui's shaders and buffers and drew everything after the first mesh of a frame as black boxes, so after the splash screen only the menu bar could be seen. On desktop OpenGL below 3.0 TraceDraw now draws with the fixed-function pipeline those drivers were made for (no shaders, no buffer objects), with the same result as on other cards. Should a driver still show nothing, TraceDraw reads two of the first frames back (with a small probe drawn last in them), notes in the log that the window came back nearly black, and starts again with the next renderer by itself instead of leaving a black window. The log now also names the driver's texture limit, extension count and any OpenGL error (#616).
- Export: choosing a format in the Export dialog brings back how that format was exported last (resolution, quality, transparency, size and the PDF choices), in this session and, with Remember the settings of the last export on, in the next. The bitmap size can be typed in px, mm, cm or in: a length sets the printed size (the scale of the artwork) at the chosen resolution, and the dialog shows the pixels, resolution and colours that come out. PNG, JPEG and BMP files now record their resolution, so they open at the size chosen (#617).
- Print Preview: the options at the left stand two to a row, and the pages to print are chosen there: All pages, Current page, a list of pages, the selection, and odd or even pages (also in the Print dialog). The printer with its settings button, its status and the copies moved to the top bar, so the left panel keeps that room for the options. Before, every option took a line of its own and only All pages could be ticked (#951, #953).
- Color viewer: in CMYK the square of the Color panel, the colour picker and the fill editors spans the inks, black ink from top to bottom and the hue's inks from left to right, so the pointer reaches every edge and corner (paper, the solid inks, K 100 and the inks over K 100) and the square shows how they print; in RGB and the other models it stays the screen's square. In CMYK the screen square was turned into the nearest printable colour, so the pointer stopped short of the dark and saturated corners (#907).
- Font lists: the Filter button opens its choices inside the list (which fonts show, the kinds of letters and what a family has), and ticking a box filters the list at once and leaves it open. The Filter and Show menus each opened as a menu of their own, which closed the font list at the first click, in the property bar, the Text and Glyphs dockers and Options (#867).
- Variable fonts: a button beside the font size on the property bar, and beside the size in the Text docker, opens the axis sliders (weight, width, slant and the font's own axes) of the selected text's font in a small window under it. The sliders were only in the Text docker's Character section, closed at first, and hard to find; the button is greyed out, with a tip saying why, when the font is not variable (#870).
- Blend tool: a blend picked with the tool, by a click on its start or end object (a text as any other) or on a step, shows its controls on the canvas: a square on the start object, a crossed one on the end object, the line between them, and two sliders on it, the object acceleration above and the colour acceleration below, each where the middle step lands. Dragging a slider toward the start or the end bunches the steps or the colours there, live, one undo step a drag, and the property bar follows. The tool showed nothing on a picked blend, so the accelerations could only be typed on the bar or the panel (#965).
- Print Merge: a text, picture or QR code field inserted with On every page and then moved or resized on its page is merged in that same place on every other page too. The copies on the other pages stayed where the field was first inserted, so from the second page on the merged objects landed somewhere else, and Place Inside Frame made a second frame there (#956).
- Print Preview: the arrows of the gap between pages sit at the right of their box again. A number field placed alone in a table of settings put its arrows in a column of their own, off to the right, away from the box (#948).
- Print Preview: the options at the left grow to their widest row, so no label is cut off at the right in any language at the window's usual size. In Spanish and other longer languages, and in English with Repeat on the sheet, the ends of some rows (Reset, Back side) were still cut until the panel was dragged wider or scrolled sideways (#862).
- Color panel: the spot ink tint slider shows its groove, and the remove button of a spot ink and the out-of-gamut sign show their icons. The groove had the white of the panel, so only the handle showed, and both icons came from characters the bundled interface font lacks, drawn as empty boxes on systems without a font for them; the gamut sign is now drawn and the remove buttons there and elsewhere use a cross the bundled font has (#949).
- Text: full justification spreads Chinese and Japanese lines to the right edge of the frame. It widened only the spaces between words, so text written without spaces stayed left aligned with room on the right; the room now goes between the characters (never after a line's last one), mixed lines widen their spaces and the gaps beside the Chinese characters, and the last line of a paragraph stays left aligned unless justification is forced (#952).
- Opening `.cdr`: text whose letters are each turned and moved on their own, as the ring of letters around a seal, opens from version 13 to 15 files with every letter in its place. Such texts were skipped and counted as objects that could not be read (#955).
- Save As .cdr: a blend goes out as one group of its start object and its steps, drawn as curves, beside its end object, instead of loose objects on the layer. A blend of two texts saved so is a plain group of a text and curves in the program that opens the file (#946).
- The main window fills the screen at start when it was left maximized, also with the interface in Chinese and a new drawing at start, where it came up with the maximized button state at its normal size in the corner of the screen. The maximize now follows the window's size instead of coming with it, and a window that says it is maximized but does not fill the screen is restored and maximized again (#787).
- Outline widths: the Outline Pen dialog's width list offers the same widths as the property bar's list, in the outline unit (0.1, 0.2, 0.25 mm and so on in millimetres). The dialog listed its own widths in points turned into the unit, so in millimetres it showed 0.088, 0.176, 0.265 while the property bar showed round values (#967).

## 0.19.7 (2026-10-27)

- Symbols: a symbol instance copied in one drawing and pasted in another brings its symbol along and shows the same objects there. It kept only the symbol's number in the list, so in a drawing without symbols it showed nothing, and in one with symbols of its own it showed one of those; symbols that show other symbols come along too, and pasting again reuses the symbol already brought (#678).
- Rotating with the Pick tool: in rotate mode a node of the selection (or a segment's midpoint) dragged turns it about the centre of rotation instead of moving it, and let go near another object's edge the node lands exactly on it, so an arc turned about one end can be laid onto a line; the pointer shows the turning arrows over such a node. Also, the turn follows the pointer when the rotate handle passes over the edge of another object or of the page. The pointer snapped to the edge, so the object stuck at one angle and then jumped while the handle crossed it; the handle still snaps to other objects' nodes, midpoints and centres to aim the turn at one (#933).
- Variable fonts: the Text docker's Character expander shows a slider for each axis of the selected text's variable font (weight, width, slant, optical size and the font's own axes). The values set go on the selected characters, are saved in .tdraw files and are drawn in the canvas, exports and PDF; Default axes takes them off (#870).
- 3-point curve, rectangle and ellipse: holding Ctrl while dragging the first line keeps it to steps of the constrain angle (15 degrees), as with the other line tools, so an arc drawn with Ctrl comes out level. Ctrl did nothing there, and the arc kept the slant of the hand. Ctrl held while placing the third point of the 3-point curve now draws a bow, the arc of the circle through the three points, which stands evenly on the line wherever the point is (#934).
- Envelope: the frame and the enveloped object stay smooth. The frame was drawn as 16 straight lines a segment and showed as a row of lines when zoomed in; a hard bend drew the object in visible facets; and Horizontal and Vertical followed a polygon through the frame instead of its curves. The frame is now drawn within a quarter pixel at any zoom, the object gets more points only where the envelope bends hard, and its sides lie on the frame (#881).
- Font lists: a Filter beside what to show picks families by the kind of letters (serif, sans serif, script, decorative) and by what they have (monospaced, symbol, variable, OpenType features), read from the fonts' own tables; the list opens as fast as before and, with thousands of fonts, fills in over a few frames the first time a box is ticked (#867).
- Opening `.cdr`: blends from versions 9 and X3 look as they did where they were made. The steps now turn from one end object into the other, curving or folding through the middle as the file's nodes pair up, and lie between the two objects instead of on top of the upper one. Before, every step kept the same upright shape on a straight line (#597).
- Properties panel: the outline's mitre limit, nib stretch and nib angle sit under an Advanced header after the common settings (caps, corners, arrows, dashes, position, behind fill, scale with object), closed at first unless one of them is set, so the outline section is shorter and its basic options are found at a glance (#487).
- Edit Fill: the gradient preview shows the start, end and skew nodes, and dragging them moves, turns and stretches the gradient as on the canvas (Ctrl turns in 15 degree steps); the preview has the object's proportions, the rotation and centre fields move nodes placed freely too, and OK now keeps nodes placed (or reset) in the dialog (#939).
- Dimensions: the bar's Options menu can join the extension lines and the dimension line (or an angle's arc) into one line that turns a corner instead of running past it, and a corner radius rounds those corners. Both are kept in the dimension and in saved files (#641).
- Drop shadow: making or editing a shadow follows the pointer smoothly, and a click on a shadow picks its object for editing. Each frame blurred and shaded a layer the size of the whole window for every shadow, however small; a shadow is now worked out on just the pixels it reaches, the picture unchanged, and in a 1168 x 733 window a redraw went from about 119 ms to under 10 ms. A click on the shadow with the Pick tool (or any tool that selects on a click) selects its object; with the Drop Shadow tool it shows the shadow's handles, and a drag on the shadow moves it (#935).
- Dialogs (Outline Pen, Fill, the effect dialogs and the others) can be dragged anywhere by their title bar, also partly out of the main window, so they no longer have to cover the drawing. A dialog was held wholly inside the window, so one nearly as tall or wide as the window, as on a laptop screen, could hardly move at all; now its title bar stays below the window's top edge and at least 100 px of it stays inside the other edges. Dialogs still open centred, or where they were left with Center Dialog Boxes off (#908).
- Pick tool: the pointer shows the move arrows only over what is selected. Over any other object it keeps the plain arrow (or the cross hair), as a press there picks that object first, so starting a marquee or picking objects that lie inside a frame or over other objects is no longer hidden under the move arrows. Dragging an unselected object still picks and moves it, and Alt+click still picks the object beneath (#885).
- Save As .cdr with Embed fonts: the embedded fonts are written as files that embed fonts have them. Each font in the file's font list now carries the number the text uses for it (the faces of a family share it), the embedded fonts follow those numbers, and their headers have the names, character set and checksums such files hold, so other programs can match the text to its embedded font; before, the fonts were numbered 1, 2, 3 in turn, which only TraceDraw itself did not need (#352).
- Shape tool: a corner rounded with its corner widget can be rounded again. Selecting the curve later shows a widget on each rounded corner, and dragging it changes the radius from the one it has, further in or back to a sharp corner; the widget had disappeared once the corner was rounded. This works for corners rounded between straight sides, also in saved drawings and with the Corners panel (#872).
- Shape tool: the rings inside the corners of a curve being edited, dragged to round them, are now a preference, Options > Nodes and Handles > Show corner rounding rings on curve nodes, and are off at first, so they no longer get in the way of editing nodes. Turn it on to round the corners of curves by dragging again; rectangles round their corners as before (#884).
- Print Merge: a Number field has a Format: 1, 2, 3, the letters A, B, C or a, b, c (Z is followed by AA), or Custom, which puts a text of your own before and after the number, for tickets and serial numbers such as "Ticket T-0098". Chosen in Add Column and in Configure Columns (#930).
- Open Palette: XML palette files open, with their colours' names and CMYK, RGB or Lab values (spot colours are skipped, and a file with nothing readable says so), and the file filter lists the extensions it accepts. `.xml` files were not offered at all, and the filter showed only "Palettes" (#882).

## 0.19.6 (2026-10-26)

- Multipage View: zooming with every page in view stays smooth, and the other pages scale with the zoom instead of jiggling and stretching. Each page was drawn anew on every frame of the zoom, and the picture shown meanwhile was stretched over the page's visible part; now the last picture follows its page until the view rests, and the pages are then drawn a few per frame. On nine pages of 1500 curves each, a zoom frame went from about 300 ms to under 1 ms (#903, #902).
- Blend: the objects acceleration slider moves the steps toward the start or the end object, also when the blend follows no path. It changed only their sizes (with Apply to sizing) and turns, so moving it seemed to do nothing (#910).
- Print Preview: the options at the left are wider and their edge drags wider, and what still does not fit scrolls sideways, so longer languages (Spanish and others) are no longer cut off at the right (#862).
- Print: the Layout tab has the paper margins (left, right, top, bottom), as Print Preview does (#909).
- Text panel, Paragraph: the icons beside the indent and spacing fields are drawn, so they show on every system; on Windows they were empty boxes, the interface fonts there lacking those arrows (#863).
- Color Management Settings and Document Properties read in the dark themes: the page under their tabs is drawn in the theme's field colour instead of white, under the light labels (#904).
- Envelope on text: letters given their own colour (or outline) keep it when the text is wrapped in an envelope, and the same holds for perspective and distortion. The warped text was drawn as one curve in the object's colour; it is now drawn as one curve for each colour, warped together as the whole text is (#912).

## 0.19.5 (2026-10-26)

- Outlines: new outlines start with a mitre limit of 45 degrees instead of 5, so a thick outline on text or on shapes with narrow turns bevels those corners instead of growing long spikes. Outlines already in drawings keep their limit (#894).
- Artistic and paragraph text: letters given another colour keep the font's kerning with the letters beside them. A text was shaped one colour at a time, so changing the colour of the "V" in "VA" dropped the pair's kerning and moved the "A" away; letters of the same font, size and placement are now shaped together whatever their colours, and no ligature joins two letters of different colours (#911).
- Drop Shadow on a clip frame or a group: the shadow now shows, beneath the frame and everything placed inside it. The tool gave the frame a shadow but it was never drawn, so only an object inside the frame could have one. The shadow follows what the frame shows: its fill and the contents within it, not the parts of the contents outside the frame; groups cast the shadow of each of their objects, and Break Drop Shadow Apart makes the same picture (#873).
- Text in a semibold face (or a light, medium or black one) shows that face everywhere: picking SemiBold under a family in the font list sets "Titillium Web SemiBold" instead of the family with Bold on, so the property bar and the Character docker no longer show Bold for it, and the docker's style list names the family's own faces (Regular, SemiBold, Bold ...). Such a face is no longer reported as a missing font substituted when its family is installed (#892, #624).
- Save Drawing: New folder stays in the folder shown and starts renaming the new folder; any folder or file in the list can be renamed with F2 or a right click; typing letters in the list jumps to the entries starting with them, and Enter opens the folder picked (#913).
- Chinese font names: a .cdr file saved in version 17 or later that names a Chinese font (方正黑体_GBK, 方正大黑_GBK) shows the name as written instead of garbled letters, and finds the installed font instead of reporting it missing; drawings TraceDraw saves keep such names too. Installed fonts that give their Chinese, Japanese or Korean family name only in a GBK, Big5, Shift JIS or Wansung name record are found by that name and listed under it (#898, #860).
- Contour, blend and extrusion keep the CPU low while dragging and pointing: a frame no longer works the effect out again to draw, to measure and to pick the same object. On a line of letters with a contour, a hit test with the pointer resting went from about 22 ms to 0.1 ms, and a drag works the contour out once for each move (#897).
- Save As: Embed fonts starts off, for `.tdraw` and `.cdr` files alike. It started on, so every drawing saved carried its own copy of the fonts its text uses; tick the box for the files that are going to another machine, and the choice stays for the next Save As (#886).
- Area Fill: a drawing of many lines that nearly meet (a line stopping a few ten thousandths of a millimetre short of the next, as files carry from rounding) now fills the cell under the click, not the whole drawing. Ends of open lines within 0.01 mm of an outline, or of their own start, count as meeting it (#899).
- Start on Windows: a start whose graphics driver hangs while the window is set up no longer stays open with no window. After 20 seconds without one, TraceDraw starts again with the next renderer by itself (OpenGL, then the software adapter), so the first start opens instead of only the one after it, and a drawing opened meanwhile gets a window of its own instead of going to the one that never showed (#570).
- Font Manager: the fonts of a folder added to the libraries can be used in drawings straight away and after every restart, without installing them one by one; removing the folder takes them out again, except fonts installed on their own (#866).
- What is being drawn shows over a black or dark fill: a freehand stroke, a polyline, 2-point line, B-spline or 3-point curve in progress and the 3-point tools' base line are a black hairline on a white line, the Bezier and Pen segments placed so far lie on a white line under their outline colour, the connector line and the zoom, text frame and rubber band boxes are on a white line too, and the node squares of a curve in progress have a white edge (#672).
- Shaping with open curves: an open curve or line as the target of Weld, Trim or Intersect is now cut by the closed objects and stays open (Trim keeps the parts outside, Intersect the part inside, Weld adds the parts outside to the welded shapes). An open curve acting on a closed object no longer distorts it: it takes no part, and the status bar points to the Knife tool for cutting a shape along a line (#828).
- The main window starts as it was left: maximized when it was left maximized (a maximize the window system dropped while the window opened is asked for again, and the first frames no longer overwrite the kept state), and a kept size that comes back the same with a larger interface or system text size, where it shrank at every start. A kept size larger than the monitor starts maximized (#787).
- Print Preview shows the printer choice at the top of its options, with the gear button that opens the chosen printer's Properties, its status and the number of copies with Collate, so the printer can be changed without going back to the Print dialog (#792).
- Print Merge docker: On every page is offered for Text and QR code fields too, not only pictures. Insert Field then puts the field in the same place on every page of the open drawing, and Create Merged Document fills those pages, one record each, instead of making a new drawing (#793).
- Fonts embedded in an opened `.cdr` file (or carried by a `.tdraw` file) that are not installed say "Embedded" at the end of their row in the font lists and after the name in the font box, so it is clear the text is drawn with the copy that came with the drawing (#518).
- A single click on an object with a drawing or shaping tool (the rectangle, ellipse, polygon and other box tools, Spiral, Common Shapes, Table outside a table, Knife, the connectors, Blend, Block Shadow, the shape brushes and others whose click had no job) selects it, Shift adds or takes it away, Ctrl reaches into groups, and a click on empty space lets go; a drag still draws. The line tools and Text keep their clicks (#271, by zalmanleib).
- The shape brushes (Smooth, Smear, Twirl, Attract and Repel, Smudge, Roughen) change only the selected objects: with nothing selected a drag no longer takes the object under the pointer, and the status bar says to select one first (#271, by zalmanleib).
- While typing text, Ctrl+L, Ctrl+E, Ctrl+R, Ctrl+J, Ctrl+H and Ctrl+N align it left, centre, right, justified, force justified or not at all, and a new alignment keeps text typed at a point in place at the reference point of the X and Y fields instead of moving it (#271, by zalmanleib).

## 0.19.4 (2026-10-26)

- Print Preview: a page smaller than the paper repeated on it (step and repeat, several pages per sheet) shows every copy. Only the copies in one band of the preview were drawn and white covered the rest; any group or clip frame moved by its own transform was left out the same way in large renders (#848).
- PDF import: Hindi written by office suites (the size set with the font, each glyph placed with a plain text matrix) comes in as whole words. A vowel sign placed a fraction of a point off the line, or the next syllable starting just before a sign's end, began a new text in the middle of a word, so क्षेत्र, शुद्धि or प्रार्थना came in as two texts and broke apart when edited; all 59 words of the test corpus and whole sentences now come back as typed (#771).
- Toolbars move into rows: the standard toolbar, the property bar, the Text, Zoom and Transform toolbars and the time tracker, docked at the top or the bottom, can share a row. Drag a toolbar's grip onto another toolbar's row to put it beside that one, or onto the line between two rows to give it a row of its own; the Text, Zoom and Transform toolbars and the time tracker now have grips and float or dock like the others. The rows are kept with the preferences (#837, #855).

- Print Preview with Print separations: the inks unticked under it no longer print; every plate printed before (#846).
- Number fields keep the mouse wheel: turning it over a field on the property bar (the angle, the outline width, any spin box) or in a docker or dialog steps the value notch after notch. The bar or panel around the field slid on with the wheel after the first notch and carried the field away from under the pointer, so the wheel stopped working on it. Parts of a notch from a touchpad now add up to one step instead of a step each (#783).
- Pick: an object chosen inside a group with Ctrl+click is dragged on its own from anywhere on it. Only a press on its centre mark kept it; anywhere else the drag took the whole group (#849).
- Clip frames, Edit Contents: the faded objects outside the frame can no longer be picked (click, marquee, Select All, Tab) while the contents are edited. Undo stops at the start of the edit: undoing past it leaves the edit with the frame as it was, instead of undoing the drawing behind it while the edit still showed; redo enters the edit again, and undoing Finish goes back into it (#851).
- Pictures look right at every zoom: pictures laid side by side (a map in tiles) no longer show pale lines where they meet, and a picture zoomed far out is drawn from a smaller copy of itself, so it shows its colours evenly instead of scattered, broken-up pixels. Pages of pictures zoomed out also draw faster (200 tiles at 21 %: 9 ms instead of 19 ms), so the new picture replaces the one shown while zooming sooner (#853).
- Variable Outline: a wide node at a corner no longer leaves a round bulge and a notch on the inside of the corner where the width falls away; the inside follows the width smoothly. Round corners and quick changes of width are drawn in finer steps, so the band's edges look smooth even when it is very wide (#776).
- Mesh Fill: a double-click inside the mesh puts a new node exactly where it was, on a new row and column through that point, and selects it; the new line went halfway between the nearest nodes instead, away from the click (#804).
- Shape tool: a node being dragged snaps to the other nodes and segments of its own curve too, not only to other objects (#673).
- PDF/X: with no CMYK profile set in Colour Management (or `TRACEDRAW_ICC` for `tracedraw-cli convert`), the output intent now embeds the CMYK profile the drawing carries, the profile its CMYK values are in, and an empty output condition takes that profile's name. It used to say "CGATS TR 001" with no profile, the name of another press (#643).
- PDF and SVG export: an outline set to scale with its object is written as wide as the drawing window draws it, its width times the object's scale. It was written at its own width, so a 10 mm outline on an object scaled to a quarter came out 10 mm wide instead of 2.5 mm, in .cdr files and in drawings scaled here alike.
- Opening .cdr files: objects of type 0x26 that keep a list of points (seen in version 22 files as the frames of clip frames) are read as the closed outline through those points. They were read as a two-point connector line, whose points then fell far off the page, so the pictures clipped to such a frame did not show at all.
- Paragraph text in Chinese and Japanese breaks lines between characters by the line-start and line-end rules: punctuation such as ，。、；：？！ and closing brackets and quotes no longer start a line, and opening brackets and quotes no longer end one; the character before goes to the next line with it (#847).

## 0.19.3 (2026-10-26)

- Blend panel: the Color Blend choices (Direct, Clockwise, Counterclockwise) show again; the Reset button's row took the rest of the panel and pushed them out of sight. An Apply button blends the two selected objects with the panel's settings (#815).
- Colour pickers have an eyedropper: in the picker of every colour swatch (Outline Pen, the Interactive Fill colours, the dockers) and in Edit Fill, press it and click the drawing to take the colour there; Esc stops it. It works while a dialog is open (#814, #817, #819).
- Shift+F3 opens Text > Change Case for the selected text, and the menu row shows it (#801).

## 0.19.2 (2026-10-26)

- Dimensions hold to the object they measure: a dimension whose ends were placed on nodes of an object (its corners, say) follows it when the object is moved, resized or reshaped, and its value changes with it. Moving the dimension by itself lets it go (#838).
- Opening a .cdr file with photos takes far less memory: the pictures are kept packed about half as small (22 MB instead of 40 MB for a 4000 x 3000 photo), and a picture placed many times is kept once instead of once per object (also in .cmx files). Saving a .cdr no longer takes several times a bitmap pattern fill's size in memory while the file is written.
- Outline Pen dialog: the selected objects now show each change (width, colour, style, corners, caps, arrowheads, calligraphy, variable widths, overprint) live while the dialog is open; OK keeps it as one undo step and Cancel or Esc puts the outline back (#741).
- Shape tool: two or more neighbouring nodes deleted together leave one curve between the nodes either side, fitted to the old outline, so it keeps its general shape. Line nodes deleted together left a straight line (#675).
- Ungroup keeps a group's perspective: each object takes a Perspective effect of its own, first in its stack (folded into a perspective it had), and stays drawn as it was in the group, its own contour or distortion included. The perspective disappeared when the group was ungrouped (#674).
- Contour: the offset is now the distance on the page also for a scaled, stretched or rotated object, so an 80 mm inside step lies 80 mm from the edges instead of 47 mm (#754).
- Area Fill: the new object now follows the outlines around the clicked area exactly, with their own curves and nodes only where they cross, instead of a jagged outline with many extra nodes traced from pixels; an overlap of two objects is filled exactly (#734).
- Hindi half letters and conjuncts with vowel signs: a syllable is never cut when text is shaped or broken into lines. A vowel sign or the second half of a conjunct in another colour or style, a syllable whose letters the font only partly has, and a word broken at the end of a line no longer show a halant under a half letter, a dotted circle or a sign on its own. PDF import reads Hindi as the typed words when the file places vowel signs apart (moved down or back), draws a vowel as two glyphs (ा + े for ो), draws a reph glyph at the end of a word, or gives the text as `ActualText`; Publish to PDF writes `ActualText` for such clusters, so its Hindi comes back exactly (all 59 words of the test corpus, 35 came back wrong before) (#599).
- Pen and Bezier: Ctrl+Z through the steps of a curve drawn with them goes back to drawing it. The tool comes back, and the line goes on from the last node left with a curved preview along the mirror of its previous handle, wherever the pointer is. This also works after closing: undoing the close reconnects to the last node. You can undo anchor by anchor and go on drawing at once (#631).
- Alt codes while editing text: the keypad digits typed under Alt are no longer typed too (with Num Lock on they also came as text, so Alt+0193 could give "0193" before the character), and a character the system composes as well is typed once. Fonts made for the symbol encoding (legacy script and dingbat fonts) now list their glyphs in the Glyphs panel with the keystroke that types them, and text typed in them draws their glyphs instead of another font's (#469).
- Colour picker and Edit Fill: the Model list offers the registration colour (it prints on every separation) after the models, as the reference program's list does; it could not be chosen anywhere (#286).
- Text: right-to-left words in left-to-right text (Arabic after an English word, or English inside Arabic) are now shaped and laid out right to left, joined, in the order the Unicode bidirectional rules give, instead of being laid out left to right like the English part. Digits inside right-to-left text read left to right, and paragraph text breaks such lines in reading order (#628).
- Text tool: in right-to-left text (Arabic, Hebrew) Ctrl+A and other selections now shade every selected letter, not only the first one at the right end, and the caret sits at the right of the letter it stands before, as it is read (#629).

## 0.19.1 (2026-10-25)

- Contour (outward, inward, to center) and the other offsets give clean curves: a circle's contour is a circle of four curves instead of hundreds of short straight pieces, sharp and bevelled corners stay sharp, and contours are drawn a little faster (#821).
- Convert to Curves and Break Apart of an enveloped, perspective or distorted object (text too) give smooth curves with their corners kept: a square in perspective is its four corners, a letter a few curves, where each was a row of short lines that Reduce Nodes then spoiled. Break Apart is in the canvas context menu for warped objects and artistic text. Letters an envelope pushes together stay filled where they overlap (#647, #840).
- Convert to Curves of a text keeps every letter filled: the strokes of a glyph that cross each other, as in many Chinese fonts, left hollow holes where they met once the text was curves; Break Apart of a warped text does the same (#800).
- Printing to a printer no longer freezes the window: the sheets are drawn and sent to the printer in the background, with "Printing on ..." in the status bar and the outcome there when it is done. On Windows a slow or stuck spooler left the window on "Processing" with a PowerShell window open until the program was closed (#810).
- Print Preview > Print to the system's PDF viewer opens each print in a page of its own; with the last print still open in the viewer, the new one did not open (#809).
- Edit Fill, the colour picker and the Color docker: a click near the black corner of the colour square in CMYK keeps its ring where it was put. All those points give K 100, which looks lighter on screen, so the ring jumped back up as if the click had not taken (#761).
- Layout > Insert Page Number on a master layer: one page number field goes on the master layer and shows the number of each page it is drawn on, on screen, in print and in exports, so one number manages them all. Before, nothing went on the master page; the numbers went on the pages instead (#807).
- Brush Strokes (Preset, Brush, Calligraphic, Expression): a stroke stays the path drawn, open and made of a few curves, with the brush as its live outline, so its nodes can be edited afterwards and the brush follows; it was a closed shape of many straight pieces. Apply to curve keeps the curve. The Sprayer still makes objects. Pages of such outlines draw as quickly as before (#829).
- Trim, Intersect and Weld with a picture as the target crop the picture to the result: its frame takes the shape and the picture shows inside it, also for a picture already cropped. The result was an empty curve that showed only in Wireframe (#827).
- Options > Customization > Commands: a tool or command can have several shortcuts. A new key joins the ones it has (a built-in key keeps working beside it) instead of replacing them, and each key has its own Delete (#822).
- Options > Customization > Commands has a Toolbar buttons list: every button of the Standard and Zoom toolbars (Zoom to Page, Fullscreen, Snap To and the others, and the separator). A button dragged off a bar can be dragged back from there; before, only Reset brought it back (#836).
- A right click anywhere on the Standard or Property bar, on its buttons and boxes too, opens the bar's menu, as on the toolbox (#835).
- Objects panel: every object's row has its own eye and padlock, to hide or show and lock or unlock it from the list, a locked object too (#832).

## 0.19.0 (2026-10-25)

- Opening `.cdr`: blends from versions 9 and X3 (and others storing them the same way) show their steps as a live blend between the two end objects; only the two ends were drawn before (#597).
- Print Merge: Add Column for a Number asks for its digits, starting value, "Continually increment value" with its step and an ending value; with an ending value the records from the start to the end are made at once, for serial, ticket or roll numbers without a data file (#791).
- Windows: printing to a printer from the Print dialog works. A print of one sheet, the usual case, failed before anything reached the printer, local or on the network, because the list of sheet sizes flattened in the PowerShell script; nothing came out and the spooler showed no job. The script also shows no "Printing page" box, gives up after five minutes instead of hanging, and writes what came of each print to `tracedraw.log` (#614).
- The mouse wheel zooms smoothly: each notch zooms in a glide over a few frames, as a scroll moves, instead of one jump of 15 %; a notch still zooms 15 % about the pointer (#766).

## 0.18.0 (2026-10-25)

- Large drawings with pictures use far less memory while editing: every undo step, background render and copied object now shares the picture data instead of copying it, so moving one curve in a drawing with 244 MB of pictures no longer adds 244 MB per step. Edit Fill and the font list preview no longer copy every step that can be redone. Saved files do not change.
- A Variable Length outline converted to an object (Ctrl+Shift+Q) is now a clean shape: a few real curves along each side and straight butt ends, instead of a whole row of anchors joined by straight lines. Along a curve, the band no longer has hairline cracks between its pieces, on screen or in exports (#627).
- The Glyphs panel's recent glyphs are readable again in the dark themes: their box takes the theme's field colour instead of staying white behind light glyphs (#660).
- Print Merge: with On every page, Create Merged Document leaves a picture frame whose cell is empty or whose picture does not load as it is instead of emptying it, says when nothing was left to fill (or when the pages hold no fields) instead of claiming it created pages, and Place Inside Frame on every page no longer stacks a second picture in a frame that already holds one (#618).
- The default palette of an RGB drawing is now a palette of RGB values defined directly (10% greys and multiples of 51, such as Navy Blue 0 51 153 and Purple 153 0 204) instead of values converted from CMYK at run time, and other CMYK colours with black in them convert as 255 (1 - min(1, ink + K)) per channel (#620).
- The OpenGL renderer now opens on graphics cards with OpenGL 2.1 only, such as the Intel HD Graphics of 2010 PCs: after OpenGL 3.3 and OpenGL ES it asks for a compatibility context, so such a PC draws on its card instead of the CPU's software adapter. The font list on the property bar no longer copies every font name on each frame while it is closed, which cost about 0.1 to 0.25 ms a frame with 1300 font families (#616).
- Tracing: neighbouring colour regions now overlap by 2 px under the lighter one, so no white hairlines of the page show between them; thin anti-aliased bands between two colours go to those colours instead of becoming slivers (#611).
- Save As starts at a drawing's own `.cdr` version when it was opened from a `.cdr` file, and at the version of Options > Global > File Formats otherwise; the hint there now says what the version and the Save new drawings as .cdr box do, and Save As no longer shows a meaningless "Version: v0" for TraceDraw files (#613).
- The wheel over the outline width box steps once per notch: a touchpad's small movements add up instead of running through the whole list, and a browser's notch (100 points or 3 lines) is one step, not two or three (#663).
- Editing a symmetry ends when another drawing comes to the front or a new one opens; the mode and its "Editing the symmetry" message in the status bar stayed on over the next drawing (#442).
- The Convert to Curves page says that a whole ellipse becomes four symmetrical nodes, as its checks and the test already did, and names the nodes in the drawing's Y-up order: right, top, left and bottom (#625).
- Print: the printer list is read off the interface's thread, so a slow print server no longer holds the window when the dialog opens; the printers are in name order with the default chosen where it stands, and the General tab shows the chosen printer's Status, Location and Comment, with a Properties button that opens the printer's own settings (#614).
- A combined curve with tens of thousands of nodes (a combined ornament) can be moved again: with snapping on, every frame over it compared each node with every other to find the snap points near the pointer, about 1.8 s a frame for 40,000 nodes, so a drag seemed to do nothing. Now a frame over it takes about 9 ms and a drag frame about 5 ms (#764).
- When the window is drawn on the CPU because Use hardware acceleration is off in Options, the status bar now says so, and a click on the note opens Options > General to turn it on; before, it only suggested installing the graphics card's driver (#616).
- The arrow keys nudge the selected guidelines when no object is selected: by the nudge distance, Shift by the super nudge and Ctrl by the micro nudge, one undo step per press; locked guidelines stay (#781).
- Windows: when the graphics driver crashes or hangs TraceDraw while the window is being opened, with nothing shown and the log ending at "Using wgpu", the next start now opens with OpenGL by itself (and the software adapter after that), and keeps using it; `--renderer=gl` no longer has to be typed (#570).
- A gradient fill and a gradient transparency now turn, mirror and skew with their object (the Pick tool's handles, the angle and mirror buttons, the Transformations docker, a turned group); before, the gradient kept running the same way on the page (#775).
- Print with Color: Separations now prints a plate for each spot ink, after the cyan, magenta, yellow and black ones, and the print preview offers its box again: the printed sheets had lost the drawing's list of spot inks, so only the four process plates came out (#779).
- The page tabs under the drawing slide to show the current page's tab whenever the page changes (a tab, the page navigator, PgUp and PgDn, Go To Page), and the mouse wheel over them slides them. In a drawing of many pages, a merged document for example, the tabs past the end of the row never showed before, so those pages could only be reached with Go To Page (#794).
- Drawings with much large text are smooth again: the box of a text object, asked for many times a frame by hovering, snapping and the Pen preview, copied the text's whole outline each time. It now comes from a box kept with the cached outline. On the reported drawing (45 lines of 257 pt text) a Pen preview frame went from 7.3 to 1.6 ms (#802).

- Browser version: the module is optimised for size with binaryen's wasm-opt and loses its name, debug and producer sections, 36.9 MB down to 31.6 MB (13.3 to 12.7 MB compressed). Nothing changes in what it does, and frame and script times stay the same.

- The default palette's menu has Rows > 1 Row, 2 Rows, 3 Rows, and with more than one row the swatches fill line by line (the second line goes on where the first ends) and scroll one swatch at a time (#610).
- .cdr files: text set in a semibold face (or a light, medium, extra bold or black one) now opens in that face instead of the bold or regular one, so it keeps its width on the page and in exported PDFs. The family is shown with its weight, "Titillium Web SemiBold", and draws with the installed face of that weight (#624).
- Dimensions: the label can be edited in place, with a double click on it (Pick tool) or a click with the Text tool. Text typed around the measured value becomes the prefix and suffix and the number keeps following the size; text without the value stays as typed, and deleting the label's text brings the measured value back; the label's font, size and colour are kept. Before, whatever was typed was lost at the next change. Break Apart now frees the lines, arrowheads and label as separate objects instead of doing nothing, and a new Options menu on the bar sets the extension lines' gap from the object and how far they run past the dimension line and a horizontal label; it stays open while its fields are typed in (#644).
- Blend: a new blend takes 20 steps instead of 5, so a front object blended to a copy just behind it gives a smooth side for 3D lettering instead of visible bands. A steps value already kept in the preferences stays as it is (#232).
- The Autofit Page popup no longer closes when its Live preview box or its margin field is clicked, so the preview can be switched on and off and watched; the box now sits on the margin's row (#659).
- Copy and paste no longer stall with a large picture on the system clipboard: Ctrl+C notes the text it wrote instead of reading the clipboard back, Windows tells by its clipboard change count whether anything was copied elsewhere, and elsewhere the picture is decoded only when there is no text or file to go by, or when it is what gets pasted, and sampled instead of hashed whole. With a 4000 x 3000 picture, pasting the copy here went from 177 ms to 118 ms on Linux and needs no read at all on Windows (#183).
- Properties docker: with nothing selected, the default outline and fill now sit under the same Outline and Fill header bars as with an object selected, so the two rows of paint kinds are named and the layout no longer changes on selecting; Add Outline and Add Fill show only once the object has an outline or a fill (the Outline box and the fill kinds give the first one); the Rectangle section's corner names wrap instead of being cut to "Top le..." (#487).
- A Variable Length outline, or any wide outline, inside a group or a clip frame is no longer cut flat at the edges of the strips the page is drawn in: a group is left out of a strip only when every object in it is (#626).
- A test now checks that Convert Outline to Object of a curved line with a thick butt-ended outline gives four cusp nodes and one curve per side (#622).
- .cdr files: rounded rectangles now open with Relative Corner Scaling off, like rectangles drawn in TraceDraw, so resizing one no longer stretches its corners and there is no need to switch the setting off rectangle by rectangle. A rectangle the file stores stretched shows round corners instead of oval ones (#619).
- Mask Outside Page is opaque, as in the reference program: nothing outside the page shows through, the mask is a near-black when the desktop is white, the selected objects' outlines are drawn over it in blue so their parts off the page can still be found, and the item sits in the View menu right after Pixels (#615).

- Convert Outline to Object (Ctrl+Shift+Q) on an outline with round ends or round corners: each round end is now two real curves meeting at its tip, with nodes where it meets the sides, and each round corner a curve, instead of many tiny straight segments. Curved sides also get fewer nodes (#623).

- File > Export now starts with the choices of the last export of the session (format, resolution, quality, transparency, pages and PDF settings; Selected only starts off again) instead of resetting them each time. To keep them for the next session too, turn on Options > Global > File Formats > Remember the settings of the last export, which new preferences now have on and which now keeps WebP lossless as well (#617).

- Print Preview: the box for each ink now stands under Print separations in the left panel, as asked, instead of in the top bar, where a drawing with many spot inks pushed Zoom and Close out of the window (#571).

- The Welcome Screen tab can be closed while a drawing is open: it shrinks to its house, which shows the Welcome Screen again, and stays closed after a restart until the Welcome Screen is shown (#677).
- Export: the Preset list now shows only the presets saved for the chosen format, and changing the format leaves a preset of another format unchosen. Each format has its own preset names, so saving a preset under a name another format's preset uses no longer replaces that one (#612).
- Shape tool: a click on a control handle keeps its node chosen, so the handle can be dragged next and the node stays where it is. The click used to choose the segment the handle lay on, and the drag that followed bent that segment, pulling the node's side along. The two handles a chosen segment shows can be dragged too (#639).
- Wrap Paragraph Text: the wrap style list beside the property bar button is now laid out as in the reference program: a Wrap style title, None (which stops the selected objects pushing paragraph text), the Contour and Square styles under their own headings, and the text wrap offset, greyed out while nothing wraps. Text already flows around shapes and pictures marked to wrap it (#661).

- `CONTRIBUTORS.md` lists the people who contributed code, with what each contributed, also those whose commits were recorded under another name; the README links it.
- The page size list on the property bar ends with Custom..., which opens the Page Size dialog with the page's width and height to type any size; it is the entry ticked when the page matches no preset (#650).

- The log now names the processor, its threads and the Windows build, says at the first frame how long the start took and how large the window is in pixels, and with `RUST_LOG=debug` sums up the frames every 10 s (count, input, build times, gaps and why they were asked for), so a report of a slow window shows where the time goes (#616).

- The preferences are no longer serialized on every frame to see whether they changed: only on frames with a key, text, a click or a dropped file, and at most once a second otherwise. A pointer move now costs about 0.1 µs there instead of about 20 µs, more on an older PC (#616).

- Pictures are decoded once and kept for every thread that draws the page. The canvas draws in bands on one thread per core, and each band decoded every picture it crossed again on every redraw: six large photos took about 1.1 s and 540 MB extra per redraw on four cores (more with more cores). Now a redraw after the first takes about 20 ms, thumbnails and the navigator reuse the same pictures, and the decoded pictures kept are capped at 512 MB.

- Tracing (Quick Trace and Outline Trace): outlines come out as smooth Bezier curves fitted to the shapes instead of stair-stepped polylines, with far fewer nodes; corners stay sharp (Corner smoothness rounds more of them), and stray specks and the fringes of soft edges no longer become curves of their own (fewer the lower the detail). Smoothing 0 still follows the pixel edges (#611).

- Saving a `.tdraw` and auto-backup now write the file as it is made instead of building the whole file in memory first: saving a drawing of 183 MB peaked at 2.4 GB and left about 2 GB held afterwards; now it stays near the drawing's own size. Auto-backup also runs on a thread of its own, so the window no longer stops every 20 minutes while a large drawing is backed up, and a backup cut short never replaces a whole one.

- Variable Length tool: a stroke made much wider than its own outline width is now drawn whole on screen. Parts of it reaching far from the path could be left out of the picture, so the stroke looked cut off with a flat edge on one side (#626).

- Canvas overlays show on any colour: a shape being drawn has a black hairline on a white line over its outline, so it no longer disappears into a black outline; the selection handles are black with a white edge; and the Shape tool's and curve tools' control handles are dashed on a white line, with white-edged arrow heads (#670).

- Shape tool: corner widgets. A curve being edited shows a small circle inside each corner; dragging one into its corner rounds the corners on the canvas as you drag, every corner of the curve together, or only the chosen nodes' corners when nodes are chosen. Works on open curves too; the status bar shows the radius, and the drag is one undo step (#635).

- View > Page > Mask Outside Page: an optional mode that covers everything outside the page with the desktop colour, with the page border drawn over the objects, so the page shows as it will print while you keep drawing and editing. Objects outside the page still show faintly and stay editable; the setting is kept with the other view settings (#615).

- `tracedraw-cli convert`: what the .cdr reader skipped or guessed (a fill it cannot read yet, a page size it does not take, objects left out) now goes to stderr as warnings, as for the other formats it reads. Only `info` showed it, so a converted file could miss part of the drawing without a word.

- Opening .cdr files: pages longer than 10 m on a side, such as a 22 m banner, keep their size up to 100 m, the bound the CMX reader already had. They opened as A4, with the warning that the page size looked wrong.

- Publish to PDF: a picture wholly outside the page and its bleed stays out of the file. It could not show, but its pixels were written: a photo left beside the page could make up most of the file's size (#642).

- When a software adapter draws the window (no usable graphics card driver), TraceDraw keeps to 30 frames a second, draws no shadows and uses the system's pointers instead of the drawn cross hair and I-beam, and says so once in the status bar. On an older PC every pointer move redrew the whole window on the CPU, which took every core (#616).

- With hardware acceleration on, TraceDraw no longer draws the window on the CPU's software adapter when no graphics card is usable: it tries OpenGL next and the software adapter only last, keeps going when the OpenGL set-up panics, says in a message box when no renderer can open the window, and logs every graphics adapter with why it was taken or left. An older PC sat at full CPU with the window crawling, with hardware acceleration on or off, and `--renderer=gl` showed nothing (#616).

- Print on Windows: the Printer list now shows the printers Windows has, network printers included, the default one first, and prints straight to the one chosen with the copies asked for. Before, Windows offered only the PDF viewer and PDF or PostScript files (#614).

- Property bar: the mouse wheel over the outline width box now changes the selected objects' outline to the next width of the list up or down, one undo step a notch (#663).

- Convert Outline to Object (Ctrl+Shift+Q) now gives clean shapes: curves stay real curves with handles, straight parts stay straight, and the result has as few nodes as keep its shape, instead of hundreds of tiny straight segments. The outline of an ellipse also no longer has a notch where the ellipse starts (#622).

- Print Merge, On every page: Create Merged Document and Print Merged Document now fill the pages of the open drawing, one record per page, instead of making a new drawing. With no placeholder picture but a pictures folder chosen, Place Inside Frame gives each page of the drawing its own picture from the folder, in name order, in the same frame (#618).

- Pen and Bezier: a click on an end node of an open curve, selected or not, now goes on drawing that curve as if you had never stopped. The next segment leaves along the mirror of the end's handle, so the preview is curved and the join is smooth. The curve had to be selected first, and the new segment always started straight (#630).

- Font lists: the mouse wheel over the font name box (with the list closed) now steps to the next or previous font, changing the selected or edited text at each notch (#665).

- Shape tool on rectangles: corner nodes can be chosen several at a time, with Shift+click or Ctrl+click (which also takes a chosen corner out) or with a marquee or freehand marquee over them, and dragging a chosen corner then rounds just the chosen corners, so the two top corners or any three can be rounded together. Only all four corners or a single one could be rounded before (#671).

- Tools > Options is now one window with a tab for each set of options along the top (TraceDraw, Customization, Tools, Global, Workspaces, AI and Document Options while a drawing is open), so every option can be reached without closing the dialog and opening another menu item. Each tab comes back to the page it showed last, and OK or Cancel covers the changes made on every tab (#633).

- The default colour palette can show two or three rows of swatches (columns when docked on a side), set in Options > Customization > Color Palette > Rows, so more colours are in reach without scrolling; the arrows and the wheel then scroll a column at a time (#610).

- Convert to Curves (Ctrl+Q) now works on dimensions: the dimension becomes a group of plain curves (its lines, its filled arrowheads and its label as curves) that no longer measures, each editable with the Shape tool. It used to do nothing. An angular dimension's arc is now a few smooth curves instead of dozens of short lines (#640).

- Text editing on the canvas: the caret is now a black bar with a white edge instead of the interface's text colour, which in the dark themes was a light grey lost on the white page, and selected characters get a clearly darker shade. Table cells being edited show the same caret and shade (#632).

- Font lists now follow the dark themes: the font names are drawn in the theme's text colour instead of near black on the dark list, and so are the characters of the Glyphs docker. The preview under the list and the font sampler keep dark text on their white card (#660).

- The default palette of an RGB drawing now holds clean RGB values instead of the CMYK colours as the press would print them: black is 0 0 0, white 255 255 255, red 255 0 0, and the tints are the matching RGB tints. CMYK drawings keep the CMYK palette (#620).

- Autofit Page: the popup has a Live preview check box. On (as before), the page shows the size it would get while the margin is typed; off, the page stays as it is until Autofit Page is clicked. The choice is kept (#659).

- An ellipse converted to curves is now one closed curve of four smooth nodes with equal handles, at its right, top, left and bottom. It was an open curve with a fifth node on top of the first, and large ellipses got more nodes still, so it had to be closed by hand (#625).

- Save As now starts at the `.cdr` version chosen in Options > Global > File Formats instead of always offering version 27. A new option there, Save new drawings as .cdr files, makes Save As start at the CDR type for a drawing without a file (#613).

## 0.17.0 (2026-10-24)

- Convert to Curves (Ctrl+Q) and Break Apart (Ctrl+K) now make an envelope, perspective or distortion part of the curves. Convert to Curves did nothing on an enveloped curve, and Break Apart lost the warp. Enveloped text breaks into one curve per letter, each warped as the whole text was, holes kept; a group with an envelope stays a group of warped curves. A warp after a contour, blend or extrusion stays live, and bitmaps keep theirs (#647).

- Outline Pen: the Variable Outline width nodes can be dragged along their bar, staying between their neighbours, with the fields following (a drag there moved the whole dialog). A drag on empty bar adds a node and slides it, and a typed position keeps the nodes in order (#652).

- Edit Fill: the dialog keeps its size and place, and OK and Cancel stay put, when switching between no fill, uniform, gradient, pattern, texture and mesh (#654).

- The drawing tools' cross hair and the text I-beam are now drawn black with a white edge, so they show on a page of any colour; on Windows the system ones could appear white on a white page. A shape being drawn now follows the pointer from the first frame of the drag, and the window keeps one frame queued instead of two, so what is drawn under the pointer trails it less (#655).

- Opening a file from the file manager or the command line while TraceDraw is running now opens it in a new tab of the running window, which comes to the front, instead of starting the whole program again. `--new-instance` still opens a separate window. On Linux, selecting several files opens them all in one window (#646).

- Opening a file that is already open (Welcome Screen, File > Open, Open Recent, a dropped file, the command line) now shows its tab instead of opening it again in a new one, unsaved changes kept. A .cdr or other drawing counts as that file until it is saved under another name; templates still open as new drawings (#645).

- Symmetry: after Finish, a symmetry moves, scales, rotates and mirrors as one object, with its copies and mirror lines. The lines used to stay where they were, so the copies mirrored the move and a handle rearranged them, as if the symmetry were still being edited. The drag outline shows the copies too. Duplicate, Step and Repeat and copies made during a drag take the lines along, and a symmetry inside a group follows the group. While editing, moving the object still leaves the lines in place.

- Property bars: the up and down arrows (and the mouse wheel) of the rotation angle, scale %, arc angles, dimension text size and line width, corner radius and Transform toolbar fields now apply at every click. After clicking into the box, only the first arrow click worked.

- A right click while dragging (move, scale, rotate, skew, and mirroring with Ctrl and a side handle) now leaves the original and makes the copy whichever button you release first. Releasing the left button first, or both at once, used to count as a click: on empty canvas it emptied the selection and nothing was copied. The scale, rotate and skew pointers now also show the copy's plus (#528).

- Segment Delete: dragging a box deletes every segment the box touches, whole, up to the crossings at its ends (a closed shape nothing crosses goes whole), as other programs do; it used to delete only the segments lying wholly inside, so a box across an ornament deleted nothing. A click that slips a few pixels still deletes, and a drag on a crowded page is about ten times faster (#528).

## 0.16.0 (2026-10-24)

- Text > Join Lines into Paragraph: the selected lines of artistic text (a PDF brings its text one object per line) become one paragraph text with the same first baseline and line spacing, so text typed into it flows on to the next lines (#599).

- Text: Hindi (and the other Indian scripts) after a Latin word on the same line keeps its half letters and conjuncts; the line was shaped with the Latin rules, which showed the halant under half letters. Seen in text taken from PDFs (#599).

- Print Preview: Transform shows handles around the artwork to stretch it on either axis (or the x and y percentages typed) and a drag inside moves it; Print prints it so (#595). Print: a Smart Board 16:9 sheet, Custom sheet sizes that can be saved with a name, and Stretch to fill, which fills the sheet inside the margins on each axis (#591).

- Export: WebP can be lossy (Lossless off, the default): the quality then shrinks the file as other programs do, a tenth of the lossless size at low qualities. Lossless keeps the earlier behaviour. The browser version stays lossless (#592).

- Autofit Page: while its popup is open, the page itself grows or shrinks live to the size it would take around the drawing, following the margin as it is typed (#594, #604).

- Print Merge: "On every page" beside the placeholder picture puts it in the same shape and place on every page with Place Inside Frame, in the matching shape of each page or a copy of the frame (#598).

- Edit Bitmap opens pictures in the bitmap editor of your choice (Tools > Options > Global > File Locations: GIMP or any program); empty, it uses the system's program for PNG files as before (#593).

## 0.15.0 (2026-10-24)

- Windows: the program carries the C runtime it needs (no Visual C++ redistributable required). A start that fails now says so: the log is written to `tracedraw.log` in the settings folder (`%APPDATA%\TraceDraw`, or `tracedraw-settings` beside a portable copy), and a crash shows a message and writes `crash.log` there; nothing was shown before (#570).

- PDF import in Text mode: Hindi and other Devanagari text comes back in the order it is typed. The short i sign (ि) and the reph (र्) were taken in the order the PDF draws them, so they sat on the wrong letters (#585).

- Print Preview: paper margins for each side, typed in the left panel or set by dragging their dashed lines on the sheet; the pages go inside them, clear of what the printer cannot print (#580, #582).

- Print Merge: Place Inside Frame with nothing selected waits for a click on the shape, with the Place Inside arrow as the pointer, and takes the shape clicked (also inside a group). It was greyed until a shape was selected (#577).

- Print Preview with separations: a box for each ink, always in view, instead of a list. One ink shows its plate; several show what those inks print together, to check two or three colours at once (#571).

- Duplicate (Ctrl+D) repeats what was done to the copy: duplicate an object, move, scale or turn the copy while it stays selected, and each next Duplicate takes the same step from the last copy, as in the reference program. It always used the duplicate offset (#559).

- Print Preview: a drag on the sheet moves the artwork on the paper before printing (clear of the margins the printer cannot print), with Artwork position fields and Reset on the left; Print prints it where it was put. A drag moved only the page information (#536).

- Export: presets with names. "+" saves the format and its settings (resolution, quality, transparency, selection only, all pages, PDF choices) as a preset; the Preset list puts them back, each shown with its format; presets can be renamed and deleted, and stay in the preferences (#547).

- Publish to PDF: Objects > Bitmaps downsamples colour, gray and black-and-white pictures to a resolution of your choice, compresses colour and gray pictures as JPEG with a quality, and can leave text and line art uncompressed. Gray pictures take one channel. The Web preset now makes small files (150 dpi, JPEG 75). Pictures always went in at full resolution, lossless (#550).

- Dimensions: the property bar of the dimension tools sets the label's font and colour, the line colour and width, the arrowheads (arrow, open arrow, dot, tick, none) and rounded line ends, for new dimensions and the selected ones. Lines were always black hairlines and the label took the current font (#475).

- Text: Options > Text > Display > Text units sets the unit font sizes are shown and typed in (points at first; millimetres, centimetres, inches and the other units of the rulers), for large work. A unit typed with the value counts, as in the other fields (#529).

- Text: a font or size picked on the property bar or the Text docker with nothing selected asks whether it becomes the default for new text, as the reference program does. It changed the default without a word, so new text could start in another font than Arial after a pick made by mistake.

- Fit Text to Path: while it waits for the path, the pointer near a path shows the text where a click puts it, centred on the nearest point and as far from the path as the pointer, with the distance and a red mark where the text begins; the click places it so. Nothing showed until the click, which always put the text on the path itself.

- Toolbars: a new installation starts with Small buttons (Default Button Size), as the reference program does; it started with Medium. A size already saved in the preferences stays.

- Symmetry: Finish selects the whole symmetry. The object picked inside it while editing (the last one drawn, for one) stayed selected, so the next drag moved that object alone, as if it had been double-clicked into.

- Toolbars: the buttons of the Standard and Zoom toolbars can be changed. With Tools > Options > Customization open (or with Alt held), drag a button along its bar to move it, onto the other bar, or off the bars to remove it; drag a command or a tool from the Commands page onto a bar to add it. Reset on the Command Bars page brings the first buttons back. The Zoom toolbar shows pictures, as the Zoom tool's property bar does; it showed words only (#563).

- Downloads: Linux on 64-bit ARM (arm64) gets its own `.deb`, AppImage and `.tar.gz`, built on an ARM machine (#561).

- Browser version: with the interface in Chinese or Japanese, the menus and dialogs show their text; they showed empty boxes, the browser having no system fonts. The page fetches Noto Sans SC, TC or JP from its font folder when it starts. Variable interface fonts are drawn at Regular weight, not at their default, which is Thin in some (#561).

- Shaping: Weld, Trim and Intersect work with one object selected: choose the command (Object > Shaping, the Shaping panel or the property bar), then click the object it acts on, as in other drawing programs. The buttons were greyed until two objects were selected, and which one was cut depended on the selection order (#562).

- Pen: the preview follows the pointer as smoothly as in 0.13.0; its white line, drawn as a path shape, cost about a third of each frame. The white line is now as thin as the dashes, so the preview reads mostly blue with short white gaps (#560).

- Symmetry: after Finish the mirror lines no longer show on the selected symmetry, so it reads as one object; they stayed whenever it was selected and editing looked still open.

## 0.14.0 (2026-10-23)

- Variable Outline: a width node dragged along the path slides along it, and its reach handles change its width; a drag on a node could only change its width (#479).

- Variable Outline: the width eases along a smooth curve between nodes however close they are; short spans on long paths ran in a few straight pieces. Shift or Alt for one side counts while dragging, not only when the drag starts (#492).

- Connectors: a copy of a connected object (Duplicate, right-click drag copy, Repeat, Copy and Paste) comes with a copy of its connector, of the same kind and style, joining the object at the other end to the copy (#476).

- Convert Outline to Object (Ctrl+Shift+Q) keeps a dashed outline's dashes: each dash becomes a closed shape and the gaps stay empty; the result was a solid band (#490).

- Repeat (Ctrl+R) after a copy made with a right click during a drag is one undo step, as is that copy itself; the first Undo only moved the new copy back onto the original (#471).

- Export: WebP takes the quality setting; it was ignored. The files stay lossless, and below 100 their colours are first rounded to fewer levels (near-lossless), so a lower quality makes a smaller file (#544).

- Edit Fill > Uniform: the colour's model list and a field for each value (C, M, Y, K or R, G, B and so on), as in the colour picker; it showed only the swatch and the hex code (#468).

- Colour palette: in an RGB drawing the default palette shows and applies its colours as RGB (converted, names kept); it applied CMYK colours. CMYK drawings keep the CMYK palette (#483).

- Welcome Screen: the colour palette shows pale and takes no clicks, and the dockers that work on a drawing say to open or create one, instead of showing and changing the selection of the drawing behind it (#482).

- Toolbox: docked on a side it fits the window's height, its buttons and icons getting smaller (down to 22 px) instead of the tools running off the bottom and needing the scroll arrows (#494).

- Property bar: the page size, nudge, duplicate distance, autofit margin and origin fields have one set of up/down arrows; a second pair beside each field did the same job. The field's own arrows and wheel step as the extra pair did (1 unit, 0.1 for nudge and duplicate) (#488).

- Shaping: Back Minus Front, Front Minus Back and Trim on a group cut each of its objects, each keeping its own fill and outline; the result had no fill or outline and showed only in Wireframe. Weld and Intersect with a group take its first filled object's attributes (#539).

- Clip frame (Place Inside Frame) Edit Contents: a group inside can be ungrouped and its members stay in the frame, drawn in full and back inside on Finish; objects drawn or pasted while editing go in too. The members faded as if outside and Finish emptied the frame (#537).

- Print: Fit to page shrinks objects that run off the page (or the selection printed) to fit inside it, centred; they were cut at the page's edge. Print Preview's Close button sits at the right end of its bar (#534).

- Options > Customization > Appearance: the desktop colour button shows the colour around the page, and white picked makes the desktop white; white stood for the theme's colour, so picking it changed nothing until another colour had been picked (#530).

- Smart Fill fills enclosed areas off the page, on the desktop, and in drawings of any size; it worked only inside the page and gave up on drawings much larger than it. An area is open when the flood reaches past the outlines, not when it covers almost the whole page (#499).

- Print: Keep page size keeps the pages at their size for Several Pages per Sheet and Step and Repeat too; the pages shrank once the columns and rows no longer fit the sheet. The grid stays as set, and a warning says when pages run off the sheet. Columns and rows go up to 99, not 20 (#496).

- Print Merge: the data grid selects a block of cells by dragging across them (or over the record numbers or field names for whole records or fields), with Add (as many empty records after it), Delete (its records) and Clear (its cells, also the Delete key) (#501).

- Layout > Delete Page opens a dialog: the page to delete (the current one to start with) and, with Through to page, the last page of a range, deleted in one undo step; it deleted the current page only (#500).

- Print Merge: CSV files saved by Excel on Windows import; they are in code page 1252, not UTF-8, and failed to read. UTF-16 files (Excel's Unicode Text) and tab-separated files import too, and a UTF-8 byte order mark no longer sticks to the first field's name (#495).

- Paragraph text: the bottom handle of a selected frame drags it taller; the flow tab sat under that handle and took the press, so the next click linked the text to another object (after Text > Convert, for one). The tab now hangs below the handle (#503).

- Fonts: on Windows the font lists offer only the fonts Windows lists as installed; a font uninstalled while in use left its file in the font folder until the next restart and was still offered. A font for new text that is no longer installed gives way to Arial at start, instead of being named in the bars while text was drawn with a stand-in (#504).

- Text: typing in a text keeps its own hyphenation; it took the setting for new text, so editing a text turned its hyphenation on or off behind the menu's back. Text > Use Hyphenation while editing acts on the paragraphs at the caret, and its check mark shows them (#470).

- Text: Alt codes type their character on Windows (Alt+0193 gives Á, or the glyph a symbol font draws there, as the Glyphs docker's keystroke says); nothing was typed (#469).

- Text: Ctrl+Space while editing text finishes the text and switches to the Pick tool, as in the reference program; it typed a space (#478).

- Shape tool on text: in right-to-left text (Arabic, Hebrew) each character's node sits under its own letter; they were bunched at one end in left-to-right order (#474).

- Pen: the dashed blue preview to the pointer is drawn over a solid white line, so it stays one line over dark objects and any colour, as in the reference program (#489).

- Pen and Bezier: the constrain key (Ctrl) keeps the closing handle to the constrain angle when dragging on the first node to close a curve, as on every other node (#473).

- Shape tool: deleting the first node of a closed curve keeps the curve's shape, as deleting any other node does; the two segments around it kept their old handles and the curve changed (#472).

- Shape tool: Ctrl or Shift while dragging a control handle keeps it to the constrain angle (15° by default), and a smooth node's other handle keeps following; Ctrl turned the node into a cusp instead and Shift did nothing (#510).

- Interactive Fill: the colour node's bar and the colour picker popup have a dotted grip that drags them off the object they cover; they stay where they were put while open.

- About TraceDraw: a green ALPHA badge beside the version, with a tooltip saying what the alpha stage means.

- Symmetry: a double click on a symmetry goes back into editing it, as in the reference program; it opened the group's focus mode instead, so after Finish the only way back was the menu.

- Guidelines: Copy, Cut and Paste work on selected guidelines (no objects selected), pasting them in the same place, and the Transformations docker moves, rotates, scales and copies them; the Help says where the Ctrl+D duplicate offset is set (#376).

- Hyphenation: a word with a comma, full stop or quotation mark next to it can break at a line's end (before, any punctuation kept the whole word on the next line, in every language), words with an apostrophe break too (Italian *dell'informazione*), and Text > Use Hyphenation now turns hyphenation on for the selected text instead of only for text made afterwards. Checked with the Italian, English, Portuguese and German dictionaries (#255).

- Text on a path: changing the distance from the path, or pressing a mirror button with a distance set, no longer slides the text round the path; text centred on the bottom of a circle (Placement: Bottom, or both mirrors) stays centred as it moves in or out, as in the reference program (#193).

- Print Merge docker: Place Inside Frame makes a shape of your own (a circle, an oval, any outline, also inside a grouped card design) the frame of a picture field, so each card's photo goes inside it, placed by Align and scale; before, the photo went into a square frame the docker made, and a shape inside a group could not be a frame at all (#460).

- Browser version: every macro command (Macro Manager, Run Macro, Macro Editor, Load Macro Project) opens a message that macros run in the desktop version, with a button to its downloads; before, the commands did nothing visible or only wrote a line in the status bar.

- Symmetry editing: a symmetry made of one object picks the object alone, its selection box no longer spanning the copies; a click on an object where a mirror line crosses it picks the object instead of showing the line's turning handles. The symmetry bar's X and Y fields keep their label, box and arrows on one line.

- Browser version: opening or creating a drawing froze the page in 0.13.0 (the time measured for the lag fix of #441 is not available in the browser and stopped the program); it opens and draws again.

- Status bar: a double click on the fill opens the Edit Fill dialog box and on the outline the Outline Pen dialog box, as in the reference program, instead of the Properties panel (#465).

## 0.13.0 (2026-10-23)

- Print Merge docker: a Pictures folder for the photos the data names, so a folder of 50 photos fills 50 ID cards, each with its own; a name without an extension finds the .jpg or .png (#460).

- Tracing: Repeat Last Trace and Open Last Trace Settings in the Bitmaps menu, and a bitmap's right-click menu now lists them with Centerline Trace and Outline Trace and all their presets, not only Quick Trace (#452).

- Glyphs docker: the font list is the property bar's, so a font can be found by typing its name, and the line above the glyphs shows the keystroke that types the glyph (Alt+0169 for ©), as in the reference program. While editing text, Alt and a number types the character on every system, not only on Windows (#447).

- Transparency tool: a gradient transparency shows its nodes on the object, as Interactive Fill does: start, end, skew, nodes added on the line and the midpoint slider between two nodes to set where the blend is half way. A drag sets the gradient from where it began to where it ended, not only its angle (#448).

- Options > Text > Display: the default font and size can be changed while a text is selected, and they no longer turn into the selected text's font and size; with nothing selected the Text tool's bar shows them (#449).

- Envelope, perspective and distortions keep curves smooth: the letters of an enveloped text were drawn in visible facets (#455).

- T-shirt mockups work as in the reference program, and a new Help page walks through one (Effects > Making a T-shirt mockup). Three fixes made it possible:
  - An envelope, perspective or distortion on a group now bends each object from the box of the whole group and keeps its colours; before, the group vanished.
  - Uniform transparency keeps its merge mode (Multiply and the others), where it used to keep Normal only.
  - Objects inside a clip frame blend with what lies beneath the frame through their merge mode, so a print clipped to a shirt keeps the cloth's shading.

- Colour picker: Lab, Grayscale and YIQ can be chosen again in the Model list; the click on them closed the popup instead (#440). The Interactive Fill bar, the Text docker, shadows, gradient stops, tables, the palette editor and the page background now use the same picker as the Properties docker, with its Model list, and dragging in the bar's colour is one undo step that adds one colour to the document palette, not one for every colour passed over (#442).

- Options > General: Use hardware acceleration, with the graphics card to draw with, as in the reference program (#445). The window is now drawn with Direct3D 12, Vulkan or Metal. Turned off, or when the card cannot open the window (a virtual machine, for example), TraceDraw uses the software renderer and then OpenGL by itself and remembers what worked, so it opens where it used to fail (#439).

- Print Merge works as in the reference program: Finish opens the Print Merge docker instead of merging at once. There you insert Text, Image and QR code fields from the data source's columns. An image field takes a placeholder picture and an Align and scale setting (anchor point, then Actual size, Fit proportionally, Fill proportionally or Stretch to fill). Update Field changes the selected placeholder, and Create Merged Document or Print Merged Document makes the merged pages in a new drawing (#437).

- Tools > Options > AI > Service: the Model field has a list to pick from, with the models the service offers (Refresh list asks it for them) or the usual ones of its kind, as the AI bar has.

- Edit Fill (F11): the selection shows the fill as you change it, as in the reference program, so a gradient can be judged on the object. OK keeps it as one undo step, and Cancel or Esc puts the old fill back (#436).

- Less lag on heavy drawings (#345): the page is drawn on all the processor cores at once, and a page that takes long to draw is drawn on a thread of its own while the last picture stays on screen, so the window keeps answering. Each undo step keeps one copy of the drawing instead of two, objects are numbered with a faster table, snapping finds crossings without solving every pair of curves, and the bars, the cursor and the hover marks measure a large selection once instead of every frame. On a page of 5000 curves, moving an object went from 126 ms to 18 ms a frame, and hovering over all of them selected from 82 ms to 20 ms.

- Toolbox: a right click on a tool button or on its empty part opens the bars' menu (Customize, button size, the bars), as on the other bars; it used to open the flyout or nothing.

- Symmetry editing as in the reference program: a click or a marquee picks one of the objects inside (not the whole symmetry, not a copy), and the bar and the lines stay while you work inside it. The mirror lines can be dragged to move them, and a click on a line shows handles that turn them. The bar has icons, spin fields, Show full preview (copies as red outlines when off) and Show symmetry lines.
- Number fields everywhere (bars, dockers, dialogs, options) have spin arrows and step with the mouse wheel; Ctrl takes a tenth of a step and Shift ten steps.

- Toolbars: the Default Button Size starts at Medium on a new installation; a size already chosen is kept.

- Shape tool on a B-spline: control points can be chosen by a click, Shift+click or a marquee, as nodes are; Delete removes only the chosen points (it deleted the whole curve), and dragging a chosen point moves all the chosen ones.

- Segment Delete: curves inside a group (an imported ornament, for example) are now cut by a click and by a drag, as in the reference program; nothing happened before. A drag also takes level and upright lines, which it used to skip (#427).

## 0.12.0 (2026-10-23)

- Break Apart on a contour, blend or extrusion: Ctrl+K and the right-click menu now offer Break Contour (Blend, Extrude) Group Apart, as in the reference program, instead of a greyed-out Break Apart. The object stays and the steps come out as a group beside it, also from the dockers' Break apart buttons (#424).

- Colour Eyedropper: after a colour is sampled the pointer turns into the paint bucket, as in the reference program, and pictures give their colour again when they are grouped, inside a clip frame or kept as JPEG (#403).

- Hyphenation in any language: put a hyphenation dictionary (a `hyph_*.dic` pattern file) where the spelling dictionaries go and choose it under Options > Text > Paragraph Text > Hyphenation patterns (#255).

- Spell checking in any language: put Hunspell dictionaries (the `.dic` and `.aff` files many programs use) in the folder Options > Text > Spelling opens, or point it at a folder that has them. Words are checked with their prefixes, suffixes and compounds, suggestions use the dictionary's own replacement table, and each dictionary can be switched off (#255).

- Options > Text: Display, Paragraph Text, Fonts, Spelling and Autocorrect tabs, as in the reference program. Autocorrect can now capitalize the names of days, turn typed web addresses into hyperlinks, use the quotation marks of another language or ones you choose, keep straight quotes after numbers, and switch the replacement table off; Text > Writing Tools > Autocorrect opens that tab. Fonts sets how many recently used fonts the font lists show (up to 20), and Spelling keeps the spell checker options and can add its corrections to Autocorrect (#369).

- Segment Delete: after one stretch of a curve was deleted, clicking the stretch past the next crossing could delete the whole curve, because the first cut ended a hair off the outline it crossed. Crossings now lie exactly on both outlines (#354).

- Glyphs panel: a search field finds characters by themselves or by Unicode code point (`U+20AC`, `0x20ac`, `20AC`), and a line above the grid shows the code point, character and range of the glyph under the pointer (#375).

- Print and Print Preview: Keep page size stops pages from shrinking on a fixed sheet to make room for the marks, and a drag on the sheet in Print Preview moves the page information; Keep page size is remembered for the next print (#388).

- Print and Print Preview: a Back side option for n-up and step-and-repeat layouts runs the pages right to left on the sheet, so the back of double-sided ID cards prints right behind the front, without mirroring the text (#387).

- PDF and AI files: opening or importing one asks how to read its text, as in the reference program. Text keeps it editable; Curves draws each letter from the fonts the file embeds, so the text looks exactly as in the file even when those fonts are not installed. Options > File Formats keeps the choice (#386).

- Print Merge: imports Excel workbooks (`.xlsx`) as well as CSV, guesses the picture, QR and text fields and shows their types after the import. Fields are now filled inside groups and frames, so grouped ID cards merge; a photo placeholder inside a frame fills the frame, and a shape named `<Field>` takes the picture inside it. Page copies with frames used to share ids with the template, so a merge could change the template instead of the copy (#377, #378).

- Edit Fill (F11): a dialog as in the reference program, with the fill types across the top and, for gradients, a preview, the fill picker, type and repeat, a band of colours whose nodes can be picked, dragged, added and removed, the node's colour, transparency, position and midpoint, Reverse, rotation, centre and edge pad, and Overprint fill. OK changes only what was changed. F11 opened the Properties panel before (#361).

- Outline Pen (F12): a dialog with every outline setting laid out in groups as in the reference program: General (colour, width with its unit, style, mitre limit, dashes, corners, line caps, position), Arrowheads, Calligraphy with a picture of the nib, Behind fill, Scale with object, Variable Outline (width nodes, sides and total) and Overprint outline. OK changes only what was changed; with nothing selected it sets the outline of new objects. F12 opened the Properties panel before (#357).

- Bitmap effects on vector objects (3D Rotate and the others) are drawn at the drawing's rendering resolution from Layout > Page Setup instead of a fixed 300 dpi, up to 16 million pixels instead of 4, and are made again when the resolution changes; set it higher for sharper results on screen, in PDF and in exported images (#382).

- Paragraph text: frames have a flow tab under their bottom edge, as in the reference program. A click on it, then a click on another frame links them, even on another page; a click or drag on an empty spot makes a new frame there that the text flows into. Linking used to need both frames selected, so frames on different pages could not be linked (#371).

- Place Inside Frame, Order > In Front Of and Behind: while they wait for the click on the target object, the pointer is a thick arrow pointing at it, as in the reference program; it stayed the ordinary pointer (#379).

- Variable Outline tool: Shift and Ctrl now work as in the reference program. A drag started with Shift changes only the side the pointer is on (Alt still does too); Shift+click picks a run of width nodes and Ctrl+click adds one or takes it out; a drag or the property bar then changes all the picked nodes, and Delete removes them (#374).

- Guidelines: several selected guidelines move together when one of them is dragged, and a right click during the drag leaves copies. Edit > Duplicate (Ctrl+D) copies the selected guidelines, and the Guidelines docker gets Duplicate and Clear (every guideline of the type shown) buttons (#376).

- Text: the font list and the size list show the selected text in the family or size under the pointer while they are open, as the reference program does; the text goes back as it was when the pointer leaves or the list closes, and the preview leaves the undo history alone (#370).

- Pick tool: scaling with a corner handle follows the pointer smoothly. The size followed the faster of the two sides, so a line of text grew at the pace of its short side and jumped with small moves; it now goes by where the pointer falls along the box's diagonal. The handles also no longer jump a few pixels when the drag starts (#383).

- Pen tool: the pointer is a pen nib with a dot at its tip, and the node goes exactly under the dot; the close, continue, add, delete and edit signs show beside it (#362).

- Pen and Bezier: a finished curve keeps its anchors in the history, as in the reference program. After it is closed or deselected, Ctrl+Z opens it first and then takes the anchors back one by one; it used to take the whole curve away at once (#360).

- Shape tool: with two or more nodes selected, the align keys (L, R, T, B, E) and the Object > Align commands align the nodes, as in the reference program; handles move with their nodes and it is one undo step (#364).

- Pick tool: a marquee (or the Freehand Pick lasso) dragged with Shift turns over the objects it takes, as in the reference program: those already selected are deselected and the others added. It only added before (#401).

- Interactive Fill: colour nodes on the gradient's line, as in the reference program. A double click on the line adds a node in the colour the fill has there, a double click on a node removes it, and nodes drag along the line. The bar between two nodes moves where their colours meet half way. A click on a node opens a small bar under it with its colour (with the colour picker) and its transparency; the property bar shows the node's colour, transparency and position, and a palette colour goes to the picked node. Node transparency shows on screen and in SVG (#330, #285).

- Interactive Fill: gradient nodes are placed freely, as in the reference program. Dragging on an object puts the start and end nodes where the drag began and ended; the start, the end and a new skew handle move on their own (the start moves the fill, the end sets length and angle, the skew handle makes a radial fill elliptic or slants a linear one). The fill keeps its nodes when the object moves or scales, and SVG and PDF exports draw it the same way (#330, #285).

- Select All on a drawing with thousands of objects no longer slows the window down: the selection is gathered once per change instead of searched for object by object several times a frame (a frame with 3000 objects selected went from about 200 ms to about 7 ms, #367).

- Text > Edit Text (Ctrl+Shift+T) opens the Edit Text dialog with the whole text in a box; OK applies it as one step and the characters left as they were keep their formatting. It only started editing in place, with no window (#355).

- Font Manager: the samples were drawn in black on the dark themes, nearly unreadable; they now take the theme's text colour (#380). Showing the details pane made the window grow a little on every frame, off the screen; the pane now takes its room from the list (#381).

- Eyedroppers (the tools and the document palette's): they read nothing from a locked object or one on a locked layer; they now read colours and attributes from them too, while applying to a locked object stays blocked (#365).

- Pen and Bezier: closing a curve with a click on its first node keeps the last segment as the preview showed it (it turned straight, #359); dragging from the first node to close puts its handle where the pointer goes (it was the mirror, #358); and a curve of two nodes closes, for a leaf or a lens of two nodes (#356).

- Repeat (Ctrl+R): after an undo it repeated the action just undone; it now repeats the last action still in effect (#366). After a copy made with a right click during a drag it moved the selection; it now makes another copy as far again (#363).

- `.cdr` (version 16 and later): outlines keep their mitre limit; it was read as 5 degrees whatever the file said, so sharp corners were cut where they should not be, and it was not saved (#384).

- `.cdr` (version 16 and later): text with letters outside Windows-1252 (Cyrillic, Greek, most Central European letters) was saved as unmarked UTF-8 and came back as garbage, in TraceDraw and in other programs; such letters are now saved as marked two-byte units, and files saved the old way read back right (#368).

- Text: editing a second text put the first one back to its text before its last edit; each text's typing is now an undo step of its own and editing another text never changes it (#373).

- Browser build: the interface followed the browser's dark mode and switched to a dark style of its own on reload; the theme is now the one chosen in the options alone.

- Interface size: 100 % is now one interface point per pixel at the system's scaling on every system (it was 0.8 on macOS and in the browser and 12 / 13 on Windows), and the bars' Default Button Size starts at Small, so the bars stay compact while text and menus take the system's size. A chosen interface size or button size is kept.

- Toolbox: the Variable Outline tool has a button of its own at the bottom of the toolbox, after the Fill flyout; it was inside the Shape tool's flyout, where it was hard to find (#339).

- Page bar: the Autofit Page button opens a popup with a Margin field (kept between sessions) and the Autofit Page button, so the page can be fitted to the drawing with room left around it; it fitted the page with no margin at once (#340).

- Text: typing in the property bar's font search field (or any other field with the keyboard) while a text is being edited put the characters into the text too; they now go to the field alone (#342).

## 0.11.0 (2026-10-22)

- Panels: any panel can be dragged by its tab or its title bar to the left side of the window, to the right side or anywhere over the drawing as a floating window (resizable; the tab's and the title's right-click menu offer the three places too), so panels can sit on both sides at once; every panel still starts at the right. Where each panel is survives a restart.

- Bars: the right-click menu of any bar lists every toolbar with a check (as Window > Toolbars does) and has Customize: Default Button Size (small, medium, large), Toolbox Toolbar (Reset to Default, its own button size, Image Only, Caption Only or Caption to Right) and Workspace import and export; it listed only five bars and had no sizes or captions.

- B-spline: a curve drawn with the B-spline tool keeps its control points. The Shape tool shows them with the dashed control polygon instead of the curve's nodes: drag one to reshape the spline, double-click the polygon to add one, double-click a point to remove it. Convert To Curves (or any other edit of the outline) makes it an ordinary curve. The control points were dropped as soon as the curve was finished.

- Smooth tool: a stroke eases the wobbles out of the curve under the nib and leaves a handful of nodes behind it; it left a node every few millimetres and broke the curve into short pieces as the pointer advanced. Smear and Twirl measure the curve finer under the nib and leave smoother curves too.

- Clip frames: while objects are moved over a clip frame it lights up and a note under the pointer says what the release does; W held adds the objects to the frame, K held replaces its contents. Edit Contents fades the rest of the page while the frame and its contents are edited.

- Clip frames: with a clip frame selected a bar at the top left of the drawing window offers Edit, Select Contents, Extract Contents, the contents lock and Fit Contents (centre, fit or fill proportionally, stretch); while the contents are edited it shows Finish.

- 3-point curve: a curve drawn from an end node of the selected curve continues it, and one ending on its other end node closes it, as the other curve tools do; it always made a curve of its own.

- Fonts embedded in a `.cdr` whose OS/2 table sets only a reserved embedding bit (old Devanagari fonts set bit 0) were refused as "restricted" and their text fell back to another font; the embedding bits are now read as the specification has them, so those fonts serve their text (#313).

- Number fields (property bar, panels, dialogs): spin arrows at the right of every field, and the mouse wheel over a field steps its value (Shift ten times as much, Ctrl a tenth); values could only be typed.

- Symmetry: several selected objects become one symmetry group (each still to be picked inside it with Ctrl+click), and objects drawn, pasted or duplicated while the symmetry is edited join it; before, each object got a symmetry of its own and nothing could be added.

- Panels: the plus button under the tabs lists the panels in the groups of Window > Panels, with a box ticked at each open one; unticking closes it.

- Panels: the pointer over a panel tab shows a close cross at the top of the tab, and a click on it closes the panel.

## 0.10.2 (2026-10-22)

- Opening `.cdr`: the bitmaps of a big drawing (a `Bitmaps.dat` member inflating past 512 MB, two large pictures on a 1.5 m page) were cut off at the limit, so the pictures after the cut were missing with "objects could not be read"; the member now inflates to the size the archive declares, up to 3 GB (#323).

- Browser build: Ctrl+R is Edit > Repeat, as on the desktop, instead of reloading the page (the browser's reload button still does).

- Pen and Bezier tools: a click after a node whose handle was pulled gives the curved segment the dashed preview showed, leaving along that handle and arriving straight at the new node; the segment came out straight, the handle thrown away (#314).

- Vertical text: Text > Vertical Text and two property bar buttons set text in columns read top to bottom, right to left, as CJK typesetting does, with ideographs, kana and hangul upright and Latin letters turned; carets, selections, clicks and the arrow keys follow the columns. Offered with a Japanese or Chinese interface language, and for any text in those scripts (#257).

- Property bar: with nothing selected, a Fit page to drawing button sits after the page width and height, the Layout command at hand (#236).

- Segment Delete: a curve drawn over itself (a loop inside one segment, or a segment coming back over the one before it) loses only the stretch clicked between the crossings; the whole curve went. Pen, Bezier, Polyline and B-spline: Esc finishes the curve being drawn as it stands, keeping the nodes placed and the tool, instead of discarding it (#314).

- Remove Background (on this computer): the contiguous fill now follows a backdrop that drifts in colour (a texture, a gradient the edges did not show, a soft shadow under the subject), stopping at the subject's sharp edge, so such backdrops go whole instead of leaving patches; up to six background colours are learnt from the edges; and the command is in a bitmap's context menu beside the AI one (#231).

- Print: the Printer list holds the system's printers on Linux and macOS (CUPS), the default one first, and a print on one of them goes straight to it with its copies, instead of only through the PDF viewer; the printer is remembered for the next print (#300).

- Save As: Embed fonts for TraceDraw files too: the installed fonts the text uses go into the `.tdraw` file when their licence allows it (one file per face), so the drawing keeps its letters on a machine without them; the box starts on for `.tdraw` and `.cdr` files alike (#275).

- Bitmaps: a linked bitmap (an imported file, or one sent to the system image editor by Edit Bitmap) follows its file on the desktop: a save in the outside editor shows in the drawing within a second, as Update from Link does by hand (#284).

- Guidelines: a value typed in the Guidelines panel's x, y or angle field moves the selected guideline as soon as it is entered, without the Modify button; the fields follow the selected guideline; a double click on a guideline opens the panel on it; and the property bar shows the selected guideline's type and position, where a typed value moves it too (#306).

- Properties docker: with characters selected in the text being edited, Fill, Outline and Character show those characters' own values (the object's where they have none) and edit them alone; the docker showed the object's values and could not adjust the characters' (#304).

- Curve tools: Options > Tools > Curve tools > Constrain angle key chooses the key that keeps the next segment to the constrain angle, Ctrl (as before), Shift or Alt (#247).

- Layout > Fit Page to Selection and Fit Page to Drawing (also on the Page Size options page): the page takes the size of the selection or of everything on it, and the objects move onto it; one undo step (#236).

- Alignment and dynamic guides: preferences saved by a version from before the guides existed had both switches on with nothing behind them, so after the update the magenta alignment lines showed for everyone with an older configuration. They are switched off once at the update, as on a first start; switched on by hand afterwards they stay on.

- Colour pickers (Properties fill and outline swatches and every other swatch that opens the popup): a Model list switches the colour's model on demand (RGB, CMYK, HSB, HSL, Lab, Grayscale, YIQ); the colour converts and the object keeps a colour of that model (#286).

- AI options: the OpenCode preset goes to the Console Inference API (`https://opencode.ai/inference/openai/v1/chat/completions`), which has replaced the Zen and Go addresses; the Zen and Go models are chosen by name there, so the two presets are one (#173).

## 0.10.1 (2026-10-21)

- Drawing speed: objects outside the part of the page on screen are no longer drawn, and while the view pans, scrolls or zooms the last render is shown moved into place and the page is rendered anew once the view rests for a moment, so a drawing of thousands of objects pans at the screen's rate instead of redrawing every object on every frame (#253).

- Pointers: with a dark theme the tool's icon on the drawing tools' cross hair showed as a plain grey box; the icon is now painted black on its light patch under every theme (#244).

- Windows portable zip: ships the `tracedraw-settings` folder beside the program, so the unzipped copy keeps its preferences there and writes nothing to %APPDATA% (#292).

- CMYK drawings: the object defaults kept in the configuration file by an earlier version no longer keep new text RGB black; a black text default, RGB or CMYK, is the drawing's own black, while a colour of the user's own stays (#259).

- Shaping (Weld, Trim, Intersect, Simplify, Front minus back, Back minus front, Boundary): the result keeps the curve segments of the objects, with new nodes only where their outlines cross; it was made of hundreds of tiny straight lines, hard to edit and faceted when enlarged. Two welded circles now give a handful of curve segments. Crop, Knife and Eraser cuts keep the curves the same way (#252).

- Shape brushes (Smooth, Smear, Twirl, Smudge, Roughen): a stroke reshapes only the stretch of the curve within the nib's reach and leaves the rest exactly as it was; it rebuilt the whole curve through points a few millimetres apart, so one stroke of Smooth over a freehand curve turned it into hundreds of nodes with the outline twisted and crossing itself. The reshaped stretch is fitted with few curves, and Smooth thins jags smaller than its rate before fitting (#227).

- Linked paragraph text frames: a copy of one frame (Duplicate, a right click during a move, Paste) is a frame of its own, so typing in it no longer flows into the original's next frame; a chain copied whole is a chain of the copies (#162).

- Clip frames: an object that holds others (Place Inside Frame) can be painted and reshaped again: a fill or outline goes to its frame, the status bar and Properties show the frame's, and the Shape tool edits the frame's nodes (a rectangle's corners round, an ellipse takes a pie or arc, a curve node by node) with the contents staying inside; the frame inside a clip frame now has an id of its own (#273).

- Browser: a right click during a move, scale, rotation or skew leaves the original and gives the change to a copy, as on the desktop; the browser reports a button pressed while another is held as a move, so the click was lost (#278).

- Shape brushes (Smooth, Smear, Twirl, Attract and Repel, Smudge, Roughen): the nib, the pointer's move and the settings' millimetres are measured on the page; on a scaled object (text converted to curves, a resized drawing) the brush reshaped the outline within a nib scaled with the object, so Smooth with a 10 mm nib could flatten 50 mm of a curve, and Smear moved it by a fifth of the drag (#227).

- Segment Delete: after a stretch of one outline is deleted, the piece left ends on the outline it crossed; clicking that other outline now deletes only the stretch up to the crossing instead of the whole outline (#227).

- PDF, print, PostScript, EPS and SVG: an outline set behind the fill is painted first and the fill over it, as on screen; a white outline behind black letters no longer covers them, so black text no longer prints white (#235, #191).

- Windows: at interface size 100 % the text is as large as the system's own (12 px at 100 % scaling), as the setting says; it was 10.4 px. A size raised to make up for it may be set back to 100 % (#229).

- Blend: the steps lie between the two objects in the stacking order; a front object blended to one behind it no longer has the steps drawn over it (#232).

- CMYK drawings: new artistic and paragraph text is CMYK black, and an outline turned on in Properties is CMYK black, so their colour pickers start in CMYK; RGB drawings keep RGB black (#259).

- Properties > Outline: dragging in the colour picker is one undo step and adds only the final colour to the document palette, when the button is let go, as the fill picker does (#258).

- Shape tool: Smooth and Symmetrical now act on nodes beside a straight segment: next to a line the curve's handle lines up with it, and between two lines both become curves and the node is smoothed (#228).

- Shape tool: a click takes the nearest node, handle or segment, measured on the page; on curves with a scale of their own, such as text converted to curves, a click no longer took a node far from the pointer (#242).

- Text: the size field shows the size as seen on the page after the text is scaled with the Pick tool (24 pt doubled reads 48 pt), and a size typed there is the size the text then shows (#243).

- Pick tool: while a rectangle with rounded corners is resized, the preview outline keeps the corners round as the result will, instead of stretching them (#239).

- Welcome Screen: the property bar (page size, orientation, units, nudge, duplicate offset), the toolbox, Snap To and the standard bar's drawing commands are pale, also with drawings open behind it; New, Open, Options, Launch and AI Generate stay on (#246).

- Transformations, Rotate: the centre of rotation moved on the canvas is the centre the docker turns about (and shows with Relative center off), and after Apply the new copies keep it, so repeated rotated copies go round one point instead of drifting (#230).

- Dimension tools: Ctrl keeps the leg being given (the second point of a linear dimension, each side of an angular one) to steps of the constrain angle, preview included (#251).

- Options > Customization > Commands: tools take shortcuts with Ctrl, Alt and Shift like menu commands, and Alt with a letter given to a command no longer opens a menu (#248); Delete also removes a built-in shortcut, leaving the command without one (#249); Reset to defaults resets only the category shown, and a separate Reset all shortcuts asks first (#250).

- A right click on a locked object (with the Pick tool) opens a menu with Unlock Object, which unlocks and selects it, and Unlock All Objects; the object stays unselected and uneditable until then (#256).

- Curve tools (Pen, Bezier and the others drawing node by node): Ctrl+Z while a curve is drawn takes back the last node, handle pull or click on a node, one at a time, before going on to the document's history; Ctrl+Shift+Z puts them back (#233).

- Shape tool: Join now makes the two chosen end nodes one node halfway between them, with no straight segment added: the ends of one curve close it there, ends of two curves make one curve. Dropping a dragged end node on another end node joins them the same way (#254).

- Pen: with Auto add/delete on, a click by the first node of the curve being drawn closes the curve again instead of deleting that node; the first and last nodes now answer within the larger of the join and pick distances (#238).

- Shape tool: a click on a segment now chooses the segment itself (drawn highlighted with its handles) instead of showing its end node as chosen, and Delete removes just that segment: an open curve splits there, a closed one opens there, the nodes stay (#240).

- Shape tool: deleting a node between segments now refits a single curve from the neighbour before to the neighbour after when either side is a curve, so the shape is kept instead of reshaping (or going straight). A node between two straight segments still leaves a straight join (#241, #228).

## 0.10.0 (2026-10-20)

- The `.tdraw` format is version 0 again: the 1 written by 0.5 to 0.9 had the same layout and is read as 0, and Save As shows "v0" with nothing to choose. The next real change of layout will be version 2.

- Every property bar, page bar and tool bar button now shows a rich tip like the standard bar: its name in bold and what it does under it (descriptions in all 18 languages). The Options button of the standard bar has a down arrow, since it opens a list.

- Dialogs taller than their first frame no longer stay cut off at the bottom (hiding OK and Cancel); they grow to fit the window before they scroll.

- Bitmaps > Remove Background...: makes the background of photos transparent on this computer, without an AI service. The background colours are learnt from the picture's edges (or picked with an eyedropper), with tolerance, edge softness, Contiguous, Remove specks and Clean edge colors, and a before and after preview; also on the bitmap property bar. The AI command is now Remove Background with AI (#192).

- AI service presets for OpenCode Zen and OpenCode Go (drawings only); requests to them carry the `x-opencode-session` header those services require, one id per run (#173).

- Dialogs can be dragged by their title bar; they open centred (or where they were left, with Center dialog boxes off).

- CMX files: text exported in several colours is read with each part in its colour and place (the long instruction form, character styles and per character styles are read), and version 8 `.cdr` files from the bidirectional edition read their paragraph text (#187).

- Tables: cells are selected by dragging across them, Ctrl+click,
  Shift+click, the row and column arrows at the left and top edges, the
  corner and Ctrl+A; selected whole rows or columns move by dragging;
  row and column lines resize by dragging, and W and H on the bar size
  the selected cells. Insert adds as many rows or columns as are
  selected, and Insert Rows, Insert Columns and Split into N ask how
  many. Selected cells get their own background (also from the
  palette), borders per edge (all, outside, inside, top, bottom, left,
  right, inner) with width, colour and style, and the bar's and Text
  docker's text settings; tables get margins per side and separated
  borders. Options > Tools > Table chooses the Tab order and whether
  Tab types a tab.
- Outline widths are shown and typed in a unit of their own, points by
  default (millimetres, centimetres, inches or pixels on request), and
  the width list offers round values in it.
- Outlines: position outside, centred or inside the path; a mitre limit
  angle (5 degrees for new outlines); Align dashes fits a dash pattern
  so corners and line ends fall on dashes; Edit Line Style makes custom
  dash patterns on a grid, kept in the settings and listed with the
  presets. SVG and PDF exports draw them the same.
- Drop shadows: perspective shadows dragged from an edge (angle,
  stretch, fade), merge modes (Multiply for new shadows), feather
  direction and edges, presets, on-canvas handles (end with colour drop,
  opacity bar, start) and Break Drop Shadow Apart (a bitmap behind the
  object, one undo step). Shadows in older files look as before.
- Ctrl+click selects an object inside a clip frame (where the frame
  shows it), and Ctrl+Alt+click digs through a clip frame's contents,
  as in groups; text over a rectangle in a `.cdr` drawing is hit first,
  and an empty rectangle over text gives way to Alt+click (#190).
- Fit Text to Path on a closed path centres the text where the path is
  clicked (the bottom of an ellipse too), the mirrors keep the letters
  in place along the path, and the Text placement list has Top centre
  and Bottom centre on a closed path, the bottom one reading left to
  right inside the curve; Tick snapping moves the text off the path in
  steps of its spacing (#193).
- Print > Prepress > Invert prints a true negative: the sheet's paper
  turns black as well, so black text no longer comes out white on white
  paper (#191).
- The Properties docker keeps its width when an object is selected:
  the Transparency section sits in three rows, the Rectangle section
  lists the corner kinds one under the other with the radii two by two,
  and no section can push the docker wider (#196).
- Properties docker, Outline Pen and property bar with several objects
  selected (#204): changing one value (such as the outline width) no
  longer gives every object the first object's whole outline, fill or
  transparency; only the value changed is set and each object keeps the
  rest. Values that differ across the selection show as mixed (an empty
  field, no colour, nothing highlighted); a value set there goes to all
  (docs/behavior/mixed-selection.md). The property bar's width, line
  style and arrowheads and a palette outline colour also keep each
  object's other outline values, inside groups too.
- Page tab menu: six rows with a picture each and no separators (Rename
  Page..., Insert Page After, Insert Page Before, Duplicate Page...,
  Delete Page, Switch Page Orientation); Page Size and Page Background
  stay in the Layout menu. Duplicate Page... (also in the Layout menu and
  the Pages panel) opens a dialog: insert the copy before or after the
  selected page, and copy the layers only or with their contents. The
  copy becomes the current page and one undo removes it.
- Rectangles: new rectangles keep their corner size when resized
  (Relative corner scaling is off by default on the property bar and in
  Options > Toolbox > Rectangle tool), so a rounded rectangle stretched in
  one direction keeps circular corners of the same radius. Turn the
  setting on to have corners scale with the rectangle. Drawings saved
  earlier open unchanged; preferences from older versions start with it
  off once. Scaling a group evenly keeps such corners fixed (grown by the
  factor) instead of turning a stretched rectangle into a curve, and
  `.cdr` versions with rectangle objects save a stretched one as a
  rectangle with its radius.
- Pen: Preview mode (on by default, on the property bar and in Options >
  Tools > Curve tools) shows the next segment dashed in blue from the last
  node to the pointer before the click, curved by the node's handle. Auto
  add/delete (on by default): a click on a segment of the curve being
  drawn or of the selected curve adds a node, a click on a node deletes
  it, with plus and minus pointers. While drawing with the Pen or Bezier
  the placed segments show in the outline colour new curves get, the
  last node filled blue and the others hollow. C held while pulling a
  handle makes a cusp even when pressed before the drag, and no longer
  aligns the selection. The snap screen tips also name the grid and
  guidelines.

- VBA macros: the object model they are written against (Application,
  Document, Page, Layer, Shapes and ShapeRange, Shape with Fill,
  Outline, Transparency, Curve, Text and its views, Color, the cdr*
  constants), with lengths in the document unit and the reference
  point (docs/behavior/vba.md); constant values and parameter names
  checked against the published developer reference. Ids minted by
  scripts no longer collide with the drawing's own after a run that
  groups or adds pages.

- VBA macros: a compiler from VBA modules, classes and form code to
  scripts over the TraceDraw object model, with the VBA run-time library,
  Collection and Scripting.Dictionary.
- `.cdr` saving writes only the versions checked in other programs (14
  to 27). Save on a drawing opened from an older version asks to choose a
  version first ("Older file version", then Save As on version 14)
  instead of writing the old version again. The command line writes
  version 27 by default and the experimental 7 to 13 only with
  `TRACEDRAW_CDR_EXPERIMENTAL=1`; the MCP `save` tool takes 14 to 27.
- Help: a user guide of 120 pages (getting started, lines and shapes,
  objects, color, effects, text, templates and styles, pages, bitmaps,
  printing, web graphics, file formats, customizing, reference) with
  screenshots of the dialogs and panels, published with the browser
  version in every interface language, with contents, search, previous
  and next, light and dark and right-to-left Arabic. Every "?" button
  opens the page of its dialog or options page, F1 the page of the open
  dialog, panel or tool, and Help > Help Topics the first page, in the
  interface language.
- Wrap Paragraph Text now works: paragraph text flows around the objects
  marked to wrap it that lie over its frame, along their outline or
  their bounding box, with the text on the left, the right, both sides
  or above and below, and a text wrap offset (the arrow beside the
  property bar button). Drawing, printing, exports, clicks and the caret
  follow the wrapped lines.
- Alignment, spacing, indents, bullets, drop caps, tabs and hyphenation
  are kept per paragraph: while a text is edited they change the
  paragraphs of the selection, with the whole object selected every
  paragraph (only the settings changed). Older drawings open with their
  object-wide formatting on every paragraph; `.cdr` files keep each
  paragraph's alignment both ways.
- The context menu's Wrap Paragraph Text is a submenu with the seven
  wrap styles and the offset. The Shape tool's spacing arrows change
  every paragraph, keeping their differences. `.cdr` files keep each
  paragraph's line spacing from version 16, and paragraph spacing
  before and after is read from version 16 style strings.
- Align to Baseline Grid puts the baselines on the page's baseline grid
  (from the page top and the grid's start offset), so frames line up
  with one another.
- Alignment guides (View > Alignment Guides, Alt+Shift+A) now work:
  while moving, drawing or resizing, lines show where the selection or
  the pointer lines up with other objects' edges and centres, the page
  or margin lines, and the move snaps there; equal gaps between
  neighbours snap and are marked, and a box being drawn or resized
  takes another object's width or height, marked on both. In the
  Multipage View the other pages' objects count too. Off by default (the code default was
  on). Options > Alignment Guides sets the object (the selection or
  each object), edges, centres, page, margins with offset and inset,
  equal spacing, equal sizes, colour and line style.
- Dynamic guides (View > Dynamic Guides, Alt+Shift+D) now work: guides
  at chosen angles run from the snap points the pointer acquired and
  from the line's start; points snap onto them and their crossings,
  with the distance and angle beside the pointer and optional tick
  marks. Options > Dynamic Guides sets the angles, tip, ticks, colour
  and line style. Both also sit in the standard bar's Snap To list.
- Text: Chinese, Japanese, Korean and other input methods work while
  typing on the canvas and in table cells (the text being composed shows
  at the caret). Break Apart (Ctrl+K) on artistic text splits it into
  lines, words or letters, each a text object of its own.
- A right click on a page tab opens the page menu: rename, insert a page
  before or after, duplicate, delete, switch orientation, page size and
  background.
- Blends of text and of shapes with several parts (letters, holes)
  morph part by part, so the steps between two texts read as letters.
- Pen and Bezier: a plain click after a pulled node gives a straight
  segment (the node becomes a cusp), clicking the last node again makes
  it a cusp, C while pulling a handle makes the node a cusp and Alt
  moves it, and dragging from the first node shapes only the closing
  segment. With no curve being drawn they add a node on a segment of the
  selected curve and drag its nodes. The pointer shows what a press
  does: start, continue, add a node, close, edit a node.
- Shape tool: nodes and handles are taken within a pick distance (6
  pixels, Options > Nodes and Handles), and a near miss no longer
  deselects the object. A rectangle, ellipse or polygon says what it is
  on the property bar and the status bar, apart from a curve.
- Linux: the window can be resized and maximized under every window
  manager (Xfwm4 kept it at one size) and fits small screens: a smaller
  minimum size, maximized at start when the kept size does not fit.
  Bars longer than the window slide with "«" and "»" buttons (and the
  mouse wheel), and dialogs taller than the window scroll (#168).
- VBA macro files are read (new `tracedraw-vba-project` crate): `.gms`
  and other compound files with a VBA project (modules, their kinds and
  code in the project's code page, references, protection, modules saved
  without source), exported `.bas`, `.cls` and `.frm`/`.frx` files, and
  the forms with their controls (labels, text boxes, buttons, check and
  option boxes, lists, frames, pages, images, spin buttons, scroll bars,
  tab strips). Projects can be written back as module files or as a
  compound file.
- PDF files keep text as text: it can be searched, selected and copied.
  The fonts go in as subsets (only the glyphs used), TrueType and CFF
  outlines alike, with a map back to the characters. Publish to PDF >
  Objects has Export all text as curves (off), Embed fonts in document,
  Embed base 14 fonts and Subset fonts below a share of glyphs. Fonts
  whose licence forbids embedding are written as curves, named on the
  status line and noted by the preflight. Text written as curves (and
  any curve in PDF and EPS files) keeps the font's exact curves:
  quadratic segments become the matching cubic instead of a misshapen
  one.

- Convert to Curves turns a table into a group of curves (cell fills,
  borders and text) and a symbol instance into its symbol's objects as
  curves, instead of a plain rectangle; a clip frame keeps its contents
  (frame and contents become curves and stay clipped); bitmaps are left
  as they are
  (the command is greyed for them, with a note in the status bar).
- Trim, Weld, Intersect and Join Curves take the bottom-most object as
  the target when the selection was made with a marquee; objects picked
  one by one keep the one selected last as the target.
- Colour Management: the Assign and Convert choices described each
  other. Assign keeps the colour values (the look may change), Convert
  keeps the look (the values change), in all 18 languages.
- Imported bitmaps come in at the resolution their file records (PNG,
  JPEG, TIFF, BMP), 96 dpi when it records none.
- Space finishes a curve drawn with the Bezier tool (and then switches
  to the Pick tool, as Space does).
- Edit > Clone makes a linked clone that follows its master's fill,
  outline and shape until the clone's own is edited, with Select
  Master, Select Clones, Revert to Master and Break Clone Link in the
  context menu (it used to make a plain duplicate).
- The feature list and guides no longer claim live colour styles or
  Find and Replace by properties; File > Open does not open bitmaps (File > Import does).
- VBA macros: a compiler from VBA modules, classes and form code to
  scripts over the TraceDraw object model, with the VBA run-time library,
  Collection and Scripting.Dictionary.
- VBA macros: the object model they are written against (Application,
  Document, Page, Layer, Shapes and ShapeRange, Shape with Fill,
  Outline, Transparency, Curve, Text and its views, Color, the cdr*
  constants), with lengths in the document unit and the reference
  point (docs/behavior/vba.md). Ids minted by scripts no longer collide
  with the drawing's own after a run that groups or adds pages.
- VBA macros in the app (desktop): Tools > Macros with the Macro
  Manager docker, Run Macro (Alt+Shift+F8), Load Macro Project and the Macro
  Editor (Alt+F11). Projects are kept in the settings folder and load at
  start-up; macros run on their own thread with MsgBox and InputBox
  dialogs, their user forms on screen with events, Stop and Esc, an
  error box with the VBA line and one undo step per run. Each macro can
  get a shortcut, and the Scripts panel runs VBA snippets too.

- Moving objects with the Pick tool shows the arrow with a dotted box,
  and the box gets a plus after a right click (the move leaves the
  original and a copy lands where the left button goes up). The pointer
  no longer vanishes after that right click.
- Ctrl+V pastes what was copied last: a picture or text copied in
  another program after a copy in TraceDraw is no longer ignored, files
  copied in a file manager are imported, and Ctrl+V works when the
  clipboard holds only a picture or files.
- Document Properties: language, title, subject, author, copyright,
  tags, notes and a rating list, applied on OK in one undo step, over a
  report of the file, the document, its colour settings, objects and
  points, text, bitmaps, styles, effects, fills, outlines, colour models
  and symbols. PDF files carry the language.
- Characters selected in an artistic or paragraph text being edited take
  a colour picked from a palette, the document palette, the Properties
  docker's fill or the Color docker as their own fill (right click: their
  own outline), each its own undo step; the rest of the text keeps the
  object's colours. Characters with their own colours are now drawn in
  them on screen and in bitmap, SVG and PDF exports, so the per-character
  colours of opened `.cdr` files show too (#187). The property bar's
  font, size and style change only what they change on a selection.
- Dragging in the Properties fill colour picker is one undo step and
  adds only the final colour to the document palette, when the button
  is let go (#188).
- A right click while dragging an object no longer drops the copy at
  once: the outline follows the pointer until the left button goes up.
- Files dropped on the window open or import: with a drawing shown they
  are imported into it in the middle of the view, otherwise each opens
  in its own tab (a picture gets a new drawing). `.psd` and every other
  format File > Open and File > Import read are supported.

## 0.9.4 (2026-10-19)

- Color Management Settings: Document and Default tabs, the drawing's
  current profiles, assign or convert new RGB, CMYK and grayscale
  profiles chosen from the profiles found on the computer, primary mode
  and intent; presets, colour engine, preserve pure black, map gray to
  CMYK black, open and import policies with warnings, spot colour
  definition.
- The standard bar's Options button opens the list of option pages; its
  buttons show their title, shortcut and what they do; two buttons open
  from and save to the drawing's Git repository.
- Object > Insert > Barcode is a three-step Barcode Wizard with a sample
  preview: Codabar, Code 39, Code 128, EAN-13, EAN-8, UPC-A, UPC-E and
  Interleaved 2 of 5 (with EAN and UPC add-ons); printer resolution, bar
  width reduction, magnification, bar height, wide to narrow ratio,
  check character and quiet zone; font, size, weight, alignment and the
  text options.
- QR codes are live objects: Object > Insert > QR Code places one at
  once, and Properties edits its kind of content (URL, email, phone,
  SMS, contact, text, location) and fields, pixel and background fills,
  pixel outline and pixel size, with Validate reading the code back.
- Effects menu: Flatten Effects, the effect groups, then the effect
  panels, each checked while open (Effect Picker moves to the Window
  menu); Object > Create keeps Arrowhead and Pattern Fill, which asks
  for a vector or bitmap pattern. The Picture Mosaic, Vector Mosaic,
  Extrude (pages with a type picture and Measure from), Envelope (24
  preset shapes in a grid) and Blend panels are laid out anew; blends
  gain spacing along a path, loop, along the full path, acceleration of
  sizes and clockwise or counterclockwise colours.
- Text on a path has its own property bar: text orientation (rotated,
  vertical or horizontal skew, upright), vertical placement (baseline,
  ascender, descender, centred), placement along the path, distance,
  offset, both mirrors and Straighten Text. Dragging a fitted text slides
  it along its path; refitting a text to another path of the same kind
  no longer shows the old letters.
- Text fitted to a closed path stands on its outside, upright and
  centred on its top, instead of starting at its right end on the
  inside; the Text tool starts text on a path when clicked on its
  outline.
- Font Manager: a window of its own with libraries (installed fonts,
  added folders, Favorites and collections), filters by status,
  embedding rights, technology, weight, width, style, scripts and
  OpenType features, a sample text and search, every face drawn in
  itself and grouped by family, details, and installing font files for
  TraceDraw.
- Text docker laid out in Character, Paragraph and Frame sections with
  the buttons that show it from a section on, Apply to default text and
  the row boxes: style list, range kerning, text fill, character
  background and outline rows, six alignments (no alignment and forced
  justification added), line spacing in % of the character height,
  points or % of the size, and the frame's background colour.
- Glyphs docker: recent glyphs, font and range lists, a grid of the
  font's glyphs with their alternates.
- Text > Writing Tools is one dialog with Spell Checker, Grammar and
  Thesaurus tabs; the grammar checker also flags "a"/"an" before the
  wrong sound and sentences with no end mark.
- Text > Custom Optional Hyphens: words hyphenated by hand break only
  there, turned on for text typed, pasted or imported with them.
- Publish to PDF opens PDF Settings with General, Color, Document,
  Objects, Prepress, Security and issues tabs: the file name, every open
  drawing or a page list as the range, the page size of the selection,
  and PDF presets saved with "+" and deleted with "-".
- Export lists formats as "EXT - Description (*.ext)", JPG, PDF, AI and
  PNG first; "Do not show filter dialog" skips the dialog next time, and
  PDF and AI take a title and tags.
- The Snap To list stays open while its boxes are ticked and has a close
  button.
- Bitmap Mask docker: ten colour slots with their own tolerance, the
  eyedropper and "Select colours from bitmap", saved and opened masks,
  and Hide selected or Show selected.
- Border and Grommet: a preview of the final page, Source (page or
  selection), Add border with type and size, Add grommets with Use
  border area, size and margin, placement on the corners and each edge,
  number or spacing, and Reset and help buttons.
- Dialogs are modal: while one is open, clicks, menus, dockers, the
  drawing and canvas typing wait until it closes.
- Create New Table asks for the number of rows and columns, height and
  width (in the document's units), with a help button.
- `.cdr` saving from version 16: the font table lists every face the
  text uses, and Save As has an "Embed fonts" box that stores the faces
  whose licence allows it (the command line with
  `TRACEDRAW_CDR_EMBED_FONTS=1`). Save As lists versions 27 down to 14.
- `.cdr` saving: version 14 is no longer marked experimental, and
  version 13 files carry the zero argument 12000 on layers, groups and
  objects, flag 0x40 on pages and an object box on the master page, as
  files of that version have them (the other program closed on them).
- Save As has a version list for TraceDraw files too: the `.tdraw`
  format versions, newest first, with the release that first wrote
  each; Save keeps the version a drawing was opened or saved in.
- `.cdr` version 15 is no longer marked experimental: files written in
  it open with every object and their text looks.
- Save As offers the newest `.cdr` version (27) first, drops the long
  version hint, and has an "Embed colour profile" box with the
  profile's name and size (version 16 and later).
- `.cdr` saving before version 16: the font table lists each bold or
  italic face the text uses, so bold text no longer opens in a
  stand-in font.
- `.cdr` saving in versions 7 to 15: text records are laid out as files
  of those versions hold them, one byte a letter from version 12 (text
  was dropped as an invalid object, or the other program closed), each
  text's frame numbered as its object, and curves end with the 8 bytes
  those versions keep after the nodes (rectangles, written as curves,
  were lost); layers and groups carry the zero argument 12000 from
  version 14, as files of versions 14 to 27 do; text before 16 takes its font, size,
  weight, colour and alignment from a style of its own (it all came out
  in the default style); curve nodes carry the marks of every version's
  files (filled curves before 16 were dropped).
- `.tdraw`: drawings whose grouped curves keep node types open again
  (they were refused with "expected usize").
- Save As lists each `.cdr` version with the release that writes it
  ("v13 (X3)", "v27 (2026)"); versions 23 to 27 are no longer marked
  experimental (files written here open in the 2026 release).
- `.cdr` saving in versions 13 to 15: no invalid object when a shape is
  partly transparent (the transparency is not kept in those versions
  yet); version 15 text carries the argument its files have.
- `.cdr` saving: images show in other programs (bitmap objects now carry
  their pixel size and image box; files opened without their images).
- `.cdr` versions 23 to 27: files open (version 23 files were refused),
  and saving offers them too (experimental: written with the records of
  version 22, which those versions share; not yet checked to open in
  other programs).
- `.cdr` version 21: the page background colour, hidden objects (kept,
  not drawn) and facing pages (objects were half a page off) are read;
  saving in version 21 or 22 keeps the background and hidden objects.
- `.cdr`: a page's layers come in the right order (the top layer was
  drawn at the bottom, under a full-page background).
- `.cdr`: arcs of ellipses drawn downwards turn the right way (smiles
  came out as frowns).
- `.cdrt` templates of version 21 open (form type `CDT`).
- `.cdr` saving: the layers are written top first, and textures and
  pattern fills, not written yet, go out as their mean colour instead of
  their first one.
- `.cdr` versions 13 to 15: artistic text no longer lands metres off the
  page when its record holds a number where later versions keep a place.
- `.cdr` versions 7 to 11: letters take the style their character
  record numbers, so a word with one letter in another font or a big
  first letter of a paragraph comes out right (it went to the end or
  vanished).
- Text in the Symbol font draws its Greek letters, arrows and math signs
  when that font is not installed (it showed accented Latin letters).
- A big first letter no longer opens up the spacing of every line of
  its paragraph.
- `.cdr` saving: a clip frame keeps the fill of its frame (it was lost).
- CMX files whose chunk sizes run short (pattern files of version 7)
  open; they came up empty.
- `.cdr` saving: paragraph text is written as one artistic text per line
  in its place, so it no longer runs together on one line; the frame
  itself is not written yet.
- `.cdr` saving: images keep their see-through pixels (they were put over
  white).
- The Print dialog has tabs: General (destination: system printer, PDF
  or PostScript file; orientation; current document, page, page list or
  selection; copies with collate; print as bitmap), Color (separations,
  spot colours as CMYK), Composite (text in black, overprint black),
  Layout (as in document, fit, scale, bleed limit, imposition), Prepress
  (invert, mirror, file information, page numbers, crop, registration,
  calibration), PostScript (language level) and the preflight's Issues.
- Print Merge is laid out around its records: Import File, Add Column
  (name and type), Configure Columns, View Single Record, record
  navigation, Save Data Source and Finish.
- `.cdr`: an object whose transform holds several matrices takes all of
  them, one after the other (only the first was used, so labels of a
  version 5 chart piled up in one place); version 5 text no longer gets
  an empty line after every line break.
- `.cdr` version 3: envelopes bend the shapes they hold (perspective
  charts and lettering); text under an envelope keeps its matrices.
- `.cdr` version 3: two-colour pattern fills (from the file's tiles);
  PostScript texture and vector pattern fills, in every version that has
  them, show a stand-in of grey stripes instead of nothing.

- `.cdr` files of versions 3 and 4 open properly (they opened blank or
  out of shape): version 3 objects with their own matrices, fills,
  fountains, outlines with caps and arrowheads, groups and artistic text;
  version 4 matrices and colours; arrowheads of both; curves of glyph
  outlines. Numbered library colours become spot inks coloured from the
  file's thumbnail.

- File > Print Preview shows the sheets as they print: imposition,
  marks, mirror, invert and each separation plate, with the print options
  beside them and zoom (it showed the page full screen).
- Print Merge has a record grid: fields and records are added, edited
  and removed in the dialog and saved as CSV; fields can be counters,
  pictures or QR codes. Merged pages follow the records' order (they came
  out reversed).

- `.cdr` files of versions 1 and 2 (the `WL` files before RIFF) open:
  rectangles, ellipses, curves, text with its font, size, alignment and
  line spacing, uniform fills and outlines in the stacking order of the
  file (they were refused).

- An outline set behind the fill is drawn under it also when it is the
  object's only outline (it was drawn over the fill, so thick white
  outlines thinned or hid letters).

- Templates (`.cdt`, `.cdrt`) open as untitled drawings and can
  be imported.
- Save As says that programs from 2021 on open `.cdr` version 15 and newer
  only.

- `.cdr` form-type letters skip `O` too: a `CDRR` file is version 25 (it
  was labelled 26).

- The B-Spline tool places control points: the curve runs smooth near
  them, with the dashed control polygon shown while drawing (it drew
  straight segments through the points).

- `.cdr` files of versions 5 and 6 show their text (it was skipped), and
  version 5 bitmaps their picture; version 3 files open (assumed layout).
- Graded transparency is saved as a gradient in `.cdr` versions 16 and
  later (it was saved as a uniform one).
- `.cdr` reading caps what a file may inflate to (512 MB a member, 1 GB of
  compressed blocks in all), so a decompression bomb cannot exhaust
  memory; the reader forbids unsafe code.

## 0.9.3 (2026-10-17)

- Wireframe view draws every object as a hairline: no outline widths,
  fills, opacity or effects, bitmaps as grey boxes (#122).
- A fill or outline colour given to a group reaches the objects in it,
  so a traced bitmap takes a palette colour with one click (#123).
- Outline widths can be typed: the property bar's width is a box with
  the usual widths in its list, and the Properties width box applies
  what is typed (#124).
- Selection handles hide while text is typed or edited (#125).
- The first built-in gradient runs between the current colours, the
  last fill and outline colours picked (#4).

- Wireframe hairlines take each object's outline colour (black without
  one), as in the reference screenshot of #122.
- Tracing a black and white logo keeps white shapes white: the colour
  quantising no longer averages black and white to grey.

- The Transformations panel has a new layout:
  spin boxes with the unit after them, the reference point under the
  check box with the direction picture beside it, a line before Copies.

## 0.9.2 (2026-10-17)

- `.cdr` files: graded transparency (version 16+ files) is read as a
  transparency mask, so highlights fade as drawn, and colours of model 25
  (library colours) take their look from the file's preview instead of
  black. Save As marks the `.cdr` versions before 16 experimental: their
  files may not open in other programs yet.
- Clip containers of version 12 `.cdr` files show their contents (the
  objects drawn inside a shape were missing).
- Vector pattern fills of `.cdr` files are read: the pattern's drawing,
  stored in the file as a CMX file, tiles the shape (they had no fill).

## 0.9.1 (2026-10-17)

Version 15 `.cdr` files read their text in full, `.cdr` files saved for
versions 12 to 20 carry the configuration record those versions write
(text saved in versions 16 to 22 now opens in other programs), and the
Transformations panel's Position page picks a direction for copies.

- `.cdr` files of versions 13 to 20 saved by TraceDraw carry the whole
  document configuration record those versions write (it held only the
  page size, and other programs refused the file), version 12 files the
  8152-byte record of that version, and version 16 text has the record
  header that version writes (#98). Text saved in versions 16 to 22 and
  groups in 17 to 22 open in the program that wrote the samples;
  versions 7 to 15 do not open yet.
- Version 15 `.cdr` text reads in full: style records with the 0x20 flag
  byte no longer cut the text off (55 text objects of one file were
  dropped), each letter takes the style its record names (sizes and
  fonts of mixed runs) and artistic text whose coordinates hold an id
  is no longer placed 41 m away (#116).
- Transformations panel, Position page: with Relative position on, the
  3 x 3 boxes pick a direction and fill X and Y with the selection's
  width or height that way, so Apply with copies lays them side by side;
  the values follow the selected object. With nothing selected the whole
  page is pale (#115).
- Version 16 `.cdr` text is written one byte a letter, as files saved for
  that version hold it (UTF-16 letters were read as half the text), and
  such files are read correctly (#98).

## 0.9.0 (2026-10-16)

`.cdr` files of versions 16 and later saved by TraceDraw with text and
curves now open in other programs: a drawing saved as version 21, with
text, text on a circle and curves, opens complete and as drawn. Version
7 to 13 files are compressed and summarised as those versions write
them, and a drawing's CMYK profile is stored in its package. Also:
outlines take every fill kind as their paint, gradients repeat, mirror,
invert and follow a shape's outline, five landscape gradients, and AI
options with one service for drawings and pictures.

- `.cdr` files of versions 7 to 13 saved by TraceDraw are compressed
  (`cmpr`) and end with the summary (`sumi`), as files of those versions
  are (#98).
- `.cdr` files of versions 16 and later store the drawing's CMYK colour
  profile in the package and name it (#98).
- `tracedraw-cli` ends quietly when the program reading its output stops
  reading (a pipe into `head`, say).
- `.cdr` files of versions 16 and later saved by TraceDraw open in other
  programs again: text frames are numbered as their objects, text
  carries its complete default style and run records with their letter
  counts, and curve nodes carry the marks those versions write, so curves
  also keep their fill (#98).
- `.cdr` files of versions 11 and later saved by TraceDraw give every
  text the line breaking tables files of those versions carry after it
  (`txtj`), as other programs write them (#98).
- `.cdr` files of versions 18 to 22 saved by TraceDraw give every
  object's transform the tag files of those versions carry (#98).
- Outlines take every fill kind as their paint (none, uniform, gradient,
  pattern, texture, mesh) with the fill editors; SVG and PDF export them
  (issue #4).
- Gradients: repeat (sawtooth or mirror, with a cycle count), bilinear,
  invert colours and a shaped type that follows the object's outline;
  `.cdr` files get the closest plain gradient (issue #4).
- Five landscape gradients among the built-in ones (issue #4).
- Tools > Options > AI has one AI service instead of separate drawing
  and image services: the Service page sets the preset, address, text
  model, one API key and, when the service can also make pictures
  (OpenAI, Google Gemini, or an OpenAI-compatible address of your own),
  the image model. With a service that makes drawings only (Anthropic,
  Ollama, LM Studio, llama.cpp, vLLM) AI Generate's pictures, Remove
  Background and AI Art Styles stay greyed, and their tip says why.
  Preferences with the old separate services are folded into the one
  service on start, keeping remembered keys.

## 0.8.0 (2026-10-15)

`.cdr` files saved by TraceDraw now carry everything files of their
version hold, so other programs can open them (a version 17 file was
reported unreadable); version 12 text, centring, line spacing and text
on circles, radial and square gradients and brushed strokes open as
drawn, and a large file that took minutes opens in a fraction of a
second. Also: the Pixels view shows document pixels when zoomed in, AI
Generate gathers all AI drawing in one panel, Window > Panels in groups,
Bitmaps > Resample, bitmap effects as lenses, Put on Plane in
perspective drawing, Save Bitmap As, a frame around the selection and
renaming a page from the Objects panel.

- `.cdr` files saved by TraceDraw carry what files of each version hold
  so other programs can open them: the document preferences and default
  styles (versions 16+), the summary, names and thumbnails of versions 14
  and 15, and the picture and info lists of RIFF files (before 14). A
  version 17 file saved this way was reported unreadable (issue #98).
- Save Bitmap As in a bitmap's context menu writes its pixels to a PNG.
- Shift + double-click on the Rectangle tool draws a rectangle around
  the selection.
- `.cdr` radial and square gradients are no longer inside out: files
  list their colours from the edge to the centre (written that way too).
- `.cdr`: strokes an artistic brush draws along a curve (linked groups)
  are placed with their group's transform; in a version 10 logo they
  floated far from the drawing.
- `.cdr` files saved as version 16 or later are complete packages: a
  media type member first, the manifest naming the drawing, metadata,
  layer and object names, colour settings and a picture of each page.
  Files saved without them were reported not to open in other programs.
- Opening a `.cdr` file with long curves is fast again: a 4 MB tutorial
  file took over three minutes, now a fraction of a second (node types
  were worked out once a node instead of once a curve). Saving `.cdr`
  and showing nodes in the Shape tool had the same slowdown.
- View > Pixels shows the document's pixels when zoomed in: the drawing
  is rendered at the document resolution and enlarged as square blocks,
  lined up with the pixel grid, so edges look as they will once exported.
- Objects panel: double-click the page (or its context menu) to rename
  it.
- `.cdr` version 12 text: accented and other letters stored as two bytes
  among one-byte letters are read (such text used to come out as Chinese
  characters); centred text is centred (it was read as right-aligned);
  line spacing given as a distance and negative character spacing are
  read, so multi-line text no longer collapses onto one line; text fitted
  to a circle without readable settings follows the circle its box rests
  on instead of running straight.
- `cdr_preview_check` example: puts a `.cdr` file's own preview picture
  beside our render of the same area, for checking imports.
- Window > Panels in groups, in the usual order of a Dockers menu
  (Properties, Objects, Pages, Comments... then Transformations, Guidelines,
  alignment, shaping, text, colour), with the effect panels in an Effects
  submenu; the menu no longer runs off the screen.
- AI in one place, as in the AI Generate docker of other editors: AI
  Drawing is now its fourth mode, Draw with words (also Object > AI
  Generate > Draw with Words), and no longer a separate menu item and
  panel.
- Bitmaps > Resample: width and height in pixels or percent, the
  resolution, aspect ratio kept or not, and four methods, among them Hard
  pixels for pixel art and Enlarge with detail.
- Draw in Perspective: Put on Plane lays existing objects onto the
  active plane.
- Bitmap effects as lenses: a Bitmap Effects lens runs any of the bitmap
  effects over what lies beneath it (Lens panel, or Effect Picker >
  Apply as a lens).

## 0.7.0 (2026-10-15)

Start faster, bring your own AI and draw in new ways: a New From
Template dialog with designed templates; AI Generate (create, remix and
vectorize pictures), AI Art Styles and Remove Background through the
online or local services you connect with your own keys; painterly
brushes, variable-width outlines and drawing in perspective; focus
mode, a Multipage View, review comments, an Export panel for many
assets and a Font Sampler; the desktop's text size and high contrast
themes followed; a darker dark theme; and `.cdr` files saved by the
newest versions opening with their fills and spot colours.

- Faster textures and other per-pixel fills on screen (#4): worked out
  on all processor cores, on a coarser grid when they cover much of the
  window (exports keep every pixel), with their end colours converted
  once instead of per pixel, and kept between redraws, so dragging other
  objects or changing the selection does not compute them again. A
  window-sized cloud texture draws about three to four times faster on
  two cores, and again from memory when unchanged.
- AI, with the user's own API keys: Tools > Options > AI turns AI
  features on or off and sets up a drawing service and an image service
  (Google Gemini API or OpenAI Images API or compatible), each with Test
  Connection. The AI Generate docker (also a button on the standard
  toolbar) creates pictures from a prompt with aspect ratio, style and
  palette presets, remixes the selected bitmap, and creates vectors (a
  generated illustration traced into curves). Remove Background (Bitmaps
  menu, property bar, context menu) keeps a picture's subject and makes
  the rest transparent. AI commands are greyed out, and the dockers show
  a banner with Set Up AI, while their service is not ready.
- Fountain fills offer 24 built-in gradients (metals, sunsets, skies,
  pastels and more) as swatches in the fill editor (#4).
- Texture and other procedural fills work out only the part in the
  window, so zooming into a cloud-filled shape stays quick (#4).
- Outlines can be painted with a gradient (Properties docker, outline
  Paint: Gradient), without converting them to curves first (#4).
- Options > Customization > Appearance: Shadows under menus and dialogs
  can be turned off (#4).
- AI Generate docker, more complete: mode buttons and an information
  page (services, models, costs, what is sent); a reference picture from
  the selection or a file (the source of Remix, an example for Create)
  with "match reference image"; ready-made remix requests (mockup, photo
  restoration, colorize sketch, replace background, change season,
  recolor); model notes; Reset per section; more aspect ratios; vector
  styles; Create vectors also as an SVG drawing from the drawing service,
  imported as editable shapes; Results replaced by each Generate, with
  Insert into drawing; Generate kept at the bottom. Object > AI Generate
  has every mode and Remove Background.
- AI service presets (Anthropic, OpenAI, Google Gemini, and Ollama, LM
  Studio, llama.cpp and vLLM on the same computer for AI Drawing; LocalAI
  for pictures) fill in the address, which stays editable. A new "AI
  features" topic in the Hints panel explains online and local services.
- Focus mode: double-click a group or clip frame, or Bring into Focus
  in the context menu, to edit its objects with the rest of the drawing
  faded (Tools > Options > TraceDraw > Edit sets how much); a breadcrumb at the
  top of the window goes back up, as does Esc.
- Grouping objects from several layers keeps their drawing order.
- Text > Font Sampler: compare fonts with your own sample text, side by
  side, and apply one with a click.
- Bitmaps > AI Art Styles: repaint the selection as an oil painting,
  watercolour, pencil sketch, ink drawing, chalk pastel, pop art, woodcut
  or stained glass through your image service.
- View > Draw in Perspective: one-, two- and three-point grids; what you
  draw lands on the ground, a wall or the ceiling in perspective and
  stays editable.
- View > Multipage View shows every page in a grid; click any page to
  work on it in place.
- Painterly Brush tool (Brush Strokes flyout): strokes that look painted
  with chalk, charcoal, pastel, ink wash, watercolour, dry brush,
  bristles, fur, spray, speckles, clouds or silk ribbons; still curves
  that the Shape tool reshapes.
- Appearance > Coloured tool icons: each toolbox group in a colour of
  its own (#4).
- The interface follows Windows' text size and high contrast themes: a
  High Contrast theme in the desktop's colours, also selectable by hand
  (#4).
- Effects > Effect Picker: a floating window with every bitmap effect
  as a picture, searchable, by group or recently used; a click applies
  it (#4).
- Comments docker: review notes pinned to a point, a box or an arrow on
  the page, with replies, resolve and reopen; saved with the drawing.
- Export docker: list pages and selections as assets, each with its
  own name, format and resolution, and export them all at once.
- Variable Outline tool (Shape tool flyout): drag across an outline to
  make it thicker or thinner at that point, one side with Alt; widths on
  the property bar.
- The dark theme's picked items show white text on blue.
- README: new screenshots (templates, AI Generate, painterly brushes,
  drawing in perspective) and some in the dark themes.
- File > New From Template is a dialog: search, a list of where
  templates come from and one of categories, thumbnails with a size
  slider, and a details pane. Besides blank pages it offers designed
  templates (business cards, flyers, a poster, a letterhead, a
  certificate, an invoice, a banner, a social post and a label) and the
  user's own `.tdt` files. The Welcome Screen's Templates page shows them
  as thumbnails.
- Dark theme: darker bars and menus, a grey desktop around the page
  (unless another desktop colour is set) and blue for the tab, row and
  tool in front.

## 0.6.0 (2026-10-15)

Closer to everyday drawing work: menus with pictures and access keys,
themes from light to dark, a Customization dialog with command bars,
commands and their shortcut keys, bitmap effects that keep vector
objects editable, symmetry editing, the Shape tool cropping pictures,
drawings opened from and saved to GitHub, and `.cdr` files saved by the
newest versions opening with their fills and spot colours.

- `.cdr` files saved by the newest versions (also when saved for older
  versions) keep their fills: text and backgrounds no longer go missing.
  Colours from spot colour libraries, which the file names by number
  only, take the look the file's own page picture shows instead of
  black, and character spacing is counted in spaces, so widely spaced
  headings fit their frames (#67).
- Fountain fills: Import Gradient reads a `.ggr` gradient file into the
  stops (#4).
- `.cdr`: an object embedded from another program with no picture to
  show (version 21 media objects) gets its own warning instead of
  "object type 0x0a not modelled yet".
- Menus, lists and dialogs open at once, without fading in, and a
  folder called `tracedraw-settings` next to the program makes a
  portable copy that keeps its preferences there (#4).
- Shape tool on pictures (#60): a picture shows a node at each corner;
  dragging one crops the picture to the reshaped outline (a clip frame
  whose frame keeps editing as a curve). A double click no longer turns
  a picture, text, group or clip frame into a curve, which made a
  picture disappear.
- Export: the pixel size of bitmap formats can be typed (the resolution
  follows), and resolutions go down to 1 dpi, for small icons (#4).
- Pictures on the rows of every menu: File, Edit, Layout, Object, Effects,
  Table, Tools and Help join Text, Bitmaps and Window (submenu rows
  too). Each bitmap effect's row shows the effect applied to a small
  landscape. Welcome Screen lost a picture that did not belong to it.
- Themes: Light, Medium Light, Medium Dark and Dark in Options >
  Customization > Appearance > Color, and a light / dark button at the
  right end of the status bar. Turning proof colours on or off says so
  in the status bar.
- Menus have access keys: one letter of each title and row is
  underlined; with a menu open, pressing it picks the row (or opens its
  submenu), and Alt with a title's letter opens that menu (except the
  Alt letters commands use: E, S, X, Q, Z). Shortcuts wait while a menu
  is open.
- `.cdr` version 12: guidelines are read into the pages' guides (they
  were skipped objects), pictures embedded from a paint program show in
  their place, spot library colours show as black at their tint instead
  of solid black, and a text on a path finds its path when its box ends
  just short of the path. The empty compressed block every version 12
  file starts with no longer logs a warning.
- Options > Customization > Commands: a category list (every menu, the
  toolbox, or all) and a search above the commands with their pictures;
  beside them a General tab (name, where the command is, its keys) and a
  Shortcut Keys tab (press the new keys, the current ones with Delete,
  Reset to Defaults). Tool shortcuts are set the same way.
- Options > Customization: pages in the order Appearance, Command Bars,
  Commands, Color Palette. Appearance has Center Dialog Boxes (off,
  dialogs move by their title bar and open where they were left), a
  size slider with a percent field and Reset to Default, and a Color
  group. Command Bars lists the bars in a box with Reset, and Lock
  Toolbars beside it. On the palette, with a right click setting the
  outline colour, holding the right button for a second opens the
  palette menu.
- Effects menu: the groups in alphabetical order, Picture Mosaic before
  Vector Mosaic, and Effects > Blur > Smart Blur (smooths flat areas,
  keeps edges).
- Text, Bitmaps and Window menus: pictures on the rows; Tabs, Columns,
  Bullets and Drop Cap work with no text selected and set what new
  paragraph text gets; Fit Text to Path with no path selected waits for
  a click on the path (and starts a new text on it when no text is
  selected); Change Case is a dialog; Show Text Frames moved to
  Options > Text; the trace submenus are greyed without a bitmap; the
  drawing in front has a dot in the Window menu.
- Help > Updates checks in the browser too (it opened the releases
  page), and the check shows a turning ring with Cancel.
- Bitmap effects (Effects > Blur, Texture, Creative and the others) work
  on vector objects: the object stays editable and is drawn with its
  effects; Flatten Effects makes it a bitmap.
- Symmetry: Create New Symmetry goes straight into an editing mode (the
  original outlined in red, a bar with Finish, mirror lines, centre and
  angle at the top left of the window) instead of a dialog; Edit
  Symmetry (Alt+E), Finish Editing Symmetry (Alt+Shift+E), Remove
  Symmetry (Alt+Shift+S) and Break Symmetry Link (Alt+X) in the menu.
- File > GitHub: open a drawing from a GitHub repository, save it back
  as a commit with a message, and see and open its earlier versions.
- `.cdr` version 22: connector lines (object type 0x26) are read; they
  were skipped.
- `.cdr` versions 7 to 12: arrowheads are read and placed on their lines,
  artistic text stands where the file puts it (it was drawn far off the
  page), and version 12 text stored as one byte a character reads right.
- `tracedraw-cli inspect file.cdr <chunk>` prints the data of every
  chunk with that id in hex.
- The browser version says at each start that it is the same Rust
  program as the desktop app, compiled to WebAssembly, slower in a
  browser, and links to the desktop app.

## 0.5.0 (2026-10-14)

More for print work, and better `.cdr` files:
spot colours through to separations, opacity masks, several fills and
outlines on one object, an effect stack, 3D extrude with lights and
bevels, CMX import, session recovery after a crash, and scripts that
add file formats and can be debugged. Pictures stored in CMYK inside
`.cdr` files now open with their real colours (in 0.4.0 they looked
like negatives).

- Script debugger: breakpoints in the Scripts panel's margin, Debug,
  Continue, Step, Stop, and expressions evaluated where the script
  paused.
- Help > Updates: a Linux AppImage installs the new release in place
  (Install), after checking the download's size and SHA-256 digest.
- Script filters: a script that defines `TraceDrawFilter` becomes a file
  format of File > Import and File > Export (Scripts panel, Install as
  Filter); `Shape.PathData` gives an outline as SVG path data.
- Help > Highlight What's New marks the menu rows new in this version.
- `.cdr`: arrowheads of newer files (`arrx` records) are read, and the
  arrowhead ids of outlines are read from their right place, so dimension
  lines and arrows keep their heads.
- Colour books: the spot entries of an `.ase` palette are spot colours;
  applying one adds its ink to the drawing.
- Welcome Screen, What's New: the news covers the effect stack, 3D
  extrude, spot colours and opacity masks, and the language setting's
  place (Tools > Options > Global > General).
- Effects docker: the selected object's effect stack, each effect
  switched off and on, moved up or down, or removed.
- Several fills and outlines on one object: Add Fill and Add Outline in
  the Properties docker paint further layers over the object's own.
- Opacity masks: Object > Clip to Frame > Opacity Mask makes a clip frame's
  brightness set how much of its contents shows (Invert Mask the other
  way round), on screen and in SVG and PDF output.
- Shelf: Add Colours keeps a drawing's document palette and colour
  styles as an item, which placing adds to another drawing.
- A user guide in `docs/guide/`.
- Scripts: `Shape.Effects.Add` adds any effect in its `.tdraw` form,
  with `AddExtrude` and `AddBlockShadow` as shortcuts.
- Help > Updates checks for a newer release and offers its download;
  the same check runs once a day at start (Options > Global > General).
- Spot colours: named inks in the Colour panel, applied at a tint,
  written to PDF as Separation colour spaces and printed on a plate of
  their own in separations.
- `.tdraw` files record their format version; older files are brought up
  to date on opening and newer ones open as far as this version knows
  them (see `docs/tdraw-format.md`).
- Every object has a stable identity, kept across saving, undo and
  moves between drawings; copies get their own.
- Scripts: `Shape.Curve` gives the curve's subpaths and nodes (position,
  type, move, delete, add), `Shape.Effects` lists, removes and clears
  effects.
- Session recovery: drawings auto-backed up by a session that ended
  without closing are offered at the next start; a clean exit removes
  its auto-backups.
- Extrude: the body turns in space (x, y, z), takes up to three lights
  with their elevation, and a bevel on its front edge (panel, and the
  bar's 3D menu).
- CMX import: text comes in as artistic text at its place, font, size
  and colour.
- Options > Tools > Pick: Marquee selects touched objects (off by
  default) makes a marquee take the objects it only touches, Alt then
  taking only those wholly inside (#2).
- Window > New Window: another window of the same drawing, "name:1" and
  "name:2", each with its own view and selection; edits, undo and saving
  are shared, and closing one window keeps the drawing open in the other.
- CMX import: arrowheads come in at the size and place the file gives
  them, open heads as lines and start heads the right way up, as the
  files' previews show.
- `.cdr` files: node types (cusp, smooth, symmetrical) are read and
  written, so a cusp node with aligned handles stays a cusp.
- Shape tool: a node's type is kept with the curve once chosen, so C and S
  switch it both ways and a cusp node with aligned handles moves its
  handles apart.
- Node tracking (Options > Display, off by default): the Pick tool drags
  the nodes of a selected curve.
- Pick tool: Ctrl+click goes one group level further in at each click;
  Ctrl keeps the marquee square; Space drops a copy while scaling, turning
  or slanting too. Ctrl+Space switches to the Pick tool from any tool.
  Selection groups: Ctrl+number stores the selection, the number selects
  it again (twice: and zooms), Alt+number adds; saved with the drawing.
  Ctrl keeps the Freehand Pick selection a rectangle.
- Shape tool: the arrow keys move the chosen nodes; Home and End choose
  the curve's first and last node; C and S switch a node's type back and
  forth (cusp and smooth, symmetrical and smooth).
- The interface in six more languages: Chinese (Traditional), Italian,
  Dutch, Polish, Turkish and Czech, eighteen in all. Taiwan, Hong Kong and
  Macau locales choose Traditional Chinese.
- Import CMX exchange files: pages, layers, groups, curves, rectangles,
  ellipses, uniform and fountain fills, outlines with dashes and
  arrowheads, embedded images and clips. Text, lenses and bitmap fills
  are not read yet; the import report names what was skipped.
- Snapping: the names beside snap marks are in lower case (node, edge,
  midpoint), and the edge a drag snaps to lights up in blue.
- Shape tool: a curve being edited shows its outline in the node colour
  and its first node as a triangle; the direction arrow is off by default.
  Control handles are dashed lines ending in solid arrow heads, also
  while a curve is drawn.
- Preferences saved by older versions turn on treating all objects as
  filled once, so a click inside an unfilled object selects it there too.
- Dragging the selection (move, scale, turn, slant) leaves the objects in
  place and shows their outline where they would go, on a white halo that
  stays visible over dark fills. A move grabbed on an edge mark carries
  that point along other objects' edges; turning and slanting snap to
  other objects' points like scaling does.
- Starting to draw a new shape lets go of the selection at once.
- Selection handles: solid black squares just outside the selection box,
  which has no frame any more, so the objects' outlines stay in view; a
  handle scales from where it was grabbed. In rotate mode the arrows have
  solid heads and stand outside the box too.
- The Pick tool marks the selection's node, midpoint, centre or edge under
  the resting pointer, with its name, as during a drag.
- The Pick tool treats all objects as filled by default: a click inside
  an unfilled object selects it. Options > Tools > Pick and the property
  bar turn it off.
- Shape tool: a dragged node, control handle or shape node snaps to the
  grid, guidelines, the page and other objects like the drawing tools
  do; it ignored snapping (#3).
- Linux: after the splash screen the main window can be maximized and
  resized; it kept the splash screen's size as its maximum (#1).
- Linux: the interface is drawn a quarter larger at the default interface
  size (100 % is one interface point per pixel at the system's scaling);
  text was too small (#1). Tools > Options > Customization > Appearance >
  Interface size still sets it from 80 % to 200 %.
- Opening .cdr files: letters beyond Latin-1 (Czech, Polish, Turkish and
  other accented letters) in text of version 17 and later files read
  right; they showed as boxes and could break lines in the middle of a
  word. CMYK pictures show their own colours, converted through the
  colour profile the file carries; they looked inverted. Cut-out
  pictures keep their transparency instead of a white box. Paragraph
  text in a frame that was resized keeps its size and starts at the
  frame's top instead of being squashed or drawn above it. Rounded
  corners of rectangles in version 15 and later files have their real
  size, and shapes of version 14 and 15 files that have no outline of
  their own take their style's. Invert lenses of version 13 and later
  files show what lies beneath in its complementary colours instead of
  a solid box.
- Files with many large pictures open more than twice as fast (a 297 MB
  flyer: 20 to 8.6 seconds on two cores): pictures are decoded side by
  side and kept compressed more lightly, and a page keeps its decoded
  pictures between draws instead of decoding them again every time.
- Contour: the tool's drag draws a live contour (outside when dragged
  away from the object's middle, inside toward it); its bar used to wait
  for Apply and then made a static group of copies. The bar and the
  Contour panel share the controls: To centre, Inside, Outside, steps,
  offset, the corners (mitred, round or bevelled, round only before),
  the colour direction, the end colours and object and colour
  accelerations. To centre now spreads the colours over all its steps.
- Object > Copy Effect copies perspective, distortion, extrusion,
  contour, blend settings, lens, bevel and block shadow besides drop
  shadow, transparency and envelope; a distortion's centre, a
  perspective's corners and a vanishing point locked to the object sit
  the same way on the target.
- Blend keeps the end object as an object of its own: it can be moved,
  resized or recoloured and the steps follow, and selecting it picks the
  blend for the bar and panel. Blending used to delete the end object
  and keep a copy only inside the effect, so it could not be selected
  again; Clear Blend lost it.
- Segment Delete removes the stretch of an outline between the places
  where it crosses other outlines or itself, so overlapping lines can be
  trimmed back to their crossings; a drag removes every such stretch
  inside it. It used to remove one segment between two nodes, and a
  closed shape could lose its closing side by mistake.
- Crop: the area stays on the page with handles to move and resize it
  (and fields for its position and size) until a double-click inside,
  Enter or Crop applies it; Escape or Clear Crop Area drops it. It used
  to crop at once on release. Bitmaps are cropped (into a clip frame;
  Crop Bitmap led to a tool that skipped them), groups member by member
  (they were turned into one plain curve, losing their members), and
  open curves keep the parts inside instead of being closed.
- Extrude: six types (small and big, back and front, toward a vanishing
  point that can be locked to the object or the page, and back or
  front parallel), each with its depth; the body is coloured from the
  object, one solid colour or a shade per face, lit by one light. Only
  the faces that face the viewer are drawn, a curved side is one
  outlined surface instead of a row of slats, and every subpath is
  extruded (only the first was, so text and holes came out wrong). The
  tool's drag runs a parallel body along it or puts the vanishing point
  where it ends, its handle can be dragged, and the bar (its Apply used
  to add a separate static extrusion) and the Extrude panel share the
  same controls, showing and editing the selected extrusion.
- An outline that ends where it started (an ellipse) is treated as
  closed by Distort and Extrude.
- Distort: Push and Pull moves the nodes and bows the segments the other
  way (spikes or petals); it only scaled the object before. Zipper puts
  teeth on every segment, with Random, Smooth and Local; Twister takes
  whole turns, degrees and the direction. A drag is centred where it
  starts (the centre can be dragged afterwards), a Twister drag follows
  the turn of the pointer, and every distortion keeps the object's
  subpaths and open curves (they were merged into one closed outline).
  The bar shows and edits the selected distortion, with Center
  Distortion, Clear Distortion and Convert to Curves; its Apply button
  used to rewrite the object's outline instead of adding the effect.
- 3-point callout: a drag from the tip to the elbow works besides two
  clicks (the hint said so; only clicks were taken), the text goes on
  the side of the elbow it was placed on and opens with its sample text
  selected, so typing replaces it (it was typed after it). The leader
  and a new shape round the text (underline, rectangle, rounded
  rectangle or ellipse) follow the text as it is typed, resized or
  moved; a tip or elbow moved with the Shape tool stays. The tool has a
  property bar (outline width, line style, head at the tip, shape, gap),
  also for callouts selected with the Pick tool.
- Knife: a freehand cut besides the two-point line, a gap along the cut,
  and the pieces kept apart or as one object, from its property bar.
  Every separate piece becomes an object of its own, and open curves are
  split where the cut crosses them (they were closed into filled pieces).
- The Blend tool's bar sets the rotation and the object and colour
  accelerations besides the steps, for the next blend and live for a
  selected one, with Clear Blend and Break Apart. Dragging a slider in an
  effect panel (blend, contour, extrude and the others) or the drop and
  block shadow drags made an undo step per movement; a run of changes is
  now one step.
- Shape Recognition keeps shapes as drawn: a tilted rectangle or ellipse
  stays tilted (they came out level in the stroke's box), triangles and
  other four-sided shapes keep their corners (they became regular
  triangles and rectangles), and circles are told from squares by their
  area. Its property bar sets the recognition and smoothing levels;
  Sketch's sets joining to the selected curve, the distance and the
  smoothing.
- Connectors stay attached: move, scale or reshape an object and the
  connectors joined to it follow, in the same undo step; moved on its
  own, a connector is let go. They used to be plain lines. The connector
  tools have a property bar: straight, right angle or rounded (changing
  selected connectors too), the corner radius, and the outline of the
  selected ones.
- Dimensions: the tools have a property bar (precision, units, show
  units, prefix, suffix, text above, in or below the line, label size),
  shown also when dimensions are selected, and changing it redraws them.
  Dimensions are dynamic: scaled or skewed, they read the new length with
  upright text. The label is centred (it sat a quarter of the length to
  the left), extension lines run past the line on its side, and the one
  being placed follows the pointer. Parallel and Horizontal or Vertical
  take a drag as their hints said; Segment measures the segment clicked.
- The shaping brushes have their own bars: Attract and Repel, Smudge and
  Roughen showed only their name and could not change the nib size.
  Smooth has a rate, Smear and Smudge a pressure, Twirl a speed and its
  turn, Attract and Repel a speed and the mode, Roughen the spike
  frequency and size. A stroke is one undo step (two strokes in a row
  used to merge). The brushes work on each subpath they reach: before,
  brushing one part of a compound object joined all its parts into one
  outline, and Roughen closed open curves.
- Mesh Fill tool: its property bar sets the rows and columns (new meshes
  take them; the selected mesh changes and keeps its colours), Smooth
  colour, the selected node's transparency and Clear Mesh. A palette
  colour now colours the selected node; before, it replaced the whole
  mesh with a flat fill. Delete removes the grid lines through the
  selected node. Smooth colour now draws (it was only a checkbox): the
  colours flow through the nodes. The tool stays the Mesh Fill tool
  after making a mesh.
- Area Fill: the property bar chooses the fill (default, a colour, none)
  and the outline (default, a width and colour, none) of the objects it
  makes; objects with live effects count as drawn.
- Envelopes rebuilt. The Envelope tool keeps its own property bar
  (presets, the four modes, Add New, mapping, Keep Lines, Copy, Clear)
  instead of the distortion controls, and edits the frame itself. The
  modes now shape the frame (they were only buttons): Straight Line,
  Single Arc, Double Arc and Unconstrained, where the selected node's
  handles bend a segment alone. Double-clicking the frame adds a node,
  a node removes it. The mappings Horizontal, Original, Putty and
  Vertical change how the object fills the frame. Keep Lines and the
  mapping change the selected envelope at once. Envelopes from earlier
  files look as they did.
- Objects with live effects are picked and boxed where they are drawn:
  a click on the arch of an enveloped object selects it, and the
  selection box, the property bar's size and alignment take the warped
  outline and the objects the effects add.
- View > Proof Colors changes the drawing on screen: its colours go to
  CMYK and back, through the CMYK profile in use or the built-in model,
  so colours CMYK cannot print show duller. Before, the button only
  lit up.
- The object context menu's Frame Type works: Create Empty Clip Frame
  makes clip frames that stay in place when the object was turned (they
  were turned twice), No Frame turns empty clip frames back into their
  objects (it used to remove the outline), and Create Empty Text Frame
  replaces the object with an empty paragraph text frame of its size (it
  did nothing).
- Shaping: Trim keeps the objects that cut and Intersect keeps the
  originals, as their pictures always showed; Weld and the others
  replace them. The Shaping panel's "Leave original" boxes now do what
  they say (they were not used), each operation keeping its own choice,
  and the panel shows the operations as pictures with a large picture
  and a description of the one chosen.
- The Text docker has Character, Paragraph and Frame sections with
  header bars and buttons to open and close them (kept), the size list
  and bold, italic and underline as on the property bar, alignment
  pictures, and fields in rows. The new Frame section sets columns and
  their gutter, bullets, drop caps, hyphenation, fitting the text to the
  frame and where paragraph text sits from top to bottom: at the top, in
  the middle or at the bottom of its frame (new, saved with the drawing).
- Window > Cascade, Tile Horizontally and Tile Vertically put every open
  drawing in a window of its own, with a title bar, a button back to the
  tabs and a close button; the active one has its rulers and tools, the
  others show their page, and a click makes one active. Combine Windows
  and Dock Window bring the tabs back. Each drawing keeps its rendered
  page and its place in its window.
- Preferences are saved as they change (looked at once a second), not
  only when TraceDraw closes, so they last in the browser too (in its
  local storage) and after a crash. More of them are kept: the tool
  settings new objects take (polygon points, star sharpness, corners,
  spiral revolutions, table and graph paper size, eraser, text font and
  size, ...), what the window shows (grid, guidelines, page border,
  bleed, printable area, pixel and baseline grids, view mode, ...), all
  snapping toggles, and the Color and Align and Distribute dockers'
  choices.
- The Color docker has a color viewer (a saturation and brightness
  square beside a hue strip), the sliders and a palette's swatches, each
  shown or hidden by a button at the top; the selected object's colour
  sits beside the new one, and the new colour follows the selection. The
  HSB and HSL hue sliders count 0 to 360 degrees (they took the hue for
  a fraction of a turn before).
- The Align and Distribute docker has picture buttons. Align To (the
  active object, the page's edges, the page's centre, the grid) now does
  what it says, where before every choice aligned to the active object;
  Use Outline aligns the outer edges of the outlines. Distribute adds
  left, right, top and bottom edges to centres and gaps, and can spread
  the objects across the page (two objects are enough there); the
  Object menu lists the new ones too.
- The Transformations docker has picture buttons for its pages
  (Position, Rotate, Scale and Mirror, Size, Skew), a 3 x 3 reference
  point, Copies and one Apply: copies are made in one undo step, each the
  one before transformed again. Position and the rotation centre can be
  relative or absolute (from the ruler origin), the mirror buttons are
  applied with the scale, scale and size can stay proportional, skew can
  use the anchor point, and the size and absolute fields follow the
  selection. Alt+F8, Alt+F9 and Alt+F10 open Rotate, Scale and Mirror
  and Size as Alt+F7 opens Position, and Object > Transformations lists
  the five. Relative Position can be turned off again (it came back on).
- The Objects docker shows the page and the Master Page as a tree: the
  master layers, the guides (show, lock all) and the document grid (show)
  under the Master Page; each layer has a printer to turn printing and
  export on or off; the active layer, where new objects go, is in bold
  and is picked by clicking a layer (or an object on it); New Layer, New
  Master Layer (all pages, or odd or even from its context menu) and
  Delete (the selection, else the active layer) are at the bottom. A
  master layer can be moved, deleted and set to all, odd or even pages
  from its context menu, and its objects can be clicked, marquee
  selected and snapped to on every page it shows on.
- The Properties docker: its sections come open, in the order Outline,
  Fill, Transparency (new there: the Transparency tool's settings), the
  kind's own (corners, effects, character) and Summary, each with a header
  bar, and a row of buttons opens and closes them; the sections closed
  are kept. The fill kinds are pictures.
- The text property bar: the font size has a list of the usual sizes, the
  alignment is one button with its picture and a list, and Bullets, Drop
  Cap, Text Properties and Edit Text have buttons. The shape tools' bars
  (rectangle, ellipse, polygon, star) with an object selected end with
  Wrap Paragraph Text, To Front, To Back and Convert to Curves, as the
  Pick tool's bar does (Wrap Paragraph Text is new there too).
- The document palette stays a row under the drawing window wherever the
  colour palette is (on a side it was a column beside it). The colour
  palette starts at the bottom; moved to a side, along the top or
  floating, it stays there across restarts, in the browser too. Nightly
  builds that stood it at the right edge by default put it back at the
  bottom once.
- The browser version offers every family of the google/fonts collection
  (about 2,000 more), fetched from GitHub the first time a drawing's text
  needs one or the pointer rests on it in the font list; they show in
  the font lists with the download mark.
- A new font list in the property bar, the Text and Glyphs dockers and
  Options: type in the field to find a family (the name, then names
  starting with it, then the rest; Down, Up, Enter, Escape), the number
  of fonts and what to show (all, installed, in this drawing, to
  download), the five used last, every name drawn in its own font with
  an icon of how it is built (TrueType, OpenType, variable, came with the
  drawing, to download), the styles of each family under it, a slider for
  the sample size and a preview of "AaBbYyZz" with its guide lines.
- Dockers: the close button and a tab's Close Docker take the docker out
  of the tab column (Hints, Properties and Objects could not be closed),
  and the column with the docker in front is kept when TraceDraw closes,
  so a docker closed stays closed the next time. Help > Hints closes
  Hints when it is ticked.
- `.cdr` writing, made to match the sample files table by table: each
  page now lists the master page's guides, desktop and grid layers by the
  master layers' own GUIDs (they had GUIDs of their own, and the grid
  entry was missing), and the style table numbers its entries once and
  gives every style a fill, an outline, fonts for all three scripts,
  spacing and decoration settings that exist (styles named entry 0
  before). Text objects get numbers of their own and graphics a graphic
  style. These are the likeliest reasons written files closed the
  program that opened them.
- The browser version loads behind the desktop app's splash screen on
  the grey of the drawing window, with the download's progress in
  megabytes and percent on its bar, and under it what TraceDraw is (an
  open-source vector editor that opens `.cdr` files, alpha, written in
  Rust, MIT or Apache-2.0), which screens it suits and links to the
  desktop downloads, the code and the issues. Once up it simply runs:
  the demo message at the start, the menu bar's demo link and the
  Welcome Screen's demo note are gone.
- The `.cdr` issue form asks for the file's version and what the drawing
  uses (B-splines, lenses, live effects, pattern fills, paragraph
  settings, fitted text, pictures, pages).
- With two or more objects selected (or a group, or a curve of several
  parts) the Pick tool's property bar shows Group, Ungroup, Ungroup All,
  Combine, Break Apart, the shaping operations (Weld, Trim, Intersect,
  Simplify, Front Minus Back, Back Minus Front, Boundary) and Align and
  Distribute.
- `.ai` files written as PostScript (the format's early versions) open
  through the PostScript interpreter instead of failing as PDF.
- PostScript: `.prn` print files open and import (a printer job header
  before the program is skipped), multi-page `.ps` and `.prn` files open
  with one page per page of the file at its own size, and File > Export
  writes PostScript (`.ps`) and PRN (`.prn`) with every page.
- `.cdr` files: fonts embedded in the file are used for its text when
  they are not installed (and their licence allows it), so text keeps its
  letters instead of falling back to another font; the drawing carries
  them, saved in `.tdraw` files, and the Fonts docker marks them
  "embedded". Coloured runs of text in files from recent versions take
  their colour (stored inline in those files).
- The missing-font check also looks at text inside groups, clip frames,
  table cells and symbols.
- A missing font is drawn with a stand-in of the same widths when one is
  installed (Liberation Sans for Arial and Helvetica, Liberation Serif
  for Times New Roman, Liberation Mono for Courier New, Carlito for
  Calibri, Caladea for Cambria, Gelasio for Georgia, the TeX Gyre faces
  for the classic PostScript ones), else of the same style (serif,
  monospaced, casual or sans-serif), so text keeps its line breaks; the
  messages name the stand-in, as in "Arial (Liberation Sans)".
- `.cdr` files, checked against the previews files carry (public samples
  of versions 11 to 22 and files of the project's author): line breaks in
  artistic text of recent versions (they showed as boxes and the text ran
  on one line), centred lines, bold, semibold, italic and underlined runs
  from the style strings, rectangles of version 15 and later at their own
  size (the scale was applied twice) with corners in the right unit,
  polygons and stars at their size (they came out as a point), the size
  of each page, layer names, uniform transparency, and CMYK colours shown
  through the profile the file carries (the colours of the previews).
  Every face of an embedded family is used (a bold face next to the
  regular one), fonts are found by legacy names such as "Arial Narrow",
  and the palette keeps colours in the CMY, HSB, HLS, Lab and YIQ models.
- `.cdr` files: text fitted to a path follows its path when the two are
  not grouped (it was drawn straight, or off the page), at the offset and
  distance the file sets, mirrored text reading upright on the other side
  of the path. Objects of lists nested in a layer keep their stacking
  order.
- Text on a path: the letters stand on the path moved by the distance,
  spaced along it; on closed paths text runs on past the start, and
  mirrored text reads the right way round on the other side.
- Outlines thinner than a pixel are drawn one pixel wide on screen and in
  bitmap exports instead of fading out.
- `tracedraw-cli`: `TRACEDRAW_PAGE` chooses the page a bitmap or single-page
  export writes, and `TRACEDRAW_OBJECTS=1 tracedraw-cli info` lists every
  object the reader made.
- Letters a font lacks are drawn with a font that has them, so Chinese,
  Japanese, Korean, Arabic, Hebrew or Thai text set in a Latin font shows
  its letters instead of empty boxes.
- Variable fonts draw bold and italic text with their weight and slant
  axes instead of the default instance.
- The browser version offers about a hundred free font families (the
  stand-ins of the same widths for Calibri, Cambria, Georgia, Palatino,
  Century Gothic and others, popular families such as Open Sans, Roboto,
  Montserrat, Lato and Poppins, and fonts for Chinese, Japanese, Korean,
  Arabic, Hebrew and Thai). The font lists show them; a family's files
  are fetched the first time a drawing's text needs it.
- `.cdr` files before version 12: text, font names and layer and object
  names written in Cyrillic, Greek, Turkish, Central European, Baltic,
  Hebrew, Arabic, Thai, Vietnamese, Japanese, Chinese or Korean read in
  their own code page (they came out as Western European letters).
- File > Save As opens a Save Drawing dialog of its own: folder bar,
  shortcuts (home, desktop, documents, downloads, pictures, drives), the
  folder's contents with dates, types and sizes, file name, type, the
  `.cdr` version and the document's title, subject and rating, asking
  before replacing a file; in the browser it keeps the name, type,
  version and properties of the download.
- Save As `.cdr` writes any version from 7 to 22, chosen in that dialog
  (programs open files of their own version and older ones). Versions 7
  to 13 are RIFF files, 14 and 15 ZIP archives around one, 16 on ZIP
  archives with the chunk tree apart from its data, each with the
  records, tables, master page and layers of that version. A drawing
  opened from a `.cdr` file saves in its version unless another is
  chosen; Options > Global > File Formats sets the version of new
  drawings (12 at first). `tracedraw-cli` reads the
  version from `TRACEDRAW_CDR_VERSION` and the MCP `save` tool takes a
  `version`.
- `.cdr` writing: files of versions 21 and 22 carry the configuration
  record of those versions (page orientation, ruler origin, grid,
  units, nudge, duplicate offset, drawing scales) instead of an empty
  one, and every file has a font table for its styles.
- `.cdr` files: version 13 and later outlines read their nib angle and
  width scale from the right place, guide and grid layers stay hidden,
  and empty master layers no longer show up as layers of their own.
- File > Close All closes every drawing (it closed only the active one).
- AI Drawing (Tools > AI Drawing): describe what to draw or change and
  an AI service you set up with your own key (the Messages API, or any
  chat completions service, local model servers included) writes a
  script for the object model; it runs as one undo step, without file
  access and all or nothing, and a failing script is sent back once with
  its error. The key stays in memory unless you ask to remember it.
  Works on the desktop and in the browser. While a service is set up, a
  prompt bar above the colour palette takes requests (Enter draws) with
  a model list fetched from the service; Window > Toolbars hides it.
- Scripts: `CreatePath(d)` draws curves from SVG path data.
- AI agents can edit the drawings of the open window: with Options >
  Global > General > "Let AI agents on this computer edit the open
  drawing" on, an MCP client that starts `tracedraw --mcp-window` gets
  the tools of `tracedraw --mcp` on the window's drawings, with undo and
  shown at once. The window listens on this computer only, with a token.

- Ctrl+C, Ctrl+X and Ctrl+V copy, cut and paste objects in the desktop
  app (the windowing layer delivers them as clipboard events, not as
  key presses, so they did nothing); Ctrl+Shift+V pastes in the middle
  of the view.

**Known limits**
- Downloads are not code-signed yet: Windows SmartScreen asks for "More
  info > Run anyway", and macOS needs right-click > Open the first time.
- The `.cdr` reader is written from public notes and sample files; some
  files open with objects missing or looking different. Reports with a
  file are very welcome (there is an issue template for them).
- Installing an update in place works for the Linux AppImage only;
  elsewhere Help > Updates offers the download.
- Gaps are listed in [`docs/blueprint-gaps.md`](docs/blueprint-gaps.md).

## 0.4.0 (2026-10-11)

The Pick and Shape tools now behave as the keys and the pointer lead
you to expect: modifiers while dragging, choosing inside groups,
bending segments, a pointer that shows what a press will do.

**Selecting and transforming (Pick tool)**
- Shift scales from the centre, Ctrl scales in 100 % steps (dragging
  across gives the mirror image at the same size), Alt frees a corner,
  Ctrl keeps a move straight; a right click during a drag leaves the
  original, Space drops a copy, Esc cancels.
- In rotate mode the side handles skew, and the centre of rotation can
  be moved; it stays with the object (selecting it again finds it where
  it was put, and it moves with the object).
- Ctrl+click chooses an object inside a group (round handles; it moves,
  takes colours and is deleted on its own); Alt+click digs to the object
  beneath; a marquee with Alt takes what it touches.
- A click on any selected object, or on the centre mark, turns the
  handles; the centre mark can be dragged; scale handles snap to other
  objects, guidelines, the grid and the page; `+` duplicates in place.
- The Freehand Pick has the Pick tool's handles and takes what its lasso
  holds wholly (with Alt, what it touches).

**Curves and nodes**
- Shape tool: dragging a segment bends it (a straight one becomes a
  curve, smooth nodes stay smooth); a click on a segment marks where Add
  Node (or `+`) puts the new node; C, S and Y make the chosen nodes cusp,
  smooth or symmetrical; `-` deletes them; a right click chooses the
  node or segment under the pointer before the menu opens.
- A line drawn from (or to) an end node of the selected curve continues
  that curve, and closes it when it meets the other end.
- Ctrl keeps lines and Bezier nodes to steps of the constrain angle;
  with Freehand a click and then another click draw a straight line.
- The curve tools' property bar has the outline width, line style,
  arrowheads and Close Curve, and Freehand its smoothing (also in
  Options > Tools > Curve tools).

**The pointer**
- It shows what a press would do: resize arrows on the handles, move
  arrows on objects and the centre mark, turning and slant arrows in
  rotate mode, arrows across guidelines, the copy arrow when a move will
  leave the original.
- The Shape tool's own arrow marks nodes, control points and segments,
  and shows the resize, turning and slant arrows over the handles that
  frame selected nodes.
- Drawing tools show their icon by the cross hair, with a sign where a
  line would continue the selected curve or close the one being drawn;
  the Eraser and the brushes show their nib at its size; the Text tool
  shows the I-beam only over text; Interactive Fill shows the move
  arrows over the fill's handles.

**Text**
- Paragraph text frames show as dashed lines (Text > Show Text Frames;
  red with a down arrow when the text does not fit), the handles sit on
  the frame, and scaling it with the Pick tool resizes the frame so the
  text flows anew instead of stretching the letters. Lines that do not
  fit are no longer drawn over the others.

**Colour**
- Drag a colour from the palette onto an object: inside it fills it, on
  its outline it colours the outline; onto the document palette it joins
  it. Objects dropped on the document palette give it their colours.

**Interface**
- Interface size 100 % now draws menus, bars and text as large as other
  programs at 100 % (a quarter smaller than in 0.3.0 on a monitor the
  system scales by 125 %); drawings keep their real size at zoom 100 %.
- Touch workspace (Window > Workspace > Touch): a larger interface for
  touch screens; two fingers pan and pinch zooms.
- Shelf panel: keep objects for any drawing and after restarts, place
  them by double click or by dragging onto the page.
- Options > Global: Printing and File Formats set what the Print and
  Export dialogs start with (and can remember the last ones), where
  imported objects go, Bitmap Tracing sets Quick Trace's preset, and
  Warnings turns messages on and off.
- The status bar tells how far, how much and what angle while dragging.
- A double click on the drawing with the Rectangle tool no longer adds a
  page frame (double-click the tool's button for that).

**Files**
- Opening a `.cdr` file with objects that could not be read, or a
  drawing with fonts that are not installed, now says so in a message
  (with a Report button for the file).
- The browser demo's loading page says only that it is loading; the demo
  message shows each time the page opens.

**Known limits**
- Downloads are not code-signed yet: Windows SmartScreen asks for "More
  info > Run anyway", and macOS needs right-click > Open the first time.
- The `.cdr` reader is written from public notes and sample files; some
  files open with objects missing or looking different. Reports with a
  file are very welcome (there is an issue template for them).
- Gaps are listed in [`docs/blueprint-gaps.md`](docs/blueprint-gaps.md).

## 0.3.0 (2026-10-11)

The first release meant for everyone to try. It is an alpha: real
use will turn up bugs, and "Known limits" below lists what is known.

**Working in the window**
- Several drawings open in tabs; a Create a New Document dialog.
- The toolbox, palette, toolbars and status bar dock on any side or
  float, and the window remembers where they were.
- Keyboard shortcuts of your own for every menu command (Options >
  Customization > Commands), shown in the menus.
- Workspace files: export and import shortcuts, bars and toolbox,
  palette options and appearance, part by part; the bars shown come back
  after a restart.
- The interface size follows the scaling the system gives each monitor,
  like other programs (100 % by default, 80 % to 200 % relative to it).
- A splash screen, an About box, an interface size option, tool cursors,
  a Hints panel with a page per tool, typed number fields and a status bar
  with object details and coordinates.
- Right-button drag of objects with a drop menu; menus switch as the
  pointer moves along the menu bar.

**Drawing and shaping**
- Shape tools draw a live preview while dragging; rectangle corners
  (round, scalloped, chamfered) per corner; complex stars.
- Common Shapes: 71 shapes in five sets (basic, arrows, flowchart,
  banners, callouts).
- Shape tool node transforms, reflection and segment copy; Corners, Join
  Curves and Coordinates panels.
- Snapping modes with marks and screen tips; a drawing scale and ruler
  calibration.
- Linked object styles and default properties for graphics, artistic text
  and paragraph text.

**Text**
- Drop caps sit on their lines and hyphenation breaks where it should;
  character offsets and angles; the Shape tool on text.

**Bitmaps**
- Colour modes, bitmap effects kept as an editable list apart from the
  pixels, image adjustments with tone curves, Straighten Image, Manually
  Inflate Bitmap and Convert to Bitmap options.
- Vector Mosaic and Picture Mosaic.

**Print and PDF**
- Preflight checks before printing; imposition and printers' marks.
- Publish to PDF with bookmarks, document properties, links and
  passwords (AES-256); protected PDFs open after asking for their
  password.
- Time Tracker: time spent on a drawing, task by task.

**Platforms**
- The whole editor runs in the browser (WebAssembly), as a demo that
  says what it leaves out and links to the desktop app.
- Installers for Windows, macOS and Linux on every release, a Microsoft
  Store package in the build, and code signing ready to switch on.

**Known limits**
- Downloads are not code-signed yet: Windows SmartScreen asks for "More
  info > Run anyway", and macOS needs right-click > Open the first time.
- The `.cdr` reader is written from public notes and sample files; some
  files open with objects missing or looking different. Reports with a
  file are very welcome (there is an issue template for them).
- Gaps are listed in [`docs/blueprint-gaps.md`](docs/blueprint-gaps.md).

## 0.1.0 (2026-10-09)

First tagged version: the document model and command engine, the desktop
editor, the `.cdr` reader and writer, SVG, PDF and the other formats,
effects, text and the command-line tool.
