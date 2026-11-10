# Das Andockfenster Effekte

Ein Objekt kann mehrere Live-Effekte gleichzeitig tragen, die nacheinander
angewendet werden. Das Andockfenster [[docker.effects]] listet die Effekte des
ausgewählten Objekts in der Reihenfolge auf, in der sie angewendet werden, von
oben nach unten. Öffnen Sie es mit
[[menu.window > menu.window.dockers > menu.window.dockers_effects]].

![Das Andockfenster Effekte](../../img/effects-effects-docker.png)

## Effekte verwalten {#stack}

Jede Zeile der Liste hat:

- ein Kontrollkästchen, das den Effekt ein- oder ausschaltet. Ein
  ausgeschalteter Effekt behält Platz und Einstellungen, wird aber nicht
  gezeichnet, exportiert oder gedruckt;
- den Namen des Effekts, abgeblendet, solange er aus ist;
- Pfeile nach oben und unten, die den Effekt früher oder später anwenden, was
  das Ergebnis ändert (zum Beispiel eine Kontur einer Hülle oder eine Hülle
  einer Kontur);
- ein Kreuz, das den Effekt entfernt.

Jede Änderung ist ein Rückgängig-Schritt. Jeder Effekt wird weiterhin mit
seinem eigenen Werkzeug oder Andockfenster bearbeitet.

## Weitere Effektbefehle {#commands}

- [[menu.effects > menu.effects.flatten]] wandelt die Effekte der ausgewählten
  Objekte in einfache Objekte um.
- [[menu.effects.picker]] zeigt eine Liste von Effekten, die auf die Auswahl
  angewendet werden können.
- [[menu.object > menu.object.clear_effect]] entfernt den Effekt der Auswahl.
- [[menu.object > menu.object.copy_effect]] kopiert einen Effekt von einem
  anderen Objekt, und [[menu.object.clone_effect]] erstellt einen Klon, der
  ihm folgt; siehe [Effekte kopieren und klonen](copy-and-clone.md).
