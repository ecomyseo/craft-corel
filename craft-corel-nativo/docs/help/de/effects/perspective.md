# Perspektive

Perspektive lässt ein Objekt so aussehen, als läge es auf einer Ebene, die in
die Ferne zurückweicht. Sie können einem gezeichneten Objekt Perspektive geben
oder direkt auf einem Perspektivraster zeichnen.

## Einem Objekt Perspektive geben {#add}

1. Wählen Sie ein Objekt aus.
2. Wählen Sie [[menu.object > menu.object.add_perspective]].
3. Ziehen Sie die vier Eckknoten mit dem Werkzeug [[tool.shape]], um es zu
   kippen.

Um die Perspektive eines anderen Objekts zu kopieren, verwenden Sie
[[menu.object > menu.object.copy_effect > menu.object.copy_perspective_from]].

## Perspektivisch zeichnen {#draw}

[[menu.view > menu.view.draw_in_perspective]] zeigt ein Perspektivraster über
der Seite. Solange es aktiv ist, wird jedes Objekt, das ein Zeichenwerkzeug
erstellt (Rechtecke, Ellipsen, Polygone, Kurven, Pinselstriche, Textrahmen),
auf die aktive Ebene des Rasters gelegt. Objekte, die Sie einfügen,
duplizieren oder mit dem Werkzeug [[tool.pick]] verschieben, bleiben, wie sie
sind.

Die Leiste oben links im Zeichenfenster enthält:

- **Fertig stellen**: verlässt den Modus; die Objekte behalten ihre
  Perspektive;
- **1-Punkt**, **2-Punkt** und **3-Punkt**: die Art des Rasters;
- die Ebenen: **Boden**, **Links**, **Rechts** (und **Oben** bei einem
  1-Punkt-Raster). Die aktive Ebene wird kräftig gezeichnet;
- **Raster**: zeigt die Rasterlinien;
- **Tiefe** (5 bis 98 %): wie weit das Raster in Richtung der Fluchtpunkte
  reicht;
- **Auf Ebene legen**: legt die ausgewählten Objekte auf die aktive Ebene.

Ziehen Sie die Fluchtpunkte (Kreise) und die vorderen Ecken (Quadrate), um das
Raster zu ändern. Ein auf eine Ebene gelegtes Objekt erhält einen gewöhnlichen
Perspektiveffekt, dessen vier Ecken Sie weiterhin mit dem Werkzeug
[[tool.shape]] bearbeiten können.
