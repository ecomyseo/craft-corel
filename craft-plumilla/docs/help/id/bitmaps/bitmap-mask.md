# Menyamarkan warna dari bitmap

Panel [[docker.bitmap_mask]] menyembunyikan atau mempertahankan piksel dengan
warna tertentu di dalam bitmap, misalnya untuk membuat latar belakang putih
menjadi transparan. Buka dengan [[menu.bitmaps > menu.bitmaps.bitmap_mask]].

![Panel Masker Bitmap](../../img/bitmaps-bitmap-mask.png)

## Untuk menyamarkan warna {#mask}

1. Pilih bitmap.
2. Ambil warna ke sepuluh slot warna:
   - klik pipet lalu klik warna di bitmap; setiap warna mengisi slot yang
     dipilih dan mencentangnya;
   - atau klik **Pilih warna dari bitmap** untuk mengisi slot dengan sepuluh
     warna yang paling banyak dipakai bitmap.
3. Pilih slot dan atur **toleransinya** (0 sampai 100%): seberapa jauh warna
   piksel boleh berbeda dari warna slot dan tetap dianggap cocok.
4. Pilih **Sembunyikan yang dipilih** (piksel yang cocok menjadi transparan)
   atau **Tampilkan yang dipilih** (awalnya hanya piksel yang cocok yang
   tersisa).
5. Klik [[docker.apply]].

**Simpan masker** dan **Buka masker** menyimpan slot dalam berkas `.msk`;
**Hapus** mengosongkan slot yang dipilih. Setiap bitmap berubah dalam satu
langkah urungkan.
