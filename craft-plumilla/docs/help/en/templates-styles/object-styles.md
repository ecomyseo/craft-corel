# Object styles and default properties

The [[docker.object_styles]] panel has two parts: the
[[defaults.title]] of new objects, and the styles of the drawing. Both are
saved with the drawing. Open it from [[menu.window > menu.window.dockers]],
or right-click an object and choose the object styles panel.

![The Object Styles panel](../../img/templates-styles-object-styles.png)

## Default properties {#defaults}

New objects take the fill and outline of their kind:

| Kind | In a new drawing |
|---|---|
| [[defaults.graphic]]: shapes, lines, tables and everything except text | no fill, a thin black outline |
| [[defaults.artistic_text]]: text without a frame | black fill, no outline |
| [[defaults.paragraph_text]]: text in a frame | black fill, no outline |

In the panel, each kind shows its fill and outline, with
[[docker.from_selection]] (take the fill and outline of the selected object)
and [[docker.reset]] (back to the defaults of a new drawing).

### Changing defaults with nothing selected {#document-defaults}

When nothing is selected and you choose a fill or outline (click a palette
color, use a fill tool, change the outline color or width), the change goes to
new objects. TraceDraw asks which kinds of new objects it applies to:

1. Tick [[defaults.graphic]], [[defaults.artistic_text]] and
   [[defaults.paragraph_text]] as needed. The kinds you ticked last time are
   ticked.
2. Tick [[defaults.dont_ask]] to skip the question from now on.
3. Click [[dialog.ok]]. [[dialog.cancel]] changes nothing.

![The Default Properties question](../../img/templates-styles-object-styles-document-defaults.png)

The check box [[defaults.ask]] in the panel turns the question on or off. The
[[docker.properties]] panel with nothing selected also edits the
[[defaults.graphic]] defaults. To keep the defaults for every new drawing,
choose [[menu.tools > menu.tools.save_settings_as_default]].

## Styles {#styles}

A style is a named set of properties: [[docker.style_fill]],
[[docker.style_outline]] (its color, width, line style and arrowheads) and
[[docker.style_transparency]]. Objects you give a style stay linked to it:
change the style and they all change.

### To make a style {#create}

1. Select an object with the look you want.
2. Click [[docker.new_style_from_selection]].

The style is named "Style 1" and so on, takes the fill, outline and
transparency of the object, and is linked to every selected object.

### Using and changing styles {#use}

Each style shows its swatches, its name (click it to rename), how many objects
are linked to it, and its parts as check boxes:

- [[docker.apply]] gives the selection the style and links it.
- [[docker.update_from_selection]] takes the fill, outline and transparency of
  the selected object; every linked object follows.
- Turning a part off leaves that property to each object.
- [[docker.delete]] removes the style; linked objects keep their look.
- [[docker.break_style_link]] unlinks the selected objects, which keep their
  look.

You can still change a linked object directly; the next change of its style
sets the style's parts again. Right-click an object to apply a style from its
menu.
