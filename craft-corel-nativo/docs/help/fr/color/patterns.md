# Surfaces motif

Une surface motif répète une vignette sur l'intérieur d'un objet. La
vignette peut être faite de deux couleurs, d'un ensemble d'objets vectoriels
(un motif vectoriel en couleurs) ou d'une image bitmap. Les vignettes sont
ancrées à la page : elles ne glissent donc pas quand vous déplacez l'objet.

## Créer un motif à partir de vos objets {#create-pattern}

1. Dessinez les objets qui forment une vignette et sélectionnez-les.
2. Choisissez [[menu.object > menu.object.create > menu.object.create_pattern]].
3. Dans la boîte de dialogue [[dialog.create_pattern]], choisissez le
   [[dialog.create_pattern_type]] : [[dialog.create_pattern_vector]] garde
   les objets sous forme vectorielle, nets à toute taille ;
   [[dialog.create_pattern_bitmap]] en fait une image.
4. Cliquez sur [[dialog.ok]].

![La boîte de dialogue Créer un motif](../../img/color-patterns-create-pattern.png)

Le motif devient la surface des nouveaux objets. Pour le donner à des objets
déjà dessinés, choisissez [[docker.fill_pattern]] dans la section surface du
panneau [[docker.properties]].

## Réglages du motif {#settings}

Dans la section surface du panneau [[docker.properties]], choisissez
[[docker.fill_pattern]], puis la sorte de motif et ses réglages. Pour un
motif vectoriel, le panneau indique combien d'objets contient la vignette,
ainsi que sa largeur et sa hauteur : les modifier met à l'échelle le contenu
de la vignette avec elle.

## Les motifs dans les fichiers exportés {#export}

Les fichiers SVG gardent les motifs vectoriels sous forme vectorielle. Les
fichiers PDF et EPS les reçoivent sous forme d'images à la résolution
d'exportation, découpées à la forme de l'objet.
