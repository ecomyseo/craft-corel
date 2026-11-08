# QR codes

Object > Insert > QR Code places a QR code object at the centre of the
page at once, selected, and opens Properties. The object is a group of
two children made from its settings: the background (a square over the
code and a quiet zone of 2 pixels on every side) and the dark pixels
(one path, each row's runs of dark pixels a rectangle). Ungroup turns it
into those two plain objects.

## Properties

With a QR code object selected, Properties shows a QR Code section first
(its button a small QR picture):

- The kind of content: URL, Email, Phone number, SMS, Contact (vCard),
  Text or Location, and its fields. What the code holds:
  - URL and Text: the text as typed.
  - Email: `MATMSG:TO:address;SUB:subject;BODY:message;;`.
  - Phone number: `tel:number`.
  - SMS: `SMSTO:number:message`.
  - Contact: a vCard 3.0 with N, FN and, when filled in, ORG, TEL, EMAIL,
    URL and ADR.
  - Location: `geo:latitude,longitude`.
  Changing the kind keeps the fields both kinds share (the phone number
  from Phone number to SMS); the others start empty. A field is applied
  when it loses the focus. A kind whose fields make no content yet (Email
  picked on a URL code: nothing carries over) stays picked, its fields
  ready to fill in, with "Enter the text to encode." in red; the object
  keeps its code, and nothing goes into the history, until the fields
  make content, which makes the code of the new kind (#1845). Emptying
  the fields works the same way.
- Pixel fill and Background fill (any fill), Pixel outline (none at
  first), Pixel size (2.5 mm at first, 0.1 to 50 mm).
- Validate: draws the code with its fills' brightness and reads it back
  with a QR decoder; green "The code reads back correctly.", or red when
  the pixels are not clearly darker than the background, nothing
  readable is found, or it reads as other content.

Each change makes the children again at the same top-left corner, in one
undo step ("QR Code"); the group's transform (moved, scaled, rotated)
stays. Content that cannot be encoded (nothing filled in, too much for a
code) changes nothing and is shown in red.

New objects encode `https://github.com/tracedraw/tracedraw`, black on white.

## Files

- `.tdraw`: the group keeps its settings in `qr` (kind, fields,
  `pixel_mm`, `dark`, `light`, `outline`); older files without it read
  as before.
- `.cdr` and other formats: written as the group of its two shapes.

## Checks (tests in `qr.rs`)

- Insert places a selected group of two children whose code reads back
  as its content.
- Editing the kind, fields, pixel size and fill makes the code again at
  the same corner with the new fill and side, in one undo step; empty
  content changes nothing.
- Each kind writes the form above, and a code of every kind reads back;
  fields shared by name carry over.
- Every kind can be picked from a new URL code and filled in field by
  field; until its fields make content the object keeps its code with
  no undo step (`every_kind_can_be_picked_and_filled_in`, #1845).
- Validate reports grey on white and white on black as unreadable.
- The settings survive `.tdraw`; a version 27 `.cdr` reads back the
  group of two shapes.
