# Kommentare

Das Andockfenster [[docker.comments]] sammelt Prüfnotizen zu einer Zeichnung:
Pins, Rahmen und Pfeile auf der Seite, jeweils mit einem Text und Antworten.
Kommentare werden mit der Zeichnung gespeichert (in `.tdraw`-Dateien) und nie
gedruckt oder exportiert. Öffnen Sie das Andockfenster über
[[menu.window > menu.window.dockers]].

![Das Andockfenster Kommentare](../../img/objects-comments.png)

## Einen Kommentar hinzufügen {#add}

1. Geben Sie Ihren Namen in [[comments.name]] ein. Er unterzeichnet Ihre
   Kommentare und Antworten. Bleibt das Feld leer, wird der Autor der Zeichnung
   verwendet.
2. Klicken Sie neben [[comments.add]] auf [[comments.pin]], [[comments.rect]]
   oder [[comments.arrow]].
3. Klicken Sie auf der Seite, um einen Pin zu setzen, oder ziehen Sie, um einen
   Rahmen um etwas oder einen Pfeil darauf zu zeichnen.
4. Geben Sie den Kommentar ein.

Jeder Kommentar zeigt seine Nummer in einer orangefarbenen Blase auf der Seite.

## Lesen und antworten {#list}

Das Andockfenster listet die Kommentare auf, mit der Anzahl der offenen, jeden
mit Nummer, Autor, Datum und Uhrzeit, Seite, Text und Antworten. Klicken Sie
auf eine Karte, um den Kommentar zu wählen und zu seiner Seite zu springen; bei
geöffnetem Andockfenster können Sie auch auf eine Blase oder einen Rahmen auf
der Seite klicken.

- Bearbeiten Sie den Text des gewählten Kommentars, oder schreiben Sie in das
  Feld [[comments.reply_hint]] und drücken Sie <kbd>Enter</kbd> oder klicken
  Sie auf [[comments.reply]].
- [[comments.resolve]] markiert einen Kommentar als erledigt;
  [[comments.reopen]] öffnet ihn wieder. Erledigte Kommentare sind
  ausgeblendet, sofern [[comments.show_resolved]] nicht aktiviert ist.
- [[comments.hide_marks]] blendet die Blasen, Rahmen und Pfeile auf der Seite
  aus.
- [[comments.delete]] entfernt einen Kommentar und seine Antworten.

Jede Änderung ist ein Rückgängig-Schritt. Wird eine Seite gelöscht, werden auch
ihre Kommentare gelöscht.
