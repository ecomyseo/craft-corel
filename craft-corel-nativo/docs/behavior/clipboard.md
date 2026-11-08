# Copy and paste

## Copy (Ctrl+C) and Cut (Ctrl+X)

The selection goes to TraceDraw's own clipboard, which keeps everything
(styles, effects, links to object styles), and as SVG text to the system
clipboard, so other programs can paste it. Right after the copy TraceDraw
notes what the system clipboard holds, without reading it back: the text
it wrote (writing text replaces everything there), and on Windows the
system's clipboard change count. A copy that writes nothing there (nodes,
guidelines) reads it: the count on Windows, else its text and files, and
a sample of its picture (the size and about 4096 bytes) only when there
is neither. Ctrl+C with nothing selected changes nothing.

## Paste (Ctrl+V, Edit > Paste)

- The system clipboard still holds what the last copy here left there
  (or there is no system clipboard): the objects copied here are pasted.
  Ctrl+V finds that out as cheaply as it can (#183): on Windows an
  unchanged change count pastes without reading the clipboard at all;
  elsewhere its text and files decide, and its picture is decoded only
  when there is neither text nor a file, or when the picture is what
   gets pasted (and then once). Large pictures can be expensive to decode,
   so Ctrl+C does not decode the existing picture and Ctrl+V reads it only
   when it must. Headless provider timings do not measure a desktop clipboard.
- Something was copied in another program since, or nothing was copied
  here: what the system offers, in this order:
  1. files copied in a file manager: each is imported like File >
     Import (one undo step each); a line of the list that ends in CR LF
     loses the CR;
  2. SVG text: its objects;
  3. an image: a bitmap object;
  4. other text: artistic text in the middle of the page.

Ctrl+Shift+V pastes the objects copied here in the middle of the view.
Paste on the page's context menu centres what it pastes where the menu
was opened.

### In a browser and on Android (#1403)

A browser hands the clipboard only to a paste event and to its
clipboard API, never to a plain read:

- Ctrl+V: the paste event's text goes the way above; its files (a
  picture copied in an image program or a web page, files copied in a
  file manager) are read in the background and pasted a frame later.
  The shortcut's own frame then pastes nothing, so the copy made here
  does not come as well. One picture alone is placed as a bitmap; other
  files are imported like File > Import. While a text is edited, a
  picture is not put into it.
- Edit > Paste (and the context menu's Paste) reads the clipboard with
  the browser's clipboard API (`navigator.clipboard.read()`): the first
  time the browser asks for permission (Firefox shows a Paste button).
  Its picture and text are pasted as for Ctrl+V; text that is still the
  copy made here pastes the objects copied here. When the browser
  refuses or has no such API, the copy made here is pasted and the
  status bar says that Ctrl+V pastes what other programs copied.

On Android the activity reads the system clipboard (`ClipboardManager`):
its text, and the documents other apps copied (content URIs, which
reading the clip grants the editor the right to read). One picture alone
(copied in a gallery, a browser or a chat) is the clipboard's picture,
placed as a bitmap; other documents are imported like files from the
document picker. A document's URI is never pasted as text.

Symbol instances pasted into another drawing (or inside pasted groups and
clip frames) bring their symbols along: a symbol the drawing holds
already, the same objects under the same name, is used; any other is
added to the drawing's symbols, with the symbols it shows, in the same
step as the paste. In the drawing copied from, an instance stays an
instance of its own symbol (#678,
`ops2::tests::an_instance_pasted_in_another_drawing_brings_its_symbol`,
`symbol_paste::tests`).

## When the clipboard holds no text

The windowing layer turns Ctrl+V into a paste event only while the
system clipboard holds text; with only an image or files on it, it drops
the key press and sends nothing but the release of V. A release of V
with Ctrl (Cmd), Shift+Insert or the Paste key that came without a paste
event or press before it pastes too, once. V let go after Ctrl is typing
and does not paste.

## Pictures on a Linux desktop (X11)

A screenshot tool keeps its picture on the clipboard itself and hands it
over when asked; a large picture comes in parts, each one after the
reader took the one before. The clipboard library gave each part 10 ms
and asked for PNG only, so a busy program's picture often came out
empty and took two or three pastes to arrive (#1488). On an X11 desktop
(no `WAYLAND_DISPLAY`) the paste now reads the clipboard's list of types
first and asks for the best picture type it offers (PNG, then BMP, TIFF,
WebP, GIF, JPEG), gives the owner 3 s to answer and each part 2 s, and
tries again twice, 150 ms apart, when no answer came
(`x11_clipboard.rs`). A clipboard without a picture type is not waited
for. On a Wayland desktop the clipboard library reads first and the X11
clipboard the desktop keeps beside it is read when it finds nothing.
TraceDraw keeps one connection to the system clipboard for the session
instead of opening one for each read.

On an X11 desktop the clipboard's text and its list of copied files are
asked for only when its list of types offers them (`UTF8_STRING` or
`text/plain;charset=utf-8` for text, `text/uri-list` for files), both
by the paste itself and by the windowing layer, which reads the text on
every paste shortcut (`clipboard::paste_may_read_text`, the gate in
`vendor/eframe`, see `vendor/README.md`). A program that holds a picture
alone, such as `xclip -selection clipboard -t image/png`, may answer a
text request with its picture sent in parts; the text read stopped
after the first part, and the program then waited for the rest and
answered nobody, so a picture of about 1 MB never pasted and the window
froze for about 24 s (#1488). An owner that gives no list of types, or
no program owning the clipboard, is not asked for text either.

## Browser build

Text events, handed files and asynchronous clipboard requests follow the
browser/Android section above. Availability depends on what the browser hands
over. A successful WebAssembly build alone does not verify those runtime APIs.

## Native selection transport, owned-format checks

The preserved 185821 preview uses an in-memory native selection and external
SVG. Preview 210340 implements a separate versioned selection inside SVG
metadata, using the namespace `urn:craft-corel:selection`. The SVG remains a
fallback for programs that do not read these metadata. This does not implement
another application's private clipboard formats.

The native block carries the selected geometry and its transitive symbols,
styles, spot inks, document fonts, actual profiles and bitmap ink planes.
Typed supplements retain text layout state deliberately omitted from ordinary
document serialization. A received block is always foreign, including when a
numeric source-instance identifier happens to match the destination. External
anchors are detached and internal references are remapped by the import
transaction. Unknown versions, damaged blocks and budget violations must fail
without falling back to a lossy SVG paste.

Publication uses a fallible provider. Copy stages its content before replacing
the internal copy, and Cut must not delete its source if publication fails.
Tests inject captured bytes and rejected or partial publication; native Windows
clipboard adapters are excluded from those tests. The large-picture benchmark
also uses a memory provider. Neither test execution nor the benchmark reads or
replaces the user's clipboard.

Checkpoint8 passed 79 executed application/codec/regression tests, including an
isolated 512-square CMYKA copy/paste job with every ink, alpha and preview byte
checked. Timing records describe minimal virtual egui frames, not the full editor.
The actual preview EXE/DLL passed independent-process transfers for all three
CMYK policies, complete RGBA/ink/profile/vector checks, selection, atomic refusal,
and one undo/redo. Wrong RGB with a recomputed fingerprint, missing vectors,
hidden bitmaps and collapsed bitmap geometry were rejected by the checker.

The same product passed native paste, save, recopy, undo and redo for 20, 64 and
128 nested groups on its unchanged one-MiB PE main stack. A 129-group packet was
rejected without losing an existing redo. The SVG's XML budget is separate from
the model's physical-depth budget: 128 groups generate 132 XML ancestors.
The rejected 205021 candidate remains preserved and is not recommended.

Evidence lives under `evidencias/craft-corel-nativo/portapapeles` in the enclosing
workspace. These checks use owned files, isolated settings and captured bytes;
they do not exercise Windows clipboard publication, the original editor's
private formats, visible GUI/GPU behaviour or browser clipboard APIs. WebAssembly
compilation passed; browser execution and its missing font bundle remain pending.

## Checks

- `clipboard::tests`: the copy here wins while the system clipboard
  holds what it left, or holds nothing; a picture or text copied
  elsewhere afterwards wins; copied files are imported; copied file paths
  lose the line end; the change check and the paste decode a picture only
  when they must (`the_change_check_decodes_a_picture_only_when_it_must`,
  `a_paste_decodes_the_picture_only_when_it_needs_it`).
- `clipboard::clipboard_bench::clipboard_costs` (ignored): an isolated memory
  provider carrying a 4000 x 3000 picture; it does not need a system clipboard.
- `pick_drag_tests::ctrl_v_pastes_when_only_its_release_comes_through`
  and `copy_cut_and_paste_events_work_on_objects`.
- `x11_clipboard::tests::a_picture_sent_slowly_in_parts_arrives_whole`
  (under the X server of `DISPLAY`, else an Xvfb it starts; skipped
  without either): a program that owns the clipboard sends a PNG in
  2 KB parts, 30 ms apart, and the first read gets it whole; once the
  program is gone a read finds nothing within 2 s.
  `picture_types_go_best_first` and `pictures_decode_by_type_or_by_signature`.
- Upstream checks under Xvfb with `xclip`: a picture (`image/png`) and a file
  (`text/uri-list` with CR LF) copied after a copy here paste as a bitmap
  and as an imported file; copying here again pastes the copy. These historical
  Linux checks are not evidence of this fork's Windows clipboard transport.
