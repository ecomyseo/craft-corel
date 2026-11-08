# Obszary robocze

Obszar roboczy to sposób urządzenia okna: które paski są widoczne i gdzie,
przybornik, skróty klawiaturowe, opcje palety kolorów i wygląd interfejsu.
Wybierz go w menu [[menu.window > menu.window.workspace]] lub na stronie
[[options.workspace]] w
[[menu.tools > menu.tools.options > menu.tools.options_workspaces]].

![Opcje obszarów roboczych](../../img/customize-workspaces.png)

## Wbudowane obszary robocze {#built-in}

- [[menu.window.workspace_default]]: zwykła konfiguracja.
- [[menu.window.workspace_lite]]: mniej pasków, na pierwsze uruchomienie.
- [[menu.window.workspace_classic]], [[menu.window.workspace_illustration]]
  i [[menu.window.workspace_page_layout]]: paski i panele do tych rodzajów
  pracy.
- [[menu.window.workspace_touch]]: do ekranów dotykowych. Interfejs jest
  rysowany w większym rozmiarze, linijki, panele i pasek stanu są ukryte, a dwa
  palce przesuwają widok i powiększają go gestem szczypania przy dowolnym
  narzędziu. Rysunki zachowują swój rzeczywisty rozmiar.

## Co jest zachowywane {#kept}

Wszystko, co zmienisz, jest zachowywane po zamknięciu TraceDraw i wraca przy
następnym uruchomieniu: opcje, położenie każdego paska i to, które paski są
widoczne, narzędzie widoczne w każdej grupie przybornika, otwarte panele,
jednostki, przyciąganie, ustawienia przyjmowane przez nowe obiekty i to, co
pokazuje okno.

## Eksportowanie i importowanie konfiguracji {#export-import}

Aby przenieść konfigurację na inny komputer lub ją zabezpieczyć:

1. Na stronie [[options.workspace]], w sekcji [[options.workspace_transfer]],
   zaznacz części do uwzględnienia: skróty klawiaturowe; paski narzędzi,
   przybornik i pasek stanu; opcje palety kolorów; wygląd.
2. Kliknij [[options.workspace_export]] i zapisz plik `.tdws`.
3. Na drugim komputerze kliknij [[options.workspace_import]] i wybierz plik.
   Stosowane są tylko zaznaczone części.

[[menu.window > menu.window.workspace > menu.window.workspace_export]]
i [[menu.window.workspace_import]] robią to samo ze wszystkimi częściami.
