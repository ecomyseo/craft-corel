# Správa barev

Obrazovky zobrazují barvy červeným, zeleným a modrým světlem; tiskárny míchají
azurovou, purpurovou, žlutou a černou barvu. Správa barev popisuje každé
zařízení pomocí **barevných profilů**, takže barva si zachová vzhled od
obrazovky až po vytištěnou stránku. Nastavení najdete v
[[menu.tools > menu.tools.color_management]], v dialogovém okně
[[color_mgmt.title]]. Nic se nezmění, dokud nekliknete na [[dialog.ok]].

![Dialogové okno Nastavení správy barev](../../img/color-color-management.png)

## Profily kresby {#document}

Karta [[color_mgmt.tab_document]] nastavuje profily aktivní kresby.

- [[color_mgmt.current_profiles]] ukazuje profily RGB, CMYK a stupňů šedi,
  které kresba právě používá.
- Zvolte, jak se nový profil použije:
  - [[color_mgmt.assign]]: hodnoty barev zůstanou a jejich vzhled se změní
    podle nového profilu;
  - [[color_mgmt.convert]]: barvy CMYK se převedou tak, aby v novém profilu
    vypadaly stejně.
- Zvolte profily [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]] a
  [[color_mgmt.grayscale]]. Každý seznam obsahuje výchozí profily a za nimi
  všechny profily nalezené v počítači. [[color_mgmt.load_profiles]] přidá
  soubory profilů (`.icc`, `.icm`).
- [[color_mgmt.primary_mode]]: CMYK nebo RGB.
- [[color_mgmt.intent]]: jak se převedou barvy, které nelze zobrazit:
  [[color_mgmt.intent_relative]] (výchozí), [[color_mgmt.intent_perceptual]],
  [[color_mgmt.intent_saturation]] nebo [[color_mgmt.intent_absolute]].

Profil CMYK se ukládá s kresbou, také do souborů `.cdr` od verze 16.

## Výchozí nastavení nových kreseb {#default}

Karta [[color_mgmt.tab_default]] nastavuje, co používají nové kresby a jak se
převádějí barvy.

- [[color_mgmt.preset]]: hotová nastavení pro Severní Ameriku, Evropu,
  Japonsko nebo práci pro web a obrazovku. [[color_mgmt.save_preset]] uloží
  vaše vlastní nastavení pod zadaným [[color_mgmt.preset_name]];
  [[color_mgmt.delete_preset]] předvolbu odebere.
- Výchozí [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]] a
  [[color_mgmt.gray_profile]], [[color_mgmt.primary_mode]] a
  [[color_mgmt.intent]].
- [[color_mgmt.conversion]]:
  - [[color_mgmt.engine]]: [[color_mgmt.engine_icc]], který používá profily,
    nebo [[color_mgmt.engine_builtin]];
  - [[color_mgmt.preserve_black]]: barvy tvořené jen černou barvou zůstanou
    po převodu jen černou;
  - [[color_mgmt.map_gray]] (výchozí zapnuto): šedé převedené do CMYK
    používají jen černou barvu;
  - [[dialog.black_point]] (výchozí zapnuto): udrží nejtmavší odstíny od
    sebe odlišené.
- [[color_mgmt.open_policy]]: co se stane s profilem otevíraného souboru,
  pro RGB, CMYK a stupně šedi: [[color_mgmt.open_use_embedded]],
  [[color_mgmt.open_assign_default]] nebo [[color_mgmt.open_convert_default]].
  [[color_mgmt.warn_mismatch_box]] a [[color_mgmt.warn_missing_box]] zobrazí
  zprávu ve stavovém řádku, když má soubor jiný profil nebo žádný.
- [[color_mgmt.import_policy]]: [[color_mgmt.import_convert]] nebo
  [[color_mgmt.import_assign]] pro to, co importujete nebo vkládáte.
- [[color_mgmt.spot_definition]]: zda jsou přímé barvy definovány pomocí
  [[color_mgmt.spot_lab]], [[color_mgmt.spot_cmyk]] nebo
  [[color_mgmt.spot_rgb]].

## Náhled tiskových barev {#proof}

[[menu.view > menu.view.proof_colors]] nebo tlačítko nátisku na pravém konci
stavového řádku zobrazí kresbu tak, jak by se vytiskla: barvy, které tiskové
barvy CMYK nedokážou vytisknout, vypadají matněji. Na takové barvy upozorňuje
také panel [[docker.color]].

Bez načtených profilů převádí TraceDraw barvy vlastním modelem procesních
barev na křídovém papíře.
