# Feuille de route : Phases 11 et 12 — Animation et cinématiques

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phases 9 et 10 — Beau sur petit matériel, contenu et production](09-10-beau-sur-petit-materiel.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 13 et 14 — Types de mondes, 2D et 3D ➡️](13-14-mondes-et-2d-3d.md)

---

## Phase 11 — Animation

**Le but** : des personnages et des objets qui **bougent de façon crédible** : marcher,
courir, sauter, poser les pieds sur une pente, parler, tomber. C'est aussi la base des
cinématiques (Phase 12).

**Le principe** :
- une animation est une suite de **poses clés** dans le temps, pour chaque os d'un squelette
  (position, rotation, échelle), ou pour chaque objet ;
- le moteur **interpole** entre les clés (linéaire, palier, ou courbe cubique, comme le
  prévoit glTF) ;
- il **mélange** plusieurs animations (marche et course selon la vitesse) ;
- le *skinning* déforme le maillage selon les os, dans le vertex shader.

**Avant de commencer** : la hiérarchie de transformations (étape I) et le chargeur glTF
(étape K) sont déjà là. L'animation peut donc commencer **tôt**, sans attendre les Phases 5
à 9.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **11.1 — Animations rigides glTF** | Lire les *channels* et *samplers* glTF ; animer les `Transform` des nœuds (portes, hélices, objets articulés) ; un composant `AnimationPlayer` (lecture, pause, boucle, vitesse) | ⭐⭐ |
| **11.2 — Squelettes et skinning** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **11.3 — Morph targets** | Les déformations par « formes cibles » de glTF : expressions du visage, synchronisation des lèvres avec la voix (Phase 12), muscles. Les poids sont animés comme le reste | ⭐⭐⭐ |
| **11.4 — Mélanges et couches** | Fondu entre deux animations, **couches** avec masques d'os (le haut du corps tire pendant que les jambes courent), animations **additives** (respiration, recul d'une arme) | ⭐⭐⭐ |
| **11.5 — Machines à états et blend spaces** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **11.6 — Root motion et cinématique inverse** | Le déplacement vient de l'animation (*root motion*) et passe au contrôleur de personnage. IK à deux os (jambes, bras), **pieds posés sur le terrain** par des lancers de rayons rapier, regard qui suit une cible | ⭐⭐⭐⭐ |
| **11.7 — Animation physique** | *Ragdolls* (articulations de la Phase 7) avec passage animation → ragdoll à la chute, mouvements secondaires par ressorts (cheveux, queues, sacoches), tissu simulé (capes, drapeaux) | ⭐⭐⭐⭐ |
| **11.8 — Animation pour petit matériel** | Voir le détail sous le tableau | ⭐⭐⭐ |

Détail de l'étape 11.2 :
- les attributs de sommets `JOINTS_0` et `WEIGHTS_0` (4 os par sommet) ;
- les matrices inverses de liaison (*inverse bind matrices*) ;
- chaque image, la pose finale de chaque os est calculée sur le CPU (en parallèle avec rayon
  s'il y a beaucoup de personnages), puis envoyée au GPU dans un *storage buffer* ;
- le vertex shader mélange les 4 matrices ; la passe d'ombres (shadow.wgsl) doit faire de
  même, sinon l'ombre reste figée ;
- les boîtes englobantes du culling doivent couvrir **toutes** les poses de l'animation, sinon
  un bras disparaît au bord de l'écran.

Détail de l'étape 11.5 :
- une **machine à états** décrit les états (repos, marche, saut, chute) et les transitions,
  avec leurs conditions (vitesse, au sol ou non) et leur durée de fondu ;
- un **blend space** mélange plusieurs animations selon un ou deux paramètres (vitesse,
  direction) ;
- les **événements d'animation** déclenchent du code à un instant précis : un bruit de pas
  (kira), une particule de poussière, un coup qui touche ;
- les graphes sont des fichiers éditables, avec un aperçu dans l'inspecteur.

Détail de l'étape 11.8 :
- **LOD d'animation** : les personnages lointains sont mis à jour moins souvent (une image
  sur 2, 4, 8), avec moins d'os, et plus du tout hors de l'écran ;
- **skinning calculé une seule fois** par image dans un compute shader, réutilisé par la passe
  d'ombres et la passe principale ;
- **textures d'animation (VAT)** : des animations précalculées dans des textures et jouées par
  le vertex shader, pour les **foules** et les objets secondaires (drapeaux, destructions), pour
  presque rien.

**Références** :
- la spécification glTF 2.0 (animations, skins, morph targets) ;
- *GDC : Animation Bootcamp* (machines à états, blend spaces) ;
- l'animation de Bevy (Rust), et la bibliothèque `ozz-animation` (C++) pour la performance.

---

## Phase 12 — Cinématiques et cutscenes

**Le but** : raconter. Des scènes mises en scène, avec caméras, personnages, dialogues,
musique et lumières, **jouées par le moteur en temps réel** (dans le décor du jeu, sans
coupure), ou rendues en vidéo pour les bandes-annonces.

**Trois usages, un seul outil** :
- la **cinématique** : le joueur regarde, le jeu reprend ensuite ;
- l'**événement scénarisé** : un pont qui s'effondre pendant que le joueur garde le contrôle ;
- la **bande-annonce** : la même séquence, rendue image par image en haute qualité.

**Le principe** : une **séquence** est un fichier qui contient une ligne de temps et des
**pistes**. Chaque piste anime une chose : la position d'un objet, une animation de personnage
(Phase 11), une propriété (l'intensité d'une lampe, la couleur d'un matériau), un son, un
sous-titre, un événement. Évaluer la séquence au temps *t* donne toujours le même résultat :
on peut avancer, reculer, sauter à n'importe quel instant.

**Avant de commencer** : la Phase 11 (au moins 11.1 et 11.4), l'interface de jeu et les
déclencheurs de la Phase 7. La caméra cinéma de la Phase 9 rend les plans beaucoup plus
beaux, mais n'est pas obligatoire.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **12.1 — Séquences et pistes** | Le format de fichier (sérialisé comme les scènes), les clés et leurs **courbes** (linéaire, Bézier, *easing*), un lecteur (lecture, pause, vitesse, saut à un instant). Les pistes de base : transformation, animation, propriété, son, événement | ⭐⭐⭐ |
| **12.2 — Caméras de cinéma** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **12.3 — Liaison avec le jeu** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **12.4 — Dialogues et sous-titres** | Piste de dialogues : voix (kira), sous-titres affichés par l'interface de jeu, synchronisation des lèvres par morph targets (11.3), textes traduisibles (fichiers de langue) | ⭐⭐⭐ |
| **12.5 — Éditeur de séquences** | Un panneau « ligne de temps » dans l'inspecteur egui : pistes, clés déplaçables, curseur de lecture qu'on fait glisser avec aperçu immédiat. **Enregistrer la caméra libre** : on place la vue, une touche ajoute une clé | ⭐⭐⭐⭐ |
| **12.6 — Qualité cinéma** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **12.7 — Rendu vidéo et vidéos précalculées** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |

Détail de l'étape 12.2 :
- un composant `CineCamera` réglé comme une vraie caméra : focale en millimètres, taille du
  capteur, distance de mise au point, ouverture (qui pilote la profondeur de champ de la
  Phase 9) ;
- une piste de **coupes** : quelle caméra est active à quel instant ;
- des **rails** (courbes de Catmull-Rom) le long desquels la caméra glisse, une cible à suivre
  du regard, des tremblements réglables (caméra à l'épaule, explosion).

Détail de l'étape 12.3 :
- les pistes visent des objets **par nom ou par étiquette** : la même séquence fonctionne avec
  le personnage du joueur, quel que soit son équipement ;
- deux sortes d'objets : ceux qui existent déjà dans la scène (*possessables*) et ceux que la
  séquence crée puis supprime (*spawnables*) ;
- le démarrage par un **déclencheur** (zone de la Phase 7) ou par un script ;
- le jeu cède le contrôle (entrées, IA, physique des objets animés), affiche des bandes noires
  si on le souhaite, permet de **passer** la séquence, puis rend la main **en douceur** : la
  caméra revient vers la caméra de jeu par un fondu de position.

Détail de l'étape 12.6 :
- un **préréglage « cinématique »** : pendant une séquence, le joueur ne joue pas, on peut donc
  dépenser le budget autrement : ombres plus fines, profondeur de champ, flou de mouvement, et
  sur petit matériel une cadence de 30 images/s assumée, comme au cinéma ;
- le **préchargement** : les ressources et les cellules du monde (Phase 13) utilisées par la
  séquence sont chargées avant son début, pour éviter toute saccade ;
- l'éclairage par plan : des lampes qui n'existent que pendant un plan, pour éclairer un
  visage.

Détail de l'étape 12.7 :
- un **rendu hors ligne** : la séquence est jouée avec un pas de temps fixe, sans fenêtre (tests
  d'images de la Phase 7), et chaque image est enregistrée. On peut accumuler plusieurs rendus
  par image pour un vrai flou de mouvement et un anticrénelage parfait : la qualité d'une
  bande-annonce, même sur un petit GPU, puisque le temps ne compte plus ;
- la **lecture de vidéos précalculées** dans le jeu (introduction, fin) : c'est une solution
  de repli pour le matériel très faible, mais elle demande un décodeur vidéo (AV1 ou VP9,
  par une bibliothèque externe).

**Références** :
- le *Sequencer* d'Unreal et la *Timeline* d'Unity (le modèle des pistes) ;
- *Cinematic Camera* : les réglages de focale et de capteur des vraies caméras ;
- Theatre.js, open source, pour l'ergonomie d'une ligne de temps.

---

[⬅️ Phases 9 et 10 — Beau sur petit matériel, contenu et production](09-10-beau-sur-petit-materiel.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 13 et 14 — Types de mondes, 2D et 3D ➡️](13-14-mondes-et-2d-3d.md)
