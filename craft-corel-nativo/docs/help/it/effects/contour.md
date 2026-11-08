# Contorni concentrici

Un contorno concentrico disegna copie equidistanti del contorno di un oggetto
al suo interno, al suo esterno o fino al centro, come le linee di una carta
geografica. È un effetto dinamico: modifica l'oggetto e il contorno
concentrico lo segue.

## Per aggiungere un contorno concentrico {#create}

Con lo strumento:

1. Scegli lo strumento [[tool.contour]] dal menu a comparsa Effetti.
2. Trascina da un oggetto: allontanandoti dal suo centro per un contorno
   esterno, verso il centro per uno interno. Più trascini, maggiore è la
   spaziatura.

Con il pannello:

1. Seleziona un oggetto.
2. Apri il pannello [[docker.contour]] con
   [[menu.effects > menu.effects.contour]] (<kbd>Ctrl+F9</kbd>).
3. Imposta le opzioni e fai clic su [[docker.apply]].

![Il pannello Contorno](../../img/effects-contour.png)

I gruppi e le bitmap non possono ricevere un contorno concentrico.

## Impostazioni del contorno concentrico {#settings}

La barra delle proprietà e il pannello mostrano le impostazioni del contorno
selezionato e lo modificano subito.

- **Verso il centro**, **Interno** o **Esterno** (in partenza).
- **Passaggi**: 3 in partenza, fino a 999. Verso il centro ne crea quanti ne
  entrano.
- **Spostamento**: la distanza tra i passaggi (2 mm in partenza).
- **Angoli**: a spigolo vivo (in partenza), arrotondati o smussati.
- **Direzione del colore**: diretta, in senso orario o antiorario attorno alla
  ruota dei colori, dai colori dell'oggetto a quelli dell'ultimo passaggio.
- I **colori di riempimento e di contorno dell'ultimo passaggio** (bianco e
  nero in partenza). Un oggetto senza riempimento dà passaggi senza
  riempimento.
- **Accelerazione degli oggetti** e **accelerazione del colore** (da -100 a
  100%) addensano i passaggi o i colori verso l'oggetto o verso l'ultimo
  passaggio.

I passaggi esterni vengono disegnati dietro l'oggetto, quelli interni davanti.

**Cancella contorno** rimuove il contorno concentrico; **Separa** trasforma i
passaggi in oggetti.
I passaggi escono raggruppati accanto all'oggetto. [[menu.object > menu.object.break_contour_apart]] (<kbd>Ctrl+K</kbd>,
anche nel menu del tasto destro) fa lo stesso.
