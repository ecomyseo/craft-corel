# Maskování barev v bitmapách

Panel [[docker.bitmap_mask]] skryje nebo ponechá v bitmapě pixely zvolených
barev, například aby se bílé pozadí stalo průhledným. Otevřete jej příkazem
[[menu.bitmaps > menu.bitmaps.bitmap_mask]].

![Panel Bitmapová maska](../../img/bitmaps-bitmap-mask.png)

## Maskování barev {#mask}

1. Vyberte bitmapu.
2. Naberte barvy do deseti barevných polí:
   - klikněte na kapátko a pak na barvy v bitmapě; každá vyplní vybrané pole
     a zaškrtne je;
   - nebo klikněte na **Vybrat barvy z bitmapy** a pole se vyplní deseti
     barvami, které bitmapa používá nejvíce.
3. Vyberte pole a nastavte jeho **toleranci** (0 až 100 %): jak daleko smí
   být barva pixelu od barvy pole, aby se stále shodovala.
4. Zvolte **Skrýt vybrané** (shodné pixely se stanou průhlednými) nebo
   **Zobrazit vybrané** (zůstanou jen shodné pixely, výchozí stav).
5. Klikněte na [[docker.apply]].

**Uložit masku** a **Otevřít masku** uchovávají pole v souboru `.msk`;
**Odstranit** vyprázdní vybrané pole. Každá bitmapa se změní v jednom kroku
zpět.
