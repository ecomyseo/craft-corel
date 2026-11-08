# O programie TraceDraw i aktualizacje

TraceDraw jest wolny i ma otwarty kod. Ta strona wyjaśnia, jak sprawdzić, którą
wersję masz, jak zdobyć nowszą i gdzie prosić o pomoc.

## Okno O programie {#about}

Wybierz [[menu.help > menu.help.about]], aby zobaczyć uruchomioną wersję,
autora, licencję, łącze do kodu źródłowego i to, z czego zbudowany jest
program. Naciśnij [[dialog.close]], <kbd>Enter</kbd> lub <kbd>Esc</kbd>, aby je
zamknąć.

![Okno O programie TraceDraw](../../img/welcome-about-and-updates-about.png)

## Sprawdzanie aktualizacji {#updates}

Na komputerze TraceDraw może powiadomić Cię o wydaniu nowszej wersji.

Aby sprawdzić teraz:

1. Wybierz [[menu.help > menu.help.updates]].
2. TraceDraw pyta stronę wydań projektu o najnowszą wersję i mówi, czy jest
   nowsza od Twojej. Wersje testowe i kompilacje nocne nigdy nie są
   proponowane.
3. Gdy istnieje nowsza wersja, wybierz, co zrobić:
   - **Pobierz** otwiera pobieranie pliku dla Twojego systemu (instalatora lub
     archiwum przenośnego w Windows, obrazu dysku dla procesora Twojego Maca,
     AppImage lub archiwum w Linuksie). Instalację wykonuje potem instalator
     systemu.
   - **Informacje o wydaniu** otwiera listę zmian tej wersji.
   - **Pomiń tę wersję** przestaje proponować tę wersję.
   - [[dialog.close]] zamyka okno.

Gdy TraceDraw działa jako AppImage w Linuksie, okno może też zaproponować
**Zainstaluj**: pobiera nowy AppImage obok używanego, sprawdza, czy pobieranie
jest kompletne i autentyczne, i wstawia go w miejsce starego pliku. Nowa wersja
uruchomi się przy następnym otwarciu TraceDraw. Jeśli coś się nie zgadza, stary
plik pozostaje bez zmian, a okno wyjaśnia dlaczego.

Jeśli sprawdzanie się nie uda (na przykład bez połączenia z internetem), okno o
tym informuje i proponuje stronę wydań.

![Okno Aktualizacje](../../img/welcome-about-and-updates-updates.png)

### Automatyczne sprawdzanie {#automatic-check}

[[menu.tools > menu.tools.options > menu.tools.options_global]], strona
[[options.general]], ma ustawienie sprawdzania aktualizacji przy starcie.
Początkowo jest włączone: raz dziennie TraceDraw po cichu sprawdza przy starcie
i odzywa się tylko wtedy, gdy wyszła nowsza wersja, której nie pominięto.

Przy pierwszym uruchomieniu TraceDraw pyta, czy sprawdzać aktualizacje: [[updates.policy_always]] lub [[updates.policy_never]]. Dopóki nie odpowiesz, TraceDraw sam nie nawiązuje żadnego połączenia. Odpowiedź ustawia tę opcję, którą możesz tu zmienić w każdej chwili.

> **Uwaga:** Wersja w przeglądarce jest zawsze najnowsza. Tam
> [[menu.help > menu.help.updates]] otwiera stronę wydań.

## Ekran powitalny {#welcome-screen}

Ekran powitalny to pierwsza karta po uruchomieniu TraceDraw. Można z niego
utworzyć rysunek, otworzyć jeden z ostatnio używanych rysunków (każdy z
miniaturą; [[menu.file.clear_recent]] je zapomina), zacząć od szablonu,
przeczytać nowości i znaleźć materiały do nauki. Wybierz
[[menu.help > menu.help.welcome_screen]], aby go przywrócić.
[[menu.help > menu.help.video_tutorials]], [[menu.help.quick_start_guide]] i
[[menu.help.user_guide]] otwierają jego stronę nauki, a
[[menu.help.whats_new]] nowości.

## Społeczność i wsparcie {#community}

- [[menu.help > menu.help.community]] otwiera forum dyskusyjne projektu, gdzie
  można zadawać pytania i dzielić się pomysłami.
- [[menu.help > menu.help.support]] otwiera system zgłoszeń, gdzie można
  zgłosić problem.
- [[menu.help > menu.help.message_settings]] otwiera opcje, w których
  ustawia się komunikaty TraceDraw.
