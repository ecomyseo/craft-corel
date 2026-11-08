# Patroonvullingen

Een patroonvulling herhaalt een tegel over de binnenkant van een object. De
tegel kan uit twee kleuren bestaan, uit een set vectorobjecten (een
vectorpatroon in volle kleur) of uit een bitmap. Tegels zijn aan de pagina
verankerd, zodat ze niet verschuiven als u het object verplaatst.

## Een patroon maken van uw objecten {#create-pattern}

1. Teken de objecten die samen één tegel vormen en selecteer ze.
2. Kies [[menu.object > menu.object.create > menu.object.create_pattern]].
3. Kies in het dialoogvenster [[dialog.create_pattern]] het
   [[dialog.create_pattern_type]]: [[dialog.create_pattern_vector]] houdt de
   objecten als vectoren, scherp op elke grootte;
   [[dialog.create_pattern_bitmap]] maakt er een afbeelding van.
4. Klik op [[dialog.ok]].

![Het dialoogvenster Patroon maken](../../img/color-patterns-create-pattern.png)

Het patroon wordt de vulling van nieuwe objecten. Om het te geven aan objecten
die u al getekend hebt, kiest u [[docker.fill_pattern]] in het vulonderdeel van
het paneel [[docker.properties]].

## Patrooninstellingen {#settings}

Kies in het vulonderdeel van het paneel [[docker.properties]]
[[docker.fill_pattern]] en daarna de soort patroon en de instellingen ervan.
Voor een vectorpatroon toont het paneel hoeveel objecten de tegel bevat en de
breedte en hoogte ervan: als u die wijzigt, wordt de inhoud van de tegel
meegeschaald.

## Patronen in geëxporteerde bestanden {#export}

SVG-bestanden behouden vectorpatronen als vectoren. PDF- en EPS-bestanden
krijgen ze als afbeeldingen op de exportresolutie, bijgesneden tot het object.
