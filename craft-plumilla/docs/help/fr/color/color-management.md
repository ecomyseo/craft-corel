# Gestion des couleurs

Les écrans affichent les couleurs avec de la lumière rouge, verte et bleue ;
les imprimantes mélangent des encres cyan, magenta, jaune et noire. La
gestion des couleurs utilise des **profils de couleur** pour décrire chaque
appareil, afin qu'une couleur garde son aspect de l'écran à la page
imprimée. Les réglages se trouvent dans
[[menu.tools > menu.tools.color_management]], la boîte de dialogue
[[color_mgmt.title]]. Rien ne change avant que vous cliquiez sur
[[dialog.ok]].

![La boîte de dialogue des paramètres de gestion des couleurs](../../img/color-color-management.png)

## Les profils du dessin {#document}

L'onglet [[color_mgmt.tab_document]] règle les profils du dessin actif.

- [[color_mgmt.current_profiles]] affiche les profils RVB, CMJN et niveaux
  de gris que le dessin utilise actuellement.
- Choisissez comment un nouveau profil est appliqué :
  - [[color_mgmt.assign]] : les valeurs de couleur restent et leur aspect
    change selon le nouveau profil ;
  - [[color_mgmt.convert]] : les couleurs CMJN sont converties pour garder le
    même aspect dans le nouveau profil.
- Choisissez les profils [[color_mgmt.rgb_profile]],
  [[color_mgmt.cmyk_profile]] et [[color_mgmt.grayscale]]. Chaque liste
  contient les profils par défaut, puis tous les profils trouvés sur votre
  ordinateur. [[color_mgmt.load_profiles]] ajoute des fichiers de profil
  (`.icc`, `.icm`).
- [[color_mgmt.primary_mode]] : CMJN ou RVB.
- [[color_mgmt.intent]] : comment les couleurs impossibles à reproduire sont
  ramenées : [[color_mgmt.intent_relative]] (au départ),
  [[color_mgmt.intent_perceptual]], [[color_mgmt.intent_saturation]] ou
  [[color_mgmt.intent_absolute]].

Le profil CMJN accompagne le dessin, y compris dans les fichiers `.cdr` à
partir de la version 16.

## Valeurs par défaut des nouveaux dessins {#default}

L'onglet [[color_mgmt.tab_default]] règle ce qu'utilisent les nouveaux
dessins et la façon dont les couleurs sont converties.

- [[color_mgmt.preset]] : des réglages prêts à l'emploi pour l'Amérique du
  Nord, l'Europe, le Japon, ou le travail pour le web et l'écran.
  [[color_mgmt.save_preset]] enregistre vos propres réglages sous un
  [[color_mgmt.preset_name]] ; [[color_mgmt.delete_preset]] en supprime un.
- Les profils [[color_mgmt.rgb_profile]], [[color_mgmt.cmyk_profile]] et
  [[color_mgmt.gray_profile]] par défaut, [[color_mgmt.primary_mode]] et
  [[color_mgmt.intent]].
- [[color_mgmt.conversion]] :
  - [[color_mgmt.engine]] : [[color_mgmt.engine_icc]], qui utilise les
    profils, ou [[color_mgmt.engine_builtin]] ;
  - [[color_mgmt.preserve_black]] : les couleurs faites uniquement d'encre
    noire restent uniquement en encre noire après conversion ;
  - [[color_mgmt.map_gray]] (actif au départ) : les gris convertis en CMJN
    n'utilisent que l'encre noire ;
  - [[dialog.black_point]] (actif au départ) : garde les nuances les plus
    sombres distinctes.
- [[color_mgmt.open_policy]] : ce qui arrive au profil d'un fichier que vous
  ouvrez, pour le RVB, le CMJN et les niveaux de gris :
  [[color_mgmt.open_use_embedded]], [[color_mgmt.open_assign_default]] ou
  [[color_mgmt.open_convert_default]]. [[color_mgmt.warn_mismatch_box]] et
  [[color_mgmt.warn_missing_box]] affichent un message dans la barre d'état
  quand un fichier a un autre profil ou n'en a aucun.
- [[color_mgmt.import_policy]] : [[color_mgmt.import_convert]] ou
  [[color_mgmt.import_assign]], pour ce que vous importez ou collez.
- [[color_mgmt.spot_definition]] : si les couleurs d'accompagnement sont
  définies par [[color_mgmt.spot_lab]], [[color_mgmt.spot_cmyk]] ou
  [[color_mgmt.spot_rgb]].

## Prévisualiser les couleurs d'impression {#proof}

[[menu.view > menu.view.proof_colors]], ou le bouton d'épreuve à l'extrémité
droite de la barre d'état, montre le dessin tel qu'il serait imprimé : les
couleurs que les encres CMJN ne peuvent pas imprimer paraissent plus ternes.
Le panneau [[docker.color]] signale aussi ces couleurs.

Sans profils chargés, TraceDraw convertit les couleurs avec son propre
modèle d'encres quadrichromiques sur papier couché.
