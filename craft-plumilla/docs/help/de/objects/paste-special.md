# Inhalte einfügen

[[menu.edit > menu.edit.paste_special]] legt fest, wie der Inhalt der
Zwischenablage in die Zeichnung kommt, wenn er auf mehr als eine Weise kommen
kann: Text, der in einem anderen Programm kopiert wurde, kann zum Beispiel ein
Textobjekt werden, und ein von einer Webseite kopiertes SVG kann zu
bearbeitbaren Formen oder zu einem Bild werden.

![Das Dialogfeld Inhalte einfügen](../../img/objects-paste-special.png)

## In einer anderen Form einfügen {#paste-as}

1. Kopieren Sie etwas, in TraceDraw oder in einem anderen Programm.
2. Wählen Sie [[menu.edit > menu.edit.paste_special]].
3. Wählen Sie unter [[dialog.paste_as]] eine der Formen, die die
   Zwischenablage anbietet:
   - [[dialog.paste_objects]]: in TraceDraw kopierte Objekte, unverändert;
   - [[dialog.paste_svg]]: SVG-Text, umgewandelt in Formen, die Sie bearbeiten
     können;
   - [[dialog.paste_bitmap]]: ein Bild, als Bitmap;
   - [[dialog.paste_text]]: einfacher Text, als Textzeile.
4. Klicken Sie auf [[dialog.ok]].

Enthält die Zwischenablage nichts, was eingefügt werden kann, zeigt das
Dialogfeld [[dialog.clipboard_empty]].

Das gewöhnliche [[menu.edit.paste]] (<kbd>Ctrl+V</kbd>) fügt in der ersten
verfügbaren Form ein, und [[menu.edit.paste_in_view]]
(<kbd>Ctrl+Shift+V</kbd>) fügt in der Mitte des Fensters ein.
