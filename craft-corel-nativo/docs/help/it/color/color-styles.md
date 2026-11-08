# Stili colore

Uno **stile colore** è un colore con un nome usato dagli oggetti del disegno.
Cambia lo stile e cambia anche ogni oggetto riempito con esso: questo mantiene
coerenti i colori di un progetto e permette di provare rapidamente altre
combinazioni di colori. Gli stili sono elencati nel pannello
[[docker.color_styles]]; aprilo da [[menu.window > menu.window.dockers]].

![Il pannello Stili colore](../../img/color-color-styles.png)

## Creare stili colore {#create}

- [[docker.new_from_selection]] aggiunge uno stile per ogni colore di
  riempimento degli oggetti selezionati che non è ancora uno stile.
- [[docker.new_color_style]] aggiunge uno stile con il colore corrente del
  pannello [[docker.color]].

## Usare e modificare gli stili {#edit}

Ogni stile mostra il suo colore, il suo nome e tre pulsanti:

- fai clic sul colore per cambiarlo: gli oggetti riempiti con quel colore lo
  seguono;
- fai clic sul nome per rinominarlo;
- [[docker.fill]] riempie la selezione con lo stile, [[docker.outline]] ne
  colora il contorno;
- **x** rimuove lo stile (gli oggetti mantengono il loro colore).

## Armonie {#harmonies}

L'[[docker.harmony_editor]] crea un insieme di stili che stanno bene insieme, a
partire da un [[docker.base_color]]:

1. Scegli il [[docker.base_color]].
2. Fai clic su un'armonia: [[harmony.analogous]], [[harmony.complementary]],
   [[harmony.split_complementary]], [[harmony.triadic]],
   [[harmony.tetradic]] o [[harmony.monochromatic]].

I colori dell'armonia vengono aggiunti come stili, con il suo nome.
