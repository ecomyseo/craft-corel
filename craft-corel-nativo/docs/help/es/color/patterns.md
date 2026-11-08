# Rellenos de patrón

Un relleno de patrón repite un mosaico por el interior de un objeto. El
mosaico puede ser de dos colores, un conjunto de objetos vectoriales (un
patrón vectorial a todo color) o un mapa de bits. Los mosaicos están anclados
a la página, así que no se deslizan cuando mueve el objeto.

## Crear un patrón a partir de sus objetos {#create-pattern}

1. Dibuje los objetos que forman un mosaico y selecciónelos.
2. Elija [[menu.object > menu.object.create > menu.object.create_pattern]].
3. En el cuadro de diálogo [[dialog.create_pattern]], elija el
   [[dialog.create_pattern_type]]: [[dialog.create_pattern_vector]] conserva
   los objetos como vectores, nítidos a cualquier tamaño;
   [[dialog.create_pattern_bitmap]] crea una imagen de ellos.
4. Haga clic en [[dialog.ok]].

![El cuadro de diálogo Crear patrón](../../img/color-patterns-create-pattern.png)

El patrón pasa a ser el relleno de los objetos nuevos. Para dárselo a objetos
que ya ha dibujado, elija [[docker.fill_pattern]] en la sección de relleno
del panel [[docker.properties]].

## Ajustes del patrón {#settings}

En la sección de relleno del panel [[docker.properties]], elija
[[docker.fill_pattern]] y luego el tipo de patrón y sus ajustes. Para un
patrón vectorial, el panel muestra cuántos objetos contiene el mosaico, y su
anchura y altura: al cambiarlas se escala con ellas el contenido del mosaico.

## Patrones en archivos exportados {#export}

Los archivos SVG conservan los patrones vectoriales como vectores. Los
archivos PDF y EPS los reciben como imágenes a la resolución de exportación,
recortadas al objeto.
