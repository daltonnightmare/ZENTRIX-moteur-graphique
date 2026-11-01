# Feuille de route : Phases 21 à 23 — Effets, apprentissage automatique, destruction

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phases 18 à 20 — Masse et gravité, saisons, PNJ](18-20-physique-saisons-pnj.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 24 à 27 — Systèmes de jeu, réseau, audio, plateformes ➡️](24-27-jeu-reseau-audio-plateformes.md)

---

## Phase 21 — Effets spéciaux

**Le but** : la foudre qui illumine la vallée, une explosion qui souffle les caisses, des
étincelles quand une balle touche du métal. Des effets **spectaculaires**, mais qui tiennent
sur un petit GPU.

**Le principe** : un effet est un **assemblage de briques** décrit dans un fichier : des
émetteurs de particules, des decals, un éclair de lumière, une distorsion de l'écran, un son,
un tremblement de caméra, une impulsion physique. Le jeu le déclenche en une ligne, avec une
position et une orientation (`spawn_effect("explosion_grande", position, normale)`). Les
effets sont **recyclés** (*pooling*) au lieu d'être créés et détruits à chaque fois.

**Avant de commencer** : les particules GPU et les decals de la Phase 7. Le bloom et le HDR
actuels donnent déjà leur éclat au feu et aux étincelles.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **21.1 — Système d'effets** | Le format de fichier (une ligne de temps de briques, comme les séquences de la Phase 12), le déclenchement par événement, le recyclage, un **budget** global (nombre d'effets et de particules actifs) | ⭐⭐⭐ |
| **21.2 — Particules avancées** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **21.3 — Foudre** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **21.4 — Explosions** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **21.5 — Impacts de balles** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **21.6 — Traînées, rayons et effets d'écran** | Traînées (épées, missiles, pneus), rayons (lasers, arcs électriques), **distorsion** (air chaud, onde de choc), gouttes sur la caméra, vignette de dégâts, éblouissement | ⭐⭐⭐ |
| **21.7 — Feu, fumée et volumes** | Fumée et feu injectés dans le brouillard volumétrique (Phase 7) pour qu'ils soient éclairés et projettent de l'ombre ; sur petit matériel, des **flipbooks** précalculés à partir de simulations (Blender, EmberGen) plutôt qu'une simulation en temps réel | ⭐⭐⭐⭐ |
| **21.8 — Effets pour petit matériel et éditeur** | Voir le détail sous le tableau | ⭐⭐⭐ |

Détail de l'étape 21.2 :
- des **modules** empilables : taux d'émission, vitesse initiale, **gravité** (celle de la
  Phase 18 : les étincelles tombent aussi en gravité inversée), frottement de l'air, vent
  (Phase 19), couleur et taille au fil de la vie ;
- des **collisions** avec la profondeur de l'écran : les étincelles rebondissent sur le sol
  sans coûter de physique ;
- des textures animées (*flipbooks*), des **particules douces** (elles s'estompent près des
  surfaces au lieu de les couper net), des particules **éclairées** par le soleil et les
  lampes ;
- des particules qui émettent leur propre lumière, comptées dans les lampes locales (au plus
  256 par image aujourd'hui).

Détail de l'étape 21.3 :
- l'éclair est **généré** à chaque fois : un tracé brisé par déplacements aléatoires du milieu,
  avec des branches, dessiné en rubans très lumineux (HDR, donc le bloom le fait rayonner) ;
- un **flash** : une lumière intense pendant quelques images, qui éclaire toute la scène et
  le ciel, avec ses ombres ;
- le **tonnerre** arrive avec un retard qui dépend de la distance (le son parcourt 343 m/s) ;
- la foudre est déclenchée par l'orage de la météo (Phase 19), et peut **frapper** un objet :
  un arbre prend feu, une caisse est détruite (Phase 23).

Détail de l'étape 21.4 :
- une **boule de feu** (flipbook ou sprites en couches), puis de la **fumée** qui monte et se
  dissipe ;
- des **débris** : de vrais morceaux physiques, ou les fragments d'un objet détruit
  (Phase 23) ;
- une **onde de choc** : la distorsion de l'écran, une **impulsion radiale** qui projette les
  objets selon leur masse (18.4), des dégâts qui diminuent avec la distance ;
- un **flash** de lumière, un **tremblement de caméra** selon la distance, une **trace
  brûlée** au sol (decal) ;
- un **son** retardé selon la distance et assourdi au loin (filtre passe-bas).

Détail de l'étape 21.5 :
- le tir est un lancer de rayon (`Physics::cast_ray` existe déjà) ; le **matériau physique**
  touché (18.3) choisit l'effet : échardes pour le bois, étincelles pour le métal, poussière
  pour le béton, gerbe pour l'eau ;
- un **decal d'impact** orienté selon la normale ; les plus anciens disparaissent en fondu
  quand on dépasse la limite (une file circulaire) ;
- les **traçantes**, la **lueur de bouche** (une lampe de quelques images), le son de
  l'impact selon le matériau ;
- la **pénétration** : une balle traverse une planche mais pas un mur, selon l'épaisseur et le
  matériau.

Détail de l'étape 21.8 :
- **l'ennemi est le remplissage** (*overdraw*) : des centaines de particules transparentes
  superposées couvrent les mêmes pixels encore et encore. Les particules sont donc dessinées à
  **demi-résolution**, puis remises à l'échelle ;
- un **LOD** par distance (moins de particules au loin), un nombre maximal par préréglage de
  qualité (Phase 9), le culling des effets hors de l'écran ;
- un **éditeur d'effets** dans l'inspecteur : aperçu en boucle, réglages en direct,
  rechargement à chaud.

**Références** :
- *Niagara* (Unreal) et *VFX Graph* (Unity) pour l'organisation en modules ;
- les GDC sur les effets visuels (*The VFX of Doom*, *Visual Effects of Diablo*) ;
- *Real-Time VFX* (realtimevfx.com), la communauté de référence.

---

## Phase 22 — Apprentissage automatique pour les PNJ

**Le but** : pour **certains cas précis**, des PNJ dont le comportement est **appris** plutôt
qu'écrit à la main : un pilote de course, une créature qui marche sur ses pattes, un adversaire
qui s'adapte au joueur.

**Le principe : un outil, pas la règle.** L'IA classique de la Phase 20 reste la base. Elle est
prévisible, réglable par un designer, et ne coûte presque rien. L'apprentissage automatique
est réservé aux cas où écrire les règles à la main est trop difficile.
- L'**entraînement** a lieu **hors du jeu** : en Python, avec PyTorch, sur une machine de
  développement.
- Le jeu ne fait que l'**inférence** : il exécute un petit réseau déjà entraîné, sur le CPU,
  quelques fois par seconde.
- Le réseau prend place dans un arbre de comportement (20.2) comme un **nœud** parmi d'autres,
  avec une solution de repli classique.

**Avant de commencer** : la Phase 20 (au moins 20.1 et 20.2), le rendu sans fenêtre et le pas
de temps fixe (tests d'images, Phase 7), l'enregistrement des entrées (15.7).

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **22.1 — Observations et actions** | Un composant `Brain` : ce que le PNJ **observe** (distances par lancers de rayons, vitesses, position de la cible, en un vecteur de nombres) et ce qu'il peut **faire** (accélérer, tourner, sauter : actions continues ou discrètes) | ⭐⭐⭐ |
| **22.2 — Mode entraînement** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **22.3 — Inférence dans le moteur** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **22.4 — Apprentissage par imitation** | Apprendre à partir de parties **jouées par des humains**, enregistrées par le rejeu des entrées (15.7) : un « fantôme » qui conduit comme le meilleur joueur, des bots au style humain | ⭐⭐⭐⭐ |
| **22.5 — Les cas d'usage conseillés** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **22.6 — Animation guidée par les données** | Le *motion matching* : à chaque instant, chercher dans une grande base de captures de mouvement l'extrait qui continue le mieux la pose actuelle vers la direction voulue. Puis sa version apprise (*learned motion matching*), qui remplace la base par de petits réseaux, beaucoup moins gourmands en mémoire | ⭐⭐⭐⭐⭐ |
| **22.7 — Dialogues générés (facultatif)** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **22.8 — Garde-fous et outils** | Visualisation des observations et des actions en debug, modèles versionnés avec les ressources du jeu, **repli** automatique sur l'IA classique, et en multijoueur une décision prise **par le serveur** (les calculs flottants diffèrent d'un processeur à l'autre) | ⭐⭐⭐ |

Détail de l'étape 22.2, sur le modèle de *Unity ML-Agents* :
- le moteur tourne **sans fenêtre**, à pas de temps fixe et en **temps accéléré**, avec
  plusieurs instances en parallèle ;
- il communique avec Python par une prise réseau locale, avec une interface compatible
  **Gymnasium** : on peut alors utiliser les bibliothèques existantes (Stable-Baselines3) ;
- les **récompenses** (franchir un point de passage, ne pas tomber, toucher la cible) et la
  remise à zéro des épisodes sont écrites dans un script.

Détail de l'étape 22.3 :
- le réseau entraîné est exporté au format **ONNX** et lu par une crate Rust : `tract` (100 %
  Rust, sur le CPU, idéal pour le petit matériel), ou `ort` (ONNX Runtime) si l'on a besoin
  de plus de vitesse ;
- tous les PNJ qui partagent un modèle sont évalués **en un seul lot** ;
- la décision est prise 5 à 10 fois par seconde, pas à chaque image ;
- les poids sont **quantifiés** en entiers 8 bits : quatre fois moins de mémoire, plus rapide
  sur les petits processeurs.

Détail de l'étape 22.5, là où l'apprentissage vaut vraiment la peine :
- les **pilotes** de course ou de vol, difficiles à régler à la main ;
- les **créatures physiques** : une locomotion apprise sur un ragdoll actif (11.7), qui
  trébuche et se rattrape de façon naturelle ;
- les **adversaires de combat** qui s'adaptent au style du joueur, et l'équilibrage de la
  difficulté ;
- les **bots de test** : des agents qui explorent le niveau pendant des heures pour trouver
  les trous dans le décor, les endroits où l'on reste coincé, les raccourcis imprévus.

Détail de l'étape 22.7 : des dialogues produits par un grand modèle de langage, pour quelques
PNJ choisis.
- **les limites** : le coût, la latence (plusieurs centaines de millisecondes), la cohérence
  avec l'histoire, le contrôle du contenu ;
- **sur petit matériel** : un modèle distant (avec connexion) ou un très petit modèle local,
  jamais au détriment de l'image ;
- **les garde-fous** : le modèle reçoit les faits du jeu (quête, relations, saison) et des
  consignes de personnage, et le dialogue écrit à la main (Phase 12) reste la solution de
  repli.

**Références** :
- *Unity ML-Agents* (le modèle de l'interface d'entraînement) et Gymnasium ;
- *Learned Motion Matching*, Holden et al. (Ubisoft La Forge), SIGGRAPH 2020 ;
- *DeepMimic*, Peng et al., SIGGRAPH 2018 (locomotion physique apprise) ;
- les crates `tract`, `ort`, `burn` et `candle`.

---

## Phase 23 — États des objets et destruction

**Le but** : des objets qui **changent d'état** : une porte ouverte, fermée ou verrouillée, une
lampe allumée ou brisée, une caisse intacte, abîmée puis détruite, un pont qui s'effondre. Et
ces changements **restent** : un mur détruit le reste quand on revient.

**Le principe** :
- chaque objet concerné a une **machine à états** (comme celle des animations, 11.5) ;
- chaque état décrit son apparence (modèle, paramètres du matériau), sa physique (collider),
  ses effets (Phase 21) et ses sons ;
- les transitions sont déclenchées par des **événements** : des dégâts, une interaction, un
  script, le temps (Phase 19) ;
- la **destruction** n'est qu'un état particulier : l'objet est remplacé par ses fragments.

**Avant de commencer** : les matériaux physiques et la masse (Phase 18), les effets (Phase 21)
et les conventions Blender (Phase 17). `zentrix-physics` calcule déjà les **impacts** (la
structure `Impact`), qui peuvent servir de source de dégâts.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **23.1 — Composant d'état** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **23.2 — Points de vie et dégâts** | Un composant `Health` ; des **types de dégâts** (choc, explosion, feu, électricité) et des résistances selon le matériau physique (18.3) : le bois brûle, le métal résiste aux balles. Des dégâts tirés de la force des collisions (les `Impact` existants), et des seuils qui déclenchent les changements d'état | ⭐⭐ |
| **23.3 — Dégâts visibles progressifs** | Fissures, brûlures et salissures peintes dans le matériau à l'endroit touché (masques de dégâts, decals), **bosses** sur la carrosserie d'une voiture (déformation des sommets près de l'impact) | ⭐⭐⭐⭐ |
| **23.4 — Fracture précalculée** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **23.5 — Destruction structurelle** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **23.6 — Fracture dynamique (facultative)** | Le découpage de Voronoï calculé **au moment de l'impact**, autour du point touché : plus réaliste, mais coûteux. Réservé aux préréglages de qualité élevés, la fracture précalculée restant le cas normal | ⭐⭐⭐⭐⭐ |
| **23.7 — Persistance et réseau** | L'état de chaque objet est **sauvegardé** par cellule (13.7). En multijoueur, on transmet les **changements d'état** (« la caisse 42 est détruite »), pas la position de chaque débris : les débris sans effet sur le jeu restent locaux | ⭐⭐⭐ |
| **23.8 — Destruction pour petit matériel et outils** | Voir le détail sous le tableau | ⭐⭐⭐ |

Détail de l'étape 23.1 :
- un composant `ObjectState` : la liste des états, l'état courant, les transitions et leurs
  conditions ;
- **par état** : le modèle affiché, les paramètres du matériau (une lampe éteinte n'émet plus
  de lumière), le collider (une porte ouverte ne bloque plus), les effets et sons
  d'**entrée** dans l'état ;
- les variantes sont modélisées dans **Blender** et reconnues par leur suffixe (`_abime`,
  `_casse`), selon les conventions de la Phase 17 ;
- les scripts (Phase 7), les PNJ (Phase 20) et les séquences (Phase 12) peuvent lire et changer
  l'état : un PNJ ferme la porte, une cinématique fait s'effondrer le pont.

Détail de l'étape 23.4 : la méthode des jeux qui détruisent beaucoup sur peu de matériel.
- l'objet est **découpé à l'avance** en fragments, par cellules de Voronoï : dans Blender
  (l'add-on *Cell Fracture*) ou à l'import ;
- à la rupture, l'objet intact est remplacé par ses fragments, qui deviennent des corps
  rigides avec l'**impulsion du coup** (direction et force de l'impact) ;
- une fracture **hiérarchique** : de gros morceaux d'abord, qui peuvent se briser à leur tour ;
- les débris **s'endorment** vite (la physique ne les calcule plus), puis disparaissent en
  fondu, ou restent comme décor statique.

Détail de l'étape 23.5 :
- les éléments d'un bâtiment (piliers, murs, planchers) forment un **graphe de soutien** ;
- quand un élément casse, un parcours du graphe vérifie ce qui touche encore le sol ;
- tout ce qui n'est plus soutenu **tombe** : un pont s'effondre quand on détruit sa pile ;
- c'est peu coûteux, parce qu'on ne calcule qu'au moment d'une rupture.

Détail de l'étape 23.8 :
- un **nombre maximal de débris** par préréglage de qualité (Phase 9) ; les plus anciens
  disparaissent en premier ;
- des fragments **peu détaillés**, et les petits morceaux remplacés par des particules (21.2),
  bien moins chères que des corps rigides ;
- les **grandes destructions de cinématique** (une tour qui s'écroule) sont précalculées dans
  Blender et rejouées en textures d'animation (VAT, 11.8) : spectaculaire, pour presque rien ;
- dans l'inspecteur : forcer un état, appliquer des dégâts, afficher le graphe de soutien.

**Références** :
- *Destruction in Red Faction: Guerrilla* (GDC 2010) pour la destruction structurelle ;
- *Chaos Destruction* (Unreal) pour la fracture hiérarchique ;
- l'add-on *Cell Fracture* de Blender ;
- *Rainbow Six Siege*, présentations sur la destruction procédurale (*RealBlast*).

---

[⬅️ Phases 18 à 20 — Masse et gravité, saisons, PNJ](18-20-physique-saisons-pnj.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 24 à 27 — Systèmes de jeu, réseau, audio, plateformes ➡️](24-27-jeu-reseau-audio-plateformes.md)
