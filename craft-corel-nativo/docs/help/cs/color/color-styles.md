# Barevné styly

**Barevný styl** je pojmenovaná barva, kterou používají objekty kresby. Když
styl změníte, změní se i každý objekt, který je jím vyplněn. Barvy návrhu tak
zůstanou jednotné a rychle vyzkoušíte jiná barevná schémata. Styly jsou
uvedeny v panelu [[docker.color_styles]]; otevřete jej z nabídky
[[menu.window > menu.window.dockers]].

![Panel Barevné styly](../../img/color-color-styles.png)

## Vytváření barevných stylů {#create}

- [[docker.new_from_selection]] přidá styl pro každou barvu výplně vybraných
  objektů, která ještě není stylem.
- [[docker.new_color_style]] přidá styl z aktuální barvy panelu
  [[docker.color]].

## Používání a úprava stylů {#edit}

Každý styl zobrazuje svou barvu, název a tři tlačítka:

- kliknutím na barvu ji změníte: objekty vyplněné touto barvou se přizpůsobí;
- kliknutím na název jej přejmenujete;
- [[docker.fill]] vyplní výběr stylem, [[docker.outline]] obarví jeho obrys;
- **x** styl odebere (objekty si svou barvu ponechají).

## Harmonie {#harmonies}

[[docker.harmony_editor]] vytvoří sadu stylů, které k sobě ladí, ze
základní barvy [[docker.base_color]]:

1. Zvolte [[docker.base_color]].
2. Klikněte na harmonii: [[harmony.analogous]], [[harmony.complementary]],
   [[harmony.split_complementary]], [[harmony.triadic]],
   [[harmony.tetradic]] nebo [[harmony.monochromatic]].

Barvy harmonie se přidají jako styly pojmenované podle ní.
