# Effekte kopieren und klonen

Statt einen Effekt erneut einzurichten, können Sie ihn von einem Objekt
kopieren, das ihn bereits hat.

## Einen Effekt kopieren {#copy}

1. Wählen Sie die Objekte aus, die den Effekt erhalten sollen.
2. Wählen Sie [[menu.object > menu.object.copy_effect]] und die Art des
   Effekts: [[menu.object.copy_shadow_from]],
   [[menu.object.copy_transparency_from]], [[menu.object.copy_envelope_from]],
   [[menu.object.copy_perspective_from]], [[menu.object.copy_distortion_from]],
   [[menu.object.copy_extrude_from]], [[menu.object.copy_contour_from]],
   [[menu.object.copy_blend_from]], [[menu.object.copy_lens_from]],
   [[menu.object.copy_bevel_from]] oder
   [[menu.object.copy_block_shadow_from]].
3. Klicken Sie auf das Objekt, von dem kopiert werden soll. Ein Rechtsklick
   bricht ab.

Der Effekt wird an jedes Objekt angepasst: Eine Hülle oder Perspektive wird auf
seine Größe gestreckt, eine Verzerrung behält ihren Mittelpunkt an derselben
Stelle innerhalb des Objekts. Hat das angeklickte Objekt keinen solchen Effekt,
ändert sich nichts, und die Statusleiste weist darauf hin. Das Kopieren ist ein
einziger Rückgängig-Schritt.

## Einen Effekt klonen {#clone}

[[menu.object > menu.object.clone_effect]] kopiert den Schatten
([[menu.object.clone_shadow_from]]) oder die Transparenz
([[menu.object.clone_transparency_from]]) eines anderen Objekts und folgt ihm
weiterhin: Ändern Sie die Quelle, ändern sich auch die Klone, bis Sie den
eigenen Effekt eines Klons bearbeiten.

[[menu.edit > menu.edit.clone]] erstellt einen Klon eines ganzen Objekts, und
[[menu.edit.copy_properties_from]] kopiert die Füllung, den Umriss oder die
Texteigenschaften eines anderen Objekts.
