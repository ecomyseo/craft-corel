# Distorsión

La herramienta [[tool.distort]] curva el contorno de un objeto de tres
maneras: **empujar y tirar** (puntas o pétalos), **cremallera** (dientes en
zigzag) y **torbellino** (un remolino). Es un efecto dinámico: puede cambiar,
mover o quitar la distorsión en cualquier momento, y el objeto conserva su
propio contorno.

## Para distorsionar un objeto {#create}

1. Elija la herramienta [[tool.distort]] en el menú desplegable Efectos.
2. Elija el tipo en la barra de propiedades.
3. Arrastre sobre el objeto. La distorsión se centra donde empieza.
   - **Empujar y tirar**: arrastre a la derecha para empujar los nodos hacia
     fuera (formas puntiagudas de lados cóncavos), a la izquierda para tirar
     de ellos hacia dentro (pétalos redondeados).
   - **Cremallera**: cuanto más arrastre, más altos serán los dientes.
   - **Torbellino**: arrastre alrededor del punto de partida; las vueltas
     completas cuentan.

El centro de una distorsión seleccionada se muestra como un rombo: arrástrelo
para mover el centro. Arrastrar de nuevo sustituye la distorsión; un objeto
tiene como máximo una. Los grupos y los mapas de bits no se distorsionan.

## Ajustes de la distorsión {#settings}

La barra de propiedades muestra y cambia la distorsión seleccionada:

- el tipo: **Empujar y tirar**, **Cremallera** o **Torbellino** (elegir otro
  tipo cambia una distorsión seleccionada);
- para empujar y tirar, la **amplitud** (de -200 a 200, 20 de forma
  predeterminada);
- para cremallera, la **amplitud** (de 0 a 100 mm, 2 mm de forma
  predeterminada) y la **frecuencia** (dientes por segmento, 6 de forma
  predeterminada), con **Aleatoria** (dientes desiguales), **Suave** (dientes
  redondeados) y **Local** (los dientes se reducen al alejarse del centro);
- para torbellino, **horario** o **antihorario**, el número de **rotaciones**
  completas y los **grados** adicionales.

[[menu.object > menu.object.clear_effect]] quita la distorsión.
