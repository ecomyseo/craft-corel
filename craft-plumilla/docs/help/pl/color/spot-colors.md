# Kolory dodatkowe

**Kolor dodatkowy** to nazwana farba drukowana na osobnej płycie, na przykład
farba metaliczna lub kolor marki, zamiast mieszana z cyjanu, magenty, żółci
i czerni. Każdy rysunek przechowuje listę swoich kolorów dodatkowych. Każda farba
ma zamiennik złożony z kolorów procesowych, używany do wyświetlania jej na
ekranie i wszędzie tam, gdzie nie można użyć tej farby.

## Dodawanie koloru dodatkowego {#add}

1. Otwórz panel [[docker.color]] i wymieszaj kolor, jak ma wyglądać farba.
2. W sekcji [[docker.spot_inks]] wpisz nazwę w polu [[docker.ink_name]].
3. Kliknij [[docker.new_ink]].

Nazwy są unikatowe w obrębie rysunku.

## Używanie kolorów dodatkowych {#use}

Lista [[docker.spot_inks]] pokazuje każdą farbę z próbką, jej nazwą oraz
przyciskami [[docker.fill]] i [[docker.outline]], które stosują ją do
zaznaczenia z ustawionym odcieniem [[docker.tint]] (od 0 do 100%, domyślnie
100%). Krzyżyk na końcu wiersza usuwa farbę z listy; obiekty, które jej używają,
zachowują jej kolor zamiennika.

## Wzorniki kolorów {#color-books}

Plik palety może być wzornikiem kolorów dodatkowych (na przykład paleta `.ase`
z pozycjami kolorów dodatkowych). Jego pozycje kolorów dodatkowych otwierają się
jako kolory dodatkowe, nazwane tak jak we wzorniku; zastosowanie jednej z nich
dodaje farbę do rysunku. TraceDraw nie dostarcza licencjonowanych wzorników
kolorów, ale posiadany wzornik otwiera się jak każda paleta.

## Kolory dodatkowe w druku i eksporcie {#output}

- W plikach PDF każda farba staje się kolorem separacji nazwanym od niej, dzięki
  czemu drukarnia otrzymuje tę farbę.
- Gdy drukujesz separacje, każda farba otrzymuje własną płytę po płytach cyjanu,
  magenty, żółci i czerni. Zobacz [Drukowanie](../printing/print.md).
- Na ekranie oraz w plikach bitmapowych, SVG i `.cdr` kolor dodatkowy jest
  widoczny jako jego zamiennik w danym odcieniu.
