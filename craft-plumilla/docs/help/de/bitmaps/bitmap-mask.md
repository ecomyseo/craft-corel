# Farben aus Bitmaps maskieren

Das Andockfenster [[docker.bitmap_mask]] blendet die Pixel ausgewählter Farben
in einer Bitmap aus oder behält nur sie, zum Beispiel um einen weißen
Hintergrund transparent zu machen. Öffnen Sie es mit
[[menu.bitmaps > menu.bitmaps.bitmap_mask]].

![Das Andockfenster Bitmap-Maske](../../img/bitmaps-bitmap-mask.png)

## Farben maskieren {#mask}

1. Wählen Sie eine Bitmap aus.
2. Nehmen Sie Farben in die zehn Farbfelder auf:
   - Klicken Sie auf die Pipette und dann auf Farben in der Bitmap; jede füllt
     das ausgewählte Feld und aktiviert es;
   - oder klicken Sie auf **Farben aus Bitmap auswählen**, um die Felder mit
     den zehn Farben zu füllen, die die Bitmap am häufigsten verwendet.
3. Wählen Sie ein Feld aus und stellen Sie seine **Toleranz** ein (0 bis
   100 %): wie weit die Farbe eines Pixels von der Farbe des Felds abweichen
   darf, um noch als Treffer zu gelten.
4. Wählen Sie **Ausgewählte ausblenden** (passende Pixel werden transparent)
   oder **Ausgewählte anzeigen** (nur passende Pixel bleiben, anfangs).
5. Klicken Sie auf [[docker.apply]].

**Maske speichern** und **Maske öffnen** bewahren die Felder in einer
`.msk`-Datei auf; **Löschen** leert das ausgewählte Feld. Jede Bitmap ändert
sich in einem einzigen Rückgängig-Schritt.
