# El espacio de trabajo

La ventana de TraceDraw contiene sus dibujos y todo lo que necesita para
trabajar en ellos. Esta página nombra sus partes, para que el resto de esta
Ayuda tenga sentido.

## Las partes de la ventana {#parts}

De arriba abajo y de izquierda a derecha:

- La **barra de menús** contiene todos los comandos, en los menús
  [[menu.file]], [[menu.edit]], [[menu.view]], [[menu.object]],
  [[menu.text]], [[menu.bitmaps]], [[menu.effects]], [[menu.layout]], [[menu.tools]], [[menu.window]] y [[menu.help]]. Cada
  comando muestra su atajo al lado. Sin ningún dibujo abierto, solo se ven
  [[menu.file]], [[menu.tools]], [[menu.window]] y [[menu.help]]. La página
  [Comandos de menú](../reference/menus.md) enumera todos los comandos.
- La **barra de herramientas estándar** contiene los comandos habituales (vea
  más abajo).
- La **barra de propiedades** muestra los ajustes de lo que está haciendo.
- Las **pestañas de documento** muestran una pestaña por dibujo abierto.
- La **caja de herramientas**, a la izquierda, contiene las herramientas.
- Las **reglas** recorren la parte superior y la izquierda de la ventana de
  dibujo.
- La **ventana de dibujo** muestra la página sobre el escritorio que la
  rodea.
- Los **paneles** están a la derecha.
- La **paleta de colores** recorre la parte inferior.
- La **barra de estado**, abajo del todo, describe la herramienta activa y la
  selección.

## La barra de herramientas estándar {#standard-toolbar}

La barra de herramientas estándar contiene, en grupos:

1. Nuevo, Abrir (su flecha enumera los últimos dibujos usados), Guardar e
   Imprimir.
2. Cortar, Copiar y Pegar.
3. Deshacer y Rehacer. Sus flechas enumeran los pasos, así que puede deshacer
   o rehacer varios a la vez eligiendo un paso.
4. Importar, Exportar y Publicar en PDF.
5. El cuadro de zoom: escriba un nivel de zoom y pulse <kbd>Enter</kbd>, o
   abra su lista para hacer zoom a lo seleccionado, a todos los objetos, a la
   página, al ancho o al alto de la página, y niveles del 10% al 1600%.
   Después, la vista previa a pantalla completa.
6. Mostrar reglas, mostrar cuadrícula y mostrar líneas guía, que aparecen
   pulsados mientras están activos.
7. Desactivar ajuste (<kbd>Alt+Q</kbd>) y la lista de elementos a los que
   ajustar.
8. Opciones, que abre la lista de páginas de opciones.
9. Iniciar: Administrador de fuentes, TraceCut y pantalla de bienvenida.

Pase el puntero sobre un botón para ver su nombre, su atajo y lo que hace.

## La barra de propiedades {#property-bar}

La barra de propiedades cambia según lo que haga. Sin nada seleccionado y con
la herramienta [[tool.pick]] activa, muestra el tamaño y la orientación de la
página, las unidades de dibujo y la distancia de desplazamiento. Con objetos
seleccionados, muestra su posición, tamaño, escala, rotación y los botones de
reflejo, y luego los ajustes de ese tipo de objeto. Cada herramienta añade sus
propios ajustes, explicados en la página de la herramienta.

En los campos numéricos puede escribir una suma como `10+5`, o un valor en otra
unidad como `1 in`, que se convierte a las unidades de dibujo.

## La caja de herramientas y los menús desplegables {#toolbox}

La caja de herramientas tiene un botón por grupo de herramientas. Un botón con
una pequeña flecha en la esquina contiene un **menú desplegable**, una lista de
herramientas relacionadas. Para abrirlo, haga clic en la flecha, haga clic con
el botón derecho en el botón o manténgalo pulsado. El botón muestra entonces la
última herramienta usada de ese grupo. Pase el puntero sobre una herramienta
para ver su nombre, su atajo y lo que hace.

Hacer doble clic en algunos botones tiene una acción rápida: [[tool.pick]]
selecciona todos los objetos, [[tool.zoom]] ajusta el zoom a todos los objetos
y [[tool.rectangle]] añade un rectángulo del tamaño de la página.

El botón **+** al final de la caja de herramientas enumera los grupos con una
casilla para mostrar u ocultar cada uno, y restablece la caja de herramientas.
La página [Caja de herramientas](../reference/toolbox.md) enumera todas las
herramientas.

## Paneles {#panels}

Los paneles contienen ajustes que se mantienen abiertos mientras trabaja, como
[[docker.properties]], [[docker.objects]] y [[docker.hints]], que están
abiertos de entrada. Los paneles abiertos son pestañas a lo largo del lado
derecho; haga clic en una pestaña para traer su panel al frente, o en la
pestaña del frente para plegar los paneles. Abra otros paneles desde
[[menu.window > menu.window.dockers]] o con sus atajos; el **+** bajo las
pestañas añade uno. Cada panel tiene botones de contraer y cerrar en su barra
de título. La página [Paneles](../reference/dockers.md) enumera todos los
paneles.

## La ventana de dibujo {#drawing-window}

La página está sobre el escritorio, la zona que la rodea, donde también puede
dejar objetos. Debajo del dibujo están:

- el **navegador de páginas**: insertar una página antes, primera página,
  página anterior, el número de página, página siguiente, última página e
  insertar una página después;
- una **pestaña por página** (haga clic para ir a una página, doble clic para
  cambiarle el nombre);
- la barra de desplazamiento horizontal. Arrastre el separador punteado entre
  las pestañas de página y la barra de desplazamiento para repartir el espacio
  entre ellas.

El botón **Navegador** en la esquina entre las barras de desplazamiento muestra
una miniatura de la página mientras lo mantiene pulsado; mueva el puntero sobre
la miniatura para mover la vista. Hacer doble clic en el borde de la página con
la herramienta [[tool.pick]] abre las opciones de tamaño de página.

## La paleta de colores {#color-palette}

Haga clic en un color de la paleta para rellenar los objetos seleccionados;
haga clic con el botón derecho para colorear su contorno. La paleta del
documento, bajo la ventana de dibujo, reúne los colores que usa su dibujo. Vea
[Paletas de colores](../color/palettes.md).

## La barra de estado {#status-bar}

La barra de estado muestra una sugerencia para la herramienta activa, lo que
está seleccionado y en qué capa, y el relleno y el contorno de la selección.
Haga doble clic en la parte del relleno o del contorno para cambiarlos. La
página [Barra de estado](../reference/status-bar.md) explica cada parte.

## Organizar las barras {#arranging}

Puede mover la caja de herramientas, la paleta de colores, la barra de
herramientas estándar, la barra de propiedades y la barra de estado: arrastre
una barra por su asa punteada a otro borde de la ventana, o lejos de los bordes
para que flote. Haga clic con el botón derecho en un asa para bloquear las
barras o restablecer sus posiciones. Para mostrar u ocultar una barra, use
[[menu.window > menu.window.toolbars]]. También puede guardar toda la
disposición como espacio de trabajo; vea
[Espacios de trabajo](../customize/workspaces.md).

## Trabajar con varios dibujos {#several-drawings}

Cada dibujo abierto tiene su pestaña encima de las reglas, con su nombre y un
`*` mientras tiene cambios sin guardar. La pestaña después de la última crea un
dibujo nuevo. Cada dibujo conserva sus propias páginas, historial de deshacer,
selección, zoom y unidades.

[[menu.window > menu.window.cascade]], [[menu.window.tile_horizontally]] y
[[menu.window.tile_vertically]] muestran cada dibujo en su propia ventana;
[[menu.window.combine_windows]] vuelve a las pestañas.
[[menu.window.new_window]] abre una segunda ventana del mismo dibujo, por
ejemplo para ver dos páginas o dos niveles de zoom a la vez; una edición en una
ventana se ve en todas.

## El tamaño de la interfaz {#interface-size}

Si los menús y los botones se ven demasiado pequeños o grandes, elija
[[menu.tools > menu.tools.options > menu.tools.options_customization]] y
defina el tamaño de la interfaz en la página [[options.appearance]]. El tamaño
de la interfaz nunca cambia el tamaño de su dibujo. Vea
[Opciones de personalización](../customize/customization.md#appearance).
