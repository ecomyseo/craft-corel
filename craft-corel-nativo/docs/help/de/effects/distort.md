# Verzerrung

Das Werkzeug [[tool.distort]] verbiegt den Umriss eines Objekts auf drei Arten:
**Drücken und Ziehen** (Zacken oder Blütenblätter), **Zickzack**
(Zickzackzähne) und **Verdrehen** (ein Wirbel). Es ist ein Live-Effekt: Sie
können die Verzerrung jederzeit ändern, verschieben oder entfernen, und das
Objekt behält seinen eigenen Umriss.

## Ein Objekt verzerren {#create}

1. Wählen Sie das Werkzeug [[tool.distort]] aus dem Flyout für Effekte.
2. Wählen Sie die Art in der Eigenschaftsleiste.
3. Ziehen Sie auf dem Objekt. Die Verzerrung wird dort zentriert, wo Sie
   beginnen.
   - **Drücken und Ziehen**: Ziehen Sie nach rechts, um die Knoten nach außen
     zu drücken (zackige Formen mit hohlen Seiten), nach links, um sie nach
     innen zu ziehen (runde Blütenblätter).
   - **Zickzack**: Je weiter Sie ziehen, desto höher die Zähne.
   - **Verdrehen**: Ziehen Sie um den Startpunkt herum; volle Umdrehungen
     zählen.

Der Mittelpunkt einer ausgewählten Verzerrung wird als Raute angezeigt: Ziehen
Sie sie, um den Mittelpunkt zu verschieben. Erneutes Ziehen ersetzt die
Verzerrung; ein Objekt hat höchstens eine. Gruppen und Bitmaps werden nicht
verzerrt.

## Verzerrungseinstellungen {#settings}

Die Eigenschaftsleiste zeigt und ändert die ausgewählte Verzerrung:

- die Art: **Drücken und Ziehen**, **Zickzack** oder **Verdrehen** (die Wahl
  einer anderen Art ändert eine ausgewählte Verzerrung);
- bei Drücken und Ziehen die **Amplitude** (-200 bis 200, anfangs 20);
- bei Zickzack die **Amplitude** (0 bis 100 mm, anfangs 2 mm) und die
  **Frequenz** (Zähne pro Segment, anfangs 6), mit **Zufällig** (ungleichmäßige
  Zähne), **Glatt** (abgerundete Zähne) und **Lokal** (Zähne werden vom
  Mittelpunkt weg kleiner);
- bei Verdrehen **im Uhrzeigersinn** oder **gegen den Uhrzeigersinn**, die
  Anzahl ganzer **Umdrehungen** und die zusätzlichen **Grad**.

[[menu.object > menu.object.clear_effect]] entfernt die Verzerrung.
