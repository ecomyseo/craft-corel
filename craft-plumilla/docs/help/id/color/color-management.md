# Manajemen warna

Layar menampilkan warna dengan cahaya merah, hijau, dan biru; printer mencampur
tinta cyan, magenta, kuning, dan hitam. Manajemen warna memakai **profil
warna** untuk menggambarkan setiap perangkat, sehingga warna tetap terlihat
sama dari layar hingga halaman cetak. Pengaturannya ada di
[[menu.tools > menu.tools.color_management]], kotak dialog
[[color_mgmt.title]]. Tidak ada yang berubah sampai Anda mengklik
[[dialog.ok]].

![Kotak dialog Pengaturan Manajemen Warna](../../img/color-color-management.png)

## Profil gambar {#document}

Tab [[color_mgmt.tab_document]] mengatur profil gambar yang aktif.

- [[color_mgmt.current_profiles]] menampilkan profil RGB, CMYK, dan skala
  abu-abu yang dipakai gambar saat ini.
- Pilih cara profil baru diterapkan:
  - [[color_mgmt.assign]]: nilai warna tetap dan tampilannya berubah mengikuti
    profil baru;
  - [[color_mgmt.convert]]: warna CMYK dikonversi agar terlihat sama di
    profil baru.
- Pilih profil [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]], dan
  [[color_mgmt.grayscale]]. Setiap daftar berisi profil bawaan, lalu setiap
  profil yang ditemukan di komputer Anda. [[color_mgmt.load_profiles]]
  menambahkan berkas profil (`.icc`, `.icm`).
- [[color_mgmt.primary_mode]]: CMYK atau RGB.
- [[color_mgmt.intent]]: cara warna yang tidak dapat ditampilkan dibawa masuk:
  [[color_mgmt.intent_relative]] (bawaan), [[color_mgmt.intent_perceptual]],
  [[color_mgmt.intent_saturation]], atau [[color_mgmt.intent_absolute]].

Profil CMYK ikut tersimpan bersama gambar, juga ke dalam berkas `.cdr` sejak
versi 16.

## Bawaan untuk gambar baru {#default}

Tab [[color_mgmt.tab_default]] mengatur apa yang dipakai gambar baru, dan cara
warna dikonversi.

- [[color_mgmt.preset]]: pengaturan siap pakai untuk Amerika Utara, Eropa,
  Jepang, atau pekerjaan web dan layar. [[color_mgmt.save_preset]] menyimpan
  pengaturan Anda sendiri dengan [[color_mgmt.preset_name]];
  [[color_mgmt.delete_preset]] menghapusnya.
- [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]], dan
  [[color_mgmt.gray_profile]] bawaan, [[color_mgmt.primary_mode]], serta
  [[color_mgmt.intent]].
- [[color_mgmt.conversion]]:
  - [[color_mgmt.engine]]: [[color_mgmt.engine_icc]], yang memakai profil,
    atau [[color_mgmt.engine_builtin]];
  - [[color_mgmt.preserve_black]]: warna yang hanya berupa tinta hitam tetap
    hanya tinta hitam saat dikonversi;
  - [[color_mgmt.map_gray]] (awalnya aktif): abu-abu yang dikonversi ke CMYK
    hanya memakai tinta hitam;
  - [[dialog.black_point]] (awalnya aktif): menjaga gradasi tergelap tetap
    terpisah.
- [[color_mgmt.open_policy]]: apa yang terjadi pada profil berkas yang Anda
  buka, untuk RGB, CMYK, dan skala abu-abu: [[color_mgmt.open_use_embedded]],
  [[color_mgmt.open_assign_default]], atau [[color_mgmt.open_convert_default]].
  [[color_mgmt.warn_mismatch_box]] dan [[color_mgmt.warn_missing_box]]
  menampilkan pesan di bilah status bila sebuah berkas memiliki profil lain
  atau tidak memiliki profil.
- [[color_mgmt.import_policy]]: [[color_mgmt.import_convert]] atau
  [[color_mgmt.import_assign]], untuk apa yang Anda impor atau tempel.
- [[color_mgmt.spot_definition]]: apakah warna spot ditentukan oleh
  [[color_mgmt.spot_lab]], [[color_mgmt.spot_cmyk]], atau
  [[color_mgmt.spot_rgb]].

## Pratinjau warna cetak {#proof}

[[menu.view > menu.view.proof_colors]], atau tombol uji cetak di ujung kanan
bilah status, menampilkan gambar seperti saat dicetak: warna yang tidak dapat
dicetak tinta CMYK terlihat lebih kusam. Panel [[docker.color]] juga
memperingatkan tentang warna semacam itu.

Tanpa profil yang dimuat, TraceDraw mengonversi warna dengan modelnya sendiri
untuk tinta proses di atas kertas berlapis.
