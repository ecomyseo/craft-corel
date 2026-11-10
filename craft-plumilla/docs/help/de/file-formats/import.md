# Dateien importieren

[[menu.file > menu.file.import]] (<kbd>Ctrl+I</kbd>) fügt den Inhalt einer
Datei in die geöffnete Zeichnung ein, auf der aktiven Ebene, als ein einziger
Rückgängig-Schritt. [[menu.file.open]] öffnet eine Datei dagegen als eigene
Zeichnung. Beide lesen die Formate, die unter
[Unterstützte Dateiformate](supported-formats.md#open) aufgeführt sind.

## Wohin importierte Objekte kommen {#placement}

Anfangs kommen importierte Objekte an die Stelle, an der sie in der Datei
liegen. Um sie stattdessen in der Ansicht zu zentrieren, ändern Sie die
Einstellung auf der Seite [[options.file_formats]] der globalen Optionen; siehe
[Globale Optionen](../customize/global-options.md#file-formats).

## Hinweise zu einigen Formaten {#formats}

- **PDF und AI**: Das Dialogfeld [[pdf_import.title]] fragt, wie der Text
  gelesen wird: als bearbeitbarer Text, der seine Schriften behält, wenn sie
  installiert sind, oder als Kurven aus den Schriften, die die Datei
  einbettet, sodass der Text genau so aussieht wie in der Datei. Optionen >
  Dateiformate behält die Wahl. Eine geschützte Datei fragt im Dialogfeld
  [[pdf_password.title]] nach ihrem Kennwort.
- **EPS und PostScript**: Jede gedruckte Seite wird zu einer Seite.
- **PSD**: Jede Ebene wird zu einer Bitmap mit Deckkraft, Sichtbarkeit und
  Maske.
- **DXF**: Die Zeichnungseinheiten werden aus der Datei übernommen.
- **Textdateien** (TXT, RTF, DOCX): werden zu Mengentext, mit Fett, Kursiv,
  Größen und Schriften, sofern die Datei sie hat.
- **Bitmaps**: werden in ihrer Auflösung platziert; siehe
  [Bitmaps](../bitmaps/index.md).

Wenn einige Objekte einer Datei nicht gelesen werden können oder Schriften
fehlen, weist eine Meldung darauf hin; siehe
[Hinweise nach dem Öffnen](../get-started/open-and-save.md#open-notes). Sie
können Dateien auch auf das Fenster ziehen oder aus anderen Programmen
einfügen; siehe [Inhalte einfügen](../objects/paste-special.md).

## PDF-Dateien mit mehreren Seiten {#pdf-pages}

Wenn Sie eine PDF- oder AI-Datei mit mehr als einer Seite importieren, öffnet
sich nach den Fragen zum Text und zum Kennwort das Dialogfeld
[[pdf_pages.title]]. Wählen Sie [[pdf_pages.all]] oder geben Sie die Seiten
ein, etwa `1-3, 5`; die Liste zeigt jede Seite mit Größe und Ausrichtung, und
ein Häkchen fügt eine Seite hinzu. Wählen Sie dann, wohin sie kommen:

- [[pdf_pages.new_pages]]: Jede Seite wird eine Seite der Zeichnung, mit
  eigener Größe und Ausrichtung, in der Reihenfolge der Datei. So werden
  PDF-Dateien zu einer Zeichnung zusammengeführt.
- [[pdf_pages.current_page]]: Die Objekte kommen auf die aktive Ebene; bei
  mehreren Seiten sind die Objekte jeder Seite eine Gruppe.

Der ganze Import ist ein einziger Rückgängig-Schritt. Eine Datei mit einer
Seite wird ohne das Dialogfeld importiert.
