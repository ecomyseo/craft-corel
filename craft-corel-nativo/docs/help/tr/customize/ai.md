# YZ ile çizme ve üretme

TraceDraw, kendi anahtarınızla seçip kurduğunuz bir YZ hizmetiyle sözcüklerle
yapılan bir açıklamadan çizim yapabilir ve resimler oluşturabilir.
TraceDraw'un kendine ait bir YZ'si ve hesabı yoktur: bir YZ komutu
çalıştırana kadar hiçbir şey gönderilmez ve o zaman da yalnızca o komutun
gerektirdiği şey kurduğunuz hizmete gider. Her istek hizmetiniz tarafından
faturalandırılır; bilgisayarınızdaki yerel model sunucuları hiçbir ücrete
mal olmaz.

## YZ hizmetini kurma {#service}

[[menu.tools > menu.tools.options > menu.tools.options_ai]] iki sayfadan
oluşur.

![YZ seçeneklerinin Hizmet sayfası](../../img/customize-ai-service.png)

**Hizmet** sayfasında her YZ özelliği için tek bir YZ hizmeti kurun:

1. Bir **Ön ayar** seçin: Anthropic, OpenAI, Google Gemini veya
   bilgisayarınızdaki bir model sunucusu (Ollama, LM Studio, llama.cpp
   server, vLLM). Ön ayar hizmet türünü ve adresi doldurur; **Özel**,
   OpenAI'ın chat completions arabirimiyle aynı dili konuşan herhangi bir
   adresi kabul eder.
2. O hizmetteki hesabınızın **API anahtarını** yazın (yerel sunucular
   gerektirmez). Anahtarı saklamak için **Anahtarı bu bilgisayarda hatırla**
   seçeneğini işaretleyin; anahtar bu durumda tercihlerinizde düz metin
   olarak saklanır.
3. **Modeli** ve resim yapabilen hizmetler için (OpenAI, Google Gemini veya
   resim modeli olan özel bir adres) **Resim modelini** seçin.
4. [[aiui.test]] düğmesine tıklayın. Hizmetten modellerini ister ve kaç
   tane olduğunu söyler ya da hatasını gösterir.

**İstem çubuğunu göster**, YZ istem çubuğunu renk paletinin üstüne koyar.

## Genel {#general}

![YZ seçeneklerinin Genel sayfası](../../img/customize-ai-general.png)

- [[aiui.enable]] (açık): kapatıldığında her YZ komutu soluk görünür.
- [[aiui.services]]: [[aiui.drawings]] ve [[aiui.pictures]] öğelerinin
  [[aiui.ready]] mi yoksa [[aiui.not_ready]] mı olduğu.
- [[aiui.features]]: her YZ komutunun ne yaptığı.
- [[aiui.privacy_title]]: yalnızca bir komutun gerektirdiği şey, kurduğunuz
  hizmete, onu çalıştırdığınızda.

## Sözcüklerle çizme {#draw-with-words}

İstem çubuğuna veya [[docker.ai_generate]] panelinin **Sözcüklerle çiz**
modunda
([[menu.object > menu.object.ai_generate > menu.object.ai_draw_words]])
"mavi tekerlekli kırmızı bir bisiklet" ya da "daireyi büyüt ve başlığı
altına koy" gibi bir istek yazın ve **Çiz** düğmesine veya <kbd>Enter</kbd>
tuşuna basın. Hizmet çizim için bir betik yazar; TraceDraw bu betiği tek bir
geri alma adımı olarak çalıştırır. Konuşma saklanır, böylece değişiklik
isteyebilirsiniz; **Yeni konuşma** baştan başlar. **Betikler'de Aç** betiği
gösterir.

## YZ Üret paneli {#generate}

[[docker.ai_generate]] paneli ([[menu.tools > menu.tools.ai_generate]] veya
standart araç çubuğundaki YZ Üret düğmesi), hizmetinizin resim modeliyle
resimler oluşturur.

![YZ Üret paneli](../../img/customize-ai-generate.png)

Üstteki düğmelerle bir mod seçin:

- **Resim oluştur**: bir açıklamadan bir resim. **Bana ilham ver** bir fikir
  doldurur. En boy oranını seçin.
- **Resmi yeniden karıştır**: bir resmi tarif ettiğiniz gibi değiştirir
  (başlangıçta seçili bitmap). Hazır istekler arasında maket, fotoğraf
  onarma, eskizi renklendirme, arka planı değiştirme, mevsimi değiştirme ve
  yeniden renklendirme bulunur.
- **Vektör oluştur**: düzenlenebilir şekillere dönüştürülmüş bir
  illüstrasyon; ya eğrilere izlenmiş bir resim ya da metin modelinin yazdığı
  bir SVG çizimi.
- **Sözcüklerle çiz** (yukarıya bakın).

Her modda bir **Model**, isteğe bağlı bir başvuru resmi (**Seçimi kullan**
veya **Dosya al**), **İstem** ve **Stil** ile **Renk paleti** kutucukları
bulunur. Altta **Resim** sayısını (1 ile 4 arası) ve **Çizime ekle**
seçeneğini ayarlayın, ardından **Üret** düğmesine tıklayın. Sonuçlar panelde
kalır; eklemek için birine tıklayın.

## Diğer YZ komutları {#other}

- [[menu.bitmaps > menu.bitmaps.remove_background]] bir resmin konusunu
  korur ve geri kalanını saydam yapar.
- [[menu.bitmaps.ai_art_styles]] seçimi sekiz stilden birinde yeniden boyar.

YZ kapalıyken veya kurulmamışken komutlar soluk görünür ve ipuçları nedenini
söyler; [[aiui.set_up]] düğmesi doğru seçenekler sayfasını açar.
