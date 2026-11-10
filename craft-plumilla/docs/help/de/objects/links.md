# Links und Lesezeichen

Das Andockfenster [[docker.links]] versieht Objekte mit einem Hyperlink, sodass
ein Klick darauf in einer exportierten PDF-, SVG-Datei oder Webseite eine
Webadresse öffnet, eine E-Mail schreibt oder zu einer anderen Seite springt. Es
listet außerdem die benannten Objekte der Zeichnung als Lesezeichen auf. Öffnen
Sie es über [[menu.window > menu.window.dockers]].

![Das Andockfenster Links und Lesezeichen](../../img/objects-links.png)

## Einen Link hinzufügen {#add}

1. Wählen Sie ein oder mehrere Objekte aus.
2. Geben Sie die Adresse in das Feld [[docker.hyperlink]] ein und verlassen
   Sie das Feld. Die Schaltflächen [[docker.link_web]], [[docker.link_mail]]
   und [[docker.link_page]] setzen `https://`, `mailto:` oder `#page=` vor den
   Text, sodass Sie für einen Seitenlink nur die Seitennummer eingeben.
3. Klicken Sie auf [[docker.remove]], um einen Link zu entfernen, oder leeren
   Sie das Feld.

## Lesezeichen {#bookmarks}

Unter [[docker.bookmarks]] listet das Andockfenster eine Schaltfläche für jedes
Objekt mit Namen auf. Klicken Sie auf eine, um das Objekt auszuwählen und darauf
zu zoomen. Wie Sie Objekte benennen, lesen Sie unter
[Objekte und Ebenen umbenennen](objects-and-layers.md#rename).

## Links in exportierten Dateien {#exports}

- In PDF-Dateien wird jedes verknüpfte Objekt zu einem anklickbaren Bereich
  über seinem Rahmen. Ein Seitenlink springt an den Anfang dieser Seite der
  Datei. PDF/X-Dateien erhalten keine Links, da diese Standards sie nicht
  erlauben.
- In SVG-Dateien und Webseiten ist das Objekt selbst der Link.
- Ein Link auf einer Gruppe gilt für alle ihre Objekte; die Links der Objekte
  darin werden weggelassen.
