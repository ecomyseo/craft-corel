# Nouveautés

Chaque version de TraceDraw ajoute des fonctions et des corrections. Cette
page explique comment savoir ce qui a changé et présente les points forts des
dernières versions.

## Voir les nouveautés dans le programme {#in-the-program}

- Choisissez [[menu.help > menu.help.whats_new]] pour ouvrir la page des
  nouveautés de l'écran d'accueil. Elle présente les dix dernières versions,
  de la plus récente à la plus ancienne, chacune avec sa date et un titre
  court. Cliquez sur une version pour lire un bref résumé de ce qui a
  changé ; la plus récente est déjà ouverte. **Envie d'en savoir plus ?**
  sous un résumé ouvre la page de cette version sur GitHub, avec la liste
  complète des changements.
- Activez [[menu.help > menu.help.highlight_whats_new]] pour marquer dans les
  menus ce qui est nouveau dans cette version : un petit point apparaît à la
  fin de chaque nouvelle commande, à côté de la flèche d'un sous-menu qui y
  mène, et au-dessus du titre d'un menu qui en contient une. Les marques
  disparaissent quand vous désactivez la commande, et la commande est de
  nouveau désactivée au prochain démarrage de TraceDraw.
- La liste complète des changements de chaque version se trouve dans le
  [journal des modifications](https://github.com/tracedraw/tracedraw/blob/main/CHANGELOG.md)
  du projet.

## Points forts des dernières versions {#highlights}

- **Tactile et téléphones.** Des boutons Maj, Ctrl et Alt dans la barre
  standard restent enfoncés pour le doigt ; sous Android, l'interface évite
  l'encoche de la caméra et les coins arrondis ; l'Apple Pencil fonctionne
  dans Safari sur iPad.
- **Fichiers.** Les fichiers .cdr gardent la règle de remplissage des courbes
  qui se chevauchent et s'ouvrent avec leurs taquets de tabulation, leur texte
  sur plusieurs colonnes, les repères communs à toutes les pages et les objets
  de la page maître.
- **Images, impression et espace de travail.** Recadrer les pixels de l'image
  dans l'outil Recadrage et [[menu.bitmaps > menu.bitmaps.crop_to_frame]]
  coupent les pixels d'une image ; impression en niveaux de gris et dans
  l'ordre inverse ; [[menu.view > menu.view.scrollbars]] masque les barres de
  défilement.
- **TraceCut.** [[menu.tools > menu.tools.tracecut]] ouvre les outils pour
  traceurs de découpe : sortie traceur, lignes de coupe, imbrication et
  mesure. Voir [TraceCut](../objects/tracecut.md).
- **Clavier, téléphones et stylets.** Les menus et les listes fonctionnent au
  clavier. Sur les téléphones Android, l'interface tient sur l'écran et une
  main posée dessus ne dessine plus pendant l'utilisation d'un stylet ; un
  stylet de tablette fonctionne sous Linux avec Wayland.
- **Fichiers.** Les fichiers .cdr s'ouvrent avec leurs taquets de tabulation,
  pages maîtres, dégradés et déformations ; les images des PDF arrivent en
  pleine résolution ; JPEG et TIFF s'exportent en CMJN.
- **Vignettes des objets.** Le panneau [[docker.objects]] montre chaque
  objet dessiné en petit à côté de son nom ; Ctrl+molette sur la liste ou le
  curseur en bas règle la taille. Voir
  [Organiser les objets et les calques](../objects/objects-and-layers.md#objects-docker).
- **L'Aide dans votre langue.** Chaque bouton **?** et <kbd>F1</kbd> ouvrent
  la page de cette Aide sur la boîte de dialogue, le panneau ou l'outil devant
  vous, dans la langue choisie dans TraceDraw.
- **Propriétés du document.**
  [[menu.file > menu.file.document_properties]] conserve la langue, le titre,
  le sujet, l'auteur, le copyright, les mots-clés, les notes et la note d'un
  dessin, au-dessus d'un rapport complet de ce que contient le dessin. Voir
  [Propriétés du document](../pages-layout/document-properties.md).
- **Gestion des couleurs.** [[menu.tools > menu.tools.color_management]]
  définit les profils du dessin et les valeurs par défaut des nouveaux, avec
  des règles pour les fichiers ouverts ou importés. Voir
  [Gestion des couleurs](../color/color-management.md).
- **Codes-barres et codes QR.** Un assistant en trois étapes place des
  codes-barres, et les codes QR sont des objets dynamiques que vous modifiez
  dans le panneau [[docker.properties]]. Voir
  [Codes-barres et codes QR](../objects/barcodes.md).
- **Effets.** Le menu [[menu.effects]] regroupe les panneaux d'effets ; les
  dégradés de formes peuvent suivre un tracé avec un espacement fixe et faire
  tourner leurs couleurs dans le sens horaire ou antihoraire. Voir
  [Dégradés de formes](../effects/blend.md).
- **Le texte sur un tracé** a ses propres réglages d'orientation, de
  placement, de distance et de décalage. Voir
  [Ajouter et modifier du texte](../text/adding-text.md).
- **Le Gestionnaire de polices** organise les polices installées, les
  dossiers, les favoris et les collections. Voir
  [Polices et glyphes](../text/fonts.md).
