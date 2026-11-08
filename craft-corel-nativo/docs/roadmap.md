# Roadmap

File compatibility target: open `.cdr` format versions 9 through 17
reliably; newer versions best effort.

## M0 Skeleton (done)
Workspace, document model, commands + undo, SVG export, native format,
`.cdr` reader skeleton, egui app with pick/rect/ellipse/polygon tools.

## M0.5 Workspace shell (done)
The full workspace layout: menus, standard toolbar, property bar,
toolbox with flyouts, rulers, palette, panels, navigator, status bar.
Status tracked in `docs/features.md`.

## M1 Real files (done for all three containers)
- `tracedraw-cli inspect` dumps chunk trees
- Text objects read as text with font and size; bitmaps read and drawn
- Layer names, page names, multi-page
- Version 16+ container (`content/root.dat`, redirected chunks); a
  version 21 file opens correctly (text, cropped bitmaps, object order)
- Open: corpus per version in `corpus/`, text on path, splines, paragraph
  formatting from the style tables

## M2 Drawing (done)
- All 64 tools, node editing, transform handles, snapping, rulers
- tiny-skia renderer: fills of every kind, dashes, hairlines, effects
- Text engine with shaping, paragraph layout, text on path, tables
- Live effects, lenses, transparency, mesh fill, symbols, master layers
- Symmetry, clip frame editing, linked text frames, connector anchors,
  writing tools, clone effects, custom arrowheads

## M3 Output (done)
- PDF export (vector, CMYK, separations), EPS, AI (PDF-based), bitmaps
- Print to PDF, print preview, print merge
- PDF/X-1a, X-3 and X-4 output (done)
- Export as PDF with document information, bookmarks and password protection (AES-256); protected PDFs of every revision open (done; see `behavior/publish-to-pdf.md`)
- `.cdr` write in versions 7 to 27, chosen when saving (RIFF, ZIP and redirected ZIP layouts; read back here and opened by another public reader) (done)

## M4 Platform (done)
- CLI batch convert (done); installers for Windows, macOS, Linux (done)
- Signed Windows downloads and a Microsoft Store package: the build signs through SignPath or Azure Artifact Signing and packs an MSIX once the accounts are set up (build ready; signing waits for the accounts, see `code-signing.md`)
- JavaScript automation with macro recording (done)
- User interface in 18 languages (done)
- JSON control channel and MCP server (`tracedraw --mcp`, see `behavior/mcp.md`) (done)
- WebAssembly build: the same editor in the browser (WebGL 2), published to GitHub Pages and attached to releases (done; see `behavior/web.md`)

## M5 Interchange
- Import PDF, AI, EPS, DXF, PSD, EMF and WMF (done); export HTML, DXF, EMF and WMF (done); CMX (import partial: no lenses or bitmap fills yet)
- External ICC profiles (done: pure-Rust ICC engine, v2/v4), PDF/X intents (done)
- The remaining items in `docs/blueprint-gaps.md`

## M6 Editing, colour and effects (planned)
- Pick and Shape tool parity: every interaction checked side by side, one behaviour at a time (in progress)
- Extrude: 3D rotation, several lights and bevels (done; see `behavior/extrude.md`)
- Opacity masks as objects of their own, next to transparency (done: a clip frame drawn as an opacity mask; see `behavior/clip-frame.md`)
- Artboards: not planned (decided 2026-10-14: pages stay the only drawing areas, as in `.cdr`)
- Spot colours with named inks through to separations (done; see `behavior/spot-colors.md`); colour books the user loads (done: spot entries of `.ase` palettes; licensed books are not bundled)
- Several fills and outlines on one object (done; see `behavior/multiple-paint.md`); live effects as an ordered stack, each one editable, hidden or removed (done: the Effects docker, see `behavior/effects-docker.md`)
- Bitmap tracing to curves (outline and centreline, with colour reduction and smoothing) (done; see `behavior/bitmap-tracing.md`)
- Node types stored with the curve (done)
- Asset library: objects, symbols, colours and styles shared between drawings (done: the Shelf keeps objects, with their object styles and symbols turned into objects, and colours with colour styles; see `behavior/shelf.md`)
- Session recovery after a crash (on top of auto-backup) (done)
- Scripting stays JavaScript only (decided 2026-10-14: one object model, recorded macros in JavaScript, runs in the browser too)
- Drawing window: focus mode, Multipage View, Draw in Perspective (one-, two- and three-point), variable-width outlines, painterly brushes, review comments and an Export panel for many assets at once (done; see the `behavior/` pages of each)
- Accessibility: the desktop's text size and high contrast themes followed, coloured tool icons (done; see `behavior/options.md`)

## M7 File format (planned)
- `.tdraw` with a format version, migrations from older versions and a stability promise (done; see `tdraw-format.md`)
- Stable object IDs, kept across saves and copies: the base for merging and collaboration, settled before M8 (done; see `architecture.md`, Object identity)
- `.cdr` target versions written down, each layout confirmed on files (public notes and other readers' documentation only to confirm a layout, never their code)

## M8 Saving and collaboration (planned)
- Git hosting: open a `.tdraw` straight from a repository, save as a commit with a message, history from the log (done with a personal access token, see `behavior/github.md`); OAuth sign-in, a branch per session, a pull request from the app, LFS for large files, a visual diff between commits, conflicts resolved in the interface, the token kept in the system keyring (open)
- Storage: Google Drive, Dropbox, OneDrive, S3-compatible storage (AWS, MinIO, Cloudflare R2), a synced local folder, WebDAV
- Real-time editing: a self-hosted WebSocket server, automatic merging with a CRDT (Yjs or Automerge), presence with each user's pointer and selection, optional locks per object, comments anchored to objects, permissions per project, share links that expire, version history with restore
- Order (decided 2026-10-14): Git hosting first, without a server of our own; storage services and real-time editing later. Stable IDs per object are done

## Later: image editing of its own (idea)
- A raster editing tool inside TraceDraw (layers, selections, retouching) so a bitmap need not leave for an outside editor; until then Edit Bitmap hands the file to the system editor and the drawing follows its saves

## M9 Release (planned)
- Automatic updates from inside the app (checking and downloading done, see `behavior/updates.md`; installing in place done for the Linux AppImage; Windows and macOS wait for code signing and a test on real machines)
- User documentation (done; see `guide/README.md`)
- Public launch and announcement once Pick and Shape tool parity is reached
