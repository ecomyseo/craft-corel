# Musterfüllungen

Eine Musterfüllung wiederholt eine Kachel im Inneren eines Objekts. Die Kachel
kann zweifarbig sein, aus Vektorobjekten bestehen (ein vollfarbiges
Vektormuster) oder eine Bitmap sein. Kacheln sind an der Seite verankert,
sodass sie nicht mitrutschen, wenn Sie das Objekt verschieben.

## Ein Muster aus Ihren Objekten erstellen {#create-pattern}

1. Zeichnen Sie die Objekte, die eine Kachel bilden, und wählen Sie sie aus.
2. Wählen Sie [[menu.object > menu.object.create > menu.object.create_pattern]].
3. Wählen Sie im Dialogfeld [[dialog.create_pattern]] den
   [[dialog.create_pattern_type]]: [[dialog.create_pattern_vector]] behält die
   Objekte als Vektoren, scharf in jeder Größe;
   [[dialog.create_pattern_bitmap]] erstellt ein Bild davon.
4. Klicken Sie auf [[dialog.ok]].

![Das Dialogfeld Muster erstellen](../../img/color-patterns-create-pattern.png)

Das Muster wird zur Füllung neuer Objekte. Um es bereits gezeichneten Objekten
zuzuweisen, wählen Sie [[docker.fill_pattern]] im Füllungsbereich des
Andockfensters [[docker.properties]].

## Mustereinstellungen {#settings}

Wählen Sie im Füllungsbereich des Andockfensters [[docker.properties]]
[[docker.fill_pattern]], dann die Art des Musters und seine Einstellungen. Bei
einem Vektormuster zeigt das Andockfenster, wie viele Objekte die Kachel
enthält, sowie ihre Breite und Höhe: Wenn Sie diese ändern, wird der Inhalt der
Kachel mitskaliert.

## Muster in exportierten Dateien {#export}

SVG-Dateien behalten Vektormuster als Vektoren. PDF- und EPS-Dateien erhalten
sie als Bilder in der Exportauflösung, auf das Objekt zugeschnitten.
