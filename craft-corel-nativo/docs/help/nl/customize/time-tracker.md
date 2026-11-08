# Tijdregistratie

De tijdregistratie legt de tijd vast die u aan een tekening besteedt, taak voor
taak, en bewaart de gegevens in de tekening (in `.tdraw`-bestanden). Tijd telt
alleen terwijl u werkt: na een paar minuten zonder activiteit wacht de klok, en
uw volgende handeling start hem weer.

## De werkbalk Tijdregistratie {#toolbar}

[[menu.window > menu.window.toolbars > menu.window.toolbar_time_tracker]] toont
een balk met:

- de lijst met taken, om de actieve taak te kiezen of een nieuwe toe te voegen;
- de tijd van de actieve taak;
- de knop voor opnemen en pauzeren;
- een lampje: rood tijdens het tellen, oranje bij inactiviteit, grijs als de
  klok gepauzeerd is of er geen taak is;
- het totaal van alle taken;
- een knop die het dialoogvenster opent.

Als u de klok start op een tekening zonder taken, wordt "Taak 1" toegevoegd.

## Het dialoogvenster Tijdregistratie {#dialog}

[[menu.tools > menu.tools.time_tracker]] toont de taken met hun tijd, begin en
einde, en het totaal.

![Het dialoogvenster Tijdregistratie](../../img/customize-time-tracker.png)

- **Nieuwe taak**, **Verwijderen**, **Actief maken** en **Tijd herstellen**
  werken op de geselecteerde taak. Dubbelklik op een taak om haar actief te
  maken.
- **Opnemen** en **Pauzeren** starten en stoppen de klok.
- Bewerk de **naam**, **tijd** (zoals `1:30:00`, `1:30` of `1.5` uur),
  **begin** en **einde** (zoals `2026-10-19 14:30`) van de geselecteerde taak.
- **Stoppen met tellen na N minuten zonder invoer** stelt de inactieve tijd in
  (in het begin 5 minuten).
- **CSV exporteren** en **Tekst exporteren** slaan de gegevens op, voor een
  spreadsheet of een rapport.

Tijd registreren is geen bewerking van de tekening: het maakt geen
ongedaan-makenstap. De gegevens worden opgeslagen bij de volgende keer dat de
tekening wordt opgeslagen.
