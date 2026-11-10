# Schuine randen

Een schuine rand laat de randen van een object verhoogd of uitgesneden lijken,
door één kant van de rand te belichten en de andere te beschaduwen. Het is een
live-effect, ingesteld in het paneel [[docker.bevel]]; open het met
[[menu.effects > menu.effects.bevel]].

![Het paneel Schuine rand](../../img/effects-bevel.png)

## Een object een schuine rand geven {#create}

1. Selecteer een gevuld object.
2. Kies in het paneel [[docker.bevel]] de stijl:
   - [[docker.soft_edge]]: de rand loopt schuin het object in, aan één kant
     belicht en aan de andere beschaduwd;
   - [[docker.emboss]]: het object lijkt uit de pagina gedrukt.
3. Stel het uiterlijk van de schuine rand in:
   - [[docker.depth]]: hoe ver de schuine rand van de rand af reikt, het
     object in bij een zachte rand en eruit bij reliëf;
   - [[docker.level]]: in hoeveel stappen licht en schaduw over de schuine
     rand worden verdeeld. 1 geeft twee vlakke tinten, meer stappen laten ze
     zacht verlopen vanaf de rand naar binnen;
   - [[docker.light_angle]]: de richting waaruit het licht komt;
   - [[docker.intensity]]: hoe sterk licht en schaduw zijn;
   - [[docker.light_color]] en [[docker.shadow_color]]: de kleuren van de
     belichte en de beschaduwde kant;
   - [[docker.emboss_color]]: de kleur waaruit de schuine rand wordt getint.
     Vink het vakje aan en kies een eigen kleur; uitgeschakeld wordt de
     vulling van het object gebruikt.
4. Klik op [[docker.apply]].

Om een schuine rand van een ander object te kopiëren, gebruikt u
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] verwijdert haar. Het paneel voor
extrusies heeft ook een schuine rand voor het voorvlak; zie
[Extrusies](extrude.md#settings).
