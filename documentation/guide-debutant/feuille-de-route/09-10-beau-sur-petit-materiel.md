# Feuille de route : Phases 9 et 10 — Beau sur petit matériel, contenu et production

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phase 8 — Lancer de rayons logiciel](08-lancer-de-rayons.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 11 et 12 — Animation et cinématiques ➡️](11-12-animation-et-cinematiques.md)

---

## Le cap : « beau sur petit matériel »

Unreal représente des milliers d'années-personnes de travail : on ne le rattrapera pas en
**largeur**. Mais un moteur ciblé peut le battre sur un terrain précis : **des visuels
éblouissants sur du matériel modeste**. Les jeux mobiles et Switch qui impressionnent reposent
sur trois leviers :
- **calculer moins de pixels** ;
- **précalculer** tout ce qui ne bouge pas ;
- **soigner le rendu « caméra »**.

**Le fil conducteur : une scène vitrine.** On définit **une** scène (par exemple : une forêt
au coucher du soleil, de l'eau, une lanterne, du brouillard). Le but : qu'elle soit
éblouissante à **60 images/s sur une Intel Iris Xe**. Chaque fonctionnalité des Phases 9 et
10 doit se justifier par cette image, et se mesurer sur elle.

---

## Phase 9 — Beau sur petit matériel

**Par où commencer** :
1. upscaling temporel et résolution dynamique ;
2. caméra cinéma ;
3. lightmapper ;
4. scalabilité automatique.

### Calculer moins de pixels

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Upscaling temporel** | Voir le détail sous le tableau | ⭐⭐⭐⭐⭐ |
| **Résolution dynamique** | La résolution de rendu baisse automatiquement quand une image dépasse son budget (mesuré sur le GPU), et remonte ensuite. L'upscaling garde l'image nette : la fluidité reste constante | ⭐⭐⭐ |
| **Cache des pipelines et précompilation** | Tous les pipelines et variantes de shaders compilés au chargement, et gardés sur disque (`PipelineCache` de wgpu) : plus de saccade la première fois qu'un effet apparaît | ⭐⭐ |

Détail de l'**upscaling temporel** :
- l'image est rendue à 50–70 % de la résolution, en décalant légèrement la caméra à chaque
  image (*jitter*) ;
- l'image pleine résolution est reconstruite à partir des images précédentes, grâce aux
  vecteurs de mouvement (étape 6.3), à la manière du TSR d'Unreal ou de FSR 2 ;
- **le** levier numéro un : il divise presque par deux le coût de tout le reste, et remplace
  le FXAA par un anticrénelage de bien meilleure qualité.

### Précalculer ce qui ne bouge pas

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Lightmapper** | Voir le détail sous le tableau | ⭐⭐⭐⭐⭐ |
| **Sondes précalculées** | Des sondes d'éclairage placées dans le niveau : les objets mobiles s'éclairent à partir des sondes voisines. Des **cube maps locales** (une pièce, un couloir, corrigées par une boîte de projection) pour des reflets justes au lieu du ciel global | ⭐⭐⭐ |
| **Éclairage hybride** | Précalculé pour le statique, dynamique (Phases 6 et 8) seulement là où le GPU le permet, avec bascule automatique selon la qualité choisie | ⭐⭐⭐ |

Détail du **lightmapper** :
- un *path tracer* **hors ligne**, construit sur le BVH de la Phase 8, calcule l'éclairage
  global des objets statiques une fois pour toutes, dans des textures (*lightmaps*) ;
- un second jeu de coordonnées de texture (UV2), dépliées automatiquement (crate `xatlas` ou
  équivalent) ;
- un débruitage, puis la compression ;
- le résultat : une qualité de film, pour un coût nul à l'exécution.

### Le rendu « caméra »

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Exposition automatique** | La luminosité moyenne de l'image (histogramme calculé sur le GPU) règle l'exposition, avec une adaptation lente : l'œil qui s'habitue en entrant dans un tunnel | ⭐⭐ |
| **Étalonnage des couleurs** | Une table de couleurs 3D (*LUT*) appliquée après le tone mapping : l'ambiance d'un film, réglée par un artiste dans un logiciel de retouche | ⭐⭐ |
| **Profondeur de champ** | Flou selon la distance au point net, avec un *bokeh* (les points lumineux deviennent des disques), à demi-résolution | ⭐⭐⭐ |
| **Flou de mouvement** | Le long des vecteurs de mouvement (étape 6.3), par tuiles pour rester économique | ⭐⭐⭐ |
| **Effets d'objectif** | Reflets d'objectif, vignettage, grain de film, légère aberration chromatique : quelques lignes chacun, réglables et désactivables | ⭐⭐ |
| **Mode photo** | Le jeu se fige, la caméra devient libre, et le joueur règle la caméra cinéma (focale, mise au point, exposition, étalonnage, effets d'objectif) avant d'enregistrer une image en haute résolution. Réutilise la Phase 12 ; la vitrine idéale du « beau sur petit matériel » | ⭐⭐ |

### La scalabilité automatique (la promesse low-end)

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Détection du matériel** | Au démarrage : type de GPU, mémoire, options disponibles. Choix d'un **préréglage** (bas, moyen, élevé, ultra) que le joueur peut changer, comme le décrit [nanites_et_lumen.md](../../nanites_et_lumen.md) | ⭐⭐ |
| **Gouverneur de budget** | Chaque effet déclare son coût mesuré (temps GPU) ; le moteur ajuste la qualité en continu (résolution, nombre de rayons, distance des ombres...) pour tenir l'objectif d'images par seconde | ⭐⭐⭐⭐ |
| **Version minimale testée en continu** | La scène vitrine est mesurée à chaque commit sur un petit GPU (benchmarks et tests d'images de la Phase 7) : une régression de performance est détectée aussitôt | ⭐⭐⭐ |

---

## Phase 10 — Contenu et production

Ce qui fait vraiment Unreal ne tient pas au seul rendu : ce sont aussi ses **outils**, qui
permettent à des artistes et des designers de créer sans programmer. Cette phase est la plus
longue ; elle se mène en parallèle des autres, piste par piste.

### Des matériaux plus riches

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Éditeur de matériaux par nœuds** | Un graphe (textures, mélanges, opérations) qui **génère du WGSL**, comme le *Material Editor* d'Unreal ; les matériaux deviennent des fichiers éditables | ⭐⭐⭐⭐ |
| **Modèles de surface supplémentaires** | Vernis (*clear coat* : voitures, objets laqués), tissu (*sheen*), métal brossé (anisotropie), peau (diffusion sous la surface, version écran), tous prévus par glTF | ⭐⭐⭐⭐ |
| **Relief sans géométrie** | *Parallax occlusion mapping* (pavés en relief sur un plan plat), textures de détail vues de près, projection triplanaire (terrain, roches sans UV) | ⭐⭐⭐ |
| **Texturing virtuel** | Les grandes textures découpées en tuiles, et seules les tuiles visibles sont en mémoire. Des mondes immenses, texturés finement, dans une mémoire GPU modeste | ⭐⭐⭐⭐⭐ |

### La vie et les mondes

L'animation, les cinématiques, les différents types de mondes et la 2D sont des chantiers à
part entière : ils ont chacun leur phase, **de 11 à 14** : voir
[Animation et cinématiques](11-12-animation-et-cinematiques.md) et
[Types de mondes, 2D et 3D](13-14-mondes-et-2d-3d.md).

### La production

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Éditeur complet** | Fenêtre de scène avec gizmos (Phase 4), explorateur de ressources avec aperçus, annuler / refaire, « jouer dans l'éditeur » sans relancer | ⭐⭐⭐⭐⭐ |
| **Script visuel par nœuds** | Comme les *Blueprints* : la logique de jeu en graphe, pour les designers, au-dessus du scripting de la Phase 7 | ⭐⭐⭐⭐⭐ |
| **Séquenceur cinématique** | Une ligne de temps pour les cinématiques et les bandes-annonces : détaillé dans la **Phase 12** | ⭐⭐⭐⭐ |
| **Préparation et packaging** | Préparer les ressources pour chaque plateforme (le *cooking* : textures compressées, meshlets, champs de distance, lightmaps), puis produire un jeu installable | ⭐⭐⭐ |
| **Rapports de plantage** | Capture de l'erreur, de la pile d'appels et des journaux, envoyés (avec accord) pour corriger les problèmes chez les joueurs | ⭐⭐ |
| **Plugins** | Étendre le moteur (nouveaux composants, outils, passes de rendu) sans modifier son code | ⭐⭐⭐⭐ |
| **Jeux d'exemple** | Trois petits projets modèles : plateformes 2D, vue à la première personne, RPG en vue de dessus. Ils servent de point de départ, de documentation vivante, et vérifient que l'API du moteur est agréable à utiliser | ⭐⭐⭐ |

---

[⬅️ Phase 8 — Lancer de rayons logiciel](08-lancer-de-rayons.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 11 et 12 — Animation et cinématiques ➡️](11-12-animation-et-cinematiques.md)
