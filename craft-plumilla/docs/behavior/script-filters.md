# Script filters

A script filter is a JavaScript script that reads or writes a file
format. It defines one global object:

```js
const TraceDrawFilter = {
  name: 'Boxes',              // shown in the file dialogs
  extensions: ['box'],        // with or without the dot, any case
  Import(text) {              // optional: read a file
    for (const line of text.split('\n')) {
      const [x, y, w, h] = line.trim().split(/\s+/).map(Number);
      if (!isNaN(h)) ActiveLayer.CreateRectangle(x, y, w, h);
    }
  },
  Export() {                  // optional: return the file's text
    return ActivePage.Shapes.All.Shapes
      .map(s => [s.PositionX, s.PositionY, s.SizeWidth, s.SizeHeight].join(' '))
      .join('\n');
  },
};
```

- Scripts panel, **Install as Filter**: checks the script in the panel
  (it runs against an empty drawing with the limits of AI Drawing
  scripts, and nothing it does is kept) and keeps it in the preferences.
  It needs a name, at least one extension (letters and digits) and
  `Import` or `Export`. A filter of the same name is replaced. The panel
  lists the installed filters with **Edit** (back into the panel) and
  **Remove**.
- File > Import: the filters that import are in the dialog's file types,
  and a file with one of their extensions goes through the filter before
  the built-in readers. `Import(text)` gets the file as text (UTF-8, a
  byte order mark dropped, bytes that are not UTF-8 replaced) and
  creates objects in the active layer with the usual object model. They
  are added as one undo step, selected, and placed like any import
  (Options > Global > File Formats). When `Import` throws, nothing is
  added and the status line shows the error. What the script prints goes
  to the Scripts panel's output.
- File > Export: the filters that export are listed after the built-in
  formats. The filter sees the page, every page (All pages) or the
  selection (Selected only) as the active document, its first page
  active; `Export()` must return a string, which is written as the file.
  What the filter changes in that copy is dropped.
- `Shape.PathData` gives an object's outline as SVG path data in page
  millimetres (y up), for writing vector formats.
- Filters are text only.

## Checks

- `script_filters::tests::a_filter_says_what_it_reads_and_writes`
- `script_filters::tests::importing_runs_the_filter_as_one_step`
- `script_filters::tests::exporting_writes_what_the_filter_returns`
- `script_filters::tests::path_data_is_in_page_millimetres`
- Looked at: the Export dialog lists an installed filter after the
  built-in formats and hides the format options when it is chosen; the
  Scripts panel shows Install as Filter and the installed filters.
