# Feuille de route : Phases 24 à 27 — Systèmes de jeu, réseau, audio, plateformes

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phases 21 à 23 — Effets, apprentissage automatique, destruction](21-23-effets-ia-destruction.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Glossaire ➡️](../08-glossaire.md)

---

## Phase 24 — Systèmes de jeu

**Le but** : les briques que presque tous les jeux réécrivent : la caméra de jeu, la
sauvegarde de partie, l'inventaire, les quêtes, les compétences. Les fournir **génériques et
pilotées par des fichiers**, pour que chaque jeu les configure au lieu de les réécrire.

**Avant de commencer** : le contrôleur de personnage (Phase 4), les actions de la Phase 15,
le scripting et les prefabs de la Phase 7.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **24.1 — Caméras de jeu** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **24.2 — Sauvegarde de partie** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **24.3 — Inventaire et objets** | Des objets décrits dans des fichiers (nom, icône de la Phase 16, poids, empilement, effets), des conteneurs (sac, coffre, boutique), le ramassage et le lâcher dans le monde, l'équipement visible sur le personnage | ⭐⭐⭐ |
| **24.4 — Quêtes et progression** | Des quêtes en étapes et objectifs (aller à, ramasser, parler à, vaincre), des conditions et récompenses, un journal de quêtes, des **drapeaux** de l'histoire lus par les dialogues (Phase 12) et les couches de données du monde (13.7) | ⭐⭐⭐ |
| **24.5 — Compétences et effets** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **24.6 — Génération procédurale** | Donjons (salles reliées par un graphe, puis habillées de prefabs), terrains (bruit, érosion), répartition d'objets ; une **graine** reproductible, pour retrouver le même monde et rejouer un bug | ⭐⭐⭐⭐ |

Détail de l'étape 24.1 :
- un **système de caméras** : plusieurs caméras virtuelles (exploration, visée, dialogue,
  véhicule), et le moteur passe de l'une à l'autre par un fondu ;
- la **troisième personne** qui évite les murs : un lancer de sphère de la cible vers la
  caméra, qui se rapproche quand un obstacle s'interpose ;
- le **verrouillage de cible**, les zones qui imposent un angle, les tremblements (21.4) ;
- l'approche de *Cinemachine* (Unity), qui sert aussi aux cinématiques (Phase 12).

Détail de l'étape 24.2 :
- à distinguer de la **sauvegarde de scène** actuelle (`F5` / `F9`), qui écrit **tout** :
  une sauvegarde de partie n'écrit que ce qui **a changé** (le joueur, l'inventaire, les
  quêtes, l'état des objets de la Phase 23, l'heure du monde de la Phase 19) ;
- des **emplacements**, la sauvegarde automatique, une vignette et la durée de jeu ;
- un **numéro de version** dans chaque fichier, et des **migrations** : une sauvegarde
  d'une ancienne version du jeu se charge encore ;
- l'écriture dans un fichier temporaire, puis le renommage : une coupure de courant ne
  corrompt jamais la sauvegarde.

Détail de l'étape 24.5, à la manière du *Gameplay Ability System* d'Unreal :
- des **attributs** (vie, endurance, vitesse) et des **effets** qui les modifient, pour une
  durée ou en continu (poison, bonus de vitesse) ;
- des **compétences** avec coût, temps de recharge, conditions, et leur animation (Phase 11)
  et leur effet visuel (Phase 21) ;
- tout est décrit dans des fichiers, et réplicable en réseau (Phase 25).

**Références** :
- *Cinemachine* (Unity) et les caméras de *Journey*, *God of War* (GDC) ;
- le *Gameplay Ability System* d'Unreal ;
- *Procedural Content Generation in Games*, Shaker, Togelius et Nelson (gratuit en ligne).

---

## Phase 25 — Réseau et multijoueur

**Le but** : jouer à plusieurs à travers Internet, avec la sensation d'un jeu local, malgré
la latence. C'est l'un des chantiers les plus difficiles d'un moteur : il touche à tout.

**Le principe** : le **serveur fait autorité** (il décide de ce qui se passe vraiment), les
clients **prédisent** pour ne pas attendre sa réponse, et se **corrigent** quand elle arrive.

**Avant de commencer** : un pas de temps fixe et une simulation **déterministe** autant que
possible (la physique tourne déjà à 1/60 s), l'enregistrement des entrées (15.7).

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **25.1 — Transport** | Des messages sur UDP, fiables ou non selon les besoins (crate `renet` ou `quinn` pour QUIC), avec la mesure de la latence et des pertes, et un simulateur de mauvais réseau pour tester | ⭐⭐⭐ |
| **25.2 — Serveur dédié** | Le moteur sans fenêtre ni rendu (comme les tests d'images de la Phase 7), qui tourne sur un serveur Linux | ⭐⭐ |
| **25.3 — Réplication des entités** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **25.4 — Prédiction et réconciliation** | Le client applique ses entrées tout de suite, garde l'historique, puis rejoue à partir de l'état corrigé par le serveur ; les autres joueurs sont **interpolés** entre deux états reçus | ⭐⭐⭐⭐⭐ |
| **25.5 — Compensation de la latence** | Le serveur « remonte le temps » pour vérifier un tir à l'instant où le joueur l'a vu : tirer juste suffit, même avec 100 ms de latence | ⭐⭐⭐⭐ |
| **25.6 — Rollback (facultatif)** | Pour les jeux de combat : tout le monde simule à l'identique, et l'on revient en arrière quand une entrée en retard arrive (à la manière de GGPO). Demande une simulation strictement déterministe | ⭐⭐⭐⭐⭐ |
| **25.7 — Salons et connexion** | Créer et rejoindre une partie, inviter un ami, traversée des NAT (le jeu entre deux box Internet), relais quand la connexion directe échoue | ⭐⭐⭐⭐ |
| **25.8 — Anti-triche de base** | Le serveur vérifie tout ce qui compte (vitesses impossibles, tirs à travers les murs) ; le client ne reçoit que ce qu'il a le droit de voir | ⭐⭐⭐ |

Détail de l'étape 25.3 :
- les composants marqués **répliqués** sont envoyés aux clients, et seulement **ce qui a
  changé** depuis le dernier état reçu (compression delta) ;
- l'**intérêt** : chaque client ne reçoit que les entités proches de lui (les cellules de la
  Phase 13) ;
- des priorités : le joueur d'à côté est mis à jour plus souvent que celui au loin ;
- les états d'objets (Phase 23) et les compétences (24.5) sont répliqués comme des
  **changements d'état**, pas comme des positions.

**Références** :
- *Overwatch Gameplay Architecture and Netcode*, Tim Ford (GDC 2017) ;
- les articles de Glenn Fiedler (gafferongames.com) sur le réseau des jeux ;
- les articles de Gabriel Gambetta sur la prédiction côté client ;
- GGPO (rollback), les crates `renet`, `lightyear` et `ggrs` (Rust).

---

## Phase 26 — Audio interactif

**Le but** : un son qui **réagit** au jeu et à l'espace : la musique qui monte au combat, la
voix étouffée derrière une porte, l'écho d'une cathédrale.

**Où on en est** : `zentrix-audio` (kira) joue des sons positionnés, en boucle ou ponctuels,
avec les sons d'impact de la physique. La Phase 7 prévoit déjà le streaming de la musique, la
réverbération par zone et l'effet Doppler.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **26.1 — Mixage par catégories** | Des **bus** (musique, effets, voix, interface, ambiance) avec leurs volumes réglables dans les options ; les voix baissent automatiquement la musique (*ducking*) ; une limite de sons simultanés avec priorités | ⭐⭐ |
| **26.2 — Musique adaptative** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **26.3 — Son spatial binaural** | Le son en 3D au casque grâce aux **HRTF** (on entend si le son vient de devant, derrière, au-dessus), avec une solution simple pour les haut-parleurs | ⭐⭐⭐ |
| **26.4 — Occlusion et obstruction** | Des lancers de rayons de la source vers l'auditeur : un mur entre les deux assourdit le son (filtre passe-bas), une porte ouverte le laisse passer. Peu de rayons, mis à jour quelques fois par seconde | ⭐⭐⭐ |
| **26.5 — Propagation dans les pièces** | Le son contourne les obstacles par les portes et les couloirs (un graphe de pièces, comme les portails de 13.2), et chaque pièce a sa réverbération | ⭐⭐⭐⭐ |
| **26.6 — Ambiances et sons procéduraux** | Des ambiances en couches (vent, oiseaux, ville) qui suivent l'heure et la saison (Phase 19), des sons variés à partir de quelques échantillons (hauteur et volume aléatoires), des moteurs de véhicules synthétisés selon le régime | ⭐⭐⭐ |

Détail de l'étape 26.2 :
- la musique est découpée en **couches** (percussions, cordes, cuivres) qui se superposent
  selon l'**intensité** du jeu : exploration, danger, combat ;
- les changements se font **en rythme** (à la fin de la mesure), pour rester musicaux ;
- des **stingers** : de courts motifs joués par-dessus lors d'un événement (victoire,
  découverte) ;
- l'intensité est un paramètre du jeu, lu par les PNJ (Phase 20) et les quêtes (24.4).

**Références** :
- FMOD et Wwise (le modèle des bus et de la musique adaptative) ;
- *Steam Audio* (HRTF, occlusion, propagation) ;
- les GDC sur l'audio de *Hitman* et de *The Last of Us* (propagation).

---

## Phase 27 — Plateformes et distribution

**Le but** : que les jeux faits avec ZENTRIX **arrivent chez les joueurs**, sur le plus
d'appareils possible, et puissent être étendus par la communauté.

**Avant de commencer** : la version web, Linux et macOS (Phase 7), le packaging (Phase 10),
les préréglages de qualité (Phase 9), le tactile (15.5).

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **27.1 — Steam Deck et PC portables** | Une cible prioritaire pour le « beau sur petit matériel » : Linux (ou Proton), manette par défaut, écran 800p, autonomie de la batterie (cadence limitée, préréglage dédié) | ⭐⭐ |
| **27.2 — Android et iOS** | wgpu fonctionne sur Vulkan (Android) et Metal (iOS) : cycle de vie de l'application (mise en arrière-plan, perte de la surface), tactile, mémoire très limitée, chaleur du téléphone (baisser la cadence quand il chauffe) | ⭐⭐⭐⭐ |
| **27.3 — Intégration Steam** | Succès, classements, sauvegardes dans le cloud, présence (« joue à... »), Workshop pour les mods ; crate `steamworks` | ⭐⭐ |
| **27.4 — Support des mods** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **27.5 — Télémétrie avec consentement** | Avec l'accord explicite du joueur : performances réelles par GPU (pour régler les préréglages), plantages (Phase 10), endroits où les joueurs abandonnent. Données anonymes, désactivables | ⭐⭐ |
| **27.6 — Écran partagé** | Plusieurs joueurs sur un écran : plusieurs vues dans la même image, une caméra et une manette par joueur (15.2), l'interface par vue | ⭐⭐⭐ |
| **27.7 — Réalité virtuelle (facultative)** | OpenXR : rendu stéréo (deux vues), suivi de la tête et des mains, 90 images/s **obligatoires** (sinon le joueur a la nausée) : un vrai test du budget de performance | ⭐⭐⭐⭐⭐ |
| **27.8 — Consoles** | Les kits de développement et leurs outils sont sous accord de confidentialité, et wgpu ne les vise pas directement : prévoir une couche de portage dans `zentrix-rhi`, le jour où un éditeur de jeux le demande | ⭐⭐⭐⭐⭐ |

Détail de l'étape 27.4 :
- les mods sont des **dossiers** de ressources (modèles, textures, scènes, sons) qui
  remplacent ou complètent ceux du jeu, dans un ordre de chargement choisi par le joueur ;
- les mods de logique utilisent le scripting de la Phase 7, dans un **bac à sable** (WASM ou
  API limitée) : un mod ne peut pas lire les fichiers du joueur ;
- un gestionnaire de mods dans le menu, et la publication sur le Workshop (27.3).

**Références** :
- la documentation de wgpu sur Android, iOS et le web ;
- *Steamworks* (documentation de Valve) et la crate `steamworks` ;
- OpenXR (Khronos) et la crate `openxr` ;
- les systèmes de mods de *Factorio* et de *Skyrim* (ordre de chargement, bac à sable).

---

[⬅️ Phases 21 à 23 — Effets, apprentissage automatique, destruction](21-23-effets-ia-destruction.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Glossaire ➡️](../08-glossaire.md)
