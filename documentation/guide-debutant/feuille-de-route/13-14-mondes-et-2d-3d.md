# Feuille de route : Phases 13 et 14 — Types de mondes, 2D et 3D

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phases 11 et 12 — Animation et cinématiques](11-12-animation-et-cinematiques.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 15 à 17 — Entrées, interface et Blender ➡️](15-17-entrees-interface-blender.md)

---

## Phase 13 — Mondes fermés, semi-ouverts et ouverts

**Le but** : que le même moteur fasse aussi bien :
- un jeu en **niveaux fermés** (couloirs, donjons, une maison) ;
- un jeu **semi-ouvert** (des zones reliées entre elles, un village central avec ses
  alentours) ;
- un **monde ouvert** sans écran de chargement.

**Le principe : un seul système de chargement, trois réglages.** Le monde est fait de
**morceaux** (des scènes, au format de sauvegarde actuel). Une **politique de chargement**
décide quand charger et décharger chaque morceau :

| Type de monde | Les morceaux | Quand ils sont chargés |
|---------------|--------------|------------------------|
| **Fermé** | Un niveau entier | Au passage d'un niveau à l'autre, derrière un écran de chargement |
| **Semi-ouvert** | Des zones et sous-niveaux | Quand le joueur entre dans un **volume** de chargement, ou s'en approche |
| **Ouvert** | Les cellules d'une grille | Selon la **distance** au joueur, en continu |

Le jeu choisit sa politique, et peut les combiner : un monde ouvert dont les grottes et les
bâtiments sont des niveaux fermés.

**Avant de commencer** : le chargement asynchrone de la Phase 7 (indispensable, sinon chaque
chargement gèle l'image) et les prefabs.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **13.1 — Niveaux et transitions (fermé)** | Une ressource `Level` (une scène), le chargement et le déchargement complets, l'écran de chargement, les **points d'entrée** (la porte par laquelle on arrive), l'état du joueur conservé d'un niveau à l'autre | ⭐⭐ |
| **13.2 — Intérieurs : pièces et portails** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **13.3 — Zones et volumes de chargement (semi-ouvert)** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **13.4 — Grille de cellules (ouvert)** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **13.5 — Grandes distances** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **13.6 — Le lointain** | **HLOD** (au loin, des groupes d'objets fusionnés en un modèle simplifié, générés à la préparation du monde), **imposteurs** octaédriques (un arbre lointain devient une image vue sous le bon angle), terrain en *clipmaps* (Phase 7), ombres lointaines simplifiées | ⭐⭐⭐⭐ |
| **13.7 — Contenu et état du monde** | **Placement procédural** (forêts, rochers, herbe répartis par règles : pente, altitude, biome), **couches de données** (le même lieu avant et après un événement), **état persistant** par cellule (une caisse détruite le reste), simulation simplifiée hors de la zone chargée | ⭐⭐⭐⭐ |
| **13.8 — Outils du monde** | Carte du monde dans l'éditeur, cellules et volumes visibles en debug (comme les cascades), statistiques de chargement dans le panneau Performance, téléportation pour tester | ⭐⭐⭐ |

Détail de l'étape 13.2 : les mondes fermés ont un avantage, les murs cachent presque tout.
- le niveau est découpé en **pièces** reliées par des **portails** (portes, fenêtres) : on ne
  dessine que les pièces visibles à travers les portails depuis la caméra ;
- c'est un culling bien plus efficace que le frustum seul, et presque gratuit ;
- les mondes fermés profitent aussi le plus de l'**éclairage précalculé** (lightmaps de la
  Phase 9) : un éclairage de qualité film sur un petit GPU.

Détail de l'étape 13.3 :
- un **volume** (une boîte, comme les déclencheurs de la Phase 7) déclenche le chargement d'une
  zone, un second volume plus grand son déchargement : l'écart évite de charger et décharger
  en boucle à la frontière ;
- les **chargements masqués** : un couloir étroit, un ascenseur, une porte qui s'ouvre
  lentement laissent le temps de charger la zone suivante sans écran de chargement ;
- les **hubs** : une zone centrale qui reste chargée, et des zones autour qui vont et viennent.

Détail de l'étape 13.4, à la manière du *World Partition* d'Unreal :
- le monde est découpé en cellules carrées ; les objets sont rangés dans les cellules à la
  préparation du monde ;
- plusieurs **grilles** de tailles différentes : les gros objets (montagnes, bâtiments) sont
  chargés de loin, les petits (herbe, cailloux, objets ramassables) seulement de près ;
- un **budget par image** (mémoire, temps de création des entités) et des priorités : ce qui
  est devant le joueur passe en premier ;
- **sur petit matériel**, le rayon de chargement dépend de la mémoire disponible, réglé par la
  scalabilité de la Phase 9.

Détail de l'étape 13.5 : un `f32` n'a qu'environ 7 chiffres significatifs.
- à 10 km de l'origine, les positions ne sont plus précises qu'au millimètre près, et à
  100 km les objets tremblent ;
- **rendu relatif à la caméra** : les positions sont envoyées au GPU par rapport à la caméra,
  pas à l'origine du monde ;
- **recentrage de l'origine** : quand le joueur s'éloigne trop, tout le monde (rendu, physique
  rapier, audio) est décalé pour le ramener près de zéro ;
- la solution complète : des positions en `f64` (`glam::DVec3`) dans la scène, converties en
  `f32` relatives à la caméra à chaque image.

**Références** :
- *World Partition* d'Unreal Engine 5 (documentation officielle) ;
- *Streaming the World of Horizon Zero Dawn* (GDC 2017) ;
- les portails et secteurs des moteurs historiques (Quake, Doom 3) ;
- *Imposters* et *HLOD* dans la documentation d'Unreal.

---

## Phase 14 — 2D et 3D

**Le but** : que ZENTRIX fasse aussi des **jeux 2D** (plateformes, vue de dessus, puzzles,
pixel art), et les **mélanges** entre les deux : la 2.5D (un jeu de plateformes en décor 3D),
ou des personnages 2D dans un décor 3D, à la manière du style « HD-2D ».

**Le principe : la 2D n'est pas un autre moteur.** C'est la même scène ECS, le même
renderer, le même post-traitement (le bloom et l'étalonnage embellissent aussi la 2D), avec :
- une **caméra orthographique** ;
- des **sprites** (des images rectangulaires) dessinés en lots ;
- un **ordre de dessin par couches** au lieu de la profondeur ;
- une **physique 2D** (`rapier2d`, de la même famille que la physique actuelle).

**Pourquoi c'est intéressant pour le petit matériel** : la 2D coûte très peu, tourne partout
(y compris dans le navigateur, Phase 7) et peut être magnifique. Le style HD-2D (sprites en
pixel art, décor 3D, lumières, profondeur de champ et bloom) est sans doute la façon la plus
économique d'obtenir une image **éblouissante**.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **14.1 — Caméra orthographique** | `Camera` gagne une projection orthographique (à côté de la perspective) ; unités en pixels ou en mètres ; contrôles 2D (déplacement, zoom) ; mode **pixel parfait** (chaque pixel du sprite tombe sur un pixel entier de l'écran) | ⭐⭐ |
| **14.2 — Sprites** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **14.3 — Animation 2D** | Animations image par image (*flipbook*) depuis un atlas, import des fichiers **Aseprite** ; plus tard, squelettes 2D à la manière de Spine, qui réutilisent les mélanges de la Phase 11 | ⭐⭐⭐ |
| **14.4 — Tilemaps** | Des grilles de tuiles découpées en morceaux dessinés en un seul appel ; import de **Tiled** et **LDtk** ; collisions générées depuis les tuiles ; tuiles animées (eau, torches) | ⭐⭐⭐ |
| **14.5 — Physique 2D** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **14.6 — Éclairage 2D** | Sprites avec **normal maps**, éclairés par les `PointLight` et `SpotLight` existants ; ombres 2D projetées par des segments ou un champ de distance ; lumière ambiante par zone | ⭐⭐⭐⭐ |
| **14.7 — Mélanges 2D et 3D** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **14.8 — Texte et formes vectorielles** | Rendu de polices nettes à toutes les tailles (champ de distance, crate `fontdue` ou `ab_glyph`), partagé avec l'interface de jeu de la Phase 7 ; formes 2D (lignes, cercles, polygones) remplies et anticrénelées | ⭐⭐⭐ |

Détail de l'étape 14.2 :
- un composant `Sprite` : texture, rectangle dans l'atlas, couleur, pivot, retournement
  horizontal et vertical ;
- tous les sprites qui partagent une texture sont dessinés **en un seul appel** (instanciation,
  comme les maillages actuels), regroupés dans des **atlas** générés à l'import ;
- le tri se fait par **couche** puis par ordre (ou par la hauteur à l'écran, pour une vue de
  dessus où un personnage passe derrière un arbre) ;
- le filtrage au plus proche pour le pixel art, linéaire pour le reste ;
- la transparence réutilise le mode `Blend` et le tri de la transparence actuels.

Détail de l'étape 14.5 :
- `rapier2d` à côté de `rapier3d`, avec ses propres composants (`RigidBody2D`, `Collider2D`) ;
- le monde de la scène choisit sa dimension : un jeu 2D ne paie pas le coût de la 3D ;
- un **contrôleur de personnage 2D** (pentes, plateformes traversables par le dessous, saut
  réglable), le cœur de tout jeu de plateformes.

Détail de l'étape 14.7 :
- **sprites dans un monde 3D** (*billboards*) : toujours tournés vers la caméra, ou seulement
  autour de l'axe vertical, avec des ombres projetées (passe d'ombres masquée existante) : le
  style HD-2D ;
- **2.5D** : un décor 3D, une caméra de côté, et une physique 3D bloquée sur un plan (rapier
  sait verrouiller des axes) ;
- **plans de parallaxe** : des couches de décor qui défilent à des vitesses différentes ;
- **modèles 3D dans un jeu 2D** (un boss, des objets qui tournent) et **interface 2D sur un
  jeu 3D**, qui partagent les sprites et le texte.

**Où ranger le code** : une crate `zentrix-2d` (sprites, tilemaps, physique 2D), qui dépend de
`zentrix-render` et `zentrix-scene`, et jamais l'inverse. Le renderer gagne seulement une
passe de sprites générique.

**Références** :
- Bevy (sprites, atlas) et Godot (nœuds 2D, éclairage 2D), tous deux open source ;
- les formats Tiled, LDtk et Aseprite ;
- les présentations de Square Enix sur le style HD-2D (*Octopath Traveler*).

---

[⬅️ Phases 11 et 12 — Animation et cinématiques](11-12-animation-et-cinematiques.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 15 à 17 — Entrées, interface et Blender ➡️](15-17-entrees-interface-blender.md)
