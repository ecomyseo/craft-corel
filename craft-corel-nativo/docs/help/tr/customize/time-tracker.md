# Zaman izleyici

Zaman izleyici bir çizim üzerinde harcadığınız süreyi görev görev kaydeder
ve kayıtları çizimin içinde (`.tdraw` dosyalarında) tutar. Süre yalnızca
çalışırken sayılır: birkaç dakika boşta kalındıktan sonra saat bekler ve bir
sonraki eyleminiz onu yeniden başlatır.

## Zaman İzleyici araç çubuğu {#toolbar}

[[menu.window > menu.window.toolbars > menu.window.toolbar_time_tracker]] şu
öğeleri içeren bir çubuk gösterir:

- etkin görevi seçmek veya yeni bir görev eklemek için görev listesi;
- etkin görevin süresi;
- kayıt ve duraklatma düğmesi;
- bir ışık: sayarken kırmızı, boştayken kehribar, duraklatıldığında veya
  görev yokken gri;
- tüm görevlerin toplamı;
- iletişim kutusunu açan bir düğme.

Görevi olmayan bir çizimde saati başlatmak "Görev 1" ekler.

## Zaman İzleyici iletişim kutusu {#dialog}

[[menu.tools > menu.tools.time_tracker]] görevleri süreleri, başlangıçları
ve bitişleriyle birlikte ve toplamı listeler.

![Zaman İzleyici iletişim kutusu](../../img/customize-time-tracker.png)

- **Yeni Görev**, **Sil**, **Etkin Yap** ve **Süreyi Sıfırla** seçili
  görevde işlem yapar. Etkin yapmak için bir göreve çift tıklayın.
- **Kayıt** ve **Duraklat** saati başlatır ve durdurur.
- Seçili görevin **adını**, **süresini** (`1:30:00`, `1:30` veya `1.5` saat
  gibi), **başlangıcını** ve **bitişini** (`2026-10-19 14:30` gibi)
  düzenleyin.
- **N dakika giriş olmadığında saymayı durdur** boşta kalma süresini ayarlar
  (başlangıçta 5 dakika).
- **CSV Olarak Dışa Aktar** ve **Metin Olarak Dışa Aktar** kayıtları bir
  elektronik tablo veya bir rapor için kaydeder.

Süre kaydetmek çizimde bir düzenleme değildir: geri alma adımı oluşturmaz.
Kayıtlar çizimin bir sonraki kaydedilişinde kaydedilir.
