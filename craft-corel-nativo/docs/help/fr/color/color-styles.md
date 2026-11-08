# Styles de couleur

Un **style de couleur** est une couleur nommée utilisée par des objets du
dessin. Modifiez le style et tous les objets remplis avec lui changent
aussi : les couleurs d'une création restent cohérentes et il est rapide
d'essayer d'autres jeux de couleurs. Les styles sont listés dans le panneau
[[docker.color_styles]] ; ouvrez-le depuis
[[menu.window > menu.window.dockers]].

![Le panneau Styles de couleur](../../img/color-color-styles.png)

## Créer des styles de couleur {#create}

- [[docker.new_from_selection]] ajoute un style pour chaque couleur de
  surface des objets sélectionnés qui n'est pas encore un style.
- [[docker.new_color_style]] ajoute un style à partir de la couleur actuelle
  du panneau [[docker.color]].

## Utiliser et modifier les styles {#edit}

Chaque style affiche sa couleur, son nom et trois boutons :

- cliquez sur la couleur pour la changer : les objets remplis avec cette
  couleur suivent ;
- cliquez sur le nom pour le renommer ;
- [[docker.fill]] remplit la sélection avec le style, [[docker.outline]]
  colore son contour ;
- **x** supprime le style (les objets gardent leur couleur).

## Harmonies {#harmonies}

L'[[docker.harmony_editor]] crée un ensemble de styles qui vont bien
ensemble, à partir d'une [[docker.base_color]] :

1. Choisissez la [[docker.base_color]].
2. Cliquez sur une harmonie : [[harmony.analogous]],
   [[harmony.complementary]], [[harmony.split_complementary]],
   [[harmony.triadic]], [[harmony.tetradic]] ou [[harmony.monochromatic]].

Les couleurs de l'harmonie sont ajoutées comme styles, nommés d'après elle.
