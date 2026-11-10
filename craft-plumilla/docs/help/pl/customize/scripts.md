# Skrypty i automatyzacja

Skrypty automatyzują powtarzalną pracę: rysują kształty na podstawie danych,
zmieniają naraz wiele obiektów lub dodają format pliku. Pisze się je w języku
JavaScript w panelu [[docker.scripts]] ([[menu.tools > menu.tools.scripts >
menu.tools.scripts_docker]]).

![Panel Skrypty](../../img/customize-scripts.png)

## Pisanie i uruchamianie skryptów {#run}

Skrypt działa na obiektach TraceDraw: `Application`, `ActiveDocument`,
`ActivePage`, `ActiveLayer` oraz kształtach, wypełnieniach, konturach,
kolorach, krzywych i efektach. Na przykład:

```js
for (let i = 0; i < 5; i++) {
  ActiveLayer.CreateRectangle(10 + i * 25, 10, 20, 20);
}
```

- [[docker.run]] uruchamia skrypt. Jego zmiany stanowią jeden krok cofania.
- [[docker.record]] nagrywa Twoje czynności jako skrypt (również
  [[menu.tools > menu.tools.scripts > menu.tools.scripts_record]]).
- [[docker.load]] i [[docker.save]] otwierają i zapisują skrypty;
  [[menu.tools.scripts_run]] uruchamia plik skryptu.
- [[docker.clear]] opróżnia edytor; [[docker.output]] pokazuje to, co skrypt
  wypisuje, oraz jego błędy.

## Debugowanie {#debug}

W wersji na komputer kliknij margines edytora, aby ustawić punkt przerwania,
a następnie kliknij [[docker.debug]]. Skrypt zatrzymuje się w punkcie
przerwania: **Kontynuuj**, **Krok** lub **Zatrzymaj**, a także wpisz wyrażenie,
aby obliczyć je w miejscu, w którym skrypt jest wstrzymany.

## Filtry skryptowe {#filters}

Skrypt, który definiuje `TraceDrawFilter` z nazwą, rozszerzeniami i funkcją
`Import` lub `Export`, dodaje format pliku do poleceń [[menu.file.import]]
i [[menu.file.export]]. Kliknij [[docker.install_filter]], aby go zachować;
[[docker.script_filters]] zawiera listę zainstalowanych filtrów z przyciskami
[[docker.edit]] i [[docker.remove]].

## Asystenci AI (MCP) {#mcp}

TraceDraw obsługuje protokół Model Context Protocol, dzięki czemu asystenci AI
na Twoim komputerze mogą otwierać, edytować, badać, eksportować i zapisywać
rysunki:

- `tracedraw --mcp` uruchamia TraceDraw bez okna dla asystenta.
- Aby pozwolić asystentowi edytować rysunek otwarty w Twoim oknie, włącz
  [[options.mcp_window]] na stronie [[options.general]] w
  [[menu.tools > menu.tools.options > menu.tools.options_global]] i podaj
  asystentowi polecenie `tracedraw --mcp-window`. Każda zmiana to jeden krok
  cofania, który możesz wycofać. Połączyć się mogą tylko programy na Twoim
  komputerze, które mają tajny token zapisywany przez TraceDraw.

## Narzędzie wiersza poleceń {#cli}

Każde pobranie zawiera też `tracedraw-cli`, które konwertuje rysunki między
formatami bez otwierania programu, do pracy wsadowej.

## Makra {#macros}

TraceDraw uruchamia projekty makr VBA napisane dla programów do rysowania (wersja na komputer). [[menu.tools > menu.tools.macros > menu.tools.macros_load]] dodaje projekt (`.gms`) lub pliki modułów (`.bas`, `.cls`, `.frm`); projekty są zachowywane i wczytywane przy starcie. Panel [[docker.macros]] pokazuje ich makra: kliknij dwukrotnie makro lub użyj [[menu.tools > menu.tools.macros > menu.tools.macros_run]] (<kbd>Alt+Shift+F8</kbd>), aby je uruchomić. Formularze, okna komunikatów i okna wprowadzania makra pojawiają się na ekranie, <kbd>Esc</kbd> lub [[menu.tools > menu.tools.macros > menu.tools.macros_stop]] je zatrzymuje, a wszystko, co zmieni jedno uruchomienie, to jeden krok cofania. [[menu.tools > menu.tools.macros > menu.tools.macro_editor]] (<kbd>Alt+F11</kbd>) pokazuje i edytuje kod. Makra wywołujące funkcje Windows lub model obiektów samego edytora nie działają.
