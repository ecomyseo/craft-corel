# Abschrägungen

Eine Abschrägung lässt die Kanten eines Objekts erhaben oder eingraviert
wirken, indem sie eine Seite der Kante beleuchtet und die andere schattiert.
Sie ist ein Live-Effekt, der im Andockfenster [[docker.bevel]] eingestellt
wird; öffnen Sie es mit [[menu.effects > menu.effects.bevel]].

![Das Andockfenster Abschrägung](../../img/effects-bevel.png)

## Ein Objekt abschrägen {#create}

1. Wählen Sie ein gefülltes Objekt aus.
2. Wählen Sie im Andockfenster [[docker.bevel]] den Stil:
   - [[docker.soft_edge]]: Die Kante fällt zum Objekt hin ab, auf einer Seite
     beleuchtet und auf der anderen schattiert;
   - [[docker.emboss]]: Das Objekt scheint aus der Seite herausgeprägt.
3. Stellen Sie das Aussehen der Abschrägung ein:
   - [[docker.depth]]: wie weit die Abschrägung von der Kante reicht, beim
     weichen Rand ins Objekt hinein, beim Relief nach außen;
   - [[docker.level]]: in wie viele Stufen Licht und Schatten über die
     Abschrägung verteilt werden. 1 ergibt zwei flache Töne, mehr Stufen
     verlaufen weich von der Kante nach innen;
   - [[docker.light_angle]]: die Richtung, aus der das Licht kommt;
   - [[docker.intensity]]: wie stark Licht und Schatten sind;
   - [[docker.light_color]] und [[docker.shadow_color]]: die Farben der
     beleuchteten und der schattierten Seite;
   - [[docker.emboss_color]]: die Farbe, aus der die Abschrägung abgetönt
     wird. Setzen Sie das Häkchen und wählen Sie eine eigene Farbe;
     ausgeschaltet gilt die Füllung des Objekts.
4. Klicken Sie auf [[docker.apply]].

Um eine Abschrägung von einem anderen Objekt zu kopieren, verwenden Sie
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] entfernt sie. Das Andockfenster für
Extrusionen hat ebenfalls eine Abschrägung für die Vorderseite; siehe
[Extrusionen](extrude.md#settings).
