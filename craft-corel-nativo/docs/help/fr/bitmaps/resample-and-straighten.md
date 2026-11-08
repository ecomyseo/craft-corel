# Rééchantillonner, redresser et agrandir les images bitmap

## Rééchantillonner {#resample}

[[menu.bitmaps > menu.bitmaps.resample]] donne aux images bitmap
sélectionnées un nouveau nombre de pixels ; leur taille sur la page ne
change pas.

![La boîte de dialogue Rééchantillonner](../../img/bitmaps-resample-and-straighten-resample.png)

1. Tapez la nouvelle taille en **pixels**, en **pourcentage**, ou sous forme
   de la **résolution** qu'elle donne à sa taille sur la page. Avec
   **Conserver les proportions** actif, la hauteur suit la largeur.
2. Choisissez la méthode :
   - **Doux** (photos), au départ ;
   - **Net**, un peu plus précis ;
   - **Pixels durs** (pixel art) : chaque pixel reste un carré d'une seule
     couleur ;
   - **Agrandir avec détails** : doux, puis accentué quand l'image grandit.
3. Cliquez sur [[dialog.ok]].

La boîte de dialogue affiche la taille d'origine, la nouvelle taille et la
mémoire occupée par la nouvelle image. Quand plusieurs images bitmap sont
sélectionnées, chacune est mise à l'échelle du même facteur.

## Redresser l'image {#straighten}

[[menu.bitmaps > menu.bitmaps.straighten_image]] corrige les photos
inclinées, vues de biais ou déformées par l'objectif.

![La boîte de dialogue Redresser l'image](../../img/bitmaps-resample-and-straighten-straighten.png)

1. Utilisez la **Grille** (sa taille et sa couleur sont réglables) pour
   aligner l'image.
2. Réglez les curseurs : **Corriger la distorsion de l'objectif** (de -100 à
   100), **Faire pivoter l'image** (de -15 à 15 degrés), **Perspective
   verticale** et **Perspective horizontale** (de -100 à 100). Les boutons de
   rotation tournent d'abord l'image d'un quart de tour.
3. Avec **Recadrer l'image** actif, le résultat est recadré au plus grand
   rectangle aux proportions d'origine ; **Recadrer et rééchantillonner à la
   taille d'origine** lui redonne le nombre de pixels d'origine. Sans
   recadrage, les coins sont transparents.
4. Cliquez sur [[dialog.ok]]. **Réinitialiser** remet tous les réglages.

Zoomez et déplacez l'aperçu avec les outils situés au-dessus. Le résultat
remplace les pixels : les effets appliqués à l'image deviennent donc
définitifs.

## Agrandir le cadre des images bitmap {#inflate}

Certains effets, comme les flous, débordent des bords d'une image bitmap.

- [[menu.bitmaps > menu.bitmaps.inflate > menu.bitmaps.inflate_auto]] (actif
  au départ) agrandit les images bitmap de ce dont leurs effets ont besoin,
  avec une marge transparente.
- [[menu.bitmaps.inflate_manual]] agrandit à la main les images bitmap
  sélectionnées : pour la largeur et la hauteur, tapez la nouvelle taille en
  pixels ou en pourcentage (de 100 à 1000 %), avec **Conserver les
  proportions**. L'image reste centrée.

![La boîte de dialogue Gonfler manuellement l'image bitmap](../../img/bitmaps-resample-and-straighten-inflate.png)

## Autres commandes bitmap {#other}

- [[menu.bitmaps.crop_bitmap]] recadre une image bitmap avec l'outil
  [[tool.crop]] ; faire glisser un coin d'une image bitmap avec l'outil
  [[tool.shape]] la recadre aussi.
- [[menu.bitmaps.edit_bitmap]] ouvre l'image bitmap dans un autre programme.
- [[menu.bitmaps.remove_background]] rend transparent le fond d'une photo.
- [[menu.bitmaps.remove_background_local]]: [Supprimer l'arrière-plan](remove-background.md).
- [[menu.bitmaps.break_link]] et [[menu.bitmaps.update_from_link]] agissent
  sur les images bitmap liées à un fichier.
- [[menu.bitmaps.save_bitmap_as]] enregistre une image bitmap dans un
  fichier PNG.
