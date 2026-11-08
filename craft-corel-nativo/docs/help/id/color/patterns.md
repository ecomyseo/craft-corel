# Isian pola

Isian pola mengulang sebuah ubin di seluruh bagian dalam objek. Ubin dapat
berupa dua warna, sekumpulan objek vektor (pola vektor penuh warna), atau
bitmap. Ubin ditambatkan ke halaman, jadi tidak bergeser saat Anda memindahkan
objek.

## Membuat pola dari objek Anda {#create-pattern}

1. Gambar objek-objek yang membentuk satu ubin lalu pilih objek-objek itu.
2. Pilih [[menu.object > menu.object.create > menu.object.create_pattern]].
3. Di kotak dialog [[dialog.create_pattern]], pilih
   [[dialog.create_pattern_type]]: [[dialog.create_pattern_vector]]
   mempertahankan objek sebagai vektor, tetap tajam pada ukuran berapa pun;
   [[dialog.create_pattern_bitmap]] membuat gambar dari objek-objek itu.
4. Klik [[dialog.ok]].

![Kotak dialog Buat Pola](../../img/color-patterns-create-pattern.png)

Pola menjadi isian objek baru. Untuk memberikannya ke objek yang sudah Anda
gambar, pilih [[docker.fill_pattern]] di bagian isian panel
[[docker.properties]].

## Pengaturan pola {#settings}

Di bagian isian panel [[docker.properties]], pilih [[docker.fill_pattern]],
lalu jenis pola dan pengaturannya. Untuk pola vektor, panel menampilkan berapa
banyak objek yang ada di dalam ubin, serta lebar dan tingginya: mengubahnya
akan ikut menskalakan isi ubin.

## Pola dalam berkas ekspor {#export}

Berkas SVG mempertahankan pola vektor sebagai vektor. Berkas PDF dan EPS
menerimanya sebagai gambar pada resolusi ekspor, dipotong mengikuti objek.
