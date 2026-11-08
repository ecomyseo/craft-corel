# Novità

Ogni versione di TraceDraw aggiunge funzioni e correzioni. Questa pagina spiega
come scoprire cosa è cambiato ed elenca i punti salienti delle ultime versioni.

## Vedere le novità nel programma {#in-the-program}

- Scegliete [[menu.help > menu.help.whats_new]] per aprire la pagina delle
  novità della schermata di benvenuto. Elenca le ultime dieci versioni,
  dalla più recente, ognuna con la data e un titolo breve. Fate clic su una
  versione per leggere un breve riassunto di ciò che è cambiato; la più
  recente è già aperta. **Vuoi saperne di più?** sotto un riassunto apre la
  pagina di quella versione su GitHub, con l'elenco completo delle modifiche.
- Attivate [[menu.help > menu.help.highlight_whats_new]] per segnare nei menu
  ciò che è nuovo in questa versione: un piccolo punto compare alla fine di
  ogni nuovo comando, accanto alla freccia di un sottomenu che porta a uno, e
  sopra il titolo di un menu che ne contiene uno. I segni spariscono quando
  disattivate il comando, e il comando torna disattivato al prossimo avvio di
  TraceDraw.
- L'elenco completo delle modifiche di ogni versione si trova nel
  [registro delle modifiche](https://github.com/tracedraw/tracedraw/blob/main/CHANGELOG.md)
  del progetto.

## Punti salienti delle ultime versioni {#highlights}

- **Touch e telefoni.** I pulsanti Maiusc, Ctrl e Alt della barra standard
  restano premuti per un dito; su Android l'interfaccia evita il foro della
  fotocamera e gli angoli arrotondati; l'Apple Pencil funziona in Safari su
  iPad.
- **File.** I file .cdr conservano la regola di riempimento delle curve
  sovrapposte e si aprono con le loro tabulazioni, il testo su più colonne, le
  guide condivise da tutte le pagine e gli oggetti della pagina master.
- **Immagini, stampa e area di lavoro.** Ritaglia i pixel dell'immagine nello
  strumento Ritaglio e [[menu.bitmaps > menu.bitmaps.crop_to_frame]]
  ritagliano i pixel di un'immagine; stampa in scala di grigi e in ordine
  inverso; [[menu.view > menu.view.scrollbars]] nasconde le barre di
  scorrimento.
- **TraceCut.** [[menu.tools > menu.tools.tracecut]] apre gli strumenti per i
  plotter da taglio: uscita al plotter, linee di taglio, disposizione e
  misura. Vedi [TraceCut](../objects/tracecut.md).
- **Tastiera, telefoni e penne.** Menu ed elenchi funzionano dalla tastiera.
  Sui telefoni Android l'interfaccia sta nello schermo e una mano appoggiata
  non disegna più mentre si usa una penna; la penna di una tavoletta funziona
  su Linux con Wayland.
- **File.** I file .cdr si aprono con tabulazioni, pagine master, sfumature e
  distorsioni; le immagini dei PDF arrivano a piena risoluzione; JPEG e TIFF
  si esportano in CMYK.
- **Miniature degli oggetti.** Il pannello [[docker.objects]] mostra ogni
  oggetto disegnato in piccolo accanto al nome; Ctrl+rotella sulla lista o il
  cursore in basso imposta la dimensione. Vedere
  [Organizzare oggetti e livelli](../objects/objects-and-layers.md#objects-docker).
- **La Guida nella vostra lingua.** Ogni pulsante **?** e <kbd>F1</kbd> aprono
  la pagina di questa Guida sulla finestra di dialogo, il pannello o lo
  strumento che avete davanti, nella lingua impostata in TraceDraw.
- **Proprietà del documento.**
  [[menu.file > menu.file.document_properties]] conserva lingua, titolo,
  oggetto, autore, copyright, parole chiave, note e valutazione di un disegno,
  sopra un rapporto completo di ciò che il disegno contiene. Vedere
  [Proprietà del documento](../pages-layout/document-properties.md).
- **Gestione del colore.** [[menu.tools > menu.tools.color_management]]
  imposta i profili del disegno e quelli predefiniti per i nuovi, con regole
  per i file che vengono aperti o importati. Vedere
  [Gestione del colore](../color/color-management.md).
- **Codici a barre e codici QR.** Una procedura guidata in tre passi colloca
  codici a barre, e i codici QR sono oggetti dinamici che si modificano nel
  pannello [[docker.properties]]. Vedere
  [Codici a barre e codici QR](../objects/barcodes.md).
- **Effetti.** Il menu [[menu.effects]] raggruppa i pannelli degli effetti; le
  fusioni possono seguire un tracciato con spaziatura fissa e ruotare i colori
  in senso orario o antiorario. Vedere [Fusioni](../effects/blend.md).
- **Il testo su tracciato** ha impostazioni proprie per orientamento,
  posizionamento, distanza e scostamento. Vedere
  [Aggiungere e modificare testo](../text/adding-text.md).
- **Gestione font** organizza i font installati, le cartelle, i preferiti e le
  raccolte. Vedere [Font e glifi](../text/fonts.md).
