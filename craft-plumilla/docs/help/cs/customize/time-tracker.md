# Sledování času

Sledování času zaznamenává čas, který nad kresbou strávíte, úkol po úkolu, a
uchovává záznamy v kresbě (v souborech `.tdraw`). Čas se počítá jen během
práce: po několika minutách nečinnosti hodiny čekají a vaše další akce je
opět spustí.

## Panel nástrojů Sledování času {#toolbar}

[[menu.window > menu.window.toolbars > menu.window.toolbar_time_tracker]]
zobrazí lištu s těmito prvky:

- seznam úkolů pro volbu aktivního úkolu nebo přidání nového;
- čas aktivního úkolu;
- tlačítko nahrávání a pozastavení;
- kontrolka: červená během počítání, oranžová při nečinnosti, šedá při
  pozastavení nebo bez úkolu;
- součet všech úkolů;
- tlačítko, které otevře dialogové okno.

Spuštění hodin v kresbě bez úkolů přidá „Úkol 1“.

## Dialogové okno Sledování času {#dialog}

[[menu.tools > menu.tools.time_tracker]] uvádí úkoly s jejich časem,
začátkem a koncem a celkový součet.

![Dialogové okno Sledování času](../../img/customize-time-tracker.png)

- **Nový úkol**, **Odstranit**, **Nastavit jako aktivní** a **Vynulovat čas**
  působí na vybraný úkol. Dvojitým kliknutím na úkol jej nastavíte jako
  aktivní.
- **Nahrávat** a **Pozastavit** spouštějí a zastavují hodiny.
- Upravte u vybraného úkolu **název**, **čas** (například `1:30:00`, `1:30`
  nebo `1.5` hodiny), **začátek** a **konec** (například `2026-10-19 14:30`).
- **Přestat počítat po N minutách bez vstupu** nastaví dobu nečinnosti
  (výchozí 5 minut).
- **Exportovat CSV** a **Exportovat text** uloží záznamy pro tabulkový
  procesor nebo do zprávy.

Zaznamenávání času není úpravou kresby: nevytváří krok zpět. Záznamy se
uloží při dalším uložení kresby.
