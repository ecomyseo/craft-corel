# Remuestrear, enderezar e inflar mapas de bits

## Remuestrear {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] da a los mapas de bits seleccionados
un nuevo número de píxeles; su tamaño en la página se mantiene.

![El cuadro de diálogo Remuestrear](../../img/bitmaps-resample-and-straighten-resample.png)

1. Escriba el nuevo tamaño en **píxeles**, como **porcentaje** o como la
   **resolución** que da a su tamaño en la página. Con **Mantener relación de
   aspecto** activado, la altura sigue a la anchura.
2. Elija el método:
   - **Suave** (fotografías), opción predeterminada;
   - **Nítido**, algo más definido;
   - **Píxeles duros** (pixel art): cada píxel sigue siendo un cuadrado de un
     solo color;
   - **Ampliar con detalle**: suave, y luego enfocado cuando la imagen crece.
3. Haga clic en [[dialog.ok]].

El cuadro de diálogo muestra los tamaños original y nuevo y la memoria que
ocupa la nueva imagen. Con varios mapas de bits seleccionados, cada uno se
escala por el mismo factor.

## Enderezar imagen {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] corrige fotografías
inclinadas, vistas en ángulo o curvadas por el objetivo.

![El cuadro de diálogo Enderezar imagen](../../img/bitmaps-resample-and-straighten-straighten.png)

1. Use la **Cuadrícula** (su tamaño y color se pueden ajustar) para alinear la
   imagen.
2. Ajuste los deslizadores: **Corregir distorsión del objetivo** (de -100 a
   100), **Girar imagen** (de -15 a 15 grados), **Perspectiva vertical** y
   **Perspectiva horizontal** (de -100 a 100). Los botones de giro dan antes
   un cuarto de vuelta a la imagen.
3. Con **Recortar imagen** activado, el resultado se recorta al rectángulo más
   grande con las proporciones originales; **Recortar y remuestrear al tamaño
   original** lo devuelve al número de píxeles original. Sin recortar, las
   esquinas quedan transparentes.
4. Haga clic en [[dialog.ok]]. **Restablecer** devuelve todos los ajustes a su
   valor inicial.

Haga zoom y desplace la vista previa con las herramientas que hay encima. El
resultado sustituye los píxeles, así que los efectos del mapa de bits se
vuelven permanentes.

## Inflar mapas de bits {#inflate}

Algunos efectos, como los desenfoques, se extienden más allá de los bordes de
un mapa de bits.

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]]
  (activado de forma predeterminada) agranda los mapas de bits lo que
  necesiten sus efectos, con un margen transparente.
- [[menu.bitmaps.inflate_manual]] agranda a mano los mapas de bits
  seleccionados: para la anchura y la altura, escriba el nuevo tamaño en
  píxeles o un porcentaje (de 100 a 1000%), con **Mantener relación de
  aspecto**. La imagen queda centrada.

![El cuadro de diálogo Inflar mapa de bits manualmente](../../img/bitmaps-resample-and-straighten-inflate.png)

## Otros comandos de mapa de bits {#other}

- [[menu.bitmaps.crop_bitmap]] recorta un mapa de bits con la herramienta
  [[tool.crop]]; arrastrar una esquina de un mapa de bits con la herramienta
  [[tool.shape]] también lo recorta.
- [[menu.bitmaps.edit_bitmap]] abre el mapa de bits en otro programa.
- [[menu.bitmaps.remove_background]] hace transparente el fondo de una
  fotografía.
- [[menu.bitmaps.remove_background_local]]: [Quitar fondo](remove-background.md).
- [[menu.bitmaps.break_link]] y [[menu.bitmaps.update_from_link]] actúan
  sobre mapas de bits vinculados a un archivo.
- [[menu.bitmaps.save_bitmap_as]] guarda un mapa de bits como archivo PNG.
