# Riempimenti a motivo

Un riempimento a motivo ripete una tessera all'interno di un oggetto. La
tessera può essere di due colori, un insieme di oggetti vettoriali (un motivo
vettoriale a colori pieni) o una bitmap. Le tessere sono ancorate alla
pagina, quindi non scivolano quando sposti l'oggetto.

## Creare un motivo dai tuoi oggetti {#create-pattern}

1. Disegna gli oggetti che formano una tessera e selezionali.
2. Scegli [[menu.object > menu.object.create > menu.object.create_pattern]].
3. Nella finestra di dialogo [[dialog.create_pattern]], scegli il
   [[dialog.create_pattern_type]]: [[dialog.create_pattern_vector]] mantiene
   gli oggetti come vettori, nitidi a qualsiasi dimensione;
   [[dialog.create_pattern_bitmap]] ne crea un'immagine.
4. Fai clic su [[dialog.ok]].

![La finestra di dialogo Crea motivo](../../img/color-patterns-create-pattern.png)

Il motivo diventa il riempimento dei nuovi oggetti. Per darlo agli oggetti
già disegnati, scegli [[docker.fill_pattern]] nella sezione riempimento del
pannello [[docker.properties]].

## Impostazioni del motivo {#settings}

Nella sezione riempimento del pannello [[docker.properties]], scegli
[[docker.fill_pattern]], poi il tipo di motivo e le sue impostazioni. Per un
motivo vettoriale, il pannello mostra quanti oggetti contiene la tessera e la
sua larghezza e altezza: modificarle scala con esse il contenuto della
tessera.

## Motivi nei file esportati {#export}

I file SVG mantengono i motivi vettoriali come vettori. I file PDF ed EPS li
ricevono come immagini alla risoluzione di esportazione, ritagliate
sull'oggetto.
