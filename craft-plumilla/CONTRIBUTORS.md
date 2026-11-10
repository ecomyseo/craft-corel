# Contributors

TraceDraw is written by its maintainers, with Claude Code as co-author (see the
README). These people contributed code that is part of TraceDraw. Thank
you!

Everyone whose code is merged also appears in GitHub's contributor list:
when a contribution was recorded under another name (for example by the
coding assistant the contributor used), it is re-applied under the
contributor's own GitHub identity.

| Contributor | Contributions |
|---|---|
| [@SofiaAFigueredo](https://github.com/SofiaAFigueredo) | Save As starts with Embed fonts off (#901, for #886); a contour on text keeps the CPU low while dragging and pointing (#906, for #897); Area Fill closes hairline gaps between lines that nearly meet (#899); Bevel: Emboss draws its band around the outline, Level and Emboss colours, and its panel (#1135); a group's uniform transparency is drawn, exported and kept on Ungroup and in `.cdr` (#1386); a locked guideline shows no rotation handles (#1385); the multipage view repaints the pages beside the current one (#1384) |
| [@belchiordameao-cmd](https://github.com/belchiordameao-cmd) | Publish to PDF leaves out pictures wholly outside the page and its bleed (#648, for #642); `.cdr` pages up to 100 m on a side keep their size (#651); `tracedraw-cli convert` reports what the `.cdr` reader skipped or guessed (#653) |
| [@LaMka21](https://github.com/LaMka21) | Moving goes on after a right click during a drag (#174); files dropped on the window are opened or imported (#177); Paste takes what was copied last, files and pictures included (#183) |

## Other pull requests

Pull requests that were not merged, or are still in review, are work given
to the project too:

| Contributor | Pull requests |
|---|---|
| [@zalmanleib](https://github.com/zalmanleib) | Click to select with drawing tools, brushes on the selection only, alignment keys while typing, right-to-left text (#271, in review) |
| [@bfs2021](https://github.com/bfs2021) | Repeat after Undo uses the last action still applied (#372; closed by its author in favour of the same fix on main) |
| [@LaMka21](https://github.com/LaMka21) | traceCUT sign making and cutting tools (#170, #245; kept for later), page tab right-click menu (#186; covered by #199 and #210) |
| [@belchiordameao-cmd](https://github.com/belchiordameao-cmd) | PDF/X with the drawing's CMYK profile (#649), outlines that scale with their object in PDF and SVG (#656), `.cdr` point lists of type 0x26 (#657); in review |

## Testing and reports

People who report bugs, send sample files and test against the reference
program help just as much; they are thanked in the issues and in
[`CHANGELOG.md`](CHANGELOG.md). Some went well beyond one report:

| Contributor | Help |
|---|---|
| [@archit995sipah-ai](https://github.com/archit995sipah-ai) | More than 130 reports that led to fixes, across printing and booklets, text and Hindi text in PDF, blends, fills, page navigation and `.cdr` files that keep their colours and pictures (#378, #771, #947, #986, #1057, #1268) |
| [@muhammadawais052400](https://github.com/muhammadawais052400) | More than 100 precise reports with steps to repeat them: the Pen tool, node and segment editing, Arabic (right-to-left) text, frame rates with many shapes and in the multipage view, Repeat after Undo (#240, #253, #359, #366, #474, #903) |
| [@2CLC](https://github.com/2CLC) | Tested build after build on a 2010 office PC (Intel HD Graphics, OpenGL 2.1) with logs and screenshots until TraceDraw ran on it with its own card, which is why older PCs are supported (#616); more than 35 other reports, from the Transformations panel to guidelines and text colours (#115, #376, #912) |
| [@vistech91](https://github.com/vistech91) | Glyphs, corners that stay editable, colour conversion between CMYK and RGB and the colour readout of the eyedropper (#469, #872, #990, #1254) |
| [@oksandao](https://github.com/oksandao) | Segment Delete, node control, Smart Fill on guidelines, fills that move with their object and text in other languages (#182, #314, #775, #899) |
| [@elmonopascual](https://github.com/elmonopascual) | Printing: spot colour separations, print preview and missing fonts; missing icons in the panels (#779, #846, #848, #1243) |
| [@Pink-ET](https://github.com/Pink-ET) | Toolbars and their customisation, the text cursor, the Object list, the update download on x86-64 and snapping marks (#632, #836, #1038, #1538) |
| [@markosc](https://github.com/markosc) | Export (WebP quality, profiles and presets), groups inside frames and the start of the app on Windows 11 (#544, #570, #612, #851) |
| [@aleksandrkirjanov-cloud](https://github.com/aleksandrkirjanov-cloud) | Perspective, the Dimension tools, the eyedropper, Drop Shadow and zoom (#458, #838, #935, #1039) |
| [@iAyadMed](https://github.com/iAyadMed) | Contours and their offset, Segment Delete, turning shapes and editing fills (#424, #528, #754, #933) |
| [@andresafu](https://github.com/andresafu) | Printing fitted to the page, text sizes in millimetres, boolean shaping and gradient handles (#529, #534, #539, #939) |
| [@giuliovale](https://github.com/giuliovale) | Hyphenation, text wrapped around a shape and CMYK separation options (#470, #571, #661) |

To contribute, read [`CONTRIBUTING.md`](CONTRIBUTING.md). When your pull
request is merged, you are added here.
