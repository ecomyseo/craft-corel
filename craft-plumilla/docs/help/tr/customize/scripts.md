# Betikler ve otomasyon

Betikler yinelenen işleri otomatikleştirir: verilerden şekiller çizmek, çok
sayıda nesneyi aynı anda değiştirmek veya bir dosya biçimi eklemek. Betikler
[[docker.scripts]] panelinde ([[menu.tools > menu.tools.scripts >
menu.tools.scripts_docker]]) JavaScript ile yazılır.

![Betikler paneli](../../img/customize-scripts.png)

## Betik yazma ve çalıştırma {#run}

Bir betik TraceDraw'un nesneleriyle çalışır: `Application`,
`ActiveDocument`, `ActivePage`, `ActiveLayer` ve şekiller, dolgular,
anahatlar, renkler, eğriler ve efektler. Örneğin:

```js
for (let i = 0; i < 5; i++) {
  ActiveLayer.CreateRectangle(10 + i * 25, 10, 20, 20);
}
```

- [[docker.run]] betiği çalıştırır. Yaptığı değişiklikler tek bir geri alma
  adımıdır.
- [[docker.record]] yaptıklarınızı bir betik olarak kaydeder (ayrıca
  [[menu.tools > menu.tools.scripts > menu.tools.scripts_record]]).
- [[docker.load]] ve [[docker.save]] betikleri açar ve kaydeder;
  [[menu.tools.scripts_run]] bir betik dosyasını çalıştırır.
- [[docker.clear]] düzenleyiciyi boşaltır; [[docker.output]] betiğin
  yazdırdıklarını ve hatalarını gösterir.

## Hata ayıklama {#debug}

Masaüstünde bir kesme noktası koymak için düzenleyicinin kenar boşluğuna
tıklayın, ardından [[docker.debug]] düğmesine tıklayın. Betik kesme noktasında
durur: onu **Devam** ettirin, **Adım** adım ilerletin veya **Durdur**un ve
betiğin duraklatıldığı yerde değerlendirmek için bir ifade yazın.

## Betik filtreleri {#filters}

Bir adı, uzantıları ve bir `Import` veya `Export` işleviyle `TraceDrawFilter`
tanımlayan bir betik, [[menu.file.import]] ve [[menu.file.export]]
komutlarına bir dosya biçimi ekler. Saklamak için [[docker.install_filter]]
düğmesine tıklayın; [[docker.script_filters]] yüklü filtreleri
[[docker.edit]] ve [[docker.remove]] ile birlikte listeler.

## YZ asistanları (MCP) {#mcp}

TraceDraw Model Context Protocol'ü konuşur; böylece bilgisayarınızdaki YZ
asistanları çizimleri açabilir, düzenleyebilir, inceleyebilir, dışa
aktarabilir ve kaydedebilir:

- `tracedraw --mcp`, TraceDraw'u bir asistan için penceresiz çalıştırır.
- Bir asistanın pencerenizde açık olan çizimi düzenlemesine izin vermek
  için [[menu.tools > menu.tools.options > menu.tools.options_global]]
  altındaki [[options.general]] sayfasında [[options.mcp_window]]
  seçeneğini açın ve asistana komut olarak `tracedraw --mcp-window` verin.
  Her değişiklik geri alabileceğiniz bir geri alma adımıdır. Yalnızca
  bilgisayarınızda bulunan ve TraceDraw'un yazdığı gizli belirteci taşıyan
  programlar bağlanabilir.

## Komut satırı aracı {#cli}

Her indirmede, toplu işler için çizimleri programı açmadan biçimler arasında
dönüştüren `tracedraw-cli` de bulunur.

## Makrolar {#macros}

TraceDraw, çizim programları için yazılmış VBA makro projelerini çalıştırır (masaüstü sürümü). [[menu.tools > menu.tools.macros > menu.tools.macros_load]] bir proje (`.gms`) veya modül dosyaları (`.bas`, `.cls`, `.frm`) ekler; projeler saklanır ve açılışta yüklenir. [[docker.macros]] paneli makroları listeler: birine çift tıklayın veya çalıştırmak için [[menu.tools > menu.tools.macros > menu.tools.macros_run]] (<kbd>Alt+Shift+F8</kbd>) kullanın. Makronun formları, ileti ve giriş kutuları ekranda görünür, <kbd>Esc</kbd> veya [[menu.tools > menu.tools.macros > menu.tools.macros_stop]] onu durdurur ve bir çalıştırmanın değiştirdiği her şey tek bir geri alma adımıdır. [[menu.tools > menu.tools.macros > menu.tools.macro_editor]] (<kbd>Alt+F11</kbd>) kodu gösterir ve düzenler. Windows işlevlerini veya düzenleyicinin kendi nesne modelini çağıran makrolar çalışmaz.
