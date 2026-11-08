# Spot renkler

**Spot renk**, camgöbeği, macenta, sarı ve siyahtan karıştırılmak yerine
kendi kalıbına basılan adlandırılmış bir mürekkeptir; örneğin metalik bir
renk veya bir marka rengi. Her çizim kendi spot mürekkeplerinin listesini
tutar. Her mürekkebin, onu ekranda göstermek için ve mürekkebin
kullanılamadığı her yerde kullanılan, işlem renklerinden yapılmış bir
yedeği vardır.

## Spot mürekkep ekleme {#add}

1. [[docker.color]] panelini açın ve mürekkebin görünmesi gereken rengi
   karıştırın.
2. [[docker.spot_inks]] altında bir [[docker.ink_name]] yazın.
3. [[docker.new_ink]] düğmesine tıklayın.

Adlar bir çizim içinde benzersizdir.

## Spot mürekkepleri kullanma {#use}

[[docker.spot_inks]] listesi her mürekkebi bir renk örneği, adı ve onu
ayarladığınız [[docker.tint]] değerinde (%0 ile %100 arası, başlangıçta %100)
seçime uygulayan [[docker.fill]] ve [[docker.outline]] düğmeleriyle gösterir.
Bir satırın sonundaki çarpı mürekkebi listeden kaldırır; onu kullanan
nesneler yedek rengini korur.

## Renk kitapları {#color-books}

Bir palet dosyası bir spot mürekkep kitabı olabilir (örneğin spot girdileri
olan bir `.ase` paleti). Spot girdileri, kitaptaki gibi adlandırılmış spot
renkler olarak açılır; birini uygulamak mürekkebi çizime ekler. TraceDraw
lisanslı renk kitaplarıyla birlikte gelmez, ancak sahip olduğunuz bir kitap
herhangi bir palet gibi açılır.

## Baskıda ve dışa aktarmada spot renkler {#output}

- PDF dosyalarında her mürekkep, adını taşıyan bir ayırma rengi olur; böylece
  matbaa mürekkebi alır.
- Ayırmaları bastığınızda her mürekkep, camgöbeği, macenta, sarı ve siyah
  kalıplardan sonra kendi kalıbını alır. Bkz.
  [Yazdırma](../printing/print.md).
- Ekranda ve bitmap, SVG ve `.cdr` dosyalarında spot renk, tonundaki yedeği
  olarak görünür.
