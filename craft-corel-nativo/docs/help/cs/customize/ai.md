# Kreslení a generování s AI

TraceDraw umí kreslit podle slovního popisu a vytvářet obrázky pomocí služby
AI, kterou si sami zvolíte a nastavíte vlastním klíčem. TraceDraw nemá
vlastní AI ani účet: nic se neodešle, dokud nespustíte příkaz AI, a i potom
se do nastavené služby odešle jen to, co daný příkaz potřebuje. Každý
požadavek vyúčtuje vaše služba; místní servery modelů ve vašem počítači nic
nestojí.

## Nastavení služby AI {#service}

[[menu.tools > menu.tools.options > menu.tools.options_ai]] má dvě stránky.

![Stránka Služba v možnostech AI](../../img/customize-ai-service.png)

Na stránce **Služba** nastavte jednu službu AI pro všechny funkce AI:

1. Zvolte **Předvolbu**: Anthropic, OpenAI, Google Gemini nebo server modelů
   ve vašem počítači (Ollama, LM Studio, llama.cpp server, vLLM). Předvolba
   vyplní druh služby a adresu; **Vlastní** přijme jakoukoli adresu, která
   mluví stejným jazykem jako chat completions od OpenAI.
2. Zadejte **Klíč API** svého účtu u této služby (místní servery jej
   nepotřebují). Zaškrtnutím **Zapamatovat si klíč na tomto počítači** jej
   uchováte; pak je uložen jako prostý text ve vašich předvolbách.
3. Zvolte **Model** a **Model obrázků** u služeb, které umějí vytvářet
   obrázky (OpenAI, Google Gemini nebo vlastní adresa s modelem obrázků).
4. Klikněte na [[aiui.test]]. Tím se služby zeptáte na její modely a uvidíte,
   kolik jich má, nebo její chybu.

**Zobrazit lištu pokynů** umístí lištu pokynů AI nad barevnou paletu.

## Obecné {#general}

![Stránka Obecné v možnostech AI](../../img/customize-ai-general.png)

- [[aiui.enable]] (zapnuto): po vypnutí jsou všechny příkazy AI zašedlé.
- [[aiui.services]]: zda jsou [[aiui.drawings]] a [[aiui.pictures]] ve stavu
  [[aiui.ready]] nebo [[aiui.not_ready]].
- [[aiui.features]]: co dělá každý příkaz AI.
- [[aiui.privacy_title]]: jen to, co příkaz potřebuje, do služby, kterou jste
  nastavili, a teprve když jej spustíte.

## Kreslení slovy {#draw-with-words}

Napište požadavek, například „červené jízdní kolo s modrými koly“ nebo
„zvětši kruh a dej pod něj nadpis“, do lišty pokynů nebo v režimu **Kreslit
slovy** panelu [[docker.ai_generate]]
([[menu.object > menu.object.ai_generate > menu.object.ai_draw_words]]) a
stiskněte **Nakreslit** nebo <kbd>Enter</kbd>. Služba napíše pro kresbu
skript, který TraceDraw spustí jako jeden krok zpět. Konverzace se uchovává,
takže můžete žádat o změny; **Nová konverzace** začne znovu. **Otevřít ve
Skriptech** zobrazí skript.

## Panel Generování AI {#generate}

Panel [[docker.ai_generate]] ([[menu.tools > menu.tools.ai_generate]] nebo
tlačítko generování AI na standardní liště) vytváří obrázky modelem obrázků
vaší služby.

![Panel Generování AI](../../img/customize-ai-generate.png)

Režim zvolte tlačítky nahoře:

- **Vytvořit obrázek**: obrázek podle popisu. **Inspiruj mě** doplní nápad.
  Zvolte poměr stran.
- **Remixovat obrázek**: změní obrázek podle vašeho popisu (výchozí je
  vybraná bitmapa). Mezi hotovými požadavky jsou maketa, obnova fotografie,
  obarvení skici, výměna pozadí, změna ročního období a přebarvení.
- **Vytvořit vektory**: ilustrace převedená na upravitelné tvary, buď obrázek
  vektorizovaný do křivek, nebo kresba SVG napsaná textovým modelem.
- **Kreslit slovy** (viz výše).

Každý režim má **Model**, volitelný referenční obrázek (**Použít výběr** nebo
**Importovat soubor**), **Pokyn** a dlaždice **Styl** a **Paleta barev**.
Dole nastavte počet **obrázků** (1 až 4) a **Vložit do kresby** a pak
klikněte na **Generovat**. Výsledky zůstanou v panelu; kliknutím na jeden jej
vložíte.

## Další příkazy AI {#other}

- [[menu.bitmaps > menu.bitmaps.remove_background]] ponechá hlavní motiv
  obrázku a zbytek zprůhlední.
- [[menu.bitmaps.ai_art_styles]] přemaluje výběr jedním z osmi stylů.

Dokud je AI vypnutá nebo nenastavená, jsou příkazy zašedlé a jejich tip říká
proč; tlačítko [[aiui.set_up]] otevře správnou stránku možností.
