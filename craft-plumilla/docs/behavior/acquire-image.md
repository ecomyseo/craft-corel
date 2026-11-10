# Acquire Image

File > Acquire Image brings pictures from a scanner or a camera into the drawing.

| Item | Enabled | What it does |
|---|---|---|
| TWAIN | not yet | The TWAIN source. Acquiring through TWAIN is not done, so the item stays off. |
| WIA | Windows Image Acquisition can be reached | Chooses WIA as the source (the default). |
| Acquire... | a drawing is open and the source chosen can be used | Opens the device's own dialog (Windows Image Acquisition: device, kind of picture, area, resolution), saves the pictures it gives to a folder of its own, imports each as File > Import does and selects them. Each picture is its own undo step. |

When no scanner or camera is connected, Acquire says so on the status bar and the
drawing does not change; nothing opens. Closing the device's dialog leaves the
drawing as it was. The temporary folder is removed afterwards.

The pictures are placed as File > Import places a picture file: their own
resolution sets their size, and their colour profile goes through the drawing's
colour management.

In the browser there is no scanner: the items stay off.
