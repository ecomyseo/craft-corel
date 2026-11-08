# Yenilikler

TraceDraw'ın her sürümü özellikler ve düzeltmeler ekler. Bu sayfa neyin
değiştiğini nasıl öğreneceğinizi açıklar ve son sürümlerin öne çıkanlarını
listeler.

## Programdaki yenilikleri görme {#in-the-program}

- Karşılama ekranının yenilikler sayfasını açmak için
  [[menu.help > menu.help.whats_new]] komutunu seçin. Sayfa son on sürümü, en yenisi
  başta olmak üzere tarihi ve kısa bir başlığıyla listeler. Neyin
  değiştiğine dair kısa bir özeti okumak için bir sürüme tıklayın; en yenisi
  zaten açıktır. Bir özetin altındaki **Daha fazlasını öğrenmek ister
  misiniz?** bağlantısı, o sürümün GitHub sayfasını tüm değişiklik
  listesiyle açar.
- Bu sürümde yeni olanları menülerde işaretlemek için
  [[menu.help > menu.help.highlight_whats_new]] komutunu açın: her yeni
  komutun sonunda, ona giden bir alt menünün okunun yanında ve yeni bir komut
  içeren menünün başlığının üstünde küçük bir nokta görünür. Komutu
  kapattığınızda işaretler kaybolur ve TraceDraw bir sonraki başlatılışında
  komut yine kapalıdır.
- Her sürümün değişikliklerinin tam listesi projenin
  [değişiklik günlüğündedir](https://github.com/tracedraw/tracedraw/blob/main/CHANGELOG.md).

## Son sürümlerin öne çıkanları {#highlights}

- **Dokunma ve telefonlar.** Standart çubuktaki Shift, Ctrl ve Alt düğmeleri
  parmak için basılı kalır; Android'de arayüz kamera çentiğinden ve yuvarlak
  köşelerden uzak durur; Apple Pencil iPad'deki Safari'de çalışır.
- **Dosyalar.** .cdr dosyaları üst üste binen eğrilerin dolgu kuralını korur
  ve sekme durakları, çok sütunlu metinleri, tüm sayfaların paylaştığı
  kılavuzları ve ana sayfa nesneleriyle açılır.
- **Resimler, yazdırma ve çalışma alanı.** Kırpma aracındaki resim
  piksellerini kırpma seçeneği ve
  [[menu.bitmaps > menu.bitmaps.crop_to_frame]] bir resmin piksellerini
  kırpar; gri tonlamalı ve ters sırada yazdırma;
  [[menu.view > menu.view.scrollbars]] kaydırma çubuklarını gizler.
- **TraceCut.** [[menu.tools > menu.tools.tracecut]] kesim plotterları için
  araçları açar: plotter çıktısı, kesim çizgileri, yerleştirme ve ölçme. Bkz.
  [TraceCut](../objects/tracecut.md).
- **Klavye, telefonlar ve kalemler.** Menüler ve listeler klavyeyle çalışır.
  Android telefonlarda arayüz ekrana sığar ve kalem kullanılırken ekrana
  dayanan el artık çizmez; tablet kalemi Wayland kullanan Linux'ta çalışır.
- **Dosyalar.** .cdr dosyaları sekme durakları, ana sayfalar, degradeler ve
  bozulmalarla açılır; PDF resimleri tam çözünürlükte gelir; JPEG ve TIFF CMYK
  olarak dışa aktarılır.
- **Nesne küçük resimleri.** [[docker.objects]] paneli her nesneyi adının
  yanında küçük olarak çizer; listenin üzerinde Ctrl+tekerlek veya alttaki
  kaydırıcı boyutu ayarlar. Bkz. [Nesneleri ve katmanları düzenleme](../objects/objects-and-layers.md#objects-docker).
- **Kendi dilinizde Yardım.** Her **?** düğmesi ve <kbd>F1</kbd>, önünüzdeki
  iletişim kutusu, panel veya araç hakkındaki Yardım sayfasını TraceDraw'ın
  ayarlı olduğu dilde açar.
- **Belge özellikleri.** [[menu.file > menu.file.document_properties]] bir
  çizimin dilini, başlığını, konusunu, yazarını, telif hakkını, etiketlerini,
  notlarını ve puanını, çizimin içeriğinin tam raporunun üstünde saklar. Bkz.
  [Belge özellikleri](../pages-layout/document-properties.md).
- **Renk yönetimi.** [[menu.tools > menu.tools.color_management]] çizimin
  profillerini ve yeni çizimlerin varsayılanlarını, açılan veya içe aktarılan
  dosyalar için kurallarla birlikte ayarlar. Bkz.
  [Renk yönetimi](../color/color-management.md).
- **Barkodlar ve QR kodları.** Üç adımlı bir sihirbaz barkod yerleştirir; QR
  kodları ise [[docker.properties]] panelinde düzenlenen canlı nesnelerdir.
  Bkz. [Barkodlar ve QR kodları](../objects/barcodes.md).
- **Efektler.** [[menu.effects]] menüsü efekt panellerini bir araya getirir;
  geçişler sabit aralıkla bir yolu izleyebilir ve renklerini saat yönünde veya
  tersine döndürebilir. Bkz. [Geçişler](../effects/blend.md).
- **Yol üzerinde metin** yön, yerleşim, uzaklık ve kaydırma için kendi
  ayarlarına sahiptir. Bkz.
  [Metin ekleme ve düzenleme](../text/adding-text.md).
- **Yazı Tipi Yöneticisi** yüklü yazı tiplerini, klasörleri, sık
  kullanılanları ve koleksiyonları düzenler. Bkz.
  [Yazı tipleri ve glifler](../text/fonts.md).
