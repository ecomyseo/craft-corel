# Wypełnienia wzorem

Wypełnienie wzorem powtarza kafelek we wnętrzu obiektu. Kafelkiem mogą być dwa
kolory, zestaw obiektów wektorowych (pełnokolorowy wzór wektorowy) lub bitmapa.
Kafelki są zakotwiczone do strony, więc nie przesuwają się, gdy przesuwasz
obiekt.

## Tworzenie wzoru z własnych obiektów {#create-pattern}

1. Narysuj obiekty, które tworzą jeden kafelek, i zaznacz je.
2. Wybierz [[menu.object > menu.object.create > menu.object.create_pattern]].
3. W oknie dialogowym [[dialog.create_pattern]] wybierz
   [[dialog.create_pattern_type]]: [[dialog.create_pattern_vector]] zachowuje
   obiekty jako wektory, ostre w każdym rozmiarze;
   [[dialog.create_pattern_bitmap]] tworzy z nich obraz.
4. Kliknij [[dialog.ok]].

![Okno dialogowe Utwórz wzór](../../img/color-patterns-create-pattern.png)

Wzór staje się wypełnieniem nowych obiektów. Aby nadać go obiektom już
narysowanym, wybierz [[docker.fill_pattern]] w sekcji wypełnienia panelu
[[docker.properties]].

## Ustawienia wzoru {#settings}

W sekcji wypełnienia panelu [[docker.properties]] wybierz
[[docker.fill_pattern]], a następnie rodzaj wzoru i jego ustawienia. Dla wzoru
wektorowego panel pokazuje, ile obiektów zawiera kafelek, oraz jego szerokość
i wysokość: ich zmiana skaluje razem z nimi zawartość kafelka.

## Wzory w eksportowanych plikach {#export}

Pliki SVG zachowują wzory wektorowe jako wektory. Pliki PDF i EPS otrzymują je
jako obrazy w rozdzielczości eksportu, przycięte do obiektu.
