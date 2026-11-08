# Scripts en automatisering

Scripts automatiseren herhalend werk: vormen tekenen op basis van gegevens,
veel objecten tegelijk wijzigen of een bestandsindeling toevoegen. Ze worden in
JavaScript geschreven in het paneel [[docker.scripts]]
([[menu.tools > menu.tools.scripts >
menu.tools.scripts_docker]]).

![Het paneel Scripts](../../img/customize-scripts.png)

## Scripts schrijven en uitvoeren {#run}

Een script werkt met de objecten van TraceDraw: `Application`,
`ActiveDocument`, `ActivePage`, `ActiveLayer` en de vormen, vullingen,
omtrekken, kleuren, curven en effecten. Bijvoorbeeld:

```js
for (let i = 0; i < 5; i++) {
  ActiveLayer.CreateRectangle(10 + i * 25, 10, 20, 20);
}
```

- [[docker.run]] voert het script uit. De wijzigingen ervan zijn één
  ongedaan-makenstap.
- [[docker.record]] neemt wat u doet op als script (ook
  [[menu.tools > menu.tools.scripts > menu.tools.scripts_record]]).
- [[docker.load]] en [[docker.save]] openen en bewaren scripts;
  [[menu.tools.scripts_run]] voert een scriptbestand uit.
- [[docker.clear]] maakt de editor leeg; de [[docker.output]] toont wat het
  script afdrukt en de fouten ervan.

## Fouten opsporen {#debug}

Klik op de desktop in de marge van de editor om een onderbrekingspunt in te
stellen en klik daarna op [[docker.debug]]. Het script stopt bij het
onderbrekingspunt: kies **Doorgaan**, **Stap** of **Stoppen**, en typ een
expressie om die te evalueren op de plek waar het script gepauzeerd is.

## Scriptfilters {#filters}

Een script dat `TraceDrawFilter` definieert, met een naam, extensies en een
functie `Import` of `Export`, voegt een bestandsindeling toe aan
[[menu.file.import]] en [[menu.file.export]]. Klik op
[[docker.install_filter]] om het te bewaren; [[docker.script_filters]] toont de
geïnstalleerde filters met [[docker.edit]] en [[docker.remove]].

## AI-assistenten (MCP) {#mcp}

TraceDraw spreekt het Model Context Protocol, zodat AI-assistenten op uw
computer tekeningen kunnen openen, bewerken, inspecteren, exporteren en
opslaan:

- `tracedraw --mcp` voert TraceDraw zonder venster uit voor een assistent.
- Om een assistent de tekening te laten bewerken die in uw venster geopend is,
  schakelt u [[options.mcp_window]] in op de pagina [[options.general]] van
  [[menu.tools > menu.tools.options > menu.tools.options_global]], en geeft u
  de assistent `tracedraw --mcp-window` als opdracht. Elke wijziging is één
  ongedaan-makenstap die u kunt terugdraaien. Alleen programma's op uw computer
  met het geheime token dat TraceDraw wegschrijft, kunnen verbinding maken.

## Het opdrachtregelprogramma {#cli}

Elke download bevat ook `tracedraw-cli`, dat tekeningen tussen indelingen
omzet zonder het programma te openen, voor batchwerk.

## Macro's {#macros}

TraceDraw voert VBA-macroprojecten uit die voor tekenprogramma's zijn geschreven (desktopversie). [[menu.tools > menu.tools.macros > menu.tools.macros_load]] voegt een project (`.gms`) of modulebestanden (`.bas`, `.cls`, `.frm`) toe; projecten worden bewaard en bij het starten geladen. Het paneel [[docker.macros]] toont de macro's: dubbelklik op een macro of gebruik [[menu.tools > menu.tools.macros > menu.tools.macros_run]] (<kbd>Alt+Shift+F8</kbd>) om hem uit te voeren. Formulieren, berichtvensters en invoervensters van de macro verschijnen op het scherm, <kbd>Esc</kbd> of [[menu.tools > menu.tools.macros > menu.tools.macros_stop]] stopt hem, en alles wat één uitvoering verandert is één stap ongedaan maken. [[menu.tools > menu.tools.macros > menu.tools.macro_editor]] (<kbd>Alt+F11</kbd>) toont en bewerkt de code. Macro's die Windows-functies of het objectmodel van de editor zelf aanroepen, werken niet.
