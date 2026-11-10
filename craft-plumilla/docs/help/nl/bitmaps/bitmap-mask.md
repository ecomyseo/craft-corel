# Kleuren uit bitmaps maskeren

Het paneel [[docker.bitmap_mask]] verbergt of behoudt de pixels van gekozen
kleuren in een bitmap, bijvoorbeeld om een witte achtergrond transparant te
maken. Open het met [[menu.bitmaps > menu.bitmaps.bitmap_mask]].

![Het paneel Bitmapmasker](../../img/bitmaps-bitmap-mask.png)

## Kleuren maskeren {#mask}

1. Selecteer een bitmap.
2. Kies kleuren voor de tien kleurvakken:
   - klik op de pipet en klik op kleuren in de bitmap; elke kleur vult het
     geselecteerde vak en vinkt het aan;
   - of klik op **Kleuren uit bitmap kiezen** om de vakken te vullen met de
     tien kleuren die de bitmap het meest gebruikt.
3. Selecteer een vak en stel de **tolerantie** in (0 tot 100%): hoe ver de
   kleur van een pixel van de kleur van het vak mag afwijken om toch overeen
   te komen.
4. Kies **Geselecteerde verbergen** (overeenkomende pixels worden
   transparant) of **Geselecteerde tonen** (alleen overeenkomende pixels
   blijven, in het begin).
5. Klik op [[docker.apply]].

**Masker opslaan** en **Masker openen** bewaren de vakken in een `.msk`-bestand;
**Verwijderen** maakt het geselecteerde vak leeg. Elke bitmap verandert in één
ongedaan-makenstap.
