# Colori spot

Un **colore spot** è un inchiostro con un nome stampato su una lastra propria,
come un colore metallizzato o un colore aziendale, invece di essere ottenuto
mescolando ciano, magenta, giallo e nero. Ogni disegno conserva un elenco dei
suoi inchiostri spot. Ogni inchiostro ha un sostituto composto da colori di
quadricromia, usato per mostrarlo sullo schermo e ovunque l'inchiostro non
possa essere usato.

## Aggiungere un inchiostro spot {#add}

1. Apri il pannello [[docker.color]] e miscela il colore che l'inchiostro
   dovrebbe avere.
2. Sotto [[docker.spot_inks]], digita un [[docker.ink_name]].
3. Fai clic su [[docker.new_ink]].

I nomi sono unici all'interno di un disegno.

## Usare gli inchiostri spot {#use}

L'elenco [[docker.spot_inks]] mostra ogni inchiostro con un campione, il suo
nome e i pulsanti [[docker.fill]] e [[docker.outline]], che lo applicano alla
selezione con la [[docker.tint]] impostata (da 0 a 100%, 100% in partenza).
La croce alla fine di una riga rimuove l'inchiostro dall'elenco; gli oggetti
che lo usano mantengono il suo colore sostitutivo.

## Cartelle colori {#color-books}

Un file di tavolozza può essere una cartella di inchiostri spot (ad esempio
una tavolozza `.ase` con voci spot). Le sue voci spot si aprono come colori
spot, con i nomi della cartella; applicarne una aggiunge l'inchiostro al
disegno. TraceDraw non include cartelle colori con licenza, ma una che
possiedi si apre come qualsiasi tavolozza.

## Colori spot in stampa e nelle esportazioni {#output}

- Nei file PDF, ogni inchiostro diventa un colore di separazione con il suo
  nome, così la tipografia riceve l'inchiostro.
- Quando stampi le separazioni, ogni inchiostro riceve una lastra propria dopo
  le lastre ciano, magenta, giallo e nero. Vedi [Stampa](../printing/print.md).
- Sullo schermo e nei file bitmap, SVG e `.cdr`, un colore spot appare come il
  suo sostituto con la sua tinta.
