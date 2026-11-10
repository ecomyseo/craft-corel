# Dessiner et générer avec l'IA

TraceDraw peut dessiner à partir d'une description en mots et créer des
images avec un service d'IA que vous choisissez et configurez avec votre
propre clé. TraceDraw n'a ni IA propre ni compte : rien n'est envoyé tant
que vous ne lancez pas une commande d'IA, et seul ce dont la commande a
besoin part alors vers le service configuré. Chaque requête est facturée par
votre service ; les serveurs de modèles locaux sur votre ordinateur ne
coûtent rien.

## Configurer le service d'IA {#service}

[[menu.tools > menu.tools.options > menu.tools.options_ai]] comporte deux
pages.

![La page Service des options d'IA](../../img/customize-ai-service.png)

Dans la page **Service**, configurez un service d'IA pour toutes les
fonctions d'IA :

1. Choisissez un **Préréglage** : Anthropic, OpenAI, Google Gemini, ou un
   serveur de modèles sur votre ordinateur (Ollama, LM Studio, llama.cpp
   server, vLLM). Un préréglage remplit la sorte de service et l'adresse ;
   **Personnalisé** accepte toute adresse qui parle le même langage que les
   chat completions d'OpenAI.
2. Tapez la **Clé d'API** de votre compte auprès de ce service (les serveurs
   locaux n'en ont pas besoin). Cochez **Retenir la clé sur cet ordinateur**
   pour la conserver ; elle est alors stockée en clair dans vos préférences.
3. Choisissez le **Modèle**, et le **Modèle d'image** pour les services qui
   savent créer des images (OpenAI, Google Gemini, ou une adresse
   personnalisée avec un modèle d'image).
4. Cliquez sur [[aiui.test]]. Il demande au service la liste de ses modèles
   et indique combien il en a, ou affiche son erreur.

**Afficher la barre d'invite** place la barre d'invite d'IA au-dessus de la
palette de couleurs.

## Général {#general}

![La page Général des options d'IA](../../img/customize-ai-general.png)

- [[aiui.enable]] (actif) : désactivé, toutes les commandes d'IA sont
  grisées.
- [[aiui.services]] : si [[aiui.drawings]] et [[aiui.pictures]] sont
  [[aiui.ready]] ou [[aiui.not_ready]].
- [[aiui.features]] : ce que fait chaque commande d'IA.
- [[aiui.privacy_title]] : seulement ce dont une commande a besoin, vers le
  service que vous avez configuré, au moment où vous la lancez.

## Dessiner avec des mots {#draw-with-words}

Tapez une demande comme "un vélo rouge aux roues bleues" ou "agrandis le
cercle et mets le titre en dessous", dans la barre d'invite ou dans le mode
**Dessiner avec des mots** du panneau [[docker.ai_generate]]
([[menu.object > menu.object.ai_generate > menu.object.ai_draw_words]]), et
appuyez sur **Dessiner** ou sur <kbd>Entrée</kbd>. Le service écrit un
script pour le dessin, que TraceDraw exécute en une seule étape
d'annulation. La conversation est conservée : vous pouvez donc demander des
modifications ; **Nouvelle conversation** recommence. **Ouvrir dans
Scripts** affiche le script.

## Le panneau Génération IA {#generate}

Le panneau [[docker.ai_generate]] ([[menu.tools > menu.tools.ai_generate]],
ou le bouton de génération IA de la barre d'outils standard) crée des images
avec le modèle d'image de votre service.

![Le panneau Génération IA](../../img/customize-ai-generate.png)

Choisissez un mode avec les boutons du haut :

- **Créer une image** : une image à partir d'une description. **Inspire-moi**
  propose une idée. Choisissez les proportions.
- **Remixer une image** : modifie une image selon votre description (l'image
  bitmap sélectionnée au départ). Les demandes prêtes à l'emploi comprennent
  maquette, restauration de photo, mise en couleur d'un croquis,
  remplacement du fond, changement de saison et recoloration.
- **Créer des vecteurs** : une illustration transformée en formes
  modifiables, soit une image vectorisée en courbes, soit un dessin SVG écrit
  par le modèle de texte.
- **Dessiner avec des mots** (voir ci-dessus).

Chaque mode a un **Modèle**, une image de référence facultative (**Utiliser
la sélection** ou **Importer un fichier**), l'**Invite**, et des vignettes
**Style** et **Palette de couleurs**. En bas, réglez le nombre d'**Images**
(de 1 à 4) et **Insérer dans le dessin**, puis cliquez sur **Générer**. Les
résultats restent dans le panneau ; cliquez sur l'un d'eux pour l'insérer.

## Autres commandes d'IA {#other}

- [[menu.bitmaps > menu.bitmaps.remove_background]] garde le sujet d'une
  image et rend le reste transparent.
- [[menu.bitmaps.ai_art_styles]] repeint la sélection dans l'un de huit
  styles.

Tant que l'IA est désactivée ou non configurée, les commandes sont grisées
et leur info-bulle explique pourquoi ; le bouton [[aiui.set_up]] ouvre la
bonne page d'options.
