# Copiar y clonar efectos

En lugar de volver a configurar un efecto, puede copiarlo de un objeto que ya
lo tenga.

## Copiar un efecto {#copy}

1. Seleccione los objetos que deben recibir el efecto.
2. Elija [[menu.object > menu.object.copy_effect]] y el tipo de efecto:
   [[menu.object.copy_shadow_from]], [[menu.object.copy_transparency_from]],
   [[menu.object.copy_envelope_from]], [[menu.object.copy_perspective_from]],
   [[menu.object.copy_distortion_from]], [[menu.object.copy_extrude_from]],
   [[menu.object.copy_contour_from]], [[menu.object.copy_blend_from]],
   [[menu.object.copy_lens_from]], [[menu.object.copy_bevel_from]] o
   [[menu.object.copy_block_shadow_from]].
3. Haga clic en el objeto del que copiar. Un clic con el botón derecho
   cancela.

El efecto se adapta a cada objeto: una envoltura o una perspectiva se estira a
su tamaño, y una distorsión mantiene su centro en el mismo lugar dentro de él.
Cuando el objeto en el que hace clic no tiene ese efecto, nada cambia y la
barra de estado lo indica. La copia es un solo paso de deshacer.

## Clonar un efecto {#clone}

[[menu.object > menu.object.clone_effect]] copia la sombra
([[menu.object.clone_shadow_from]]) o la transparencia
([[menu.object.clone_transparency_from]]) de otro objeto, y lo sigue
siguiendo: cambie el original y los clones cambian también, hasta que edite el
efecto propio de un clon.

[[menu.edit > menu.edit.clone]] crea un clon de un objeto entero, y
[[menu.edit.copy_properties_from]] copia las propiedades de relleno, contorno
o texto de otro objeto.
