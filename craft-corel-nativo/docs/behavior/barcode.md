# Barcode Wizard

Object > Insert > Barcode opens the Barcode Wizard: a picture on the
left, three steps on the right, a sample preview under each step that
shows the symbol as it will be made, and Back, Next (Finish on the last
step), Cancel and ? at the bottom. Next checks the data; data a
symbology cannot hold keeps the wizard on its step with a message.
Finish places the symbol at the page centre on the active layer, as one
group (bars, text, and the white background of the quiet zone when one
is asked for), selects it, and is one undo step.

## Step 1: the standard and the data

- "Select one of the following industry standards": Codabar, Code 39,
  Code 128, EAN-13, EAN-8, UPC-A, UPC-E, Interleaved 2 of 5. Choosing
  another standard puts its sample data in the field.
- Under it, what the standard accepts:
  - Codabar: up to 30 digits and - $ : / . +, between start and stop
    characters A to D (A and B when the data gives none).
  - Code 39: capital letters (lower case is raised), digits and
    - . space $ / + %.
  - Code 128: printable ASCII; runs of digits use the compact digit set.
  - EAN-13, EAN-8, UPC-A: 12, 7 or 11 digits (one more with the check
    digit, which must then be right), optionally a space and a 2- or
    5-digit add-on.
  - UPC-E: 6 digits, or the number system (0 or 1) and 6 digits, with
    or without the check digit.
  - Interleaved 2 of 5: up to 40 digits; a leading zero makes the count
    even.

## Step 2: printer and sizes

- Printer: 150, 203, 300 (default), 600, 1200 or 2400 dpi. The narrow
  bar is a whole number of printer dots: 0.33 mm times the
  magnification, rounded to dots (at least one).
- Units: inches, millimetres, centimetres or points, for the bar height
  and the symbol width shown beside it.
- Bar width reduction: printer dots taken off every bar for ink spread
  (1 by default); the spaces grow by the same amount and the symbol
  keeps its width.
- Magnification (scale): 100 % is the nominal size.
- Bar height (1 inch by default), with "Symbol width" of the result.
- Wide to narrow: 2.0 to 3.0, for Codabar, Code 39 and Interleaved 2 of
  5 (greyed for the others).
- Advanced... shows: Add the check character (Code 39 modulo 43,
  Interleaved 2 of 5 modulo 10) and Quiet zone, in narrow modules: a
  white rectangle behind the symbol reaching that far past the bars (0,
  none, by default).

## Step 3: text

- Font (the installed and carried families), Size in points (12),
  Weight (Normal, Normal-Italic, Bold, Bold-Italic) and Alignment (Left,
  Center, Right) of the human-readable text.
- Options, each greyed where the standard has no use for it:
  - Center sys/check (EAN-13, UPC-A, UPC-E): the system digit before the
    bars, the digit groups under the halves, the UPC check digit after
    the bars. Off: one line of digits.
  - Place text above: the text over the bars instead of under them.
  - Show add-on text and Add-on text at bottom (EAN and UPC): the
    add-on's digits over its bars (shorter, to make room) or under them.
  - Show FACT data (Code 39 with its check character): the check
    character in the text.
  - Show asterisks (Code 39): the * start and stop characters in the
    text.
  - Show quiet zone marks (EAN-13, EAN-8): > after the symbol, and < before
    an EAN-8.
  - Show start/stop (Codabar): the A to D characters in the text.
- "Make this bar code human readable (show text)": off, no text at all.
- EAN and UPC guard bars reach down into the digits' line when the text
  is under the bars.

## Checks

- Known check characters: EAN-13 590123412345 gives 7, Code 39 CODE39
  gives W, Interleaved 2 of 5 1234567 gives 0; widths: EAN-13 95
  modules, EAN-8 67, UPC-A 95, UPC-E 51, add-ons 9 + 20 or 9 + 47; Code
  39 characters have three wide elements of nine (`barcode::tests`).
- The printer resolution, magnification, bar height, bar width reduction
  and ratio change the bars as described; every text option changes the
  labels (`barcode::tests::printer_settings_size_the_bars`,
  `text_options_change_the_labels`,
  `retail_symbols_group_their_digits`).
- Enter walks the steps and Finish inserts a group of bars and text; bad
  data stays on step 1 with a message; the text takes the font, size and
  weight, and a quiet zone adds the background
  (`ui::barcode_wizard::tests`).
