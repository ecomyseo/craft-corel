# Color styles

A **color style** is a named color used by objects of the drawing. Change the
style and every object filled with it changes too, which keeps a design's
colors consistent and makes it quick to try other color schemes. The styles
are listed in the [[docker.color_styles]] panel; open it from
[[menu.window > menu.window.dockers]].

![The Color Styles panel](../../img/color-color-styles.png)

## Making color styles {#create}

- [[docker.new_from_selection]] adds a style for each fill color of the
  selected objects that is not a style yet.
- [[docker.new_color_style]] adds a style of the current color of the
  [[docker.color]] panel.

## Using and editing styles {#edit}

Each style shows its color, its name and three buttons:

- click the color to change it: the objects filled with that color follow;
- click the name to rename it;
- [[docker.fill]] fills the selection with the style, [[docker.outline]]
  colors its outline;
- **x** removes the style (the objects keep their color).

## Harmonies {#harmonies}

The [[docker.harmony_editor]] makes a set of styles that go together, from a
[[docker.base_color]]:

1. Choose the [[docker.base_color]].
2. Click a harmony: [[harmony.analogous]], [[harmony.complementary]],
   [[harmony.split_complementary]], [[harmony.triadic]],
   [[harmony.tetradic]] or [[harmony.monochromatic]].

The colors of the harmony are added as styles, named after it.
