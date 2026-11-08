# Neuigkeiten

Jede Version von TraceDraw bringt Funktionen und Korrekturen. Diese Seite
erklärt, wie Sie herausfinden, was sich geändert hat, und nennt die Highlights
der letzten Versionen.

## Neues im Programm sehen {#in-the-program}

- Wählen Sie [[menu.help > menu.help.whats_new]], um die Seite mit den
  Neuigkeiten des Begrüßungsbildschirms zu öffnen. Sie zeigt die letzten
  zehn Versionen, die neueste zuerst, jede mit Datum und kurzem Titel. Ein
  Klick auf eine Version zeigt eine kurze Zusammenfassung der Änderungen;
  die neueste ist schon geöffnet. **Mehr erfahren?** unter einer
  Zusammenfassung öffnet die Seite dieser Version auf GitHub mit der
  vollständigen Liste der Änderungen.
- Schalten Sie [[menu.help > menu.help.highlight_whats_new]] ein, um in den
  Menüs zu markieren, was in dieser Version neu ist: Ein kleiner Punkt
  erscheint am Ende jedes neuen Befehls, neben dem Pfeil eines Untermenüs, das
  zu einem führt, und über dem Titel eines Menüs, das einen enthält. Die
  Markierungen verschwinden, wenn Sie den Befehl ausschalten, und beim
  nächsten Start von TraceDraw ist der Befehl wieder aus.
- Die vollständige Liste der Änderungen jeder Version steht im
  [Änderungsprotokoll](https://github.com/tracedraw/tracedraw/blob/main/CHANGELOG.md)
  des Projekts.

## Highlights der letzten Versionen {#highlights}

- **Touch und Telefone.** Umschalt-, Strg- und Alt-Schaltflächen in der
  Standardleiste bleiben für einen Finger gedrückt; unter Android hält die
  Oberfläche Abstand zur Kameraaussparung und zu den abgerundeten Ecken; der
  Apple Pencil funktioniert in Safari auf dem iPad.
- **Dateien.** .cdr-Dateien behalten die Füllregel sich überlappender Kurven
  und öffnen sich mit ihren Tabstopps, mehrspaltigem Text, Hilfslinien für
  alle Seiten und den Objekten der Masterseite.
- **Bilder, Drucken und Arbeitsbereich.** Bildpixel zuschneiden im
  Zuschneidewerkzeug und [[menu.bitmaps > menu.bitmaps.crop_to_frame]]
  schneiden die Pixel eines Bildes zu; Drucken in Graustufen und in
  umgekehrter Reihenfolge; [[menu.view > menu.view.scrollbars]] blendet die
  Bildlaufleisten aus.
- **TraceCut.** [[menu.tools > menu.tools.tracecut]] öffnet die Werkzeuge für
  Schneideplotter: Plotterausgabe, Schnittlinien, Verschachtelung und Messen.
  Siehe [TraceCut](../objects/tracecut.md).
- **Tastatur, Telefone und Stifte.** Menüs und Listen lassen sich mit der
  Tastatur bedienen. Auf Android-Telefonen passt die Oberfläche auf den
  Bildschirm, und eine aufliegende Hand zeichnet nicht mehr, während ein Stift
  benutzt wird; ein Tablettstift funktioniert unter Linux mit Wayland.
- **Dateien.** .cdr-Dateien öffnen sich mit Tabstopps, Masterseiten, Verläufen
  und Verzerrungen; Bilder aus PDF-Dateien kommen in voller Auflösung; JPEG
  und TIFF werden in CMYK exportiert.
- **Objektminiaturen.** Das Andockfenster [[docker.objects]] zeigt jedes
  Objekt verkleinert neben seinem Namen; Strg+Mausrad über der Liste oder der
  Schieberegler unten stellt die Größe ein. Siehe
  [Objekte und Ebenen ordnen](../objects/objects-and-layers.md#objects-docker).
- **Hilfe in Ihrer Sprache.** Jede Schaltfläche **?** und <kbd>F1</kbd> öffnen
  die Seite dieser Hilfe zu dem Dialogfeld, Andockfenster oder Werkzeug vor
  Ihnen, in der Sprache, auf die TraceDraw eingestellt ist.
- **Dokumenteigenschaften.**
  [[menu.file > menu.file.document_properties]] speichert Sprache, Titel,
  Betreff, Autor, Copyright, Stichwörter, Notizen und Bewertung einer
  Zeichnung, über einem vollständigen Bericht über ihren Inhalt. Siehe
  [Dokumenteigenschaften](../pages-layout/document-properties.md).
- **Farbmanagement.** [[menu.tools > menu.tools.color_management]] legt die
  Profile der Zeichnung und die Vorgaben für neue fest, mit Regeln für Dateien,
  die geöffnet oder importiert werden. Siehe
  [Farbmanagement](../color/color-management.md).
- **Strichcodes und QR-Codes.** Ein Assistent in drei Schritten platziert
  Strichcodes, und QR-Codes sind dynamische Objekte, die Sie im Andockfenster
  [[docker.properties]] bearbeiten. Siehe
  [Strichcodes und QR-Codes](../objects/barcodes.md).
- **Effekte.** Das Menü [[menu.effects]] fasst die Effekt-Andockfenster
  zusammen; Überblendungen können einem Pfad mit festem Abstand folgen und
  ihre Farben im oder gegen den Uhrzeigersinn drehen. Siehe
  [Überblendungen](../effects/blend.md).
- **Text auf Pfad** hat eigene Einstellungen für Ausrichtung, Platzierung,
  Abstand und Versatz. Siehe
  [Text hinzufügen und bearbeiten](../text/adding-text.md).
- **Der Schriftmanager** ordnet installierte Schriften, Ordner, Favoriten und
  Sammlungen. Siehe [Schriften und Glyphen](../text/fonts.md).
