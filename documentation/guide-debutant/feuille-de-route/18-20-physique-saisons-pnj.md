# Feuille de route : Phases 18 à 20 — Masse et gravité, saisons, PNJ

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phases 15 à 17 — Entrées, interface et Blender](15-17-entrees-interface-blender.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 21 à 23 — Effets, apprentissage automatique, destruction ➡️](21-23-effets-ia-destruction.md)

---

## Phase 18 — Masse et gravité

**Le but** : des objets qui ont un **poids crédible**, et une gravité que le jeu peut
**changer** : apesanteur, gravité inversée, mini-planètes.

**Où on en est** : dans `zentrix-physics`, un `Collider` a une `density`, et rapier en déduit
la masse à partir du volume ; une seule gravité globale existe (`Physics::set_gravity`). On
ne peut pas encore écrire « cette caisse pèse 20 kg », ni donner une gravité à une zone.

**Avant de commencer** : rien d'obligatoire. Les étapes 18.1 à 18.3 peuvent démarrer **tout de
suite** ; les déclencheurs et l'eau de la Phase 7 servent aux étapes 18.4 et 18.5.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **18.1 — Masse explicite** | `RigidBody.mass: Option<f32>` : la masse en kilogrammes, et sinon celle déduite de la densité. Masse modifiable en cours de jeu (un chariot qu'on remplit, un tonneau qui se vide), visible et réglable dans l'inspecteur | ⭐⭐ |
| **18.2 — Centre de masse et inertie** | Centre de masse réglable (un camion chargé en hauteur bascule, une quille lestée se redresse), tenseur d'inertie calculé ou imposé, centre de masse dessiné en debug | ⭐⭐ |
| **18.3 — Matériaux physiques** | Voir le détail sous le tableau | ⭐⭐ |
| **18.4 — Effets de jeu de la masse** | Le joueur pousse une chaise mais pas une armoire, les explosions projettent plus loin les objets légers, **flottaison** (poussée d'Archimède selon le volume immergé, avec l'eau de la Phase 7) : le bois flotte, l'acier coule | ⭐⭐⭐ |
| **18.5 — Gravité par objet et par zone** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **18.6 — Gravité ponctuelle** | L'attraction vers un centre (mini-planètes, astéroïdes) ou vers l'extérieur (station spatiale en rotation). La gravité rapier de l'objet est coupée, et la force `m · g_local` est appliquée à chaque pas | ⭐⭐⭐ |
| **18.7 — Le « haut » local** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **18.8 — Gravité animable** | La gravité changée par un script ou par une piste de séquence (Phase 12) : une cinématique où tout se met à flotter. Changements progressifs, sauvegardés avec la partie | ⭐⭐ |

Détail de l'étape 18.3 :
- une table de **matériaux physiques** : bois, acier, pierre, caoutchouc, glace, tissu...
- chacun a sa **densité réelle** (bois environ 600 kg/m³, acier environ 7850 kg/m³), sa
  friction, son rebond et son **son d'impact** : le bruit d'une collision se choisit tout
  seul ;
- un matériau peut être indiqué depuis Blender (propriété `physics_material`, Phase 17) ;
- **un garde-fou** : le solveur devient instable quand deux objets en contact ont un rapport de
  masses supérieur à environ 1:100 ; l'import et l'inspecteur le signalent.

Détail de l'étape 18.5 :
- une **échelle de gravité** par objet (`gravity_scale` de rapier) : 0 pour un ballon, 0,2
  pour une plume, -1 pour un objet qui « tombe » vers le haut ;
- des **zones de gravité** : des volumes (comme les déclencheurs de la Phase 7) avec leur
  propre gravité : une salle en apesanteur, un couloir à gravité inversée, un tremplin ;
- des **priorités** quand des zones se chevauchent, et un **fondu** à l'entrée pour éviter les
  à-coups.

Détail de l'étape 18.7 : c'est le vrai travail de cette phase.
- le moteur suppose aujourd'hui que le haut est `Vec3::Y` à plusieurs endroits ;
- avec une gravité modifiable, le haut devient `up = -g_local` pour le contrôleur de
  personnage, la caméra (qui se réoriente en douceur), l'IK des pieds (11.6), les particules,
  le tissu et les ragdolls ;
- les animations restent jouées dans le repère du personnage, qui tourne avec la gravité.

**Références** :
- la documentation de rapier (*mass properties*, `gravity_scale`, forces) ;
- *Game Physics Engine Development*, Ian Millington (masse, inertie, forces) ;
- les présentations de développeurs de jeux à mini-planètes sur la gravité sphérique.

---

## Phase 19 — Temps du monde, météo et saisons

**Le but** : un monde qui **vit au fil du temps** : le jour et la nuit, la pluie et la neige,
le printemps et l'hiver, avec des effets sur l'image **et** sur le jeu.

**Le principe** :
- une ressource `WorldTime` (heure, jour, saison, année) avance à une vitesse réglable et
  pilote le ciel et le soleil (Phase 7) ;
- un **système météo** choisit le temps qu'il fait, selon la saison et le biome ;
- **la règle d'or pour le petit matériel** : une saison est **un jeu de paramètres**, pas un
  jeu de modèles. Les mêmes modèles changent d'aspect par le shader, pour un coût presque nul.

**Avant de commencer** : le ciel physique et le cycle jour/nuit (Phase 7), l'étalonnage par
LUT (Phase 9) et les types de mondes (Phase 13).

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **19.1 — Le temps du monde** | `WorldTime` : heure, jour, saison, année, vitesse du temps, pause ; course et hauteur du soleil selon la saison et la **latitude** ; événements (« le jour se lève », « l'hiver commence ») pour les scripts et les PNJ (Phase 20) | ⭐⭐ |
| **19.2 — La météo** | Pluie, neige, brouillard, orage, vent : des probabilités par saison et par biome, des transitions progressives, les particules (Phase 7), le brouillard volumétrique, le vent qui agite la végétation et le tissu | ⭐⭐⭐ |
| **19.3 — Les surfaces saisonnières** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **19.4 — Lumière et ambiance** | Une LUT d'étalonnage par saison et par météo (Phase 9), un ciel couvert ou dégagé, des ambiances sonores qui changent (oiseaux au printemps, vent en hiver), des animaux selon la saison | ⭐⭐ |
| **19.5 — La saison change le jeu** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **19.6 — Adaptation aux types de mondes** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **19.7 — Sauvegarde et outils** | L'heure, la saison et la météo sont sauvegardées, et les cellules rechargées reprennent l'état courant. Dans l'inspecteur : un curseur pour faire défiler l'année, et forcer une météo pour tester | ⭐⭐ |

Détail de l'étape 19.3, des paramètres globaux lus par mesh.wgsl :
- **neige** au sol et sur les toits : une couverture calculée selon la normale (`n · up`) et
  l'altitude, qui s'accumule ou fond progressivement ;
- **feuillage** : couleur prise dans un dégradé selon la saison ; chute des feuilles par
  particules ou par textures d'animation (VAT, 11.8) ;
- **sol mouillé** : rugosité plus basse, couleur plus sombre, flaques dans les creux, qui
  sèchent après la pluie ;
- les artistes peuvent marquer dans Blender (Phase 17) ce qui ne reçoit pas de neige
  (intérieurs, dessous des ponts).

Détail de l'étape 19.5 :
- l'**eau gelée** : la surface de l'eau devient de la glace, avec un collider qui apparaît et
  un matériau physique glissant (18.3) ; un lac devient un chemin ;
- la **neige profonde** ralentit le joueur et garde ses traces de pas ;
- les **routines des PNJ** changent : ils s'abritent sous la pluie, portent des vêtements
  d'hiver (Phase 20) ;
- des ressources saisonnières : fruits, champignons, animaux de passage.

Détail de l'étape 19.6 :
- **monde fermé** : la saison est **fixée par niveau**, choisie par l'histoire, et ses
  variantes sont rangées dans les couches de données (13.7). C'est le seul cas où l'on peut
  **précalculer l'éclairage** par saison (lightmaps de la Phase 9) ; c'est coûteux en disque,
  donc on se limite à deux variantes ;
- **monde semi-ouvert** : la saison est **globale** et change **par chapitres** ou au retour
  dans une zone. La variante est choisie au chargement de la zone, et le changement a toujours
  lieu hors champ (chargement masqué, cinématique) ;
- **monde ouvert** : le calendrier est **continu**, avec des transitions progressives (la
  neige s'accumule sur plusieurs jours de jeu). La saison varie aussi **dans l'espace** : neige
  au-dessus d'une altitude, désert sans hiver. L'éclairage reste donc dynamique (sondes de la
  Phase 6), ou précalculé seulement pour l'indirect.

**Références** :
- *Physically Based Sky* (Hillaire, 2020) pour le ciel selon l'heure et la saison ;
- les GDC sur la météo dynamique des mondes ouverts (accumulation de neige, surfaces
  mouillées) ;
- *Real-Time Rendering* (4ᵉ édition) pour les surfaces mouillées.

---

## Phase 20 — PNJ : comportements et animations

**Le but** : des personnages non joueurs qui **perçoivent**, **décident** et **agissent** de
façon crédible, avec des animations variées, qu'ils soient trois dans un donjon ou trois cents
dans une ville.

**Le principe : quatre couches**, chacune testable séparément :
1. la **perception** : ce que le PNJ voit, entend et se rappelle ;
2. la **décision** : ce qu'il veut faire ;
3. la **navigation** : comment il s'y rend ;
4. l'**action** : l'animation qui le montre.

**Avant de commencer** : le navmesh et les premières machines à états (Phase 7), l'animation
(Phase 11), les dialogues (Phase 12) et le retargeting (Phase 17).

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **20.1 — Perception** | Un **cône de vision** vérifié par des lancers de rayons rapier, l'**ouïe** (les sons du jeu émettent des événements avec une portée), une **mémoire** (« j'ai vu le joueur ici il y a 5 secondes »), des seuils de détection progressifs | ⭐⭐⭐ |
| **20.2 — Décision** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **20.3 — Navigation avancée** | Évitement entre agents (RVO), *flow fields* pour les foules, liens de navigation (portes, échelles, sauts), navmesh qui se met à jour quand un obstacle bouge | ⭐⭐⭐⭐ |
| **20.4 — Routines et vie sociale** | Des **horaires** liés à `WorldTime` et aux saisons (Phase 19) : dormir, travailler, aller au marché, s'abriter. Des **factions** et des relations (allié, neutre, hostile, réputation), des **dialogues à embranchements** joués par la piste de dialogue de la Phase 12 | ⭐⭐⭐ |
| **20.5 — Objets intelligents** | Un banc « annonce » qu'on peut s'y asseoir, avec son point d'ancrage et son animation ; une forge, un comptoir, une porte de même. Le PNJ n'a pas besoin de connaître chaque objet à l'avance : les objets peuplent le monde de possibilités | ⭐⭐⭐ |
| **20.6 — Animation des PNJ** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **20.7 — LOD d'IA et foules** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **20.8 — Outils** | Cônes de vision et chemins dessinés en debug (comme les cascades), état de l'arbre de comportement **en direct** dans l'inspecteur, arbres éditables dans des fichiers, rejeu des entrées (15.7) pour reproduire un comportement | ⭐⭐⭐ |

Détail de l'étape 20.2, en commençant simple :
- une **machine à états** (patrouille, alerte, poursuite, fuite), suffisante pour beaucoup de
  jeux ;
- puis des **arbres de comportement** : le standard de l'industrie (séquences, sélecteurs,
  conditions, actions), décrits dans des fichiers et rechargés à chaud ;
- puis l'**IA utilitaire** : chaque option reçoit un score (faim, peur, devoir, fatigue), et la
  meilleure l'emporte ; elle donne des PNJ moins prévisibles ;
- le scripting de la Phase 7 permet d'écrire des comportements propres à un personnage.

Détail de l'étape 20.6, construite sur la Phase 11 :
- **locomotion pilotée par la navigation** : le *blend space* (11.5) reçoit la vitesse de
  l'agent. Il faut **synchroniser** l'agent et l'animation (le *root motion* commande, ou la
  vitesse est ajustée), sinon les pieds glissent ;
- départs, arrêts, virages sur place, IK des pieds dans les escaliers (11.6) ;
- **couches** : le haut du corps porte un objet ou gesticule en parlant, réactions additives
  (sursaut, coup reçu), regard qui suit le joueur ;
- **visage** : expressions et synchronisation des lèvres pendant les dialogues (11.3, 12.4) ;
- les **objets intelligents** (20.5) fournissent leurs animations avec un point d'alignement ;
- **contre l'effet clone** : plusieurs animations de repos tirées au hasard, décalages de
  départ, vitesses légèrement différentes, et le **retargeting** (17.6) pour partager une
  bibliothèque d'animations entre des corps différents ;
- **chutes et mort** : passage animation → ragdoll (11.7), qui tient compte de la masse et de
  la gravité locale (Phase 18).

Détail de l'étape 20.7 : indispensable sur petit matériel et en monde ouvert.
- **près du joueur** : perception, décision et animation complètes ;
- **à distance moyenne** : décision toutes les 4 à 8 images, animation simplifiée ;
- **au loin** : seulement la routine (le PNJ « est au marché »), dessiné en VAT et instancié
  (11.8) ;
- **dans les cellules déchargées** (Phase 13) : une simulation statistique, pour que le
  village soit cohérent quand le joueur revient ;
- les foules utilisent les *flow fields* (20.3) plutôt qu'un chemin par personnage.

**Références** :
- *Game AI Pro* (volumes 1 à 3, gratuits en ligne) : arbres de comportement, IA utilitaire,
  objets intelligents ;
- *Behavior Trees in Robotics and AI*, Colledanchise et Ögren ;
- *Reciprocal Velocity Obstacles* (van den Berg et al., 2008) ;
- les crates Rust `bonsai-bt` (arbres de comportement) et `big-brain` (IA utilitaire, Bevy).

---

[⬅️ Phases 15 à 17 — Entrées, interface et Blender](15-17-entrees-interface-blender.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 21 à 23 — Effets, apprentissage automatique, destruction ➡️](21-23-effets-ia-destruction.md)
