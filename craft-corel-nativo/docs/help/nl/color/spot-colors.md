# Steunkleuren

Een **steunkleur** is een benoemde inkt die op een eigen drukplaat wordt
gedrukt, zoals een metallic of een huiskleur, in plaats van te worden gemengd
uit cyaan, magenta, geel en zwart. Elke tekening houdt een lijst bij van haar
steunkleurinkten. Elke inkt heeft een vervanger uit proceskleuren, die wordt
gebruikt om haar op het scherm te tonen en overal waar de inkt niet gebruikt
kan worden.

## Een steunkleurinkt toevoegen {#add}

1. Open het paneel [[docker.color]] en meng de kleur waar de inkt op moet
   lijken.
2. Typ onder [[docker.spot_inks]] een [[docker.ink_name]].
3. Klik op [[docker.new_ink]].

Namen zijn uniek binnen een tekening.

## Steunkleurinkten gebruiken {#use}

De lijst [[docker.spot_inks]] toont elke inkt met een staal, de naam en de
knoppen [[docker.fill]] en [[docker.outline]], die de inkt op de selectie
toepassen met de [[docker.tint]] die u instelt (0 tot 100%, in het begin
100%). Het kruisje aan het einde van een rij verwijdert de inkt uit de lijst;
objecten die haar gebruiken, behouden de vervangende kleur.

## Kleurenboeken {#color-books}

Een paletbestand kan een boek met steunkleurinkten zijn (bijvoorbeeld een
`.ase`-palet met steunkleuritems). De steunkleuritems openen als steunkleuren,
met de namen uit het boek; als u er een toepast, wordt de inkt aan de tekening
toegevoegd. TraceDraw levert geen gelicentieerde kleurenboeken mee, maar een
boek dat u bezit, opent als elk ander palet.

## Steunkleuren bij afdrukken en exporteren {#output}

- In PDF-bestanden wordt elke inkt een scheidingskleur met dezelfde naam,
  zodat de drukkerij de inkt krijgt.
- Als u kleurscheidingen afdrukt, krijgt elke inkt een eigen plaat na de
  platen voor cyaan, magenta, geel en zwart. Zie
  [Afdrukken](../printing/print.md).
- Op het scherm en in bitmap-, SVG- en `.cdr`-bestanden wordt een steunkleur
  getoond als de vervanger ervan op de ingestelde tint.
