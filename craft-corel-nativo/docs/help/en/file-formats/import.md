# Importing files

[[menu.file > menu.file.import]] (<kbd>Ctrl+I</kbd>) adds the content of a
file to the drawing that is open, on the active layer, as one undo step.
[[menu.file.open]] instead opens a file as a drawing of its own. Both read
the formats listed in [Supported file formats](supported-formats.md#open).

## Where imported objects go {#placement}

At first, imported objects go where the file has them. To center them in the
view instead, change the setting on the [[options.file_formats]] page of the
global options; see [Global options](../customize/global-options.md#file-formats).

## Notes on some formats {#formats}

- **PDF and AI**: the [[pdf_import.title]] dialog box asks how to read the
  text: as editable text, which keeps its fonts when they are installed, or as
  curves, drawn from the fonts the file embeds so the text looks exactly as
  it does in the file. Options > File Formats keeps the choice. A protected
  file asks for its password in the [[pdf_password.title]] dialog box.
- **EPS and PostScript**: each printed page becomes a page.
- **PSD**: each layer becomes a bitmap with its opacity, visibility and mask.
- **DXF**: drawing units are taken from the file.
- **Text files** (TXT, RTF, DOCX): become paragraph text, with bold, italic,
  sizes and fonts where the file has them.
- **Bitmaps**: placed at their resolution; see [Bitmaps](../bitmaps/index.md).

When some objects of a file cannot be read, or fonts are missing, a message
says so; see [Notes after opening](../get-started/open-and-save.md#open-notes).
You can also drag files onto the window, or paste from other programs; see
[Paste Special](../objects/paste-special.md).

## PDF files of several pages {#pdf-pages}

Importing a PDF or AI file of more than one page opens the
[[pdf_pages.title]] dialog box, after the questions about its text and its
password. Choose [[pdf_pages.all]] or type the pages, such as `1-3, 5`; the
list shows each page with its size and orientation, and ticking a page adds
it. Then choose where they go:

- [[pdf_pages.new_pages]]: each page becomes a page of the drawing, with its
  own size and orientation, in order. This merges PDF files into one drawing.
- [[pdf_pages.current_page]]: the objects go on the active layer; with
  several pages, each page's objects are a group.

The whole import is one undo step. A file of one page comes in without the
dialog box.
