# Mascherare i colori delle bitmap

Il pannello [[docker.bitmap_mask]] nasconde o mantiene i pixel dei colori
scelti in una bitmap, ad esempio per rendere trasparente uno sfondo bianco.
Aprilo con [[menu.bitmaps > menu.bitmaps.bitmap_mask]].

![Il pannello Maschera bitmap](../../img/bitmaps-bitmap-mask.png)

## Per mascherare i colori {#mask}

1. Seleziona una bitmap.
2. Inserisci i colori nelle dieci caselle di colore:
   - fai clic sul contagocce e poi sui colori della bitmap; ciascuno riempie
     la casella selezionata e la spunta;
   - oppure fai clic su **Seleziona colori dalla bitmap** per riempire le
     caselle con i dieci colori più usati nella bitmap.
3. Seleziona una casella e imposta la sua **tolleranza** (da 0 a 100%): quanto
   il colore di un pixel può discostarsi dal colore della casella e
   corrispondere comunque.
4. Scegli **Nascondi selezionati** (i pixel corrispondenti diventano
   trasparenti) o **Mostra selezionati** (restano solo i pixel corrispondenti,
   in partenza).
5. Fai clic su [[docker.apply]].

**Salva maschera** e **Apri maschera** conservano le caselle in un file `.msk`;
**Elimina** svuota la casella selezionata. Ogni bitmap cambia in un unico
passaggio di annullamento.
