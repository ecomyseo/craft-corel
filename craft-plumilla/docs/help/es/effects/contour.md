# Siluetas

Una silueta dibuja copias del contorno de un objeto espaciadas de manera
uniforme, dentro de él, fuera de él o hasta su centro, como las líneas de un
mapa. Es un efecto dinámico: cambie el objeto y la silueta lo sigue.

## Para añadir una silueta {#create}

Con la herramienta:

1. Elija la herramienta [[tool.contour]] en el menú desplegable Efectos.
2. Arrastre desde un objeto: alejándose de su centro para una silueta
   exterior, hacia él para una interior. Cuanto más arrastre, mayor será el
   espaciado.

Con el panel:

1. Seleccione un objeto.
2. Abra el panel [[docker.contour]] con
   [[menu.effects > menu.effects.contour]] (<kbd>Ctrl+F9</kbd>).
3. Ajuste las opciones y haga clic en [[docker.apply]].

![El panel Silueta](../../img/effects-contour.png)

Los grupos y los mapas de bits no admiten silueta.

## Ajustes de la silueta {#settings}

La barra de propiedades y el panel muestran los ajustes de la silueta
seleccionada y la cambian de inmediato.

- **Al centro**, **Interior** o **Exterior** (opción predeterminada).
- **Pasos**: 3 de forma predeterminada, hasta 999. Al centro crea tantos como
  quepan.
- **Desplazamiento**: la distancia entre pasos (2 mm de forma
  predeterminada).
- **Esquinas**: en inglete (agudas, opción predeterminada), redondeadas o
  biseladas.
- **Dirección del color**: directa, en sentido horario o antihorario
  alrededor de la rueda de colores, de los colores del objeto a los del
  último paso.
- Los **colores de relleno y contorno del último paso** (blanco y negro de
  forma predeterminada). Un objeto sin relleno da pasos sin relleno.
- **Aceleración de objetos** y **aceleración de color** (de -100 a 100%)
  agolpan los pasos o los colores hacia el objeto o hacia el último paso.

Los pasos exteriores se dibujan detrás del objeto, los interiores delante.

**Borrar silueta** quita la silueta; **Separar** convierte los pasos en
objetos.
Los pasos salen agrupados junto al objeto. [[menu.object > menu.object.break_contour_apart]] (<kbd>Ctrl+K</kbd>, también en
el menú del botón derecho) hace lo mismo.
