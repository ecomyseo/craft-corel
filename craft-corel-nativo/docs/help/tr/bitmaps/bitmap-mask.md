# Bitmaplerden renkleri maskeleme

[[docker.bitmap_mask]] paneli, bir bitmapte seçilen renklerin piksellerini
gizler veya korur; örneğin beyaz bir arka planı saydam yapmak için. Paneli
[[menu.bitmaps > menu.bitmaps.bitmap_mask]] ile açın.

![Bitmap Maskesi paneli](../../img/bitmaps-bitmap-mask.png)

## Renkleri maskelemek için {#mask}

1. Bir bitmap seçin.
2. On renk yuvasına renk alın:
   - damlalığa tıklayın ve bitmapteki renklere tıklayın; her biri seçili
     yuvayı doldurur ve işaretler;
   - ya da yuvaları bitmapin en çok kullandığı on renkle doldurmak için
     **Bitmapten renk seç** düğmesine tıklayın.
3. Bir yuva seçin ve **toleransını** ayarlayın (%0 ile %100 arası): bir
   pikselin renginin, yuvanın renginden ne kadar uzak olup yine de
   eşleşebileceği.
4. **Seçilenleri gizle** (eşleşen pikseller saydam olur) veya **Seçilenleri
   göster** (başlangıçta yalnızca eşleşen pikseller kalır) seçeneğini seçin.
5. [[docker.apply]] düğmesine tıklayın.

**Maskeyi kaydet** ve **Maske aç** yuvaları bir `.msk` dosyasında saklar;
**Sil** seçili yuvayı boşaltır. Her bitmap tek bir geri alma adımında
değişir.
