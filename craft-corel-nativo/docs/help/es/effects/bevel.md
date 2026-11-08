# Biseles

Un bisel hace que los bordes de un objeto parezcan en relieve o tallados,
iluminando un lado del borde y sombreando el otro. Es un efecto dinámico, que
se ajusta en el panel [[docker.bevel]]; ábralo con
[[menu.effects > menu.effects.bevel]].

![El panel Bisel](../../img/effects-bevel.png)

## Para biselar un objeto {#create}

1. Seleccione un objeto con relleno.
2. En el panel [[docker.bevel]], elija el estilo:
   - [[docker.soft_edge]]: el borde desciende hacia el interior del objeto,
     iluminado por un lado y sombreado por el otro;
   - [[docker.emboss]]: el objeto parece estampado en relieve sobre la
     página.
3. Ajuste el aspecto del bisel:
   - [[docker.depth]]: hasta dónde llega el bisel desde el borde, hacia
     dentro del objeto en el borde suave y hacia fuera en el relieve;
   - [[docker.level]]: en cuántos pasos se dividen la luz y la sombra a lo
     largo del bisel. 1 da dos tonos planos; más pasos los degradan
     suavemente desde el borde hacia dentro;
   - [[docker.light_angle]]: la dirección de la que viene la luz;
   - [[docker.intensity]]: la fuerza de la luz y de la sombra;
   - [[docker.light_color]] y [[docker.shadow_color]]: los colores del lado
     iluminado y del lado sombreado;
   - [[docker.emboss_color]]: el color a partir del cual se tonifica el
     bisel. Marque la casilla y elija un color propio; desactivado, se usa
     el relleno del objeto.
4. Haga clic en [[docker.apply]].

Para copiar un bisel de otro objeto, use
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] lo quita. El panel de extrusión
también tiene un bisel para la cara frontal; vea
[Extrusiones](extrude.md#settings).
