# Script e automazione

Gli script automatizzano il lavoro ripetitivo: disegnare forme a partire da
dati, modificare molti oggetti in una volta o aggiungere un formato di file.
Si scrivono in JavaScript nel pannello [[docker.scripts]]
([[menu.tools > menu.tools.scripts >
menu.tools.scripts_docker]]).

![Il pannello Script](../../img/customize-scripts.png)

## Scrivere ed eseguire script {#run}

Uno script lavora con gli oggetti di TraceDraw: `Application`,
`ActiveDocument`, `ActivePage`, `ActiveLayer` e le forme, i riempimenti, i
contorni, i colori, le curve e gli effetti. Ad esempio:

```js
for (let i = 0; i < 5; i++) {
  ActiveLayer.CreateRectangle(10 + i * 25, 10, 20, 20);
}
```

- [[docker.run]] esegue lo script. Le sue modifiche sono un unico passaggio di
  annullamento.
- [[docker.record]] registra ciò che fai come script (anche
  [[menu.tools > menu.tools.scripts > menu.tools.scripts_record]]).
- [[docker.load]] e [[docker.save]] aprono e salvano gli script;
  [[menu.tools.scripts_run]] esegue un file di script.
- [[docker.clear]] svuota l'editor; l'[[docker.output]] mostra ciò che lo
  script stampa e i suoi errori.

## Debug {#debug}

Sul desktop, fai clic nel margine dell'editor per impostare un punto di
interruzione, poi fai clic su [[docker.debug]]. Lo script si ferma al punto di
interruzione: **Continua**, **Passo** o **Interrompi**, e digita
un'espressione per valutarla nel punto in cui lo script è in pausa.

## Filtri script {#filters}

Uno script che definisce `TraceDrawFilter`, con un nome, delle estensioni e
una funzione `Import` o `Export`, aggiunge un formato di file a
[[menu.file.import]] e [[menu.file.export]]. Fai clic su
[[docker.install_filter]] per conservarlo; [[docker.script_filters]] elenca i
filtri installati con [[docker.edit]] e [[docker.remove]].

## Assistenti IA (MCP) {#mcp}

TraceDraw parla il Model Context Protocol, quindi gli assistenti IA sul tuo
computer possono aprire, modificare, esaminare, esportare e salvare disegni:

- `tracedraw --mcp` esegue TraceDraw senza finestra per un assistente.
- Per permettere a un assistente di modificare il disegno aperto nella tua
  finestra, attiva [[options.mcp_window]] nella pagina [[options.general]] di
  [[menu.tools > menu.tools.options > menu.tools.options_global]], e dai
  all'assistente `tracedraw --mcp-window` come comando. Ogni modifica è un
  passaggio di annullamento che puoi revocare. Possono connettersi solo i
  programmi del tuo computer che hanno il token segreto scritto da TraceDraw.

## Lo strumento da riga di comando {#cli}

Ogni download include anche `tracedraw-cli`, che converte i disegni tra
formati senza aprire il programma, per il lavoro in serie.

## Macro {#macros}

TraceDraw esegue progetti di macro VBA scritti per programmi di disegno (versione desktop). [[menu.tools > menu.tools.macros > menu.tools.macros_load]] aggiunge un progetto (`.gms`) o file di modulo (`.bas`, `.cls`, `.frm`); i progetti vengono conservati e caricati all'avvio. Il pannello [[docker.macros]] elenca le macro: fare doppio clic su una o usare [[menu.tools > menu.tools.macros > menu.tools.macros_run]] (<kbd>Alt+Shift+F8</kbd>) per eseguirla. Moduli, finestre di messaggio e di input della macro compaiono sullo schermo, <kbd>Esc</kbd> o [[menu.tools > menu.tools.macros > menu.tools.macros_stop]] la interrompe e tutto ciò che un'esecuzione cambia è un solo passo di annullamento. [[menu.tools > menu.tools.macros > menu.tools.macro_editor]] (<kbd>Alt+F11</kbd>) mostra e modifica il codice. Le macro che chiamano funzioni di Windows o il modello a oggetti dell'editor stesso non vengono eseguite.
