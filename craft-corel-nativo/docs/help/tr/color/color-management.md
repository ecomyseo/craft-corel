# Renk yönetimi

Ekranlar renkleri kırmızı, yeşil ve mavi ışıkla gösterir; yazıcılar camgöbeği,
macenta, sarı ve siyah mürekkepleri karıştırır. Renk yönetimi her aygıtı
tanımlamak için **renk profillerini** kullanır; böylece bir renk ekrandan
basılı sayfaya kadar görünümünü korur. Ayarlar
[[menu.tools > menu.tools.color_management]] altında, [[color_mgmt.title]]
iletişim kutusundadır. [[dialog.ok]] düğmesine tıklayana kadar hiçbir şey
değişmez.

![Renk Yönetimi Ayarları iletişim kutusu](../../img/color-color-management.png)

## Çizimin profilleri {#document}

[[color_mgmt.tab_document]] sekmesi etkin çizimin profillerini ayarlar.

- [[color_mgmt.current_profiles]] çizimin şu anda kullandığı RGB, CMYK ve gri
  tonlama profillerini gösterir.
- Yeni bir profilin nasıl uygulanacağını seçin:
  - [[color_mgmt.assign]]: renk değerleri aynı kalır ve görünümleri yeni
    profile göre değişir;
  - [[color_mgmt.convert]]: CMYK renkleri yeni profilde aynı görünecek
    şekilde dönüştürülür.
- [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]] ve
  [[color_mgmt.grayscale]] profillerini seçin. Her listede önce varsayılan
  profiller, ardından bilgisayarınızda bulunan her profil yer alır.
  [[color_mgmt.load_profiles]] profil dosyaları (`.icc`, `.icm`) ekler.
- [[color_mgmt.primary_mode]]: CMYK veya RGB.
- [[color_mgmt.intent]]: gösterilemeyen renklerin nasıl içeri alınacağı:
  [[color_mgmt.intent_relative]] (başlangıçta),
  [[color_mgmt.intent_perceptual]], [[color_mgmt.intent_saturation]] veya
  [[color_mgmt.intent_absolute]].

CMYK profili çizimle birlikte gider, 16. sürümden itibaren `.cdr`
dosyalarına da.

## Yeni çizimler için varsayılanlar {#default}

[[color_mgmt.tab_default]] sekmesi yeni çizimlerin ne kullanacağını ve
renklerin nasıl dönüştürüleceğini ayarlar.

- [[color_mgmt.preset]]: Kuzey Amerika, Avrupa, Japonya veya web ve ekran
  çalışmaları için hazır ayarlar. [[color_mgmt.save_preset]] kendi
  ayarlarınızı bir [[color_mgmt.preset_name]] altında kaydeder;
  [[color_mgmt.delete_preset]] birini kaldırır.
- Varsayılan [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]] ve
  [[color_mgmt.gray_profile]], [[color_mgmt.primary_mode]] ve
  [[color_mgmt.intent]].
- [[color_mgmt.conversion]]:
  - [[color_mgmt.engine]]: profilleri kullanan [[color_mgmt.engine_icc]] veya
    [[color_mgmt.engine_builtin]];
  - [[color_mgmt.preserve_black]]: yalnızca siyah mürekkepten oluşan renkler
    dönüştürüldüğünde yalnızca siyah mürekkep olarak kalır;
  - [[color_mgmt.map_gray]] (başlangıçta açık): CMYK'ye dönüştürülen griler
    yalnızca siyah mürekkep kullanır;
  - [[dialog.black_point]] (başlangıçta açık): en koyu tonları birbirinden
    ayrı tutar.
- [[color_mgmt.open_policy]]: RGB, CMYK ve gri tonlama için, açtığınız bir
  dosyanın profiline ne olacağı: [[color_mgmt.open_use_embedded]],
  [[color_mgmt.open_assign_default]] veya
  [[color_mgmt.open_convert_default]]. [[color_mgmt.warn_mismatch_box]] ve
  [[color_mgmt.warn_missing_box]], bir dosyanın başka bir profili olduğunda
  veya hiç profili olmadığında durum çubuğunda bir ileti gösterir.
- [[color_mgmt.import_policy]]: içe aktardığınız veya yapıştırdığınız şeyler
  için [[color_mgmt.import_convert]] veya [[color_mgmt.import_assign]].
- [[color_mgmt.spot_definition]]: spot renklerin [[color_mgmt.spot_lab]],
  [[color_mgmt.spot_cmyk]] veya [[color_mgmt.spot_rgb]] ile tanımlanıp
  tanımlanmayacağı.

## Baskı renklerini önizleme {#proof}

[[menu.view > menu.view.proof_colors]] veya durum çubuğunun sağ ucundaki
prova düğmesi çizimi basılacağı gibi gösterir: CMYK mürekkeplerinin
basamayacağı renkler daha mat görünür. [[docker.color]] paneli de bu tür
renkler için uyarır.

Yüklü profil olmadan TraceDraw renkleri, kuşe kâğıt üzerindeki işlem
mürekkeplerinin kendi modeliyle dönüştürür.
