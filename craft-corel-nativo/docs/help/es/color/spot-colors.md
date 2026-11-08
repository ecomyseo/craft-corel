# Colores planos

Un **color plano** es una tinta con nombre que se imprime en una plancha
propia, como un color metálico o un color de marca, en lugar de mezclarse a
partir de cian, magenta, amarillo y negro. Cada dibujo guarda una lista de sus
tintas planas. Cada tinta tiene un sustituto hecho con colores de
cuatricromía, que se usa para mostrarla en pantalla y allí donde no se puede
usar la tinta.

## Añadir una tinta plana {#add}

1. Abra el panel [[docker.color]] y mezcle el color que debe tener la tinta.
2. En [[docker.spot_inks]], escriba un [[docker.ink_name]].
3. Haga clic en [[docker.new_ink]].

Los nombres son únicos dentro de un dibujo.

## Usar tintas planas {#use}

La lista [[docker.spot_inks]] muestra cada tinta con una muestra, su nombre y
los botones [[docker.fill]] y [[docker.outline]], que la aplican a la
selección con el [[docker.tint]] que indique (de 0 a 100%, 100% de forma
predeterminada). La cruz al final de una fila quita la tinta de la lista; los
objetos que la usan conservan su color sustituto.

## Libros de colores {#color-books}

Un archivo de paleta puede ser un libro de tintas planas (por ejemplo, una
paleta `.ase` con entradas de color plano). Sus entradas de color plano se
abren como colores planos, con los nombres del libro; al aplicar una, la tinta
se añade al dibujo. TraceDraw no incluye libros de colores con licencia, pero
uno que sea suyo se abre como cualquier paleta.

## Colores planos en la impresión y en las exportaciones {#output}

- En los archivos PDF, cada tinta se convierte en un color de separación con
  su nombre, de modo que la imprenta recibe la tinta.
- Cuando imprime separaciones, cada tinta recibe una plancha propia después de
  las planchas cian, magenta, amarilla y negra. Vea
  [Imprimir](../printing/print.md).
- En pantalla y en los archivos de mapa de bits, SVG y `.cdr`, un color plano
  se muestra como su sustituto con su tinte.
