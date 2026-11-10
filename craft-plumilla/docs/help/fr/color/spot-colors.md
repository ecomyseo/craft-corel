# Couleurs d'accompagnement

Une **couleur d'accompagnement** est une encre nommée imprimée sur sa propre
plaque, comme une couleur métallisée ou une couleur de marque, au lieu
d'être obtenue en mélangeant du cyan, du magenta, du jaune et du noir.
Chaque dessin conserve la liste de ses encres d'accompagnement. Chaque encre
a un équivalent en couleurs quadrichromiques, utilisé pour l'afficher à
l'écran et partout où l'encre ne peut pas être utilisée.

## Ajouter une encre d'accompagnement {#add}

1. Ouvrez le panneau [[docker.color]] et composez la couleur que l'encre
   doit avoir.
2. Sous [[docker.spot_inks]], tapez un [[docker.ink_name]].
3. Cliquez sur [[docker.new_ink]].

Les noms sont uniques dans un dessin.

## Utiliser les encres d'accompagnement {#use}

La liste [[docker.spot_inks]] affiche chaque encre avec un échantillon, son
nom et les boutons [[docker.fill]] et [[docker.outline]], qui l'appliquent à
la sélection selon la [[docker.tint]] réglée (de 0 à 100 %, 100 % au
départ). La croix au bout d'une ligne retire l'encre de la liste ; les
objets qui l'utilisent gardent sa couleur équivalente.

## Nuanciers {#color-books}

Un fichier de palette peut être un nuancier d'encres d'accompagnement (par
exemple une palette `.ase` avec des entrées d'accompagnement). Ses entrées
d'accompagnement s'ouvrent comme des couleurs d'accompagnement, nommées
comme dans le nuancier ; en appliquer une ajoute l'encre au dessin.
TraceDraw ne fournit pas de nuanciers sous licence, mais un nuancier que
vous possédez s'ouvre comme n'importe quelle palette.

## Les couleurs d'accompagnement à l'impression et à l'exportation {#output}

- Dans les fichiers PDF, chaque encre devient une couleur de séparation
  portant son nom : l'imprimeur reçoit donc l'encre.
- Quand vous imprimez des séparations, chaque encre reçoit sa propre plaque
  après les plaques cyan, magenta, jaune et noire. Voir
  [Imprimer](../printing/print.md).
- À l'écran et dans les fichiers bitmap, SVG et `.cdr`, une couleur
  d'accompagnement s'affiche sous la forme de son équivalent, à sa teinte.
