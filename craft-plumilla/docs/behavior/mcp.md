# MCP server (`craft-plumilla --mcp`)

`craft-plumilla --mcp` runs the editor without a window and speaks the Model
Context Protocol over stdin and stdout: JSON-RPC 2.0, one message per
line, protocol version 2024-11-05, the `tools` capability. An AI agent
(or any MCP client) connected to it opens, edits, inspects, exports and
saves documents with the same commands and undo history a user has.

Client configuration, for example:

```json
{ "mcpServers": { "craft-plumilla": { "command": "craft-plumilla", "args": ["--mcp"] } } }
```

## The open window (`tracedraw --mcp-window`)

The same tools can work on the drawings of a TraceDraw window the user
has open. Options > Global > General > "Let AI agents on this computer
edit the open drawing (MCP)" (off by default, desktop only) makes the
window listen on a free port of `127.0.0.1` and write the port, a random
128-bit token and its process id to `mcp-window.json` in the preferences
folder (removed when the option is turned off or TraceDraw closes). The
client is configured with the bridge as its command:

```json
{ "mcpServers": { "tracedraw-window": { "command": "tracedraw", "args": ["--mcp-window"] } } }
```

The bridge reads the file and forwards each line of standard input as an
HTTP `POST /mcp` with `Authorization: Bearer <token>` (never through a
proxy) and writes the answer to standard output. The window answers on
its own thread between frames, as if the user had run the command: the
tools act on the active drawing (`open` and `new_document` open tabs),
each change is one undo step, and the window shows it at once. The
status bar says when agents are let in and on which port, and the option
page shows the port.

Refused: requests without the token or with another one (403), requests
carrying an `Origin` header, which web pages send (403), other methods
(405) and paths (404), bodies over 16 MB (413). Notifications are
answered `202 Accepted` with no body. A request waits up to 120 s for the
window. Without a window that lets agents in, the bridge answers every
request with a JSON-RPC error (-32000) saying how to turn the option on.

## Methods

| Method | Reply |
| --- | --- |
| `initialize` | protocol version, `tools` capability, server name and version |
| `ping` | `{}` |
| `tools/list` | the tools below with their JSON schemas |
| `tools/call` | `content: [{type: "text", text}]`, `isError` on failure |
| notifications (no id) | no reply |
| anything else | error -32601 |

## Tools

| Tool | Arguments | Effect |
| --- | --- | --- |
| `open` | `path` | Opens any supported file (`.cdr`, `.tdraw`, SVG, PDF, AI, EPS, DXF, PSD, EMF, WMF, images) and returns the document description |
| `new_document` | `width_mm`, `height_mm`, `title` (all optional; A4 default) | A fresh document |
| `save` | `path`, `version` (optional, `.cdr` only: 7 to 27) | Native `.tdraw`, or `.cdr` through the writer in `version` (default: the version of the `.cdr` the document came from or was last saved in, else the one in Options > Global > File Formats); marks the document saved |
| `export` | `path`, `page_index` (0), `dpi` (96), `quality` (90), `webp_lossless` (false), `ai_version` (`pdf` or `8`) | SVG, SVGZ, PDF, AI, EPS, PS, PRN, DXF, EMF, WMF, PLT, PSD, HTML, PNG, JPEG, BMP, TIFF, WebP or GIF by file suffix |
| `run_script` | `source`, `object_model` (`native`, default, or `vgcore`) | Runs JavaScript against the selected object model and returns printed output; changes are one undo step. The two models deliberately have distinct rectangle contracts; see below. |
| `import` | `path`, optional `x_mm` and `y_mm` together | Imports into the current drawing with one undo step and returns imported IDs, document description and notice. Optional coordinates place the imported centre. |
| `copy_selection` | optional `ids` | Returns SVG with owned versioned native-selection metadata. Explicit IDs temporarily select current-page objects; the previous selection is restored. Does not access the operating-system clipboard. |
| `paste_selection` | `svg` | Pastes ordinary SVG or the owned native selection with one undo step. Damaged owned metadata is a terminal error. Returns selected IDs, document description and notice; no operating-system clipboard access. |
| `document_info` | | Title, file, modified flag, pages with size, layers with visibility and lock, every object with id, type, name, bounds in mm, fill, outline, opacity, text and font, group children, effect count; the selection |
| `undo`, `redo` | | One step back or forward, then the description |

New objects go to the active layer: the layer picked in the Objects
docker while it shows on the page, visible and unlocked, else the
topmost layer that is visible and unlocked (a hidden layer never
receives objects; with every layer hidden, the topmost unlocked one).
The same rule applies to drawing in the window.

Page formats use the zero-based `page_index`. PDF, PDF-based AI, PS, PRN and
HTML export the document. Classic AI 8 uses the selected page. DPI must be
between 1 and 2400; quality must be an integer from 1 to 100. Invalid
options and unsupported save extensions are refused before writing. `save`
accepts only `.tdraw` and `.cdr`; other formats use `export`. Raster export
uses the same codecs as the GUI and CLI, with true lossless WebP when selected.
RGB interchange does not establish preservation of source ICC profiles or CMYK
ink planes. None of these checks establishes equivalence with another editor.

## JavaScript object models

Preview 214714 adds `object_model` selection; portable 210340 predates that option.
Omitting it, or passing `native`, preserves the existing application API in
millimetres. For example, `ActiveLayer.CreateRectangle(10,30,50,10)` creates a
50 by 10 rectangle. Existing examples, command recording and scripts retain
this contract.

`vgcore` selects the separate object model already used by compiled VBA macros.
It uses document units and one-based collections. With
`ActiveDocument.Unit = cdrMillimeter`, the same four arguments to
`CreateRectangle` are left/top/right/bottom and create a 40 by 20 rectangle.
`CreateRectangle2` instead takes position and dimensions. This is JavaScript
using the macro object model; selecting it does not execute COM, VBA source or
the original editor and does not establish complete SDK compatibility.

```json
{"name":"run_script","arguments":{"object_model":"vgcore","source":"ActiveDocument.Unit = cdrMillimeter; const r = ActiveLayer.CreateRectangle(10,30,50,10); print(r.SizeWidth, r.SizeHeight);"}}
```

Unknown object-model values or types are rejected before script evaluation,
without changing the document, allocator, selection, undo or redo. User scripts
keep commands completed before a runtime error, as before; this is distinct
from the rejection of an invalid tool argument before evaluation.

The RPC source tests exercise both contracts through the actual tool dispatcher,
including one undo/redo and preserved redo after invalid model arguments.
The actual 214714 EXE/DLL passed both contracts, all three rectangle constructors,
unit persistence across requests and a second process, native save/reopen,
invalid arguments with held redo, and physical four-corner checks. Its PNG
export and a generated CDR27 save/reopen/export matched an independent circular
mask on 695434 pixels per image, excluding a 1.5-pixel antialias boundary. Wrong
corner mapping and square-corner controls were rejected. The comparison is RGB
composited over white, not a separate alpha-preservation or GUI/GPU test.

The source now also retains `vgcore`'s `Document.Unit` across requests and TDRAW
save/reopen in optional metadata, without changing physical millimetre geometry
or the native model. Existing files with no stored unit retain the owned inch
fallback. Zero is valid, unknown identifiers are rejected explicitly, and
`SaveSettings`/`RestoreSettings` preserve the absence of a stored value. This
does not verify CDR carriage or GUI ruler-unit integration. See `vba.md` for the
rectangle contracts and the remaining reference-editor uncertainties.

## Checks

- `initialize` answers with the protocol version and server name; a
  notification gets no reply; an unknown method gets -32601.
- `new_document` 100 x 50, then a script creating a red rectangle and
  printing `ActivePage.Shapes.Count`, prints `1`; `document_info` lists
  one rectangle with fill `R:255 G:0 B:0`; `undo` leaves none.
- `export` to svg, pdf, png and emf and `save` to cdr and tdraw each
  write a non-empty file; the saved `.cdr` opens again with its ellipse.
- `save` with `version` 21 writes a file that reads back as version 21;
  versions 3, 28, -1 and a string are refused and write nothing.
- Raster exports decode as their requested formats; AI starts with PDF or
  classic AI headers and SVGZ decompresses to SVG. Invalid options and a
  `save` request targeting PNG preserve existing file contents.
- The open window: with the option on, `initialize`, a notification and
  a script drawing an ellipse sent through the bridge get the server's
  name, no answer and `ok`, the window's drawing has one more object and
  Undo takes it back; the link file goes when the link does
  (`mcp_link::tests::agents_edit_the_drawing_in_the_window`).
- Requests without the token, with a wrong one or with an `Origin`
  header get 403, a GET 405, another path 404, the right token 200
  (`mcp_link::tests::strangers_and_web_pages_are_refused`).
- Without a window, or with one that went away, the bridge answers with
  an error naming the option (`mcp_link::tests::without_a_window_the_bridge_says_so`);
  the option starts and stops listening
  (`mcp_link::tests::the_option_starts_and_stops_listening`).
- Checked on screen: `tracedraw --mcp-window` drew a star and a line of
  text in the open window.
