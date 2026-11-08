# Bitmaps herbemonsteren, rechtzetten en vergroten

## Herbemonsteren {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] geeft de geselecteerde bitmaps een
nieuw aantal pixels; hun grootte op de pagina blijft gelijk.

![Het dialoogvenster Herbemonsteren](../../img/bitmaps-resample-and-straighten-resample.png)

1. Typ de nieuwe grootte in **pixels**, als **percentage** of als de
   **resolutie** die dat geeft bij de grootte op de pagina. Met
   **Hoogte-breedteverhouding behouden** aan volgt de hoogte de breedte.
2. Kies de methode:
   - **Vloeiend** (foto's), in het begin;
   - **Scherp**, iets scherper;
   - **Harde pixels** (pixelkunst): elke pixel blijft een vierkant van één kleur;
   - **Vergroten met detail**: vloeiend, daarna verscherpt als de afbeelding
     groter wordt.
3. Klik op [[dialog.ok]].

Het dialoogvenster toont de oorspronkelijke en de nieuwe grootte en het
geheugen dat de nieuwe afbeelding inneemt. Als er meerdere bitmaps
geselecteerd zijn, wordt elke bitmap met dezelfde factor geschaald.

## Afbeelding rechtzetten {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] corrigeert foto's die scheef
staan, onder een hoek zijn genomen of door de lens zijn vervormd.

![Het dialoogvenster Afbeelding rechtzetten](../../img/bitmaps-resample-and-straighten-straighten.png)

1. Gebruik het **Raster** (grootte en kleur zijn instelbaar) om de afbeelding
   uit te lijnen.
2. Stel de schuifregelaars in: **Lensvervorming corrigeren** (-100 tot 100),
   **Afbeelding draaien** (-15 tot 15 graden), **Verticaal perspectief** en
   **Horizontaal perspectief** (-100 tot 100). De draaiknoppen draaien de
   afbeelding eerst een kwartslag.
3. Met **Afbeelding bijsnijden** aan wordt het resultaat bijgesneden tot de
   grootste rechthoek met de oorspronkelijke verhoudingen; **Bijsnijden en
   herbemonsteren naar oorspronkelijke grootte** brengt het terug naar het
   oorspronkelijke aantal pixels. Zonder bijsnijden zijn de hoeken transparant.
4. Klik op [[dialog.ok]]. **Herstellen** zet alle instellingen terug.

Zoom en pan in het voorbeeld met de gereedschappen erboven. Het resultaat
vervangt de pixels, dus effecten op de bitmap worden definitief.

## Bitmaps vergroten {#inflate}

Sommige effecten, zoals vervagingen, reiken tot voorbij de randen van een
bitmap.

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]] (in het
  begin aan) vergroot bitmaps met wat hun effecten nodig hebben, met een
  transparante marge.
- [[menu.bitmaps.inflate_manual]] vergroot de geselecteerde bitmaps met de
  hand: typ voor de breedte en hoogte de nieuwe grootte in pixels of als
  percentage (100 tot 1000%), met **Hoogte-breedteverhouding behouden**. De
  afbeelding blijft gecentreerd.

![Het dialoogvenster Bitmap handmatig vergroten](../../img/bitmaps-resample-and-straighten-inflate.png)

## Andere bitmapopdrachten {#other}

- [[menu.bitmaps.crop_bitmap]] snijdt een bitmap bij met het gereedschap
  [[tool.crop]]; een hoek van een bitmap slepen met het gereedschap
  [[tool.shape]] snijdt hem ook bij.
- [[menu.bitmaps.edit_bitmap]] opent de bitmap in een ander programma.
- [[menu.bitmaps.remove_background]] maakt de achtergrond van een foto
  transparant.
- [[menu.bitmaps.remove_background_local]]: [Achtergrond verwijderen](remove-background.md).
- [[menu.bitmaps.break_link]] en [[menu.bitmaps.update_from_link]] werken op
  bitmaps die aan een bestand gekoppeld zijn.
- [[menu.bitmaps.save_bitmap_as]] slaat een bitmap op als PNG-bestand.
