# Renk stilleri

**Renk stili**, çizimin nesnelerinin kullandığı adlandırılmış bir renktir.
Stili değiştirin, onunla doldurulmuş her nesne de değişir; bu, bir tasarımın
renklerini tutarlı tutar ve başka renk düzenlerini denemeyi hızlandırır.
Stiller [[docker.color_styles]] panelinde listelenir; paneli
[[menu.window > menu.window.dockers]] menüsünden açın.

![Renk Stilleri paneli](../../img/color-color-styles.png)

## Renk stilleri oluşturma {#create}

- [[docker.new_from_selection]], seçili nesnelerin henüz stil olmayan her
  dolgu rengi için bir stil ekler.
- [[docker.new_color_style]], [[docker.color]] panelinin geçerli renginden
  bir stil ekler.

## Stilleri kullanma ve düzenleme {#edit}

Her stil rengini, adını ve üç düğmeyi gösterir:

- değiştirmek için renge tıklayın: o renkle doldurulmuş nesneler onu izler;
- yeniden adlandırmak için ada tıklayın;
- [[docker.fill]] seçimi stille doldurur, [[docker.outline]] seçimin
  anahattını renklendirir;
- **x** stili kaldırır (nesneler renklerini korur).

## Uyumlar {#harmonies}

[[docker.harmony_editor]], bir [[docker.base_color]] öğesinden birbirine
uyan bir stil kümesi oluşturur:

1. [[docker.base_color]] öğesini seçin.
2. Bir uyuma tıklayın: [[harmony.analogous]], [[harmony.complementary]],
   [[harmony.split_complementary]], [[harmony.triadic]],
   [[harmony.tetradic]] veya [[harmony.monochromatic]].

Uyumun renkleri, onun adıyla adlandırılmış stiller olarak eklenir.
