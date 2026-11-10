# Gestione del colore

Gli schermi mostrano i colori con luce rossa, verde e blu; le stampanti
mescolano inchiostri ciano, magenta, giallo e nero. La gestione del colore usa
i **profili colore** per descrivere ogni dispositivo, così un colore mantiene
il suo aspetto dallo schermo alla pagina stampata. Le impostazioni si trovano
in [[menu.tools > menu.tools.color_management]], la finestra di dialogo
[[color_mgmt.title]]. Nulla cambia finché non fai clic su [[dialog.ok]].

![La finestra di dialogo Impostazioni di gestione del colore](../../img/color-color-management.png)

## I profili del disegno {#document}

La scheda [[color_mgmt.tab_document]] imposta i profili del disegno attivo.

- [[color_mgmt.current_profiles]] mostra i profili RGB, CMYK e scala di grigi
  che il disegno usa ora.
- Scegli come viene applicato un nuovo profilo:
  - [[color_mgmt.assign]]: i valori dei colori restano e il loro aspetto
    cambia secondo il nuovo profilo;
  - [[color_mgmt.convert]]: i colori CMYK vengono convertiti in modo da
    apparire uguali nel nuovo profilo.
- Scegli i profili [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]] e
  [[color_mgmt.grayscale]]. Ogni elenco contiene i profili predefiniti, poi
  tutti i profili trovati sul computer. [[color_mgmt.load_profiles]] aggiunge
  file di profilo (`.icc`, `.icm`).
- [[color_mgmt.primary_mode]]: CMYK o RGB.
- [[color_mgmt.intent]]: come vengono riportati i colori che non possono
  essere mostrati: [[color_mgmt.intent_relative]] (in partenza),
  [[color_mgmt.intent_perceptual]], [[color_mgmt.intent_saturation]] o
  [[color_mgmt.intent_absolute]].

Il profilo CMYK accompagna il disegno, anche nei file `.cdr` a partire dalla
versione 16.

## Predefiniti per i nuovi disegni {#default}

La scheda [[color_mgmt.tab_default]] imposta cosa usano i nuovi disegni e come
vengono convertiti i colori.

- [[color_mgmt.preset]]: impostazioni già pronte per Nord America, Europa,
  Giappone, o per il lavoro web e a schermo. [[color_mgmt.save_preset]] salva
  le tue impostazioni con un [[color_mgmt.preset_name]];
  [[color_mgmt.delete_preset]] ne rimuove uno.
- I profili predefiniti [[color_mgmt.rgb_profile]],
  [[color_mgmt.cmyk_profile]] e [[color_mgmt.gray_profile]],
  [[color_mgmt.primary_mode]] e [[color_mgmt.intent]].
- [[color_mgmt.conversion]]:
  - [[color_mgmt.engine]]: [[color_mgmt.engine_icc]], che usa i profili, o
    [[color_mgmt.engine_builtin]];
  - [[color_mgmt.preserve_black]]: i colori composti solo da inchiostro nero
    restano solo inchiostro nero quando vengono convertiti;
  - [[color_mgmt.map_gray]] (attivo in partenza): i grigi convertiti in CMYK
    usano solo inchiostro nero;
  - [[dialog.black_point]] (attivo in partenza): mantiene distinte le
    sfumature più scure.
- [[color_mgmt.open_policy]]: cosa succede al profilo di un file che apri, per
  RGB, CMYK e scala di grigi: [[color_mgmt.open_use_embedded]],
  [[color_mgmt.open_assign_default]] o [[color_mgmt.open_convert_default]].
  [[color_mgmt.warn_mismatch_box]] e [[color_mgmt.warn_missing_box]] mostrano
  un messaggio nella barra di stato quando un file ha un altro profilo o
  nessuno.
- [[color_mgmt.import_policy]]: [[color_mgmt.import_convert]] o
  [[color_mgmt.import_assign]], per ciò che importi o incolli.
- [[color_mgmt.spot_definition]]: se i colori spot sono definiti da
  [[color_mgmt.spot_lab]], [[color_mgmt.spot_cmyk]] o
  [[color_mgmt.spot_rgb]].

## Anteprima dei colori di stampa {#proof}

[[menu.view > menu.view.proof_colors]], o il pulsante di prova all'estremità
destra della barra di stato, mostra il disegno come verrebbe stampato: i
colori che gli inchiostri CMYK non possono stampare appaiono più spenti. Anche
il pannello [[docker.color]] avvisa di tali colori.

Senza profili caricati, TraceDraw converte i colori con un proprio modello di
inchiostri di quadricromia su carta patinata.
