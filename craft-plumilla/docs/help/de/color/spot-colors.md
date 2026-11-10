# Volltonfarben

Eine **Volltonfarbe** ist eine benannte Druckfarbe, die auf einer eigenen
Druckplatte gedruckt wird, etwa eine Metallic- oder eine Markenfarbe, statt aus
Cyan, Magenta, Gelb und Schwarz gemischt zu werden. Jede Zeichnung führt eine
Liste ihrer Volltonfarben. Jede Farbe hat einen Ersatz aus Prozessfarben, der
zur Anzeige auf dem Bildschirm und überall dort verwendet wird, wo die
Druckfarbe nicht verwendet werden kann.

## Eine Volltonfarbe hinzufügen {#add}

1. Öffnen Sie das Andockfenster [[docker.color]] und mischen Sie die Farbe, wie
   die Druckfarbe aussehen soll.
2. Geben Sie unter [[docker.spot_inks]] einen [[docker.ink_name]] ein.
3. Klicken Sie auf [[docker.new_ink]].

Namen sind innerhalb einer Zeichnung eindeutig.

## Volltonfarben verwenden {#use}

Die Liste [[docker.spot_inks]] zeigt jede Farbe mit einem Farbfeld, ihrem
Namen und den Schaltflächen [[docker.fill]] und [[docker.outline]], die sie mit
dem eingestellten [[docker.tint]] (0 bis 100 %, anfangs 100 %) auf die Auswahl
anwenden. Das Kreuz am Ende einer Zeile entfernt die Farbe aus der Liste;
Objekte, die sie verwenden, behalten ihre Ersatzfarbe.

## Farbbücher {#color-books}

Eine Palettendatei kann ein Buch von Volltonfarben sein (zum Beispiel eine
`.ase`-Palette mit Volltoneinträgen). Ihre Volltoneinträge öffnen sich als
Volltonfarben, benannt wie im Buch; wenn Sie einen anwenden, wird die Farbe der
Zeichnung hinzugefügt. TraceDraw liefert keine lizenzierten Farbbücher mit,
aber ein Buch, das Sie besitzen, öffnet sich wie jede Palette.

## Volltonfarben im Druck und in Exporten {#output}

- In PDF-Dateien wird jede Farbe zu einer nach ihr benannten Separationsfarbe,
  sodass die Druckerei die Druckfarbe erhält.
- Wenn Sie Farbauszüge drucken, erhält jede Farbe eine eigene Druckplatte nach
  den Platten für Cyan, Magenta, Gelb und Schwarz. Siehe
  [Drucken](../printing/print.md).
- Auf dem Bildschirm und in Bitmap-, SVG- und `.cdr`-Dateien erscheint eine
  Volltonfarbe als ihr Ersatz in ihrem Farbton.
