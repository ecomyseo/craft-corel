# Spot colors

A **spot color** is a named ink that is printed on a plate of its own, such
as a metallic or a brand color, instead of being mixed from cyan, magenta,
yellow and black. Each drawing keeps a list of its spot inks. Every ink has a
stand-in made of process colors, used to show it on screen and wherever the
ink cannot be used.

## Adding a spot ink {#add}

1. Open the [[docker.color]] panel and mix the color the ink should look
   like.
2. Under [[docker.spot_inks]], type an [[docker.ink_name]].
3. Click [[docker.new_ink]].

Names are unique within a drawing.

## Using spot inks {#use}

The [[docker.spot_inks]] list shows each ink with a swatch, its name, and
[[docker.fill]] and [[docker.outline]] buttons, which apply it to the
selection at the [[docker.tint]] you set (0 to 100%, 100% at first). The
cross at the end of a row removes the ink from the list; objects that use it
keep its stand-in color.

## Color books {#color-books}

A palette file can be a book of spot inks (for example an `.ase` palette with
spot entries). Its spot entries open as spot colors, named as in the book;
applying one adds the ink to the drawing. TraceDraw does not ship licensed
color books, but one you own opens like any palette.

## Spot colors in print and exports {#output}

- In PDF files, each ink becomes a separation color named after it, so the
  print shop gets the ink.
- When you print separations, each ink gets a plate of its own after the
  cyan, magenta, yellow and black plates. See [Printing](../printing/print.md).
- On screen and in bitmap, SVG and `.cdr` files, a spot color shows as its
  stand-in at its tint.
