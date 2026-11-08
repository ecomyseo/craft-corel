# Aree di lavoro

Un'area di lavoro è il modo in cui è configurata la finestra: quali barre sono
visibili e dove, la casella degli strumenti, le scorciatoie da tastiera, le
opzioni della tavolozza dei colori e l'aspetto dell'interfaccia. Scegline una
in [[menu.window > menu.window.workspace]], o nella pagina
[[options.workspace]] di
[[menu.tools > menu.tools.options > menu.tools.options_workspaces]].

![Le opzioni delle aree di lavoro](../../img/customize-workspaces.png)

## Aree di lavoro integrate {#built-in}

- [[menu.window.workspace_default]]: la configurazione abituale.
- [[menu.window.workspace_lite]]: meno barre, per un primo avvio.
- [[menu.window.workspace_classic]], [[menu.window.workspace_illustration]] e
  [[menu.window.workspace_page_layout]]: barre e pannelli per quei tipi di
  lavoro.
- [[menu.window.workspace_touch]]: per gli schermi tattili. L'interfaccia è
  disegnata più grande, righelli, pannelli e barra di stato sono nascosti, e
  con due dita si sposta la vista e si pizzica per lo zoom con qualsiasi
  strumento. I disegni mantengono le loro dimensioni reali.

## Cosa viene conservato {#kept}

Tutto ciò che cambi viene conservato quando TraceDraw si chiude e ritorna la
volta successiva: le opzioni, dove si trova ciascuna barra e quali barre sono
visibili, lo strumento mostrato da ciascun gruppo della casella degli
strumenti, i pannelli aperti, le unità, l'aggancio, le impostazioni dei nuovi
oggetti e ciò che mostra la finestra.

## Esportare e importare la configurazione {#export-import}

Per portare la tua configurazione su un altro computer, o tenerla al sicuro:

1. Nella pagina [[options.workspace]], sotto [[options.workspace_transfer]],
   spunta le parti da includere: scorciatoie da tastiera; barre degli
   strumenti, casella degli strumenti e barra di stato; opzioni della
   tavolozza dei colori; aspetto.
2. Fai clic su [[options.workspace_export]] e salva il file `.tdws`.
3. Sull'altro computer, fai clic su [[options.workspace_import]] e scegli il
   file. Vengono applicate solo le parti che hai spuntato.

[[menu.window > menu.window.workspace > menu.window.workspace_export]] e
[[menu.window.workspace_import]] fanno lo stesso con tutte le parti.
