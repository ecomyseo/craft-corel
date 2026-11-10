# Převzorkování, narovnání a rozšíření bitmap

## Převzorkování {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] dá vybraným bitmapám nový počet
pixelů; jejich velikost na stránce zůstane stejná.

![Dialogové okno Převzorkovat](../../img/bitmaps-resample-and-straighten-resample.png)

1. Zadejte novou velikost v **pixelech**, v **procentech** nebo jako
   **rozlišení**, které dává při své velikosti na stránce. Se zapnutou volbou
   **Zachovat poměr stran** se výška řídí šířkou.
2. Zvolte metodu:
   - **Hladké** (fotografie), výchozí;
   - **Ostré**, o něco ostřejší;
   - **Tvrdé pixely** (pixel art): každý pixel zůstane čtvercem jedné barvy;
   - **Zvětšit s detaily**: vyhlazení a poté doostření, když se obrázek
     zvětšuje.
3. Klikněte na [[dialog.ok]].

Dialogové okno ukazuje původní a novou velikost a paměť, kterou nový obrázek
zabere. Je-li vybráno více bitmap, každá se změní stejným poměrem.

## Narovnat obrázek {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] opravuje fotografie, které
jsou nakloněné, snímané pod úhlem nebo zakřivené objektivem.

![Dialogové okno Narovnat obrázek](../../img/bitmaps-resample-and-straighten-straighten.png)

1. Obrázek srovnejte podle **mřížky** (její velikost a barvu lze nastavit).
2. Nastavte posuvníky: **Opravit zkreslení objektivu** (-100 až 100),
   **Otočit obrázek** (-15 až 15 stupňů), **Svislá perspektiva** a
   **Vodorovná perspektiva** (-100 až 100). Tlačítka otočení nejprve otočí
   obrázek o čtvrt otáčky.
3. Se zapnutou volbou **Oříznout obrázek** se výsledek ořízne na největší
   obdélník s původními proporcemi; **Oříznout a převzorkovat na původní
   velikost** mu vrátí původní počet pixelů. Bez oříznutí jsou rohy
   průhledné.
4. Klikněte na [[dialog.ok]]. **Obnovit** vrátí všechna nastavení.

Náhled lze přibližovat a posouvat nástroji nad ním. Výsledek nahradí pixely,
takže efekty na bitmapě se stanou trvalými.

## Rozšíření bitmap {#inflate}

Některé efekty, například rozostření, přesahují okraje bitmapy.

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]]
  (výchozí zapnuto) zvětší bitmapy o to, co jejich efekty potřebují, s
  průhledným okrajem.
- [[menu.bitmaps.inflate_manual]] zvětší vybrané bitmapy ručně: pro šířku a
  výšku zadejte novou velikost v pixelech nebo v procentech (100 až 1000 %),
  s volbou **Zachovat poměr stran**. Obrázek zůstane na středu.

![Dialogové okno Rozšířit bitmapu ručně](../../img/bitmaps-resample-and-straighten-inflate.png)

## Další příkazy pro bitmapy {#other}

- [[menu.bitmaps.crop_bitmap]] ořízne bitmapu nástrojem [[tool.crop]];
  oříznout ji lze také tažením rohu bitmapy nástrojem [[tool.shape]].
- [[menu.bitmaps.edit_bitmap]] otevře bitmapu v jiném programu.
- [[menu.bitmaps.remove_background]] zprůhlední pozadí fotografie.
- [[menu.bitmaps.remove_background_local]]: [Odstranit pozadí](remove-background.md).
- [[menu.bitmaps.break_link]] a [[menu.bitmaps.update_from_link]] pracují s
  bitmapami propojenými se souborem.
- [[menu.bitmaps.save_bitmap_as]] uloží bitmapu jako soubor PNG.
