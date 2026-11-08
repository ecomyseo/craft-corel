# Vzorkové výplně

Vzorková výplň opakuje dlaždici po celém vnitřku objektu. Dlaždice může být
dvoubarevná, sada vektorových objektů (plnobarevný vektorový vzorek) nebo
bitmapa. Dlaždice jsou ukotveny ke stránce, takže se při posunu objektu
neposouvají.

## Vytvoření vzorku z vlastních objektů {#create-pattern}

1. Nakreslete objekty, které tvoří jednu dlaždici, a vyberte je.
2. Zvolte [[menu.object > menu.object.create > menu.object.create_pattern]].
3. V dialogovém okně [[dialog.create_pattern]] zvolte
   [[dialog.create_pattern_type]]: [[dialog.create_pattern_vector]] ponechá
   objekty jako vektory, ostré v jakékoli velikosti;
   [[dialog.create_pattern_bitmap]] z nich vytvoří obrázek.
4. Klikněte na [[dialog.ok]].

![Dialogové okno Vytvořit vzorek](../../img/color-patterns-create-pattern.png)

Vzorek se stane výplní nových objektů. Chcete-li jej dát již nakresleným
objektům, zvolte [[docker.fill_pattern]] v oddílu výplně panelu
[[docker.properties]].

## Nastavení vzorku {#settings}

V oddílu výplně panelu [[docker.properties]] zvolte [[docker.fill_pattern]],
poté druh vzorku a jeho nastavení. U vektorového vzorku panel ukazuje, kolik
objektů dlaždice obsahuje, a její šířku a výšku: jejich změnou se spolu s
nimi změní velikost obsahu dlaždice.

## Vzorky v exportovaných souborech {#export}

Soubory SVG uchovávají vektorové vzorky jako vektory. Soubory PDF a EPS je
dostanou jako obrázky v rozlišení exportu, oříznuté podle objektu.
