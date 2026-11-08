# Das Andockfenster Exportieren

Das Andockfenster [[docker.export]] führt eine Liste von **Elementen** (Seiten
und Objekten) und exportiert sie alle auf einmal, jedes in seinem eigenen
Format, in einen Ordner: die Symbole einer App in mehreren Größen, jede Seite
einer Serie von Social-Media-Beiträgen. Öffnen Sie es über
[[menu.window > menu.window.dockers]].

![Das Andockfenster Exportieren](../../img/web-graphics-export-docker.png)

## Elemente hinzufügen {#assets}

- **Seiten hinzufügen** fügt jede Seite der Zeichnung hinzu, benannt nach der
  Seite.
- **Auswahl hinzufügen** fügt die ausgewählten Objekte als ein Element hinzu,
  auf sie zugeschnitten und nach dem Objekt benannt.
- Neue Elemente übernehmen Format und Auflösung, die unter **Neue Elemente**
  eingestellt sind (anfangs PNG mit 150 dpi).

Jedes Element hat ein Häkchen (nur abgehakte Elemente werden exportiert),
seinen Namen (den Dateinamen), sein Format, seine Auflösung bei
Bitmap-Formaten (72 bis 2400 dpi) und ein Kreuz zum Entfernen. Der Link unter
dem Namen springt zur Seite oder wählt die Objekte aus.

## Exportieren {#export}

1. Klicken Sie auf **Durchsuchen**, um den Ordner zu wählen (beim ersten Export
   wird danach gefragt).
2. Klicken Sie auf **Exportieren**. Die Schaltfläche zeigt, wie viele Elemente
   geschrieben werden.

Zeichen, die in Dateinamen nicht vorkommen dürfen, werden zu `_`, und zwei
Elemente mit demselben Namen erhalten " (2)", " (3)" und so weiter. Die
Statusleiste meldet, wie viele Dateien geschrieben wurden. Im Browser wird jede
Datei heruntergeladen.
