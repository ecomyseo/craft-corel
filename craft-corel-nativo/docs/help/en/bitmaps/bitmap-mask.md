# Masking colors out of bitmaps

The [[docker.bitmap_mask]] panel hides or keeps the pixels of chosen colors in
a bitmap, for example to make a white background transparent. Open it with
[[menu.bitmaps > menu.bitmaps.bitmap_mask]].

![The Color Mask panel](../../img/bitmaps-bitmap-mask.png)

## To mask colors {#mask}

1. Select a bitmap.
2. Pick colors into the ten color slots:
   - click the eyedropper and click colors in the bitmap; each fills the
     selected slot and ticks it;
   - or click **Select colours from bitmap** to fill the slots with the ten
     colors the bitmap uses most.
3. Select a slot and set its **tolerance** (0 to 100%): how far a pixel's
   color may be from the slot's color and still match.
4. Choose **Hide selected** (matching pixels become transparent) or **Show
   selected** (only matching pixels stay, at first).
5. Click [[docker.apply]].

**Save mask** and **Open mask** keep the slots in a `.msk` file; **Delete**
empties the selected slot. Each bitmap changes in one undo step.
