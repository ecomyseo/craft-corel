# Menggambar dan membuat gambar dengan AI

TraceDraw dapat menggambar dari deskripsi kata-kata dan membuat gambar dengan
layanan AI yang Anda pilih dan atur dengan kunci Anda sendiri. TraceDraw tidak
memiliki AI sendiri dan tidak memerlukan akun: tidak ada yang dikirim sampai
Anda menjalankan perintah AI, dan saat itu hanya apa yang dibutuhkan perintah
tersebut yang dikirim ke layanan yang Anda atur. Setiap permintaan ditagih oleh
layanan Anda; server model lokal di komputer Anda tidak memakan biaya.

## Mengatur layanan AI {#service}

[[menu.tools > menu.tools.options > menu.tools.options_ai]] memiliki dua
halaman.

![Halaman Layanan pada opsi AI](../../img/customize-ai-service.png)

Di halaman **Layanan**, atur satu layanan AI untuk semua fitur AI:

1. Pilih **Preset**: Anthropic, OpenAI, Google Gemini, atau server model di
   komputer Anda (Ollama, LM Studio, llama.cpp server, vLLM). Preset mengisi
   jenis layanan dan alamatnya; **Kustom** menerima alamat apa pun yang
   berbicara dengan bahasa yang sama dengan chat completions OpenAI.
2. Ketik **kunci API** akun Anda di layanan itu (server lokal tidak
   membutuhkannya). Centang **Ingat kunci di komputer ini** untuk
   menyimpannya; kunci lalu disimpan sebagai teks biasa di preferensi Anda.
3. Pilih **Model**, dan **Model gambar** untuk layanan yang dapat membuat
   gambar (OpenAI, Google Gemini, atau alamat kustom dengan model gambar).
4. Klik [[aiui.test]]. Perintah ini meminta daftar model dari layanan dan
   menyebutkan berapa banyak yang tersedia, atau menampilkan galatnya.

**Tampilkan bilah perintah** meletakkan bilah perintah AI di atas palet warna.

## Umum {#general}

![Halaman Umum pada opsi AI](../../img/customize-ai-general.png)

- [[aiui.enable]] (aktif): bila dimatikan, setiap perintah AI menjadi abu-abu.
- [[aiui.services]]: apakah [[aiui.drawings]] dan [[aiui.pictures]] berstatus
  [[aiui.ready]] atau [[aiui.not_ready]].
- [[aiui.features]]: apa yang dilakukan setiap perintah AI.
- [[aiui.privacy_title]]: hanya yang dibutuhkan perintah, ke layanan yang Anda
  atur, saat Anda menjalankannya.

## Menggambar dengan kata-kata {#draw-with-words}

Ketik permintaan seperti "sepeda merah dengan roda biru" atau "perbesar
lingkaran dan letakkan judul di bawahnya", di bilah perintah atau di mode
**Gambar dengan kata-kata** pada panel [[docker.ai_generate]]
([[menu.object > menu.object.ai_generate > menu.object.ai_draw_words]]), lalu
tekan **Gambar** atau <kbd>Enter</kbd>. Layanan menulis skrip untuk gambar,
yang dijalankan TraceDraw sebagai satu langkah urungkan. Percakapan disimpan,
jadi Anda dapat meminta perubahan; **Percakapan baru** memulai dari awal.
**Buka di Skrip** menampilkan skripnya.

## Panel Buat dengan AI {#generate}

Panel [[docker.ai_generate]] ([[menu.tools > menu.tools.ai_generate]], atau
tombol AI di bilah alat standar) membuat gambar dengan model gambar dari
layanan Anda.

![Panel Buat dengan AI](../../img/customize-ai-generate.png)

Pilih mode dengan tombol di bagian atas:

- **Buat gambar**: gambar dari deskripsi. **Beri saya ide** mengisi sebuah
  gagasan. Pilih rasio aspek.
- **Remiks gambar**: mengubah gambar sesuai deskripsi Anda (awalnya bitmap
  yang dipilih). Permintaan siap pakai mencakup mockup, pulihkan foto, warnai
  sketsa, ganti latar belakang, ubah musim, dan warnai ulang.
- **Buat vektor**: ilustrasi yang diubah menjadi bentuk yang dapat disunting,
  baik gambar yang dijiplak menjadi kurva maupun gambar SVG yang ditulis oleh
  model teks.
- **Gambar dengan kata-kata** (lihat di atas).

Setiap mode memiliki **Model**, gambar referensi opsional (**Gunakan pilihan**
atau **Impor berkas**), **Perintah**, serta ubin **Gaya** dan **Palet warna**.
Di bagian bawah, atur jumlah **Gambar** (1 sampai 4) dan **Sisipkan ke
gambar**, lalu klik **Buat**. Hasil tetap ada di panel; klik salah satunya
untuk menyisipkannya.

## Perintah AI lainnya {#other}

- [[menu.bitmaps > menu.bitmaps.remove_background]] mempertahankan subjek
  gambar dan membuat sisanya transparan.
- [[menu.bitmaps.ai_art_styles]] melukis ulang pilihan dalam salah satu dari
  delapan gaya.

Selama AI dimatikan atau belum diatur, perintah-perintahnya berwarna abu-abu,
dan tipnya menjelaskan alasannya; tombol [[aiui.set_up]] membuka halaman opsi
yang sesuai.
