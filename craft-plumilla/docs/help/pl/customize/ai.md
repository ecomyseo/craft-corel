# Rysowanie i generowanie z AI

TraceDraw potrafi rysować na podstawie opisu słownego i tworzyć obrazy za pomocą
wybranej przez Ciebie usługi AI, skonfigurowanej z Twoim własnym kluczem.
TraceDraw nie ma własnej sztucznej inteligencji ani konta: nic nie jest
wysyłane, dopóki nie uruchomisz polecenia AI, a wtedy do skonfigurowanej usługi
trafia tylko to, czego potrzebuje dane polecenie. Każde żądanie jest rozliczane
przez Twoją usługę; lokalne serwery modeli na Twoim komputerze nic nie kosztują.

## Konfigurowanie usługi AI {#service}

[[menu.tools > menu.tools.options > menu.tools.options_ai]] ma dwie strony.

![Strona Usługa opcji AI](../../img/customize-ai-service.png)

Na stronie **Usługa** skonfiguruj jedną usługę AI dla wszystkich funkcji AI:

1. Wybierz **Ustawienie wstępne**: Anthropic, OpenAI, Google Gemini lub serwer
   modeli na Twoim komputerze (Ollama, LM Studio, llama.cpp server, vLLM).
   Ustawienie wstępne wypełnia rodzaj usługi i adres; **Własne** przyjmuje
   dowolny adres, który obsługuje ten sam protokół co chat completions OpenAI.
2. Wpisz **klucz API** swojego konta w tej usłudze (serwery lokalne go nie
   wymagają). Zaznacz **Zapamiętaj klucz na tym komputerze**, aby go zachować;
   jest on wtedy przechowywany jako zwykły tekst w Twoich preferencjach.
3. Wybierz **Model** oraz **Model obrazów** dla usług, które potrafią tworzyć
   obrazy (OpenAI, Google Gemini lub własny adres z modelem obrazów).
4. Kliknij [[aiui.test]]. Polecenie pyta usługę o jej modele i podaje, ile ich
   ma, albo wyświetla jej błąd.

**Pokaż pasek poleceń** umieszcza pasek poleceń AI nad paletą kolorów.

## Ogólne {#general}

![Strona Ogólne opcji AI](../../img/customize-ai-general.png)

- [[aiui.enable]] (włączone): po wyłączeniu każde polecenie AI jest wyszarzone.
- [[aiui.services]]: czy funkcje [[aiui.drawings]] i [[aiui.pictures]] mają
  stan [[aiui.ready]] czy [[aiui.not_ready]].
- [[aiui.features]]: co robi każde polecenie AI.
- [[aiui.privacy_title]]: tylko to, czego potrzebuje polecenie, do
  skonfigurowanej usługi, w chwili jego uruchomienia.

## Rysowanie słowami {#draw-with-words}

Wpisz żądanie, na przykład „czerwony rower z niebieskimi kołami” lub „powiększ
okrąg i umieść tytuł pod nim”, na pasku poleceń lub w trybie **Rysuj słowami**
panelu [[docker.ai_generate]]
([[menu.object > menu.object.ai_generate > menu.object.ai_draw_words]])
i naciśnij **Rysuj** lub <kbd>Enter</kbd>. Usługa pisze skrypt rysunku, który
TraceDraw wykonuje jako jeden krok cofania. Rozmowa jest zachowywana, więc
możesz prosić o zmiany; **Nowa rozmowa** zaczyna od nowa. **Otwórz w Skryptach**
pokazuje skrypt.

## Panel Generowanie AI {#generate}

Panel [[docker.ai_generate]] ([[menu.tools > menu.tools.ai_generate]] lub
przycisk Generowanie AI na standardowym pasku narzędzi) tworzy obrazy za pomocą
modelu obrazów Twojej usługi.

![Panel Generowanie AI](../../img/customize-ai-generate.png)

Wybierz tryb przyciskami u góry:

- **Utwórz obraz**: obraz na podstawie opisu. **Zainspiruj mnie** podsuwa
  pomysł. Wybierz proporcje obrazu.
- **Przerób obraz**: zmienia obraz zgodnie z opisem (domyślnie zaznaczoną
  bitmapę). Gotowe żądania obejmują makietę, renowację zdjęcia, pokolorowanie
  szkicu, zamianę tła, zmianę pory roku i zmianę kolorów.
- **Utwórz wektory**: ilustracja zamieniona w edytowalne kształty, albo obraz
  zwektoryzowany do krzywych, albo rysunek SVG napisany przez model tekstowy.
- **Rysuj słowami** (patrz wyżej).

Każdy tryb ma **Model**, opcjonalny obraz referencyjny (**Użyj zaznaczenia**
lub **Importuj plik**), **Polecenie** oraz kafelki **Styl** i **Paleta
kolorów**. U dołu ustaw liczbę **Obrazów** (od 1 do 4) i opcję **Wstaw do
rysunku**, a następnie kliknij **Generuj**. Wyniki pozostają w panelu; kliknij
jeden z nich, aby go wstawić.

## Inne polecenia AI {#other}

- [[menu.bitmaps > menu.bitmaps.remove_background]] zachowuje główny motyw
  obrazu, a resztę czyni przezroczystą.
- [[menu.bitmaps.ai_art_styles]] przemalowuje zaznaczenie w jednym z ośmiu
  stylów.

Gdy AI jest wyłączona lub nieskonfigurowana, polecenia są wyszarzone, a ich
podpowiedź wyjaśnia dlaczego; przycisk [[aiui.set_up]] otwiera właściwą stronę
opcji.
