# Maskowanie kolorów w bitmapach

Panel [[docker.bitmap_mask]] ukrywa lub zachowuje piksele wybranych kolorów
bitmapy, na przykład aby białe tło stało się przezroczyste. Otwórz go poleceniem
[[menu.bitmaps > menu.bitmaps.bitmap_mask]].

![Panel Maska bitmapy](../../img/bitmaps-bitmap-mask.png)

## Maskowanie kolorów {#mask}

1. Zaznacz bitmapę.
2. Wybierz kolory do dziesięciu gniazd kolorów:
   - kliknij kroplomierz i klikaj kolory w bitmapie; każdy z nich wypełnia
     zaznaczone gniazdo i je zaznacza;
   - albo kliknij **Wybierz kolory z bitmapy**, aby wypełnić gniazda dziesięcioma
     kolorami najczęściej używanymi w bitmapie.
3. Zaznacz gniazdo i ustaw jego **tolerancję** (od 0 do 100%): jak bardzo kolor
   piksela może się różnić od koloru gniazda, aby nadal był dopasowany.
4. Wybierz **Ukryj zaznaczone** (dopasowane piksele stają się przezroczyste) lub
   **Pokaż zaznaczone** (pozostają tylko dopasowane piksele, domyślnie).
5. Kliknij [[docker.apply]].

**Zapisz maskę** i **Otwórz maskę** przechowują gniazda w pliku `.msk`;
**Usuń** opróżnia zaznaczone gniazdo. Każda bitmapa zmienia się w jednym kroku
cofania.
