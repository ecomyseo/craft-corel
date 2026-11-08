# Créer un dessin

Chaque dessin commence par un format de page, un nombre de pages, un mode de
couleur et une résolution. Vous les choisissez dans la boîte de dialogue
[[dialog.create_new_document]].

## Pour créer un dessin {#create}

1. Choisissez [[menu.file > menu.file.new]] (<kbd>Ctrl+N</kbd>), cliquez sur
   Nouveau dans la barre d'outils standard, cliquez sur le **+** après le
   dernier onglet de document, ou cliquez sur Nouveau document dans l'écran
   d'accueil.
2. Dans la boîte de dialogue [[dialog.create_new_document]], réglez les valeurs
   décrites ci-dessous.
3. Cliquez sur [[dialog.ok]] ou appuyez sur <kbd>Entrée</kbd>.

Le dessin s'ouvre dans son propre onglet. La boîte de dialogue retient vos
valeurs pour le dessin suivant.

![La boîte de dialogue Créer un nouveau document](../../img/get-started-new-document.png)

## Général {#general}

- [[dialog.name]] nomme le dessin. Il commence par le prochain `Untitled-N`.
- [[newdoc.preset]] règle en une fois le mode de couleur, le format de page,
  les unités et la résolution. Les préréglages sont [[newdoc.preset_default]]
  et [[newdoc.preset_cmyk]] (CMJN, A4, millimètres, 300 ppp),
  [[newdoc.preset_rgb]] (RVB, A4, millimètres, 300 ppp) et
  [[newdoc.preset_web]] (RVB, 1920 x 1080 pixels, 72 ppp). Quand vous changez
  l'une de ces valeurs, le préréglage affiche [[newdoc.preset_custom]], sauf si
  les valeurs correspondent de nouveau à un préréglage.
- Le bouton à côté du préréglage contient [[newdoc.save_preset]], qui demande
  un [[newdoc.preset_name]] et ajoute les valeurs actuelles à la liste, et
  [[newdoc.delete_preset]], qui supprime un préréglage que vous avez
  enregistré. Les préréglages intégrés ne peuvent pas être supprimés.
- [[newdoc.pages]] règle le nombre de pages du dessin, de 1 à 999. Chaque page
  s'appelle "Page N" et a un calque.
- [[newdoc.color_mode]] est CMJN, pour l'impression, ou RVB, pour les écrans.

## Dimensions {#dimensions}

- [[newdoc.page_size]] liste les formats de papier, ou Personnalisé. Choisir
  un format garde l'orientation actuelle. Une largeur et une hauteur tapées
  qui correspondent à un format, dans l'une ou l'autre orientation,
  sélectionnent son nom.
- [[newdoc.width]] et [[newdoc.height]] règlent la taille de la page, dans les
  unités choisies dans la liste à côté de la largeur. Ces unités deviennent les
  unités du dessin dans la barre de propriétés.
- [[newdoc.orientation]] : [[dialog.portrait]] ou [[dialog.landscape]].
- [[newdoc.resolution]] en [[newdoc.dpi]] (300 au départ). Elle sert aux effets
  convertis en pixels et à mesurer les tailles en pixels : 1920 pixels font
  162,56 mm à 300 ppp et 677,33 mm à 72 ppp. Quand les unités sont des pixels,
  changer la résolution garde la taille de la page en pixels.

## Paramètres de couleur {#color-settings}

Ouvrez [[newdoc.color_settings]] pour choisir le [[newdoc.rgb_profile]], le
[[newdoc.cmyk_profile]] et le [[newdoc.gray_profile]] du dessin, ainsi que
l'[[newdoc.intent]]. Ils démarrent sur sRVB, le profil CMJN intégré, des
niveaux de gris avec 20 % d'engraissement et colorimétrie relative. Voir
[Gestion des couleurs](../color/color-management.md).

## Ignorer la boîte de dialogue {#skip}

Cochez [[newdoc.dont_show]] pour créer de nouveaux dessins tout de suite, avec
les dernières valeurs utilisées. Pour revoir la boîte de dialogue, activez le
réglage correspondant sur la page [[options.general]] de
[[menu.tools > menu.tools.options > menu.tools.options_app]]. Voir
[Options de TraceDraw](../customize/options-app.md#general).

## Autres façons de commencer {#other-ways}

- [[menu.file > menu.file.new_from_template]] part d'un modèle. Voir
  [Modèles](../templates-styles/templates.md).
- [[menu.file > menu.file.open]] ouvre un dessin existant. Voir
  [Ouvrir et enregistrer des dessins](open-and-save.md).
