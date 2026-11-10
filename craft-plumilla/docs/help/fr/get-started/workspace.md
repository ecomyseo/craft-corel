# L'espace de travail

La fenêtre de TraceDraw contient vos dessins et tout ce qu'il faut pour y
travailler. Cette page en nomme les parties, pour que le reste de cette Aide
soit clair.

## Les parties de la fenêtre {#parts}

De haut en bas et de gauche à droite :

- La **barre de menus** contient toutes les commandes, dans les menus
  [[menu.file]], [[menu.edit]], [[menu.view]], [[menu.object]],
  [[menu.text]], [[menu.bitmaps]], [[menu.effects]], [[menu.layout]], [[menu.tools]], [[menu.window]] et [[menu.help]]. Chaque
  commande affiche son raccourci à côté. Sans dessin ouvert, seuls
  [[menu.file]], [[menu.tools]], [[menu.window]] et [[menu.help]] sont
  affichés. La page [Commandes des menus](../reference/menus.md) liste toutes
  les commandes.
- La **barre d'outils standard** contient les commandes courantes (voir
  plus bas).
- La **barre de propriétés** affiche les réglages de ce que vous faites.
- Les **onglets de document** affichent un onglet par dessin ouvert.
- La **boîte à outils**, à gauche, contient les outils.
- Les **règles** longent le haut et la gauche de la fenêtre de dessin.
- La **fenêtre de dessin** affiche la page sur le bureau qui l'entoure.
- Les **panneaux** sont à droite.
- La **palette de couleurs** longe le bas.
- La **barre d'état**, tout en bas, décrit l'outil actif et la sélection.

## La barre d'outils standard {#standard-toolbar}

La barre d'outils standard contient, par groupes :

1. Nouveau, Ouvrir (sa flèche liste les derniers dessins utilisés),
   Enregistrer et Imprimer.
2. Couper, Copier et Coller.
3. Annuler et Rétablir. Leurs flèches listent les étapes, pour en annuler ou
   en rétablir plusieurs à la fois en choisissant une étape.
4. Importer, Exporter et Publier au format PDF.
5. La zone de zoom : tapez un niveau de zoom et appuyez sur <kbd>Entrée</kbd>,
   ou ouvrez sa liste pour zoomer sur la sélection, sur tous les objets, sur la
   page, sur la largeur ou la hauteur de la page, et des niveaux de 10 % à
   1600 %. Puis l'aperçu plein écran.
6. Afficher les règles, la grille et les repères, enfoncés quand ils sont
   actifs.
7. Désactiver le magnétisme (<kbd>Alt+Q</kbd>) et la liste des éléments
   magnétiques.
8. Options, qui ouvre la liste des pages d'options.
9. Lancer : Gestionnaire de polices, TraceCut et écran d'accueil.

Survolez un bouton pour voir son nom, son raccourci et ce qu'il fait.

## La barre de propriétés {#property-bar}

La barre de propriétés change selon ce que vous faites. Sans sélection et avec
l'outil [[tool.pick]] actif, elle affiche le format et l'orientation de la
page, les unités du dessin et le pas de déplacement. Avec des objets
sélectionnés, elle affiche leur position, leur taille, leur échelle, leur
rotation et les boutons de miroir, puis les réglages de ce type d'objet. Chaque
outil ajoute ses propres réglages, expliqués sur la page de l'outil.

Dans les champs numériques, vous pouvez taper une somme comme `10+5`, ou une
valeur dans une autre unité comme `1 in`, convertie dans les unités du dessin.

## La boîte à outils et les menus volants {#toolbox}

La boîte à outils a un bouton par groupe d'outils. Un bouton portant une petite
flèche dans son coin contient un **menu volant**, une liste d'outils
apparentés. Pour l'ouvrir, cliquez sur la flèche, cliquez avec le bouton droit
sur le bouton, ou maintenez-le enfoncé. Le bouton affiche ensuite le dernier
outil utilisé de ce groupe. Survolez un outil pour voir son nom, son raccourci
et ce qu'il fait.

Un double-clic sur certains boutons a une action rapide : [[tool.pick]]
sélectionne tous les objets, [[tool.zoom]] zoome pour afficher tous les objets
et [[tool.rectangle]] ajoute un rectangle de la taille de la page.

Le bouton **+** au bout de la boîte à outils liste les groupes avec une case
pour afficher ou masquer chacun, et réinitialise la boîte à outils. La page
[Boîte à outils](../reference/toolbox.md) liste tous les outils.

## Panneaux {#panels}

Les panneaux contiennent des réglages que vous gardez ouverts pendant le
travail, comme [[docker.properties]], [[docker.objects]] et [[docker.hints]],
ouverts au départ. Les panneaux ouverts sont des onglets le long du côté droit ;
cliquez sur un onglet pour mettre son panneau au premier plan, ou sur l'onglet
du premier plan pour replier les panneaux. Ouvrez d'autres panneaux depuis
[[menu.window > menu.window.dockers]] ou avec leurs raccourcis ; le **+** sous
les onglets en ajoute un. Chaque panneau a des boutons de réduction et de
fermeture dans sa barre de titre. La page [Panneaux](../reference/dockers.md)
liste tous les panneaux.

## La fenêtre de dessin {#drawing-window}

La page est posée sur le bureau, la zone qui l'entoure, où vous pouvez aussi
garder des objets. Sous le dessin se trouvent :

- le **navigateur de pages** : insérer une page avant, première page, page
  précédente, le numéro de page, page suivante, dernière page et insérer une
  page après ;
- un **onglet par page** (cliquez pour aller à une page, double-cliquez pour la
  renommer) ;
- la barre de défilement horizontale. Faites glisser le séparateur pointillé
  entre les onglets de page et la barre de défilement pour partager l'espace.

Le bouton **Navigateur** dans le coin entre les barres de défilement affiche
une miniature de la page tant que vous le maintenez ; déplacez le pointeur sur
la miniature pour déplacer la vue. Un double-clic sur le bord de la page avec
l'outil [[tool.pick]] ouvre les options de format de page.

## La palette de couleurs {#color-palette}

Cliquez sur une couleur de la palette pour remplir les objets sélectionnés ;
cliquez avec le bouton droit pour colorer leur contour. La palette du document,
sous la fenêtre de dessin, réunit les couleurs utilisées par le dessin. Voir
[Palettes de couleurs](../color/palettes.md).

## La barre d'état {#status-bar}

La barre d'état affiche un conseil pour l'outil actif, ce qui est sélectionné
et sur quel calque, ainsi que le remplissage et le contour de la sélection.
Double-cliquez sur la partie remplissage ou contour pour les modifier. La page
[La barre d'état](../reference/status-bar.md) explique chaque partie.

## Disposer les barres {#arranging}

Vous pouvez déplacer la boîte à outils, la palette de couleurs, la barre
d'outils standard, la barre de propriétés et la barre d'état : faites glisser
une barre par sa poignée pointillée vers un autre bord de la fenêtre, ou loin
des bords pour la rendre flottante. Un clic droit sur une poignée verrouille
les barres ou réinitialise leurs positions. Pour afficher ou masquer une barre,
utilisez [[menu.window > menu.window.toolbars]]. Vous pouvez aussi enregistrer
toute la disposition comme espace de travail ; voir
[Espaces de travail](../customize/workspaces.md).

## Travailler avec plusieurs dessins {#several-drawings}

Chaque dessin ouvert a son onglet au-dessus des règles, avec son nom et un `*`
tant qu'il a des modifications non enregistrées. L'onglet après le dernier crée
un nouveau dessin. Chaque dessin garde ses propres pages, historique
d'annulation, sélection, zoom et unités.

[[menu.window > menu.window.cascade]], [[menu.window.tile_horizontally]] et
[[menu.window.tile_vertically]] affichent chaque dessin dans sa propre fenêtre ;
[[menu.window.combine_windows]] revient aux onglets.
[[menu.window.new_window]] ouvre une deuxième fenêtre du même dessin, par
exemple pour voir deux pages ou deux niveaux de zoom à la fois ; une
modification dans une fenêtre apparaît dans toutes.

## La taille de l'interface {#interface-size}

Si les menus et les boutons paraissent trop petits ou trop grands, choisissez
[[menu.tools > menu.tools.options > menu.tools.options_customization]] et
réglez la taille de l'interface sur la page [[options.appearance]]. La taille
de l'interface ne change jamais la taille du dessin. Voir
[Options de personnalisation](../customize/customization.md#appearance).
