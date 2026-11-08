# Kleurstijlen

Een **kleurstijl** is een benoemde kleur die door objecten van de tekening
wordt gebruikt. Wijzig de stijl en elk object dat ermee gevuld is, verandert
mee. Zo blijven de kleuren van een ontwerp consistent en kunt u snel andere
kleurenschema's uitproberen. De stijlen staan in het paneel
[[docker.color_styles]]; open het via [[menu.window > menu.window.dockers]].

![Het paneel Kleurstijlen](../../img/color-color-styles.png)

## Kleurstijlen maken {#create}

- [[docker.new_from_selection]] voegt een stijl toe voor elke vulkleur van de
  geselecteerde objecten die nog geen stijl is.
- [[docker.new_color_style]] voegt een stijl toe met de huidige kleur van het
  paneel [[docker.color]].

## Stijlen gebruiken en bewerken {#edit}

Elke stijl toont zijn kleur, zijn naam en drie knoppen:

- klik op de kleur om die te wijzigen: de objecten die met die kleur gevuld
  zijn, volgen;
- klik op de naam om de stijl een andere naam te geven;
- [[docker.fill]] vult de selectie met de stijl, [[docker.outline]] kleurt de
  omtrek ervan;
- **x** verwijdert de stijl (de objecten behouden hun kleur).

## Harmonieën {#harmonies}

De [[docker.harmony_editor]] maakt een reeks stijlen die bij elkaar passen,
uitgaande van een [[docker.base_color]]:

1. Kies de [[docker.base_color]].
2. Klik op een harmonie: [[harmony.analogous]], [[harmony.complementary]],
   [[harmony.split_complementary]], [[harmony.triadic]],
   [[harmony.tetradic]] of [[harmony.monochromatic]].

De kleuren van de harmonie worden als stijlen toegevoegd, genoemd naar de
harmonie.
