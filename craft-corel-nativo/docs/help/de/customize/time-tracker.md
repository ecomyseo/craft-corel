# Zeiterfassung

Die Zeiterfassung zeichnet die Zeit auf, die Sie an einer Zeichnung arbeiten,
Aufgabe für Aufgabe, und speichert die Einträge in der Zeichnung (in
`.tdraw`-Dateien). Gezählt wird nur, während Sie arbeiten: Nach einigen
Minuten ohne Eingabe wartet die Uhr, und Ihre nächste Aktion startet sie
wieder.

## Die Symbolleiste Zeiterfassung {#toolbar}

[[menu.window > menu.window.toolbars > menu.window.toolbar_time_tracker]] zeigt
eine Leiste mit:

- der Liste der Aufgaben, um die aktive Aufgabe zu wählen oder eine neue
  hinzuzufügen;
- der Zeit der aktiven Aufgabe;
- der Schaltfläche zum Aufzeichnen und Pausieren;
- einer Leuchte: rot während der Zählung, gelb im Leerlauf, grau bei Pause
  oder ohne Aufgabe;
- der Summe aller Aufgaben;
- einer Schaltfläche, die das Dialogfeld öffnet.

Wird die Uhr in einer Zeichnung ohne Aufgaben gestartet, wird „Task 1“
hinzugefügt.

## Das Dialogfeld Zeiterfassung {#dialog}

[[menu.tools > menu.tools.time_tracker]] listet die Aufgaben mit ihrer Zeit,
ihrem Beginn und Ende sowie die Summe auf.

![Das Dialogfeld Zeiterfassung](../../img/customize-time-tracker.png)

- **Neue Aufgabe**, **Löschen**, **Aktiv setzen** und **Zeit zurücksetzen**
  wirken auf die ausgewählte Aufgabe. Doppelklicken Sie auf eine Aufgabe, um
  sie aktiv zu setzen.
- **Aufzeichnen** und **Pause** starten und stoppen die Uhr.
- Bearbeiten Sie **Name**, **Zeit** (etwa `1:30:00`, `1:30` oder `1.5`
  Stunden), **Beginn** und **Ende** (etwa `2026-10-19 14:30`) der ausgewählten
  Aufgabe.
- **Zählung nach N Minuten ohne Eingabe anhalten** legt die Leerlaufzeit fest
  (anfangs 5 Minuten).
- **CSV exportieren** und **Text exportieren** speichern die Einträge, für eine
  Tabellenkalkulation oder einen Bericht.

Das Aufzeichnen von Zeit ist keine Bearbeitung der Zeichnung: Es erzeugt keinen
Rückgängig-Schritt. Die Einträge werden beim nächsten Speichern der Zeichnung
gespeichert.
