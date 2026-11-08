# Zarządzanie kolorami

Ekrany wyświetlają kolory za pomocą światła czerwonego, zielonego i niebieskiego;
drukarki mieszają farby cyjan, magenta, żółtą i czarną. Zarządzanie kolorami
używa **profili kolorów** do opisania każdego urządzenia, dzięki czemu kolor
zachowuje swój wygląd od ekranu do wydrukowanej strony. Ustawienia znajdują się
w [[menu.tools > menu.tools.color_management]], w oknie dialogowym
[[color_mgmt.title]]. Nic się nie zmienia, dopóki nie klikniesz
[[dialog.ok]].

![Okno dialogowe ustawień zarządzania kolorami](../../img/color-color-management.png)

## Profile rysunku {#document}

Karta [[color_mgmt.tab_document]] ustawia profile aktywnego rysunku.

- [[color_mgmt.current_profiles]] pokazuje profile RGB, CMYK i skali szarości,
  których rysunek obecnie używa.
- Wybierz sposób zastosowania nowego profilu:
  - [[color_mgmt.assign]]: wartości kolorów pozostają, a ich wygląd zmienia się
    zgodnie z nowym profilem;
  - [[color_mgmt.convert]]: kolory CMYK są przekształcane tak, aby w nowym
    profilu wyglądały tak samo.
- Wybierz profile [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]]
  i [[color_mgmt.grayscale]]. Każda lista zawiera profile domyślne, a następnie
  wszystkie profile znalezione na komputerze. [[color_mgmt.load_profiles]] dodaje
  pliki profili (`.icc`, `.icm`).
- [[color_mgmt.primary_mode]]: CMYK lub RGB.
- [[color_mgmt.intent]]: sposób dopasowania kolorów, których nie można
  wyświetlić: [[color_mgmt.intent_relative]] (domyślnie),
  [[color_mgmt.intent_perceptual]], [[color_mgmt.intent_saturation]] lub
  [[color_mgmt.intent_absolute]].

Profil CMYK jest zapisywany z rysunkiem, również w plikach `.cdr` od wersji 16.

## Ustawienia domyślne nowych rysunków {#default}

Karta [[color_mgmt.tab_default]] określa, czego używają nowe rysunki i jak
kolory są przekształcane.

- [[color_mgmt.preset]]: gotowe ustawienia dla Ameryki Północnej, Europy,
  Japonii lub pracy dla internetu i ekranu. [[color_mgmt.save_preset]] zapisuje
  własne ustawienia pod nazwą [[color_mgmt.preset_name]];
  [[color_mgmt.delete_preset]] usuwa ustawienie.
- Domyślne profile [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]]
  i [[color_mgmt.gray_profile]], [[color_mgmt.primary_mode]] oraz
  [[color_mgmt.intent]].
- [[color_mgmt.conversion]]:
  - [[color_mgmt.engine]]: [[color_mgmt.engine_icc]], który używa profili, lub
    [[color_mgmt.engine_builtin]];
  - [[color_mgmt.preserve_black]]: kolory złożone wyłącznie z czarnej farby po
    przekształceniu nadal składają się wyłącznie z czarnej farby;
  - [[color_mgmt.map_gray]] (domyślnie włączone): szarości przekształcone do
    CMYK używają wyłącznie czarnej farby;
  - [[dialog.black_point]] (domyślnie włączone): utrzymuje rozróżnienie
    najciemniejszych odcieni.
- [[color_mgmt.open_policy]]: co dzieje się z profilem otwieranego pliku, dla
  RGB, CMYK i skali szarości: [[color_mgmt.open_use_embedded]],
  [[color_mgmt.open_assign_default]] lub [[color_mgmt.open_convert_default]].
  [[color_mgmt.warn_mismatch_box]] i [[color_mgmt.warn_missing_box]] wyświetlają
  komunikat na pasku stanu, gdy plik ma inny profil lub nie ma go wcale.
- [[color_mgmt.import_policy]]: [[color_mgmt.import_convert]] lub
  [[color_mgmt.import_assign]], dla tego, co importujesz lub wklejasz.
- [[color_mgmt.spot_definition]]: czy kolory dodatkowe są definiowane przez
  [[color_mgmt.spot_lab]], [[color_mgmt.spot_cmyk]] czy
  [[color_mgmt.spot_rgb]].

## Podgląd kolorów wydruku {#proof}

[[menu.view > menu.view.proof_colors]] lub przycisk próby na prawym końcu paska
stanu pokazuje rysunek tak, jak zostałby wydrukowany: kolory, których farby
CMYK nie mogą wydrukować, wyglądają na bardziej matowe. Panel [[docker.color]]
również ostrzega przed takimi kolorami.

Bez wczytanych profili TraceDraw przekształca kolory według własnego modelu
farb procesowych na papierze powlekanym.
