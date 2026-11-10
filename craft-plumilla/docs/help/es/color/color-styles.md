# Estilos de color

Un **estilo de color** es un color con nombre que usan objetos del dibujo.
Cambie el estilo y todos los objetos rellenos con él cambiarán también, lo que
mantiene coherentes los colores de un diseño y permite probar rápidamente
otras combinaciones de colores. Los estilos aparecen en el panel
[[docker.color_styles]]; ábralo desde [[menu.window > menu.window.dockers]].

![El panel Estilos de color](../../img/color-color-styles.png)

## Crear estilos de color {#create}

- [[docker.new_from_selection]] añade un estilo por cada color de relleno de
  los objetos seleccionados que aún no sea un estilo.
- [[docker.new_color_style]] añade un estilo con el color actual del panel
  [[docker.color]].

## Usar y editar estilos {#edit}

Cada estilo muestra su color, su nombre y tres botones:

- haga clic en el color para cambiarlo: los objetos rellenos con ese color lo
  siguen;
- haga clic en el nombre para cambiarlo;
- [[docker.fill]] rellena la selección con el estilo, [[docker.outline]]
  colorea su contorno;
- **x** elimina el estilo (los objetos conservan su color).

## Armonías {#harmonies}

El [[docker.harmony_editor]] crea un conjunto de estilos que combinan entre
sí, a partir de un [[docker.base_color]]:

1. Elija el [[docker.base_color]].
2. Haga clic en una armonía: [[harmony.analogous]],
   [[harmony.complementary]], [[harmony.split_complementary]],
   [[harmony.triadic]], [[harmony.tetradic]] o [[harmony.monochromatic]].

Los colores de la armonía se añaden como estilos, con el nombre de la armonía.
