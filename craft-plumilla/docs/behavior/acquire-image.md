# Acquire Image

File > Acquire Image brings pictures from a scanner or a camera into the drawing.

| Item | Enabled | What it does |
|---|---|---|
| TWAIN | the 64-bit TWAIN source manager is installed (`TWAINDSM.dll`) | Chooses TWAIN as the source. |
| WIA | Windows Image Acquisition can be reached | Chooses WIA as the source (the default). |
| Acquire... | a drawing is open and the source chosen can be used | Opens the device's own dialog (Windows Image Acquisition: device, kind of picture, area, resolution; TWAIN: the scanner's own dialog), saves the pictures it gives to a folder of its own, imports each as File > Import does and selects them. Each picture is its own undo step. |

When no scanner or camera is connected (with TWAIN: no TWAIN source installed),
Acquire says so on the status bar and the drawing does not change; nothing opens. Closing the device's dialog leaves the
drawing as it was. The temporary folder is removed afterwards.

The pictures are placed as File > Import places a picture file: their own
resolution sets their size, and their colour profile goes through the drawing's
colour management.

## TWAIN

The source manager Windows carries is opened over the window and asked for its
sources. With one, it is used; with several, the manager's own Select Source
dialog chooses (cancelling it ends Acquire). The source's own dialog is shown;
when it says pictures are ready, every one it has is taken (native transfer) and
saved as a BMP file with the resolution the source gave, then imported as above.
The source is then disabled and closed, and the manager closed, also when the
source's dialog is closed without scanning or a transfer fails. Version 2
sources are heard through the callback the specification gives; others through
the window messages handed to them.

In the browser there is no scanner: the items stay off.
