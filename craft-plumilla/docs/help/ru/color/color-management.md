# Управление цветом

Экраны показывают цвета красным, зелёным и синим светом; принтеры смешивают
голубую, пурпурную, жёлтую и чёрную краски. Управление цветом использует
**цветовые профили**, описывающие каждое устройство, чтобы цвет выглядел
одинаково на экране и на напечатанной странице. Параметры находятся в
[[menu.tools > menu.tools.color_management]], в диалоговом окне
[[color_mgmt.title]]. Ничего не меняется, пока вы не нажмёте [[dialog.ok]].

![Диалоговое окно параметров управления цветом](../../img/color-color-management.png)

## Профили рисунка {#document}

Вкладка [[color_mgmt.tab_document]] задаёт профили активного рисунка.

- [[color_mgmt.current_profiles]] показывает профили RGB, CMYK и оттенков
  серого, которые рисунок использует сейчас.
- Выберите, как применяется новый профиль:
  - [[color_mgmt.assign]]: значения цветов сохраняются, а их вид меняется под
    новый профиль;
  - [[color_mgmt.convert]]: цвета CMYK преобразуются так, чтобы в новом
    профиле выглядеть так же.
- Выберите профили [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]] и
  [[color_mgmt.grayscale]]. В каждом списке сначала профили по умолчанию,
  затем все профили, найденные на компьютере. [[color_mgmt.load_profiles]]
  добавляет файлы профилей (`.icc`, `.icm`).
- [[color_mgmt.primary_mode]]: CMYK или RGB.
- [[color_mgmt.intent]]: как приводятся цвета, которые нельзя показать:
  [[color_mgmt.intent_relative]] (по умолчанию),
  [[color_mgmt.intent_perceptual]], [[color_mgmt.intent_saturation]] или
  [[color_mgmt.intent_absolute]].

Профиль CMYK хранится вместе с рисунком, в том числе в файлах `.cdr` начиная с
версии 16.

## Значения по умолчанию для новых рисунков {#default}

Вкладка [[color_mgmt.tab_default]] задаёт, что используют новые рисунки и как
преобразуются цвета.

- [[color_mgmt.preset]]: готовые параметры для Северной Америки, Европы,
  Японии или для работы для веба и экрана. [[color_mgmt.save_preset]]
  сохраняет ваши параметры под именем [[color_mgmt.preset_name]];
  [[color_mgmt.delete_preset]] удаляет набор.
- Профили по умолчанию [[color_mgmt.rgb_profile]],
  [[color_mgmt.cmyk_profile]] и [[color_mgmt.gray_profile]],
  [[color_mgmt.primary_mode]] и [[color_mgmt.intent]].
- [[color_mgmt.conversion]]:
  - [[color_mgmt.engine]]: [[color_mgmt.engine_icc]], который использует
    профили, или [[color_mgmt.engine_builtin]];
  - [[color_mgmt.preserve_black]]: цвета только из чёрной краски при
    преобразовании остаются только чёрной краской;
  - [[color_mgmt.map_gray]] (по умолчанию включено): серые цвета,
    преобразованные в CMYK, используют только чёрную краску;
  - [[dialog.black_point]] (по умолчанию включено): самые тёмные оттенки
    остаются различимыми.
- [[color_mgmt.open_policy]]: что происходит с профилем открываемого файла,
  для RGB, CMYK и оттенков серого: [[color_mgmt.open_use_embedded]],
  [[color_mgmt.open_assign_default]] или [[color_mgmt.open_convert_default]].
  [[color_mgmt.warn_mismatch_box]] и [[color_mgmt.warn_missing_box]]
  показывают сообщение в строке состояния, когда у файла другой профиль или
  его нет.
- [[color_mgmt.import_policy]]: [[color_mgmt.import_convert]] или
  [[color_mgmt.import_assign]], для импортируемого или вставляемого.
- [[color_mgmt.spot_definition]]: определяются ли плашечные цвета через
  [[color_mgmt.spot_lab]], [[color_mgmt.spot_cmyk]] или
  [[color_mgmt.spot_rgb]].

## Предпросмотр цветов печати {#proof}

[[menu.view > menu.view.proof_colors]] или кнопка цветопробы у правого края
строки состояния показывает рисунок таким, каким он будет напечатан: цвета,
которые нельзя напечатать красками CMYK, выглядят тусклее. Панель
[[docker.color]] тоже предупреждает о таких цветах.

Без загруженных профилей TraceDraw преобразует цвета по собственной модели
триадных красок на мелованной бумаге.
