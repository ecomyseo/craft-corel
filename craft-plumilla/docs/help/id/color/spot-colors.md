# Warna spot

**Warna spot** adalah tinta bernama yang dicetak di pelatnya sendiri, misalnya
warna metalik atau warna merek, alih-alih dicampur dari cyan, magenta, kuning,
dan hitam. Setiap gambar menyimpan daftar tinta spotnya. Setiap tinta memiliki
pengganti yang terbuat dari warna proses, dipakai untuk menampilkannya di layar
dan di mana pun tinta itu tidak dapat dipakai.

## Menambahkan tinta spot {#add}

1. Buka panel [[docker.color]] dan campur warna yang akan menjadi tampilan
   tinta itu.
2. Di bawah [[docker.spot_inks]], ketik [[docker.ink_name]].
3. Klik [[docker.new_ink]].

Nama bersifat unik di dalam satu gambar.

## Memakai tinta spot {#use}

Daftar [[docker.spot_inks]] menampilkan setiap tinta dengan contoh warna,
namanya, serta tombol [[docker.fill]] dan [[docker.outline]], yang
menerapkannya ke pilihan pada [[docker.tint]] yang Anda atur (0 sampai 100%,
awalnya 100%). Tanda silang di ujung baris menghapus tinta dari daftar; objek
yang memakainya tetap mempertahankan warna penggantinya.

## Buku warna {#color-books}

Berkas palet dapat berupa buku tinta spot (misalnya palet `.ase` dengan entri
spot). Entri spotnya terbuka sebagai warna spot, dinamai seperti di buku;
menerapkan salah satunya menambahkan tinta ke gambar. TraceDraw tidak
menyertakan buku warna berlisensi, tetapi buku yang Anda miliki dapat dibuka
seperti palet lainnya.

## Warna spot dalam cetakan dan ekspor {#output}

- Dalam berkas PDF, setiap tinta menjadi warna separasi yang dinamai sesuai
  tinta itu, sehingga percetakan menerima tintanya.
- Saat Anda mencetak separasi, setiap tinta mendapat pelatnya sendiri setelah
  pelat cyan, magenta, kuning, dan hitam. Lihat [Mencetak](../printing/print.md).
- Di layar dan dalam berkas bitmap, SVG, dan `.cdr`, warna spot tampil sebagai
  penggantinya pada tingkat rona yang diatur.
