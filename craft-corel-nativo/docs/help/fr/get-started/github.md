# Conserver des dessins sur GitHub

TraceDraw peut ouvrir des dessins depuis un dépôt GitHub et les y
enregistrer sous forme de commits, sur le bureau comme dans le navigateur.
Chaque enregistrement devient une version dans l'historique du dépôt, que
vous pouvez rouvrir plus tard. Les commandes se trouvent dans
[[menu.file > menu.file.github]].

## Avant de commencer : le jeton {#token}

Les dépôts publics s'ouvrent sans jeton. Pour ouvrir des dépôts privés et
pour enregistrer, créez sur GitHub un jeton d'accès personnel à granularité
fine, avec un accès en lecture et en écriture au contenu du dépôt, et
collez-le dans le champ [[github.token]].

Le jeton est conservé tant que TraceDraw fonctionne. Cochez
[[github.remember_token]] pour le garder pour la fois suivante ; il est alors
stocké en clair dans vos préférences (dans le navigateur, dans le stockage
de ce navigateur). Les boîtes de dialogue retiennent aussi le dernier dépôt,
la dernière branche et le dernier chemin.

## Ouvrir un dessin depuis GitHub {#open}

1. Choisissez [[menu.file > menu.file.github > menu.file.github_open]].
2. Tapez le [[github.repository]] sous la forme `propriétaire/nom` ou son
   adresse github.com.
3. Tapez la [[github.branch]], ou laissez-la vide pour `main`.
4. Tapez le chemin du [[github.path]], par exemple `art/logo.tdraw`.
5. Cliquez sur [[dialog.ok]].

Le dessin s'ouvre dans un nouvel onglet, qui porte le nom du fichier. Il
retient sa provenance et sa version.

![La boîte de dialogue Ouvrir depuis GitHub](../../img/get-started-github-open.png)

## Enregistrer un dessin sur GitHub {#save}

Pour un dessin ouvert depuis GitHub, [[menu.file.save]] (<kbd>Ctrl+S</kbd>)
l'y enregistre de nouveau. Pour n'importe quel dessin, choisissez
[[menu.file > menu.file.github > menu.file.github_save]].

1. Vérifiez le dépôt, la branche et le chemin.
2. Écrivez un [[github.message]] qui dit ce qui a changé (il commence par
   "Update" suivi du nom du fichier).
3. Cliquez sur [[dialog.ok]].

Le dessin est enregistré sous forme de fichier `.tdraw`, avec ses polices,
dans un nouveau commit sur cette branche. [[menu.file.save_as]] enregistre
toujours un fichier sur votre ordinateur.

![La boîte de dialogue Enregistrer sur GitHub](../../img/get-started-github-save.png)

### Quand GitHub refuse un enregistrement {#conflict}

Si le fichier a changé sur GitHub depuis que vous l'avez ouvert, ou si un
fichier existe déjà à ce chemin et que votre dessin n'en provient pas,
GitHub refuse l'enregistrement et la boîte de dialogue l'indique.
Enregistrez sous un autre chemin ou une autre branche, ou ouvrez d'abord la
version la plus récente. La boîte de dialogue explique aussi d'autres
problèmes : un jeton refusé ou sans accès, trop de requêtes, un dépôt, une
branche ou un fichier qui n'existe pas, un chemin qui est un dossier, un
fichier qui n'est pas un dessin, ou un dessin de plus de 100 Mo.

## Ouvrir une version antérieure {#history}

Pour un dessin ouvert depuis GitHub, choisissez
[[menu.file > menu.file.github > menu.file.github_history]]. La boîte liste
les commits qui ont modifié le fichier, du plus récent au plus ancien, avec
leur identifiant court, leur date, leur auteur et leur message.
Sélectionnez-en un et cliquez sur Ouvrir : cette version s'ouvre comme un
nouveau dessin, nommé d'après le fichier et le commit, qui n'est pas lié à
GitHub.

![La boîte de dialogue Historique sur GitHub](../../img/get-started-github-history.png)
