# Menyampel ulang, meluruskan, dan memperluas bitmap

## Sampel ulang {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] memberi bitmap yang dipilih jumlah
piksel baru; ukurannya di halaman tetap.

![Kotak dialog Sampel Ulang](../../img/bitmaps-resample-and-straighten-resample.png)

1. Ketik ukuran baru dalam **piksel**, sebagai **persen**, atau sebagai
   **resolusi** yang dihasilkan pada ukurannya di halaman. Dengan
   **Pertahankan rasio aspek** aktif, tinggi mengikuti lebar.
2. Pilih metodenya:
   - **Halus** (foto), bawaan;
   - **Tajam**, sedikit lebih jelas;
   - **Piksel keras** (seni piksel): setiap piksel tetap berupa persegi satu
     warna;
   - **Perbesar dengan detail**: dihaluskan, lalu dipertajam bila gambar
     membesar.
3. Klik [[dialog.ok]].

Kotak dialog menampilkan ukuran asli dan ukuran baru serta memori yang dipakai
gambar baru. Bila beberapa bitmap dipilih, masing-masing diskalakan dengan
faktor yang sama.

## Luruskan Gambar {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] memperbaiki foto yang miring,
dipotret dari sudut tertentu, atau melengkung karena lensa.

![Kotak dialog Luruskan Gambar](../../img/bitmaps-resample-and-straighten-straighten.png)

1. Gunakan **Kisi** (ukuran dan warnanya dapat diatur) untuk menyejajarkan
   gambar.
2. Atur penggeser: **Koreksi distorsi lensa** (-100 sampai 100), **Putar
   gambar** (-15 sampai 15 derajat), **Perspektif vertikal** dan **Perspektif
   horizontal** (-100 sampai 100). Tombol putar memutar gambar seperempat
   putaran lebih dahulu.
3. Dengan **Pangkas gambar** aktif, hasilnya dipangkas ke persegi panjang
   terbesar dengan proporsi asli; **Pangkas dan sampel ulang ke ukuran asli**
   mengembalikannya ke jumlah piksel semula. Tanpa pemangkasan, sudut-sudutnya
   transparan.
4. Klik [[dialog.ok]]. **Atur ulang** mengembalikan setiap pengaturan.

Zum dan geser pratinjau dengan alat di atasnya. Hasilnya menggantikan piksel,
jadi efek pada bitmap menjadi permanen.

## Memperluas bitmap {#inflate}

Beberapa efek, seperti buram, menyebar melewati tepi bitmap.

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]] (awalnya
  aktif) memperbesar bitmap sebanyak yang dibutuhkan efeknya, dengan margin
  transparan.
- [[menu.bitmaps.inflate_manual]] memperbesar bitmap yang dipilih secara
  manual: untuk lebar dan tinggi, ketik ukuran baru dalam piksel atau persen
  (100 sampai 1000%), dengan **Pertahankan rasio aspek**. Gambar tetap di
  tengah.

![Kotak dialog Perluas Bitmap Secara Manual](../../img/bitmaps-resample-and-straighten-inflate.png)

## Perintah bitmap lainnya {#other}

- [[menu.bitmaps.crop_bitmap]] memangkas bitmap dengan alat [[tool.crop]];
  menyeret sudut bitmap dengan alat [[tool.shape]] juga memangkasnya.
- [[menu.bitmaps.edit_bitmap]] membuka bitmap di program lain.
- [[menu.bitmaps.remove_background]] membuat latar belakang foto menjadi
  transparan.
- [[menu.bitmaps.remove_background_local]]: [Hapus Latar](remove-background.md).
- [[menu.bitmaps.break_link]] dan [[menu.bitmaps.update_from_link]] berlaku
  untuk bitmap yang tertaut ke sebuah berkas.
- [[menu.bitmaps.save_bitmap_as]] menyimpan bitmap sebagai berkas PNG.
