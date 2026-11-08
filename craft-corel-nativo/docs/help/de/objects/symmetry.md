# Mit Symmetrie zeichnen

Symmetrie zeichnet live gespiegelte Kopien eines Objekts an einer oder mehreren
Spiegelachsen. Die Kopien folgen jeder Änderung am Original, sodass Sie eine
Hälfte (oder ein Segment) zeichnen und das Ganze sehen. Die Befehle finden Sie
unter [[menu.object > menu.object.symmetry]].

## Eine Symmetrie erstellen {#create}

1. Wählen Sie ein Objekt aus.
2. Wählen Sie [[menu.object > menu.object.symmetry > menu.object.symmetry_create]]
   (<kbd>Alt+S</kbd>).

Eine senkrechte Spiegelachse wird an den rechten Rand des Objekts gelegt,
sodass die Kopie daneben erscheint, und TraceDraw wechselt direkt in die
Bearbeitung der Symmetrie.

## Eine Symmetrie bearbeiten {#edit}

[[menu.object.symmetry_edit]] (<kbd>Alt+E</kbd>) bearbeitet die Symmetrie des
ausgewählten Objekts. Eine Leiste oben links im Zeichenfenster legt fest:

- die Anzahl der Spiegelachsen, 1 bis 12, gleichmäßig um den Mittelpunkt
  verteilt. Mit mehr Achsen erhalten Sie ein Kaleidoskop: Bei *n* Achsen gibt
  es 2*n* - 1 Kopien;
- den Mittelpunkt (X und Y);
- den Winkel der ersten Achse (0 bis 360 Grad);
- Vollständige Vorschau (Kopien vollständig oder, wenn aus, als rote Umrisse)
  und Symmetrielinien anzeigen.

Bearbeiten Sie die Originale wie gewohnt mit einem beliebigen Werkzeug; die
Kopien folgen. Ein Klick oder ein Auswahlrahmen wählt eines der Originale in
der Symmetrie, und Objekte, die Sie zeichnen, einfügen oder duplizieren, kommen
hinzu. Ziehen Sie eine Spiegelachse mit dem Auswahlwerkzeug, um die Achsen zu
verschieben; ein Klick auf eine Achse zeigt runde Griffe an ihren Enden, mit
denen Sie die Achsen drehen. Solange ein Objekt mit Symmetrie ausgewählt ist
oder bearbeitet wird, werden seine Spiegelachsen gestrichelt angezeigt.

Klicken Sie auf der Leiste auf Fertig stellen oder wählen Sie
[[menu.object.symmetry_finish]] (<kbd>Alt+Shift+E</kbd>), um die Bearbeitung
zu beenden.

## Kopien entfernen oder behalten {#remove}

- [[menu.object.symmetry_remove]] (<kbd>Alt+Shift+S</kbd>) entfernt die
  Symmetrie; die Kopien verschwinden.
- [[menu.object.symmetry_break]] (<kbd>Alt+X</kbd>) wandelt die Kopien in
  eigenständige Objekte um, die dem Original nicht mehr folgen.
