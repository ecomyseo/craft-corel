# Masquer des couleurs dans les images bitmap

Le panneau [[docker.bitmap_mask]] masque ou conserve les pixels de couleurs
choisies dans une image bitmap, par exemple pour rendre transparent un fond
blanc. Ouvrez-le avec [[menu.bitmaps > menu.bitmaps.bitmap_mask]].

![Le panneau Masque de couleur bitmap](../../img/bitmaps-bitmap-mask.png)

## Pour masquer des couleurs {#mask}

1. Sélectionnez une image bitmap.
2. Prélevez des couleurs dans les dix emplacements de couleur :
   - cliquez sur la pipette, puis sur des couleurs de l'image ; chacune
     remplit l'emplacement sélectionné et le coche ;
   - ou cliquez sur **Sélectionner les couleurs de l'image** pour remplir les
     emplacements avec les dix couleurs les plus utilisées par l'image.
3. Sélectionnez un emplacement et réglez sa **tolérance** (de 0 à 100 %) :
   l'écart permis entre la couleur d'un pixel et celle de l'emplacement pour
   qu'ils correspondent encore.
4. Choisissez **Masquer la sélection** (les pixels correspondants deviennent
   transparents) ou **Afficher la sélection** (seuls les pixels
   correspondants restent visibles, au départ).
5. Cliquez sur [[docker.apply]].

**Enregistrer le masque** et **Ouvrir un masque** conservent les
emplacements dans un fichier `.msk` ; **Supprimer** vide l'emplacement
sélectionné. Chaque image bitmap change en une seule étape d'annulation.
