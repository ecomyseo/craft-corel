# Zmiana rozdzielczości, prostowanie i rozszerzanie bitmap

## Zmiana rozdzielczości {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] nadaje zaznaczonym bitmapom nową liczbę
pikseli; ich rozmiar na stronie się nie zmienia.

![Okno dialogowe Zmień rozdzielczość](../../img/bitmaps-resample-and-straighten-resample.png)

1. Wpisz nowy rozmiar w **pikselach**, w **procentach** lub jako
   **rozdzielczość**, jaką bitmapa uzyska przy swoim rozmiarze na stronie. Gdy
   opcja **Zachowaj proporcje** jest włączona, wysokość podąża za szerokością.
2. Wybierz metodę:
   - **Gładka** (zdjęcia), domyślnie;
   - **Ostra**, nieco wyraźniejsza;
   - **Twarde piksele** (pixel art): każdy piksel pozostaje kwadratem jednego
     koloru;
   - **Powiększenie ze szczegółami**: wygładzanie, a następnie wyostrzanie, gdy
     obraz się powiększa.
3. Kliknij [[dialog.ok]].

Okno dialogowe pokazuje rozmiar pierwotny i nowy oraz pamięć zajmowaną przez
nowy obraz. Gdy zaznaczonych jest kilka bitmap, każda jest skalowana o ten sam
współczynnik.

## Wyprostuj obraz {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] koryguje zdjęcia przechylone,
widziane pod kątem lub zniekształcone przez obiektyw.

![Okno dialogowe Wyprostuj obraz](../../img/bitmaps-resample-and-straighten-straighten.png)

1. Użyj **Siatki** (jej rozmiar i kolor można zmieniać), aby wyrównać obraz.
2. Ustaw suwaki: **Koryguj zniekształcenie obiektywu** (od -100 do 100),
   **Obróć obraz** (od -15 do 15 stopni), **Perspektywa pionowa**
   i **Perspektywa pozioma** (od -100 do 100). Przyciski obrotu najpierw
   obracają obraz o ćwierć obrotu.
3. Gdy opcja **Kadruj obraz** jest włączona, wynik jest kadrowany do
   największego prostokąta o pierwotnych proporcjach; **Kadruj i przywróć
   pierwotny rozmiar** przywraca pierwotną liczbę pikseli. Bez kadrowania
   narożniki są przezroczyste.
4. Kliknij [[dialog.ok]]. **Resetuj** przywraca wszystkie ustawienia.

Powiększaj i przesuwaj podgląd narzędziami nad nim. Wynik zastępuje piksele,
więc efekty bitmapy stają się trwałe.

## Rozszerzanie bitmap {#inflate}

Niektóre efekty, takie jak rozmycia, wychodzą poza krawędzie bitmapy.

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]]
  (domyślnie włączone) powiększa bitmapy o tyle, ile potrzebują ich efekty,
  z przezroczystym marginesem.
- [[menu.bitmaps.inflate_manual]] powiększa zaznaczone bitmapy ręcznie: dla
  szerokości i wysokości wpisz nowy rozmiar w pikselach lub w procentach (od 100
  do 1000%), z opcją **Zachowaj proporcje**. Obraz pozostaje wyśrodkowany.

![Okno dialogowe Ręczne rozszerzanie bitmapy](../../img/bitmaps-resample-and-straighten-inflate.png)

## Inne polecenia bitmap {#other}

- [[menu.bitmaps.crop_bitmap]] kadruje bitmapę narzędziem [[tool.crop]];
  przeciągnięcie narożnika bitmapy narzędziem [[tool.shape]] również ją kadruje.
- [[menu.bitmaps.edit_bitmap]] otwiera bitmapę w innym programie.
- [[menu.bitmaps.remove_background]] sprawia, że tło zdjęcia staje się
  przezroczyste.
- [[menu.bitmaps.remove_background_local]]: [Usuń tło](remove-background.md).
- [[menu.bitmaps.break_link]] i [[menu.bitmaps.update_from_link]] działają na
  bitmapach połączonych z plikiem.
- [[menu.bitmaps.save_bitmap_as]] zapisuje bitmapę jako plik PNG.
