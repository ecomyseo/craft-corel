# Farbstile

Ein **Farbstil** ist eine benannte Farbe, die von Objekten der Zeichnung
verwendet wird. Ändern Sie den Stil, ändert sich jedes damit gefüllte Objekt
mit. So bleiben die Farben eines Entwurfs einheitlich, und andere Farbschemata
sind schnell ausprobiert. Die Stile sind im Andockfenster
[[docker.color_styles]] aufgelistet; öffnen Sie es über
[[menu.window > menu.window.dockers]].

![Das Andockfenster Farbstile](../../img/color-color-styles.png)

## Farbstile erstellen {#create}

- [[docker.new_from_selection]] fügt für jede Füllfarbe der ausgewählten
  Objekte, die noch kein Stil ist, einen Stil hinzu.
- [[docker.new_color_style]] fügt einen Stil mit der aktuellen Farbe des
  Andockfensters [[docker.color]] hinzu.

## Stile verwenden und bearbeiten {#edit}

Jeder Stil zeigt seine Farbe, seinen Namen und drei Schaltflächen:

- Klicken Sie auf die Farbe, um sie zu ändern: Die damit gefüllten Objekte
  folgen;
- klicken Sie auf den Namen, um ihn umzubenennen;
- [[docker.fill]] füllt die Auswahl mit dem Stil, [[docker.outline]] färbt
  ihren Umriss;
- **x** entfernt den Stil (die Objekte behalten ihre Farbe).

## Harmonien {#harmonies}

Der [[docker.harmony_editor]] erstellt aus einer [[docker.base_color]] einen
Satz zusammenpassender Stile:

1. Wählen Sie die [[docker.base_color]].
2. Klicken Sie auf eine Harmonie: [[harmony.analogous]],
   [[harmony.complementary]], [[harmony.split_complementary]],
   [[harmony.triadic]], [[harmony.tetradic]] oder [[harmony.monochromatic]].

Die Farben der Harmonie werden als Stile hinzugefügt und nach ihr benannt.
