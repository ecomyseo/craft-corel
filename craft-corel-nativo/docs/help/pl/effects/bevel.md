# Fazowanie

Fazowanie sprawia, że krawędzie obiektu wyglądają na wypukłe lub wyrzeźbione,
przez oświetlenie jednej strony krawędzi i zacienienie drugiej. Jest to efekt
dynamiczny, ustawiany w panelu [[docker.bevel]]; otwórz go poleceniem
[[menu.effects > menu.effects.bevel]].

![Panel Fazowanie](../../img/effects-bevel.png)

## Fazowanie obiektu {#create}

1. Zaznacz wypełniony obiekt.
2. W panelu [[docker.bevel]] wybierz styl:
   - [[docker.soft_edge]]: krawędź opada ku wnętrzu obiektu, oświetlona z jednej
     strony i zacieniona z drugiej;
   - [[docker.emboss]]: obiekt wydaje się wytłoczony ze strony.
3. Ustaw wygląd fazowania:
   - [[docker.depth]]: jak daleko fazowanie sięga od krawędzi, w głąb
     obiektu przy miękkiej krawędzi i na zewnątrz przy płaskorzeźbie;
   - [[docker.level]]: na ile kroków światło i cień są podzielone wzdłuż
     fazowania. 1 daje dwa płaskie odcienie, więcej kroków płynnie je
     przechodzi od krawędzi do wnętrza;
   - [[docker.light_angle]]: kierunek, z którego pada światło;
   - [[docker.intensity]]: siła światła i cienia;
   - [[docker.light_color]] i [[docker.shadow_color]]: kolory oświetlonej
     i zacienionej strony;
   - [[docker.emboss_color]]: kolor, z którego tonowane jest fazowanie.
     Zaznacz pole i wybierz własny kolor; wyłączony: używane jest
     wypełnienie obiektu.
4. Kliknij [[docker.apply]].

Aby skopiować fazowanie z innego obiektu, użyj
[[menu.object > menu.object.copy_effect > menu.object.copy_bevel_from]].
[[menu.object > menu.object.clear_effect]] usuwa je. Panel wytłaczania ma też
fazowanie przedniej ściany; zobacz [Wytłaczanie](extrude.md#settings).
