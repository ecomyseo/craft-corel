# El panel Efectos

Un objeto puede llevar varios efectos dinámicos a la vez, aplicados uno tras
otro. El panel [[docker.effects]] enumera los efectos del objeto seleccionado
en el orden en que se aplican, de arriba abajo. Ábralo con
[[menu.window > menu.window.dockers > menu.window.dockers_effects]].

![El panel Efectos](../../img/effects-effects-docker.png)

## Gestionar los efectos {#stack}

Cada fila de la lista tiene:

- una casilla que activa o desactiva el efecto. Un efecto desactivado
  conserva su lugar y sus ajustes, pero no se dibuja, ni se exporta, ni se
  imprime;
- el nombre del efecto, atenuado mientras está desactivado;
- flechas arriba y abajo que adelantan o retrasan el efecto, lo que cambia el
  resultado (por ejemplo, una silueta de una envoltura, o una envoltura de una
  silueta);
- una cruz que quita el efecto.

Cada cambio es un paso de deshacer. Cada efecto se sigue editando con su
propia herramienta o panel.

## Otros comandos de efectos {#commands}

- [[menu.effects > menu.effects.flatten]] convierte los efectos de los objetos
  seleccionados en objetos normales.
- [[menu.effects.picker]] muestra una lista de efectos que aplicar a la
  selección.
- [[menu.object > menu.object.clear_effect]] quita el efecto de la selección.
- [[menu.object > menu.object.copy_effect]] copia un efecto de otro objeto, y
  [[menu.object.clone_effect]] crea un clon que lo sigue; vea
  [Copiar y clonar efectos](copy-and-clone.md).
