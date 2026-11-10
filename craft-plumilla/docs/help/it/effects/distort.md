# Distorsione

Lo strumento [[tool.distort]] piega il contorno di un oggetto in tre modi:
**spingi e tira** (punte o petali), **cerniera** (denti a zigzag) e
**vortice** (un mulinello). È un effetto dinamico: puoi cambiare, spostare o
rimuovere la distorsione in qualsiasi momento, e l'oggetto mantiene il proprio
contorno.

## Per distorcere un oggetto {#create}

1. Scegli lo strumento [[tool.distort]] dal menu a comparsa Effetti.
2. Scegli il tipo nella barra delle proprietà.
3. Trascina sull'oggetto. La distorsione è centrata nel punto da cui parti.
   - **Spingi e tira**: trascina verso destra per spingere i nodi verso
     l'esterno (forme appuntite con i lati incavati), verso sinistra per
     tirarli verso l'interno (petali arrotondati).
   - **Cerniera**: più trascini, più alti sono i denti.
   - **Vortice**: trascina attorno al punto di partenza; i giri completi
     contano.

Il centro di una distorsione selezionata appare come un rombo: trascinalo per
spostare il centro. Trascinare di nuovo sostituisce la distorsione; un oggetto
ne ha al massimo una. I gruppi e le bitmap non vengono distorti.

## Impostazioni della distorsione {#settings}

La barra delle proprietà mostra e modifica la distorsione selezionata:

- il tipo: **Spingi e tira**, **Cerniera** o **Vortice** (scegliere un altro
  tipo cambia la distorsione selezionata);
- per spingi e tira, l'**ampiezza** (da -200 a 200, 20 in partenza);
- per la cerniera, l'**ampiezza** (da 0 a 100 mm, 2 mm in partenza) e la
  **frequenza** (denti per segmento, 6 in partenza), con **Casuale** (denti
  irregolari), **Morbida** (denti arrotondati) e **Locale** (i denti si
  riducono allontanandosi dal centro);
- per il vortice, **in senso orario** o **in senso antiorario**, il numero di
  **rotazioni** complete e i **gradi** aggiuntivi.

[[menu.object > menu.object.clear_effect]] rimuove la distorsione.
