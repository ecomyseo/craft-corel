# Style kolorów

**Styl koloru** to nazwany kolor używany przez obiekty rysunku. Gdy zmienisz
styl, zmienia się też każdy obiekt nim wypełniony, co utrzymuje spójność kolorów
projektu i pozwala szybko wypróbować inne schematy kolorów. Style są wymienione
w panelu [[docker.color_styles]]; otwórz go z menu
[[menu.window > menu.window.dockers]].

![Panel Style kolorów](../../img/color-color-styles.png)

## Tworzenie stylów kolorów {#create}

- [[docker.new_from_selection]] dodaje styl dla każdego koloru wypełnienia
  zaznaczonych obiektów, który nie jest jeszcze stylem.
- [[docker.new_color_style]] dodaje styl z bieżącego koloru panelu
  [[docker.color]].

## Używanie i edytowanie stylów {#edit}

Każdy styl pokazuje swój kolor, nazwę i trzy przyciski:

- kliknij kolor, aby go zmienić: obiekty wypełnione tym kolorem podążają za
  zmianą;
- kliknij nazwę, aby ją zmienić;
- [[docker.fill]] wypełnia zaznaczenie stylem, [[docker.outline]] koloruje jego
  kontur;
- **x** usuwa styl (obiekty zachowują swój kolor).

## Harmonie {#harmonies}

[[docker.harmony_editor]] tworzy zestaw pasujących do siebie stylów na podstawie
koloru [[docker.base_color]]:

1. Wybierz [[docker.base_color]].
2. Kliknij harmonię: [[harmony.analogous]], [[harmony.complementary]],
   [[harmony.split_complementary]], [[harmony.triadic]],
   [[harmony.tetradic]] lub [[harmony.monochromatic]].

Kolory harmonii są dodawane jako style nazwane od niej.
