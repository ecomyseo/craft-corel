# Smussi

Uno smusso fa sembrare i bordi di un oggetto in rilievo o scolpiti,
illuminando un lato del bordo e ombreggiando l'altro. È un effetto dinamico,
impostato nel pannello [[docker.bevel]]; aprilo con
[[menu.effects > menu.effects.bevel]].

![Il pannello Smusso](../../img/effects-bevel.png)

## Per smussare un oggetto {#create}

1. Seleziona un oggetto con riempimento.
2. Nel pannello [[docker.bevel]], scegli lo stile:
   - [[docker.soft_edge]]: il bordo scende in pendenza verso l'interno
     dell'oggetto, illuminato da un lato e ombreggiato dall'altro;
   - [[docker.emboss]]: l'oggetto sembra sbalzato fuori dalla pagina.
3. Imposta l'aspetto dello smusso:
   - [[docker.depth]]: fin dove lo smusso arriva dal bordo, dentro l'oggetto
     per il bordo morbido e fuori per il rilievo;
   - [[docker.level]]: in quanti passi luce e ombra sono suddivise lungo lo
     smusso. 1 dà due toni piatti, più passi li sfumano dolcemente dal bordo
     verso l'interno;
   - [[docker.light_angle]]: la direzione da cui arriva la luce;
   - [[docker.intensity]]: la forza della luce e dell'ombra;
   - [[docker.light_color]] e [[docker.shadow_color]]: i colori del lato
     illuminato e di quello in ombra;
   - [[docker.emboss_color]]: il colore da cui è tonalizzato lo smusso.
     Spunta la casella e scegli un colore tuo; disattivato, si usa il
     riempimento dell'oggetto.
4. Fai clic su [[docker.apply]].

Per copiare uno smusso da un altro oggetto, usa
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] lo rimuove. Anche il pannello
dell'estrusione ha uno smusso per la faccia anteriore; vedi
[Estrusioni](extrude.md#settings).
