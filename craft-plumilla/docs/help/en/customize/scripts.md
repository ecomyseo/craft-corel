# Scripts and automation

Scripts automate repetitive work: draw shapes from data, change many objects
at once, or add a file format. They are written in JavaScript in the
[[docker.scripts]] panel ([[menu.tools > menu.tools.scripts >
menu.tools.scripts_docker]]).

![The Scripts panel](../../img/customize-scripts.png)

## Writing and running scripts {#run}

A script works with the objects of TraceDraw: `Application`,
`ActiveDocument`, `ActivePage`, `ActiveLayer` and the shapes, fills,
outlines, colors, curves and effects. For example:

```js
for (let i = 0; i < 5; i++) {
  ActiveLayer.CreateRectangle(10 + i * 25, 10, 20, 20);
}
```

- [[docker.run]] runs the script. Its changes are one undo step.
- [[docker.record]] records what you do as a script (also
  [[menu.tools > menu.tools.scripts > menu.tools.scripts_record]]).
- [[docker.load]] and [[docker.save]] open and save scripts;
  [[menu.tools.scripts_run]] runs a script file.
- [[docker.clear]] empties the editor; the [[docker.output]] shows what the
  script prints and its errors.

## Debugging {#debug}

On the desktop, click in the margin of the editor to set a breakpoint, then
click [[docker.debug]]. The script stops at the breakpoint: **Continue**,
**Step** or **Stop** it, and type an expression to evaluate it where the
script is paused.

## Script filters {#filters}

A script that defines `TraceDrawFilter`, with a name, extensions and an
`Import` or `Export` function, adds a file format to
[[menu.file.import]] and [[menu.file.export]]. Click
[[docker.install_filter]] to keep it; [[docker.script_filters]] lists the
installed filters with [[docker.edit]] and [[docker.remove]].

## AI assistants (MCP) {#mcp}

TraceDraw speaks the Model Context Protocol, so AI assistants on your
computer can open, edit, inspect, export and save drawings:

- `tracedraw --mcp` runs TraceDraw without a window for an assistant.
- To let an assistant edit the drawing open in your window, turn on
  [[options.mcp_window]] on the [[options.general]] page of
  [[menu.tools > menu.tools.options > menu.tools.options_global]], and give
  the assistant `tracedraw --mcp-window` as its command. Each change is one
  undo step you can take back. Only programs on your computer with the
  secret token TraceDraw writes can connect.

## The command-line tool {#cli}

Every download also has `tracedraw-cli`, which converts drawings between
formats without opening the program, for batch work.

## Macros {#macros}

TraceDraw runs VBA macro projects written for drawing programs (desktop version). [[menu.tools > menu.tools.macros > menu.tools.macros_load]] adds a project (`.gms`) or module files (`.bas`, `.cls`, `.frm`); projects are kept and load at start-up. The [[docker.macros]] lists their macros: double-click one or use [[menu.tools > menu.tools.macros > menu.tools.macros_run]] (<kbd>Alt+Shift+F8</kbd>) to run it. A macro's forms, message boxes and input boxes appear on screen, <kbd>Esc</kbd> or [[menu.tools > menu.tools.macros > menu.tools.macros_stop]] stops it, and everything one run changes is one undo step. [[menu.tools > menu.tools.macros > menu.tools.macro_editor]] (<kbd>Alt+F11</kbd>) shows and edits the code. Macros that call Windows functions or the editor's own object model do not run.
