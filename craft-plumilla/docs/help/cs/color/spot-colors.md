# Přímé barvy

**Přímá barva** je pojmenovaná tisková barva, která se tiskne z vlastní
desky, například metalická nebo firemní barva, místo aby se míchala z
azurové, purpurové, žluté a černé. Každá kresba si vede seznam svých přímých
barev. Každá barva má náhradu složenou z procesních barev, která slouží k
jejímu zobrazení na obrazovce a všude, kde přímou barvu nelze použít.

## Přidání přímé barvy {#add}

1. Otevřete panel [[docker.color]] a namíchejte barvu, jak má přímá barva
   vypadat.
2. Pod [[docker.spot_inks]] zadejte [[docker.ink_name]].
3. Klikněte na [[docker.new_ink]].

Názvy jsou v rámci kresby jedinečné.

## Používání přímých barev {#use}

Seznam [[docker.spot_inks]] ukazuje každou barvu se vzorkem, jejím názvem a
tlačítky [[docker.fill]] a [[docker.outline]], která ji použijí na výběr v
nastaveném odstínu [[docker.tint]] (0 až 100 %, výchozí 100 %). Křížek na
konci řádku barvu ze seznamu odebere; objekty, které ji používají, si
ponechají její náhradní barvu.

## Vzorníky barev {#color-books}

Soubor palety může být vzorníkem přímých barev (například paleta `.ase` s
přímými položkami). Jeho přímé položky se otevřou jako přímé barvy,
pojmenované jako ve vzorníku; použitím jedné z nich se barva přidá do kresby.
TraceDraw nedodává licencované vzorníky barev, ale vzorník, který vlastníte,
se otevře jako jakákoli paleta.

## Přímé barvy v tisku a exportech {#output}

- V souborech PDF se každá barva stane separační barvou pojmenovanou podle
  ní, takže tiskárna barvu dostane.
- Při tisku separací dostane každá barva vlastní desku po deskách azurové,
  purpurové, žluté a černé. Viz [Tisk](../printing/print.md).
- Na obrazovce a v bitmapách, souborech SVG a `.cdr` se přímá barva zobrazí
  jako svá náhrada v daném odstínu.
