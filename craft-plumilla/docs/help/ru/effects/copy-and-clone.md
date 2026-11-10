# Копирование и клонирование эффектов

Вместо того чтобы снова настраивать эффект, его можно скопировать с объекта,
у которого он уже есть.

## Копирование эффекта {#copy}

1. Выделите объекты, которые должны получить эффект.
2. Выберите [[menu.object > menu.object.copy_effect]] и вид эффекта:
   [[menu.object.copy_shadow_from]], [[menu.object.copy_transparency_from]],
   [[menu.object.copy_envelope_from]], [[menu.object.copy_perspective_from]],
   [[menu.object.copy_distortion_from]], [[menu.object.copy_extrude_from]],
   [[menu.object.copy_contour_from]], [[menu.object.copy_blend_from]],
   [[menu.object.copy_lens_from]], [[menu.object.copy_bevel_from]] или
   [[menu.object.copy_block_shadow_from]].
3. Щёлкните объект, с которого копируете. Щелчок правой кнопкой отменяет.

Эффект подгоняется под каждый объект: оболочка или перспектива растягивается
до его размера, искажение сохраняет свой центр в том же месте внутри объекта.
Если у щёлкнутого объекта такого эффекта нет, ничего не меняется, и строка
состояния сообщает об этом. Копирование является одним шагом отмены.

## Клонирование эффекта {#clone}

[[menu.object > menu.object.clone_effect]] копирует тень
([[menu.object.clone_shadow_from]]) или прозрачность
([[menu.object.clone_transparency_from]]) другого объекта и продолжает
следовать за ним: измените источник, и клоны тоже изменятся, пока вы не
отредактируете собственный эффект клона.

[[menu.edit > menu.edit.clone]] создаёт клон целого объекта, а
[[menu.edit.copy_properties_from]] копирует свойства заливки, абриса или
текста другого объекта.
