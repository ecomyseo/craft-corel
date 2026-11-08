# Licznik czasu

Licznik czasu zapisuje czas spędzony nad rysunkiem, zadanie po zadaniu,
i przechowuje te zapisy w rysunku (w plikach `.tdraw`). Czas jest liczony tylko
podczas pracy: po kilku minutach bezczynności zegar czeka, a Twoja następna
czynność uruchamia go ponownie.

## Pasek narzędzi Licznik czasu {#toolbar}

[[menu.window > menu.window.toolbars > menu.window.toolbar_time_tracker]]
wyświetla pasek z następującymi elementami:

- lista zadań, aby wybrać aktywne zadanie lub dodać nowe;
- czas aktywnego zadania;
- przycisk nagrywania i wstrzymania;
- lampka: czerwona podczas liczenia, bursztynowa podczas bezczynności, szara po
  wstrzymaniu lub bez zadania;
- suma wszystkich zadań;
- przycisk otwierający okno dialogowe.

Uruchomienie zegara w rysunku bez zadań dodaje „Zadanie 1”.

## Okno dialogowe Licznik czasu {#dialog}

[[menu.tools > menu.tools.time_tracker]] zawiera listę zadań z ich czasem,
początkiem i końcem oraz sumę.

![Okno dialogowe Licznik czasu](../../img/customize-time-tracker.png)

- **Nowe zadanie**, **Usuń**, **Ustaw jako aktywne** i **Resetuj czas** działają
  na zaznaczonym zadaniu. Kliknij zadanie dwukrotnie, aby uczynić je aktywnym.
- **Nagrywaj** i **Wstrzymaj** uruchamiają i zatrzymują zegar.
- Edytuj **nazwę**, **czas** (na przykład `1:30:00`, `1:30` lub `1.5` godziny),
  **początek** i **koniec** (na przykład `2026-10-19 14:30`) zaznaczonego
  zadania.
- **Przestań liczyć po N minutach bez aktywności** ustawia czas bezczynności
  (domyślnie 5 minut).
- **Eksportuj CSV** i **Eksportuj tekst** zapisują zapisy do arkusza
  kalkulacyjnego lub raportu.

Zapisywanie czasu nie jest edycją rysunku: nie tworzy kroku cofania. Zapisy są
zachowywane przy następnym zapisaniu rysunku.
