# Help

The user Help: task pages for people drawing with TraceDraw, published
in every interface language, and the context help that opens them from
the app.

## Where it opens

- Every "?" button of a dialog opens the Help page of that dialog: the
  page, and the section of it, that `help.rs` maps the dialog to. The
  Options dialogs' "?" opens the section of the options page on show
  (Tools > Options > TraceDraw at Snapping opens the Snapping section of
  the TraceDraw options page).
- F1 opens the page of what is in front: the open dialog, else the
  docker on show, else the active tool. F1 works with a dialog open, and
  is left to the shortcut field while Options > Customization > Commands
  waits for a key.
- Help > Help Topics opens the first page of the Help. Help > Product
  Help (F1) opens the page of the docker on show, else of the tool.
- The Hints docker's "Learn more" link opens the page of the tool or
  topic it shows; its first page links to the first page of the Help.
- Pages open in the browser (a new tab in the browser version), at
  `https://tracedraw.github.io/tracedraw/help/<language>/<page>.html#<anchor>`
  in the interface language (Options > Global > General > Language). Every
  page exists in every language: a page not translated yet is published
  from the English page with a notice at the top, so a link never breaks.

## The site

- `web/help_site.py` builds `docs/help/` into `site/help/`, one tree per
  interface language (`site/help/<code>/`), with the stylesheet and
  script shared. CI runs it in the `web` job, so the `pages` job
  publishes it with the browser version. `site/help/index.html` sends
  the reader to the browser's language, English otherwise.
- Each page has: a top bar (name, language list, light or dark switch);
  a side panel with two tabs, Contents (the tree: one collapsible
  section per part, the current page marked and its section open) and
  Search (the language's `search.json`, matched as the reader types:
  titles first, then headings, then text; works for scripts without
  spaces); breadcrumbs (Help, section, page); the page; Previous and
  Next in the order of the contents.
- The language list keeps the page and the anchor. Arabic pages are laid
  out right to left (a page shown in English inside the Arabic Help
  keeps left to right for its text).
- Light or dark follows the system until the switch is used, then the
  choice is kept in the browser. Under 820 px wide the side panel becomes
  a drawer opened by the button at the top left.
- Headings carry a `#` link to themselves on hover.

## Writing pages

- `docs/help/contents.txt` lists the pages in order: a line at the margin
  is a section (its page is the overview, followed by a list of its
  pages), an indented line a page of it. A page is
  `docs/help/<lang>/<path>.md`; English (`en`) is the source.
- The first line is the title, `# Title`. Every `##` and `###` heading
  ends with an explicit anchor, `## Inserting pages {#insert-page}`:
  lowercase ASCII, digits and hyphens, unique in the page, identical in
  every language. Links to a section use these anchors.
- Interface names come from the language files: `[[menu.file]]` prints
  "File" in English and "Arquivo" in Portuguese; `[[menu.file >
  menu.file.save_as]]` prints a menu path. A trailing "...", "…" or ":"
  of the label is dropped. Every menu, command, dialog, docker, tab,
  button and field name that has a key in `lang/en.toml` is written this
  way, never typed.
- Links between pages are relative `.md` links (`[Fills](../color/fills.md#mesh-fill)`);
  web links are absolute. Keys are `<kbd>Ctrl+J</kbd>`.
- Supported Markdown: headings, paragraphs, `-` and `1.` lists (nested
  by indentation), `**bold**`, `*italic*`, `` `code` ``, links, tables,
  code fences and `>` notes.
- Pages are written for people using the program: what a feature is for,
  how to do each task step by step, then what every option does. Facts
  come from the behaviour notes and the code, never from guesses. No em
  dashes; the program is TraceDraw; no other graphics program is named.
- A translation keeps the same file name, the same `{#id}` anchors and
  the same `[[keys]]` as the English page; only the prose changes.

## Screenshots

- The pages show the dialogs, options pages and panels they explain, as
  TraceDraw draws them, in English. They are made once, in English only,
  and every language's pages use the same pictures (the alt text and the
  page text are translated): `docs/help/img/<topic>.png`, named after the
  Help topic with `/` and `#` as `-` (`pages-layout-pages-insert-page.png`),
  published as `site/help/img/`.
- `web/help_shots.sh` makes them again: it runs the ignored test
  `help_shots` (`src/help_shots.rs`), which opens each dialog through its
  menu command (or sets it, for dialogs without one), each options page
  and each panel, on a demo drawing of a filled rectangle, an ellipse, a
  line of text and a generated bitmap, draws the frame headlessly with
  the bundled interface font, paints egui's triangles in software and
  crops to the window (a panel to its used height). Nothing of the
  user's is read. A PNG optimizer is used when one is installed.
- A page refers to its picture as `![Alt text](../../img/<name>.png)`;
  the generator stops on a picture that does not exist.

## Checks

- `help.rs` tests: every dialog, options page, docker and tool maps to a
  page of `contents.txt` whose English file exists and has the anchor;
  addresses carry the language and fall back to English for an unknown
  one; F1 picks the dialog, then the docker, then the tool; every page
  of the contents exists in English, every translated page has the
  English anchors and labels, and every `[[key]]` it uses is in that
  language's `lang/<code>.toml`.
- `web/help_site.py` stops with an error on a link to a missing page or
  anchor, an unknown `[[key]]` or a listed page missing in English
  (`--partial` skips pages not written yet, `--lang=en,de` builds only
  some languages).
