# Bevel

Bevel membuat tepi objek terlihat timbul atau terukir, dengan menerangi satu
sisi tepi dan membayangi sisi lainnya. Ini adalah efek langsung, diatur di
panel [[docker.bevel]]; buka dengan [[menu.effects > menu.effects.bevel]].

![Panel Bevel](../../img/effects-bevel.png)

## Untuk memberi bevel pada objek {#create}

1. Pilih objek yang berisi isian.
2. Di panel [[docker.bevel]], pilih gayanya:
   - [[docker.soft_edge]]: tepi melandai ke dalam objek, terang di satu sisi
     dan berbayang di sisi lain;
   - [[docker.emboss]]: objek tampak menonjol keluar dari halaman.
3. Atur tampilan bevel:
   - [[docker.depth]]: seberapa jauh bevel menjangkau dari tepi, ke dalam
     objek pada tepi lembut dan ke luar pada timbul;
   - [[docker.level]]: berapa langkah cahaya dan bayangan dibagi di sepanjang
     bevel. 1 memberi dua warna datar, lebih banyak langkah memudarkannya
     dengan halus dari tepi ke dalam;
   - [[docker.light_angle]]: arah datangnya cahaya;
   - [[docker.intensity]]: kekuatan cahaya dan bayangan;
   - [[docker.light_color]] dan [[docker.shadow_color]]: warna sisi yang
     terang dan sisi yang berbayang;
   - [[docker.emboss_color]]: warna dasar tempat bevel diberi nada. Centang
     kotaknya dan pilih warna sendiri; jika mati, isian objek yang dipakai.
4. Klik [[docker.apply]].

Untuk menyalin bevel dari objek lain, gunakan
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] menghapusnya. Panel ekstrusi juga
memiliki bevel untuk sisi depan; lihat [Ekstrusi](extrude.md#settings).
