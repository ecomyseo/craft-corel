# Cronometro attività

Il cronometro attività registra il tempo che dedichi a un disegno, attività
per attività, e conserva le registrazioni nel disegno (nei file `.tdraw`). Il
tempo viene contato solo mentre lavori: dopo alcuni minuti di inattività
l'orologio si mette in attesa e la tua azione successiva lo fa ripartire.

## La barra degli strumenti Cronometro attività {#toolbar}

[[menu.window > menu.window.toolbars > menu.window.toolbar_time_tracker]]
mostra una barra con:

- l'elenco delle attività, per scegliere l'attività attiva o aggiungerne una
  nuova;
- il tempo dell'attività attiva;
- il pulsante di registrazione e pausa;
- una spia: rossa durante il conteggio, ambra durante l'inattività, grigia
  quando in pausa o senza attività;
- il totale di tutte le attività;
- un pulsante che apre la finestra di dialogo.

Avviare l'orologio su un disegno senza attività aggiunge "Attività 1".

## La finestra di dialogo Cronometro attività {#dialog}

[[menu.tools > menu.tools.time_tracker]] elenca le attività con il loro tempo,
inizio e fine, e il totale.

![La finestra di dialogo Cronometro attività](../../img/customize-time-tracker.png)

- **Nuova attività**, **Elimina**, **Rendi attiva** e **Azzera tempo** agiscono
  sull'attività selezionata. Fai doppio clic su un'attività per renderla
  attiva.
- **Registra** e **Pausa** avviano e fermano l'orologio.
- Modifica il **nome**, il **tempo** (ad esempio `1:30:00`, `1:30` o `1.5`
  ore), l'**inizio** e la **fine** (ad esempio `2026-10-19 14:30`)
  dell'attività selezionata.
- **Interrompi il conteggio dopo N minuti senza input** imposta il tempo di
  inattività (5 minuti in partenza).
- **Esporta CSV** ed **Esporta testo** salvano le registrazioni, per un foglio
  di calcolo o un resoconto.

Registrare il tempo non è una modifica del disegno: non crea alcun passaggio
di annullamento. Le registrazioni vengono salvate al successivo salvataggio
del disegno.
