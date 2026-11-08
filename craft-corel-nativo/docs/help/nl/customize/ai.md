# Tekenen en genereren met AI

TraceDraw kan tekenen op basis van een beschrijving in woorden en afbeeldingen
maken met een AI-dienst die u zelf kiest en met uw eigen sleutel instelt.
TraceDraw heeft geen eigen AI en geen account: er wordt niets verzonden totdat
u een AI-opdracht uitvoert, en dan gaat alleen wat die opdracht nodig heeft
naar de dienst die u hebt ingesteld. Elk verzoek wordt door uw dienst in
rekening gebracht; lokale modelservers op uw computer kosten niets.

## De AI-dienst instellen {#service}

[[menu.tools > menu.tools.options > menu.tools.options_ai]] heeft twee pagina's.

![De pagina Dienst van de AI-opties](../../img/customize-ai-service.png)

Stel op de pagina **Dienst** één AI-dienst in voor alle AI-functies:

1. Kies een **Voorinstelling**: Anthropic, OpenAI, Google Gemini, of een
   modelserver op uw computer (Ollama, LM Studio, llama.cpp server, vLLM). Een
   voorinstelling vult het soort dienst en het adres in; **Aangepast**
   accepteert elk adres dat dezelfde taal spreekt als de chat completions van
   OpenAI.
2. Typ de **API-sleutel** van uw account bij die dienst (lokale servers hebben
   er geen nodig). Vink **De sleutel op deze computer onthouden** aan om hem te
   bewaren; hij wordt dan als platte tekst in uw voorkeuren opgeslagen.
3. Kies het **Model**, en het **Beeldmodel** voor diensten die afbeeldingen
   kunnen maken (OpenAI, Google Gemini, of een aangepast adres met een
   beeldmodel).
4. Klik op [[aiui.test]]. Dit vraagt de dienst om zijn modellen en meldt
   hoeveel er zijn, of toont de foutmelding.

**De promptbalk tonen** plaatst de AI-promptbalk boven het kleurenpalet.

## Algemeen {#general}

![De pagina Algemeen van de AI-opties](../../img/customize-ai-general.png)

- [[aiui.enable]] (aan): uitgeschakeld wordt elke AI-opdracht grijs.
- [[aiui.services]]: of [[aiui.drawings]] en [[aiui.pictures]]
  [[aiui.ready]] of [[aiui.not_ready]] zijn.
- [[aiui.features]]: wat elke AI-opdracht doet.
- [[aiui.privacy_title]]: alleen wat een opdracht nodig heeft, naar de dienst
  die u hebt ingesteld, op het moment dat u haar uitvoert.

## Tekenen met woorden {#draw-with-words}

Typ een verzoek zoals "een rode fiets met blauwe wielen" of "maak de cirkel
groter en zet de titel eronder", in de promptbalk of in de modus **Tekenen met
woorden** van het paneel [[docker.ai_generate]]
([[menu.object > menu.object.ai_generate > menu.object.ai_draw_words]]), en
druk op **Tekenen** of <kbd>Enter</kbd>. De dienst schrijft een script voor de
tekening, dat TraceDraw als één ongedaan-makenstap uitvoert. Het gesprek wordt
bewaard, zodat u om wijzigingen kunt vragen; **Nieuw gesprek** begint opnieuw.
**Openen in Scripts** toont het script.

## Het paneel AI-genereren {#generate}

Het paneel [[docker.ai_generate]] ([[menu.tools > menu.tools.ai_generate]], of
de knop AI-genereren op de standaardwerkbalk) maakt afbeeldingen met het
beeldmodel van uw dienst.

![Het paneel AI-genereren](../../img/customize-ai-generate.png)

Kies een modus met de knoppen bovenaan:

- **Afbeelding maken**: een afbeelding op basis van een beschrijving.
  **Inspireer mij** vult een idee in. Kies de beeldverhouding.
- **Afbeelding remixen**: wijzigt een afbeelding zoals u beschrijft (in het
  begin de geselecteerde bitmap). Kant-en-klare verzoeken zijn onder meer
  mockup, foto herstellen, schets inkleuren, achtergrond vervangen, seizoen
  wijzigen en herkleuren.
- **Vectoren maken**: een illustratie omgezet in bewerkbare vormen, ofwel een
  afbeelding die tot curven is overgetrokken, ofwel een SVG-tekening die door
  het tekstmodel is geschreven.
- **Tekenen met woorden** (zie hierboven).

Elke modus heeft een **Model**, een optionele referentieafbeelding
(**Selectie gebruiken** of **Bestand importeren**), de **Prompt**, en de
tegels **Stijl** en **Kleurenpalet**. Stel onderaan het aantal
**Afbeeldingen** (1 tot 4) en **In tekening invoegen** in en klik daarna op
**Genereren**. Resultaten blijven in het paneel; klik op een resultaat om het
in te voegen.

## Andere AI-opdrachten {#other}

- [[menu.bitmaps > menu.bitmaps.remove_background]] behoudt het onderwerp van
  een afbeelding en maakt de rest transparant.
- [[menu.bitmaps.ai_art_styles]] schildert de selectie opnieuw in een van acht
  stijlen.

Zolang AI uit staat of niet is ingesteld, zijn de opdrachten grijs en zegt
hun tip waarom; de knop [[aiui.set_up]] opent de juiste optiepagina.
