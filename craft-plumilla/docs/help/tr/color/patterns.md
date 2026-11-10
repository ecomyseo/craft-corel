# Desen dolguları

Desen dolgusu, bir nesnenin içi boyunca bir karoyu yineler. Karo iki renk,
bir vektör nesneleri kümesi (tam renkli vektör deseni) veya bir bitmap
olabilir. Karolar sayfaya sabitlenir, bu yüzden nesneyi taşıdığınızda
kaymazlar.

## Nesnelerinizden desen oluşturma {#create-pattern}

1. Bir karoyu oluşturan nesneleri çizin ve seçin.
2. [[menu.object > menu.object.create > menu.object.create_pattern]] komutunu
   seçin.
3. [[dialog.create_pattern]] iletişim kutusunda
   [[dialog.create_pattern_type]] seçeneğini belirleyin:
   [[dialog.create_pattern_vector]] nesneleri vektör olarak, her boyutta
   keskin tutar; [[dialog.create_pattern_bitmap]] onlardan bir resim yapar.
4. [[dialog.ok]] düğmesine tıklayın.

![Desen Oluştur iletişim kutusu](../../img/color-patterns-create-pattern.png)

Desen yeni nesnelerin dolgusu olur. Onu önceden çizdiğiniz nesnelere vermek
için [[docker.properties]] panelinin dolgu bölümünde
[[docker.fill_pattern]] seçeneğini seçin.

## Desen ayarları {#settings}

[[docker.properties]] panelinin dolgu bölümünde [[docker.fill_pattern]],
ardından desen türünü ve ayarlarını seçin. Bir vektör deseni için panel,
karonun kaç nesne içerdiğini ve genişliğini ve yüksekliğini gösterir: bunları
değiştirmek karonun içeriğini de birlikte ölçekler.

## Dışa aktarılan dosyalarda desenler {#export}

SVG dosyaları vektör desenlerini vektör olarak korur. PDF ve EPS dosyaları
onları dışa aktarma çözünürlüğünde, nesneye kırpılmış resimler olarak alır.
