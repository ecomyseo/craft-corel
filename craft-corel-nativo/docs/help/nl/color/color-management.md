# Kleurbeheer

Beeldschermen tonen kleuren met rood, groen en blauw licht; printers mengen
cyaan, magenta, geel en zwarte inkt. Kleurbeheer gebruikt **kleurprofielen**
om elk apparaat te beschrijven, zodat een kleur er van het scherm tot de
afgedrukte pagina hetzelfde uitziet. De instellingen staan in
[[menu.tools > menu.tools.color_management]], het dialoogvenster
[[color_mgmt.title]]. Er verandert niets totdat u op [[dialog.ok]] klikt.

![Het dialoogvenster Instellingen voor kleurbeheer](../../img/color-color-management.png)

## De profielen van de tekening {#document}

Het tabblad [[color_mgmt.tab_document]] stelt de profielen van de actieve
tekening in.

- [[color_mgmt.current_profiles]] toont de RGB-, CMYK- en
  grijswaardenprofielen die de tekening nu gebruikt.
- Kies hoe een nieuw profiel wordt toegepast:
  - [[color_mgmt.assign]]: de kleurwaarden blijven en hun uiterlijk verandert
    naar het nieuwe profiel;
  - [[color_mgmt.convert]]: de CMYK-kleuren worden omgezet zodat ze er in het
    nieuwe profiel hetzelfde uitzien.
- Kies de profielen [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]] en
  [[color_mgmt.grayscale]]. Elke lijst bevat eerst de standaardprofielen en
  daarna elk profiel dat op uw computer is gevonden.
  [[color_mgmt.load_profiles]] voegt profielbestanden toe (`.icc`, `.icm`).
- [[color_mgmt.primary_mode]]: CMYK of RGB.
- [[color_mgmt.intent]]: hoe kleuren die niet weergegeven kunnen worden binnen
  bereik worden gebracht: [[color_mgmt.intent_relative]] (in het begin),
  [[color_mgmt.intent_perceptual]], [[color_mgmt.intent_saturation]] of
  [[color_mgmt.intent_absolute]].

Het CMYK-profiel gaat met de tekening mee, ook in `.cdr`-bestanden vanaf
versie 16.

## Standaardinstellingen voor nieuwe tekeningen {#default}

Het tabblad [[color_mgmt.tab_default]] stelt in wat nieuwe tekeningen
gebruiken en hoe kleuren worden omgezet.

- [[color_mgmt.preset]]: kant-en-klare instellingen voor Noord-Amerika,
  Europa, Japan, of werk voor web en scherm. [[color_mgmt.save_preset]] slaat
  uw eigen instellingen op onder een [[color_mgmt.preset_name]];
  [[color_mgmt.delete_preset]] verwijdert er een.
- De standaardprofielen [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]]
  en [[color_mgmt.gray_profile]], [[color_mgmt.primary_mode]] en
  [[color_mgmt.intent]].
- [[color_mgmt.conversion]]:
  - [[color_mgmt.engine]]: [[color_mgmt.engine_icc]], dat de profielen
    gebruikt, of [[color_mgmt.engine_builtin]];
  - [[color_mgmt.preserve_black]]: kleuren van alleen zwarte inkt blijven bij
    het omzetten alleen zwarte inkt;
  - [[color_mgmt.map_gray]] (in het begin aan): grijstinten die naar CMYK
    worden omgezet, gebruiken alleen zwarte inkt;
  - [[dialog.black_point]] (in het begin aan): houdt de donkerste tinten van
    elkaar te onderscheiden.
- [[color_mgmt.open_policy]]: wat er gebeurt met het profiel van een bestand
  dat u opent, voor RGB, CMYK en grijswaarden:
  [[color_mgmt.open_use_embedded]], [[color_mgmt.open_assign_default]] of
  [[color_mgmt.open_convert_default]]. [[color_mgmt.warn_mismatch_box]] en
  [[color_mgmt.warn_missing_box]] tonen een melding in de statusbalk als een
  bestand een ander profiel of geen profiel heeft.
- [[color_mgmt.import_policy]]: [[color_mgmt.import_convert]] of
  [[color_mgmt.import_assign]], voor wat u importeert of plakt.
- [[color_mgmt.spot_definition]]: of steunkleuren worden gedefinieerd door
  [[color_mgmt.spot_lab]], [[color_mgmt.spot_cmyk]] of
  [[color_mgmt.spot_rgb]].

## Drukkleuren vooraf bekijken {#proof}

[[menu.view > menu.view.proof_colors]], of de proefknop rechts in de
statusbalk, toont de tekening zoals die zou worden afgedrukt: kleuren die
CMYK-inkten niet kunnen afdrukken, zien er doffer uit. Het paneel
[[docker.color]] waarschuwt ook voor zulke kleuren.

Zonder geladen profielen zet TraceDraw kleuren om met een eigen model van
proceskleuren op gestreken papier.
