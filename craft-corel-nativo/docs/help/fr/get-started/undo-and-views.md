# Annuler, zoomer et se déplacer

Vous pouvez annuler toute modification et regarder votre dessin à n'importe
quel niveau de zoom et depuis n'importe quelle position. Zoomer et se
déplacer ne modifient jamais le dessin.

## Annuler et rétablir {#undo}

- [[menu.edit > menu.edit.undo]] (<kbd>Ctrl+Z</kbd>) annule la dernière
  modification ; [[menu.edit.redo]] (<kbd>Ctrl+Shift+Z</kbd>) la refait.
- Les flèches à côté d'Annuler et de Rétablir dans la barre d'outils standard
  listent les étapes. Choisissez une étape pour annuler ou rétablir toutes
  les modifications jusqu'à elle.
- [[menu.edit.repeat]] applique de nouveau la dernière commande, par exemple
  à l'objet suivant que vous sélectionnez.

### Le panneau Annuler {#undo-docker}

Le panneau [[docker.undo]] liste toutes les modifications du dessin, de la
plus ancienne à la plus récente. Cliquez sur une étape pour ramener le dessin
à son état après cette étape ; cliquez sur une étape plus récente pour
avancer de nouveau. Ouvrez-le depuis [[menu.window > menu.window.dockers]].

![Le panneau Annuler](../../img/get-started-undo-and-views-undo-docker.png)

## Zoomer {#zoom}

- Tournez la molette de la souris pour zoomer ou dézoomer autour du
  pointeur.
- Utilisez la zone de zoom de la barre d'outils standard : tapez un niveau,
  choisissez-en un dans sa liste (de 10 % à 1600 %), ou zoomez sur la
  sélection, sur tous les objets, sur la page, sur la largeur de la page ou
  sur sa hauteur.
- [[menu.view > menu.view.zoom_in]] et [[menu.view.zoom_out]] zooment d'un
  cran ; [[menu.view.zoom_to_fit]] (<kbd>F4</kbd>) affiche tous les objets,
  [[menu.view.zoom_to_page]] (<kbd>Shift+F4</kbd>) la page et
  [[menu.view.zoom_to_selected]] (<kbd>Shift+F2</kbd>) la sélection.
- Un pincement sur le pavé tactile zoome et, sur un écran tactile, deux
  doigts déplacent la vue et zooment avec n'importe quel outil.

### L'outil Zoom {#zoom-tool}

Choisissez l'outil [[tool.zoom]] (<kbd>Z</kbd> ou <kbd>F2</kbd>), puis :

- cliquez pour zoomer par deux à cet endroit ;
- <kbd>Shift</kbd>+clic ou clic droit pour dézoomer par deux ;
- faites glisser un rectangle pour zoomer de façon que le rectangle remplisse
  la fenêtre.

Double-cliquez sur le bouton de l'outil pour zoomer sur tous les objets.

## Se déplacer {#pan}

- Maintenez le bouton du milieu de la souris et faites glisser, avec
  n'importe quel outil.
- Choisissez l'outil [[tool.pan]] (<kbd>H</kbd>) et faites glisser.
- <kbd>Ctrl</kbd>+molette fait défiler verticalement, <kbd>Alt</kbd>+molette
  horizontalement.
- Utilisez les barres de défilement, qui couvrent la page et une taille de
  page autour d'elle.
- Maintenez le bouton Navigateur, dans le coin entre les barres de
  défilement, et déplacez le pointeur sur la vignette de la page.
- <kbd>PgUp</kbd> et <kbd>PgDn</kbd> vont à la page précédente et à la page
  suivante.

Pour que la molette fasse défiler au lieu de zoomer, changez l'action de la
molette dans la page [[options.zoom_pan]] de
[[menu.tools > menu.tools.options > menu.tools.options_tools]]. La molette
fait alors défiler, <kbd>Shift</kbd>+molette défile horizontalement et
<kbd>Ctrl</kbd>+molette zoome. Voir [Options des outils](../customize/tool-options.md#zoom-pan).

## Affichages {#views}

Le menu [[menu.view]] choisit comment les objets sont dessinés :
[[menu.view.wireframe]] n'affiche que les contours, [[menu.view.normal]] et
[[menu.view.enhanced]] dessinent les surfaces et les effets, et
[[menu.view.pixels]] montre le dessin sous forme des pixels d'une
exportation. [[menu.view.fullscreen_preview]] affiche la page seule à
l'écran ; appuyez sur une touche pour revenir.
[[menu.view.preview_selected_only]] limite cet aperçu à la sélection. Le menu
[[menu.view]] affiche ou masque aussi les règles, les grilles, les repères, le
fond perdu et la zone imprimable ; voir
[Règles, grilles et repères](../pages-layout/guidelines-and-grid.md).
