# Bitmapleri yeniden örnekleme, düzeltme ve genişletme

## Yeniden örnekle {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] seçili bitmaplere yeni bir piksel
sayısı verir; sayfadaki boyutları aynı kalır.

![Yeniden Örnekle iletişim kutusu](../../img/bitmaps-resample-and-straighten-resample.png)

1. Yeni boyutu **piksel** olarak, **yüzde** olarak veya sayfadaki boyutunda
   verdiği **çözünürlük** olarak yazın. **En boy oranını koru** açıkken
   yükseklik genişliği izler.
2. Yöntemi seçin:
   - **Düzgün** (fotoğraflar), başlangıçta;
   - **Keskin**, biraz daha net;
   - **Sert pikseller** (piksel sanatı): her piksel tek renkli bir kare
     olarak kalır;
   - **Ayrıntıyla büyüt**: düzgünleştirir, ardından resim büyüdüğünde
     keskinleştirir.
3. [[dialog.ok]] düğmesine tıklayın.

İletişim kutusu özgün ve yeni boyutları ve yeni resmin kapladığı belleği
gösterir. Birden çok bitmap seçiliyken her biri aynı oranda ölçeklenir.

## Görüntüyü Düzelt {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] eğik duran, açıyla görülen
veya mercek tarafından bükülen fotoğrafları düzeltir.

![Görüntüyü Düzelt iletişim kutusu](../../img/bitmaps-resample-and-straighten-straighten.png)

1. Resmi hizalamak için **Izgara**'yı kullanın (boyutu ve rengi
   ayarlanabilir).
2. Kaydırıcıları ayarlayın: **Mercek bozulmasını düzelt** (-100 ile 100
   arası), **Görüntüyü döndür** (-15 ile 15 derece arası), **Dikey
   perspektif** ve **Yatay perspektif** (-100 ile 100 arası). Döndürme
   düğmeleri resmi önce çeyrek tur çevirir.
3. **Görüntüyü kırp** açıkken sonuç, özgün oranlara sahip en büyük
   dikdörtgene kırpılır; **Kırp ve orijinal boyuta yeniden örnekle** onu özgün
   piksel sayısına geri getirir. Kırpma olmadan köşeler saydam olur.
4. [[dialog.ok]] düğmesine tıklayın. **Sıfırla** her ayarı geri alır.

Önizlemeyi üstündeki araçlarla yakınlaştırın ve kaydırın. Sonuç piksellerin
yerini alır, bu yüzden bitmap üzerindeki efektler kalıcı hâle gelir.

## Bitmapleri genişletme {#inflate}

Bulanıklaştırma gibi bazı efektler bir bitmapin kenarlarının dışına taşar.

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]]
  (başlangıçta açık) bitmapleri, efektlerinin gerektirdiği kadar saydam bir
  kenar boşluğuyla büyütür.
- [[menu.bitmaps.inflate_manual]] seçili bitmapleri elle büyütür: genişlik ve
  yükseklik için yeni boyutu piksel veya yüzde (%100 ile %1000 arası) olarak,
  **En boy oranını koru** ile yazın. Resim ortada kalır.

![Bitmapi Elle Genişlet iletişim kutusu](../../img/bitmaps-resample-and-straighten-inflate.png)

## Diğer bitmap komutları {#other}

- [[menu.bitmaps.crop_bitmap]] bir bitmapi [[tool.crop]] aracıyla kırpar;
  bir bitmapin köşesini [[tool.shape]] aracıyla sürüklemek de onu kırpar.
- [[menu.bitmaps.edit_bitmap]] bitmapi başka bir programda açar.
- [[menu.bitmaps.remove_background]] bir fotoğrafın arka planını saydam
  yapar.
- [[menu.bitmaps.remove_background_local]]: [Arka Planı Kaldır](remove-background.md).
- [[menu.bitmaps.break_link]] ve [[menu.bitmaps.update_from_link]] bir dosyaya
  bağlı bitmaplerde çalışır.
- [[menu.bitmaps.save_bitmap_as]] bir bitmapi PNG dosyası olarak kaydeder.
