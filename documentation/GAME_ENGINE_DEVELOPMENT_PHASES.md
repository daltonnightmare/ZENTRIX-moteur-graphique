# Phases de développement d'un moteur de jeu en Rust

## Objectif

Cette feuille de route décrit une séquence pratique pour construire un moteur de jeu en Rust. Elle est rédigée pour le mainteneur ou le contributeur du moteur qui doit décider quoi construire ensuite, quoi reporter, et comment savoir quand chaque phase est terminée.

Le plan privilégie une petite tranche verticale jouable avant les fonctionnalités étendues du moteur. Un moteur de jeu devient plus facile à concevoir une fois qu'il est contraint de supporter une boucle de jeu réelle, des actifs, des entrées, du rendu, du débogage et de l'itération.

## Principes directeurs

- Construire le plus petit moteur capable de livrer un minuscule jeu avant d'étendre l'ensemble des fonctionnalités.
- Garder les systèmes du moteur modulaires, mais éviter les abstractions de plugins tant qu'il n'y a pas au moins deux cas d'utilisation réels.
- Privilégier les conceptions orientées données pour les chemins critiques tels que les transformations, le rendu, la physique et l'animation.
- Intégrer les outils et la visibilité de débogage dans le moteur, et non comme une couche de finition finale.
- Traiter la performance, le déterminisme et l'itération des actifs comme des contraintes de conception dès le départ.
- Valider chaque phase avec une démo fonctionnelle, pas seulement avec des tests unitaires.

## Phase 0 : Vision, périmètre et contraintes

### Objectif

Définir quel type de moteur vous construisez et ce que vous ne construisez pas intentionnellement.

### Décisions clés

- Type de jeu cible : 2D, 3D, hybride, à forte composante simulation, narratif, action, orienté éditeur, ou orienté code.
- Plateformes cibles : bureau en premier, web, mobile, console, ou serveur/sans tête.
- Moteur de rendu : Vulkan, Metal, DirectX, OpenGL, ou une abstraction comme `wgpu`.
- Style de moteur : orienté ECS, graphe de scène, modèle d'objet retenu, ou hybride.
- Stratégie d'outillage : éditeur autonome, interface de débogage dans le jeu, outils en ligne de commande, ou moteur sous forme de bibliothèque uniquement.
- Stratégie de script : Rust uniquement, Lua, Rhai, WASM, ou point d'extension futur.

### Livrables

- Charte du moteur sur une page.
- Structure initiale du dépôt.
- Politique de la chaîne d'outils Rust.
- Règles de codage et règles de contribution.
- Politique de dépendances initiale.

### Critères de sortie

- La première cible de démonstration est clairement définie.
- Les plateformes cibles et la stratégie de rendu sont choisies.
- Les trois premières phases peuvent être implémentées sans rouvrir les débats sur l'orientation principale.

## Phase 1 : Fondation et boucle d'exécution

### Objectif

Créer l'environnement d'exécution minimal qui peut ouvrir une fenêtre, traiter les événements, mettre à jour l'état et rendre une image.

### Travail principal

- Cycle de vie de l'application.
- Création de fenêtres et gestion des événements.
- Stratégie de pas de temps fixe et variable.
- Modèle de propriété de la boucle principale.
- Journalisation et rapport de panique.
- Chargement de la configuration.
- Modèles de base de gestion des erreurs.
- Indicateurs de fonctionnalités pour les sous-systèmes optionnels.

### Crates Rust recommandées

- `winit` pour les fenêtres et les événements d'entrée.
- `tracing` pour la journalisation structurée.
- `anyhow` ou `eyre` pour les erreurs au niveau de l'application.
- `thiserror` pour les erreurs au niveau de la bibliothèque.
- `serde` pour les formats de configuration.

### Livrables

- Une interface d'application exécutable.
- Métriques de timing d'image.
- Démarrage et arrêt gracieux.
- Fichier de configuration d'exécution minimal.
- Mode sans tête, si la simulation serveur ou les tests sont une priorité.

### Critères de sortie

- Le moteur peut exécuter une boucle d'image vide stable.
- Le moteur rapporte le temps d'image et les événements du cycle de vie.
- Les responsabilités de la fenêtre, des événements, de la mise à jour et du rendu sont clairement séparées.

## Phase 2 : Noyau de rendu

### Objectif

Rendre de la géométrie simple de manière fiable tout en établissant l'architecture pour la croissance future du moteur de rendu.

### Travail principal

- Initialisation du périphérique GPU et de la swapchain.
- Graphe de rendu ou abstraction de passe de rendu simple.
- Flux de chargement et de compilation des shaders.
- Gestion des maillages et des tampons.
- Chargement des textures.
- Modèle de caméra.
- Représentation des matériaux.
- Chemin de dessin 2D ou 3D de base.
- Rendu de débogage pour les lignes, les grilles, les limites et les étiquettes.

### Crates Rust recommandées

- `wgpu` pour les graphismes multiplateformes.
- `image` pour le chargement d'images.
- `glam` pour les mathématiques.
- `bytemuck` pour les conversions de tampons sûres.
- `naga` ou outils de shader via `wgpu` pour le traitement des shaders.

### Livrables

- Passe de rendu à écran clair.
- Rendu de triangle ou de sprite.
- Scène contrôlée par caméra.
- Rendu de texture.
- Grille de débogage ou superposition de primitives.
- Documentation du moteur de rendu expliquant la durée de vie et la propriété des ressources.

### Critères de sortie

- Le moteur rend une scène à chaque image sans erreurs de validation.
- Les ressources du moteur de rendu peuvent être créées, réutilisées et détruites de manière prévisible.
- Le moteur de rendu de débogage peut visualiser des informations spatiales de base.

## Phase 3 : ECS, scène et modèle de transformation

### Objectif

Définir comment l'état du jeu est représenté, interrogé, mis à jour, sérialisé et connecté au rendu.

### Travail principal

- Identité et durée de vie des entités.
- Stockage des composants.
- Planification des systèmes.
- Hiérarchie des transformations ou modèle de transformation plat.
- Chargement et sauvegarde des scènes.
- Concept de préfabriqué ou d'archétype.
- Génération et suppression d'entités à l'exécution.
- Identifiants stables pour les références de l'éditeur et des actifs.

### Crates Rust recommandées

- `hecs`, `shipyard`, ou `bevy_ecs` pour l'ECS.
- `serde` pour la sérialisation des scènes.
- `uuid` ou crate d'ID similaire pour les identifiants d'actifs/éditeur stables.

### Livrables

- Entités avec composants de transformation.
- Composants rendables connectés au moteur de rendu.
- Format de sérialisation des scènes.
- Planificateur de systèmes de base.
- Scène de démonstration chargée depuis le disque.

### Critères de sortie

- Une scène peut être chargée, mise à jour, rendue et sauvegardée.
- Des entités peuvent être générées et supprimées à l'exécution.
- Le rendu ne dépend pas d'objets de démonstration codés en dur.

## Phase 4 : Pipeline des actifs

### Objectif

Construire un chemin reproductible des actifs sources aux actifs moteur prêts à l'exécution.

### Travail principal

- Registre des actifs.
- Poignées d'actifs.
- Chargement, mise en cache et déchargement.
- Rechargement à chaud pour le développement.
- Étapes d'importation pour les textures, les maillages, les shaders, les polices, l'audio et les scènes.
- Suivi des dépendances des actifs.
- Identifiants d'actifs stables.
- Empaquetage des actifs au moment de la construction.

### Crates Rust recommandées

- `notify` pour la surveillance des fichiers.
- `serde` et `ron`, `toml`, ou `json5` pour les métadonnées.
- `gltf` pour l'importation d'actifs 3D.
- `rusttype`, `ab_glyph`, ou des crates similaires pour les polices.

### Livrables

- Format du manifeste des actifs.
- Gestionnaire d'actifs à l'exécution.
- Rechargement à chaud pour au moins les shaders et les textures.
- Outil en ligne de commande pour l'importation.
- Comportement de repli pour les actifs manquants.

### Critères de sortie

- Les actifs peuvent être référencés par des poignées stables plutôt que par des chemins bruts.
- Un actif modifié peut être mis à jour dans une version de débogage en cours d'exécution.
- Les actifs empaquetés peuvent être chargés sans dépendre de la structure du répertoire source.

## Phase 5 : Entrées, caméras et interaction

### Objectif

Rendre le moteur utilisable pour les entrées de jeu réelles sur différents appareils et schémas de contrôle.

### Travail principal

- Entrée clavier, souris et manette.
- Couche de mappage des actions.
- Contextes d'entrée.
- Capture du curseur et verrouillage du pointeur.
- Contrôleurs de caméra.
- Support de la saisie de texte.
- Points d'accroche pour l'enregistrement des entrées pour les tests et les relectures.

### Crates Rust recommandées

- `gilrs` pour le support des manettes de jeu.
- Données d'événements `winit` pour le clavier et la souris.
- `serde` pour les fichiers de liaison des entrées.

### Livrables

- Liaisons d'entrée configurables.
- Requêtes d'état d'entrée à l'exécution.
- Exemple de caméra libre ou de contrôleur de caméra 2D.
- Superposition de débogage des entrées.

### Critères de sortie

- Le code du jeu peut demander des actions au lieu de touches brutes.
- Les liaisons peuvent être modifiées sans recompilation.
- Le comportement des entrées est suffisamment visible pour être débogué rapidement.

## Phase 6 : Cadre de jeu et première tranche verticale

### Objectif

Utiliser le moteur pour construire une minuscule tranche de jeu jouable et exposer les exigences manquantes du moteur.

### Travail principal

- Transitions d'état du jeu.
- Flux d'interface utilisateur de base.
- Stratégie de sauvegarde/chargement, si pertinente.
- Collision ou espace réservé pour la physique.
- Espace réservé pour l'audio.
- Génération d'entités et règles de jeu.
- Pipeline de contenu minimal pour la tranche.

### Livrables

- Une démo jouable avec une condition de début, de boucle et de fin.
- Un vrai niveau ou une arène de test.
- Contrôles de débogage pour la réinitialisation, la pause, l'avance pas à pas et le rechargement.
- Liste des points douloureux du moteur découverts lors de l'implémentation du gameplay.

### Critères de sortie

- La démo est jouable à partir d'une extraction propre.
- Le moteur prend en charge le gameplay sans code de démonstration spécial dans les systèmes principaux.
- Les prochaines priorités du moteur sont basées sur les frictions observées, et non sur des spéculations.

## Phase 7 : Physique et collision

### Objectif

Ajouter des requêtes spatiales fiables, la détection de collision et une physique adaptée au type de jeu cible.

### Travail principal

- Composants de collisionneur.
- Détection de collision à large phase et à phase étroite.
- Volumes de déclenchement.
- Raycasts et casts de forme.
- Dynamique des corps rigides, si nécessaire.
- Support du mouvement des personnages.
- Couches de collision et masques.
- Visualisation de débogage.

### Crates Rust recommandées

- `rapier2d` ou `rapier3d` pour la physique.
- `parry2d` ou `parry3d` pour les requêtes de collision.

### Livrables

- Collisionneurs statiques et dynamiques.
- Événements de collision.
- API de requête spatiale.
- Vue de débogage de la physique.
- Mise à jour physique à pas fixe déterministe.

### Critères de sortie

- Le code du jeu peut interroger les collisions et y réagir sans dépendre de l'état du moteur de rendu.
- La physique s'exécute à un pas de temps fixe stable.
- Le comportement de collision peut être inspecté visuellement.

## Phase 8 : Audio

### Objectif

Prendre en charge les effets sonores, la musique, la lecture spatiale et le contrôle à l'exécution.

### Travail principal

- Initialisation du périphérique audio.
- Chargement des actifs sonores.
- Lecture unique et en boucle.
- Groupes de volume.
- Audio spatial, si nécessaire.
- Musique en streaming.
- Contrôles de débogage pour les sons actifs.

### Crates Rust recommandées

- `kira` pour la lecture audio de plus haut niveau.
- `rodio` pour des besoins de lecture plus simples.

### Livrables

- Lecture d'effets sonores.
- Lecture de musique.
- Groupes de mixage.
- Poignées d'actifs audio.
- Commandes de coupure et de volume à l'exécution.

### Critères de sortie

- La tranche verticale peut utiliser l'audio entièrement via les API du moteur.
- Les ressources audio suivent le même modèle de propriété que les autres actifs.
- La lecture peut être contrôlée et déboguée à l'exécution.

## Phase 9 : Interface utilisateur et texte

### Objectif

Fournir l'interface utilisateur nécessaire pour les menus du jeu, les outils de débogage, les superpositions et les panneaux de l'éditeur.

### Travail principal

- Mise en forme et rendu du texte.
- Interface utilisateur de débogage en mode immédiat.
- Modèle d'interface utilisateur de jeu/exécution.
- Primitives de mise en page.
- Focus des entrées.
- Thèmes et données de style.
- Identifiants de texte prêts pour la localisation, si nécessaire.

### Crates Rust recommandées

- `egui` pour les outils de débogage et les panneaux de l'éditeur.
- `cosmic-text` ou des crates de mise en forme de texte similaires pour un texte plus avancé.

### Livrables

- Superposition d'inspecteur de débogage.
- Écran de menu de base.
- Chemin de rendu de texte.
- Gestion des entrées de l'interface utilisateur à l'exécution.

### Critères de sortie

- Le moteur peut afficher l'état de débogage et l'interface utilisateur du jeu.
- Les entrées de l'interface utilisateur et les entrées du jeu peuvent coexister sans conflit.
- Le texte est rendu via un système réutilisable, et non par un dessin de débogage ponctuel.

## Phase 10 : Éditeur et outillage

### Objectif

Améliorer la vitesse d'itération en rendant les scènes, les actifs, les entités et l'état d'exécution inspectables et modifiables.

### Travail principal

- Vue hiérarchique des entités.
- Inspecteur de composants.
- Intégration du chargement/sauvegarde des scènes.
- Navigateur d'actifs.
- Gizmos de transformation.
- Frontière entre les modes de jeu et d'édition.
- Modèle d'annulation et de rétablissement.
- Composants ou métadonnées réservés à l'éditeur.

### Livrables

- Éditeur intégré au moteur ou application éditrice compagnon.
- Inspecteur pour les entités et les composants.
- Édition et sauvegarde des scènes.
- Navigateur d'actifs.
- Annulation et rétablissement de base.

### Critères de sortie

- Une scène simple peut être construite sans modifier manuellement les fichiers de scène bruts.
- Les limites entre l'état d'exécution et l'état de l'éditeur sont explicites.
- Les modifications de l'éditeur sont sérialisées dans le même format de scène utilisé par l'exécution.

## Phase 11 : Animation

### Objectif

Prendre en charge le modèle d'animation requis par le type de jeu cible du moteur.

### Travail principal

- Animation de sprite ou animation squelettique.
- Clips d'animation.
- Machines d'état d'animation ou arbres de mélange.
- Marqueurs d'événements.
- Retargeting, si nécessaire.
- Outillage de prévisualisation d'animation.

### Livrables

- Format d'actif d'animation.
- Lecteur d'animation à l'exécution.
- Démo d'entité pilotée par animation.
- Visualisation de débogage pour les clips et les états actuels.

### Critères de sortie

- Le gameplay peut déclencher et observer l'état de l'animation.
- Les données d'animation sont chargées via le pipeline d'actifs.
- Les entités animées fonctionnent dans la tranche verticale sans code sur mesure.

## Phase 12 : Performance, mémoire et parallélisme

### Objectif

Rendre la performance mesurable, prévisible et suffisamment bonne pour le jeu cible.

### Travail principal

- Profileur d'images.
- Suivi de la mémoire.
- Système de tâches ou planificateur de tâches.
- Exécution parallèle des systèmes lorsque c'est sûr.
- Regroupement (batching) du moteur de rendu.
- Streaming des actifs.
- Optimisation des requêtes ECS.
- Réduction des allocations dans les chemins critiques.

### Crates Rust recommandées

- `rayon` pour le parallélisme des données.
- `puffin` ou `tracy-client` pour le profilage.
- `slotmap`, `generational-arena`, ou des crates similaires pour les poignées stables.

### Livrables

- Intégration du profilage.
- Budgets de performance pour le CPU, le GPU, la mémoire et le chargement des actifs.
- Scènes de test de charge.
- Règles d'allocation pour les chemins critiques documentées.

### Critères de sortie

- Le moteur peut identifier où le temps d'image est passé.
- Les scènes de charge révèlent les limites de performance.
- Les systèmes critiques évitent les allocations par image évitables.

## Phase 13 : Empaquetage, support des plateformes et distribution

### Objectif

Rendre les démos et les jeux du moteur reproductibles en dehors de l'environnement de développement.

### Travail principal

- Profils de construction.
- Empaquetage des actifs.
- Comportement spécifique à la plateforme pour les fenêtres, le système de fichiers et les entrées.
- Stratégie de rapport de crash.
- Métadonnées de version.
- Automatisation des versions.
- Empaquetage pour installateur ou archive.

### Livrables

- Commande de construction de version.
- Démo empaquetée.
- Tests de validation sur les plateformes cibles.
- Validation des bundles d'actifs.
- Liste de contrôle de base pour les versions.

### Critères de sortie

- Une machine propre peut exécuter une démo empaquetée.
- Les actifs manquants et les échecs de démarrage produisent des erreurs utiles.
- Les versions de production sont reproductibles.

## Phase 14 : Stabilité, tests et maintenance à long terme

### Objectif

Transformer le moteur d'une expérience en une base de code maintenable.

### Travail principal

- Tests unitaires pour la logique pure.
- Tests d'intégration pour le chargement des actifs, le chargement des scènes et la sérialisation.
- Tests de régression visuelle (golden tests) pour les résultats d'importation des actifs.
- Tests de rendu lorsque cela est pratique.
- Règles de stabilité de l'API.
- Politique de dépréciation.
- Fuzzing pour les analyseurs syntaxiques et les importateurs.
- Documentation pour les contributeurs du moteur.

### Crates Rust recommandées

- `insta` pour les tests instantanés.
- `proptest` pour les tests basés sur les propriétés.
- `cargo-fuzz` pour le fuzzing.
- `criterion` pour les benchmarks.

### Livrables

- Suite de tests regroupés par sous-système.
- Suite de benchmarks pour les chemins critiques.
- Guide du contributeur.
- Aperçu de l'architecture du moteur.
- Processus de notes de version.

### Critères de sortie

- Les systèmes principaux ont des tests qui détectent les régressions.
- Les API du moteur ont des attentes claires en matière de propriété et de compatibilité.
- Les nouveaux contributeurs peuvent comprendre comment ajouter une fonctionnalité sans lire l'intégralité de la base de code.

## Jalons suggérés

### Jalon 1 : Moteur vide

Inclut les phases 0 et 1.

Résultat : une application fenêtrée avec une boucle d'exécution stable, une journalisation, une configuration et un timing d'image.

### Jalon 2 : Scène rendue

Inclut les phases 2 et 3.

Résultat : une scène chargée depuis le disque avec des entités, des transformations, un contrôle de caméra et des objets rendus.

### Jalon 3 : Démo pilotée par les actifs

Inclut les phases 4 et 5.

Résultat : les actifs se chargent via des poignées, les shaders ou les textures se rechargent à chaud, et les entrées sont configurables.

### Jalon 4 : Première tranche jouable

Inclut la phase 6 ainsi que des versions minimales de la physique, de l'audio et de l'interface utilisateur.

Résultat : un minuscule jeu qui prouve que le moteur peut supporter un gameplay réel.

### Jalon 5 : Outillage utilisable

Inclut les phases 9 et 10.

Résultat : interface de débogage, inspection, édition de scène et itération plus rapide.

### Jalon 6 : Renforcement pour la production

Inclut les phases 11 à 14.

Résultat : animation, profilage, empaquetage, tests et pratiques de maintenabilité.

## Première démo recommandée

Choisissez un très petit jeu qui sollicite l'ensemble du moteur sans nécessiter de contenu avancé.

Bonnes options :

- Arène 2D vue de dessus avec mouvement, ennemis, collision, audio, interface utilisateur et particules.
- Salle de test 3D à la première personne avec mouvement de caméra, maillages statiques, collision, éclairage et interaction.
- Prototype de puzzle avec chargement de scène, interface utilisateur, état de sauvegarde, animation et besoins d'éditeur.

Évitez de commencer par un monde ouvert, un jeu multijoueur, un bac à sable physique avancé ou un éditeur complet. Ce sont des tests utiles à long terme, mais ils masquent les problèmes fondamentaux du moteur sous trop de périmètre.

## Liste de contrôle de l'architecture précoce

- Les systèmes d'exécution peuvent fonctionner sans code réservé à l'éditeur.
- Les actifs sont référencés via des poignées, et non par des chemins de fichiers bruts.
- Le moteur de rendu, la physique, l'audio et l'interface utilisateur ne possèdent pas l'état du jeu.
- Les chemins critiques évitent les allocations de tas inutiles.
- Des vues de débogage existent pour les systèmes invisibles.
- La sérialisation a des identifiants stables et des plans de versionnement.
- Chaque sous-système a une démo minimale qui peut être exécutée indépendamment.
- Le moteur a une tranche verticale principale qui prouve que tous les systèmes principaux fonctionnent ensemble.

## Ce qu'il faut reporter

- Scripting visuel complet.
- Réplication multijoueur.
- Support des plateformes consoles.
- Éditeur de matériaux complexe.
- Illumination globale avancée.
- Outillage complet de terrain.
- Marché de plugins à usage général.
- Moteur physique personnalisé.
- Abstraction de moteur de rendu personnalisée.

Ces éléments peuvent devenir précieux plus tard, mais ils sont coûteux avant que la première tranche jouable ne prouve les véritables exigences du moteur.

## Modèle de revue de phase

Utilisez cette liste de contrôle à la fin de chaque phase :

- Quelle démo prouve que cette phase fonctionne ?
- Quelle API du moteur le code du jeu a-t-il utilisée ?
- Quelle vue de débogage explique le sous-système à l'exécution ?
- Quelles données peuvent être chargées depuis le disque au lieu d'être codées en dur ?
- Quel budget de performance s'applique ?
- Quels tests ou vérifications rapides protègent le comportement ?
- Qu'est-ce que cette phase a révélé à propos de la phase suivante ?

## Prochaines étapes immédiates

1. Rédiger la charte du moteur à partir de la Phase 0.
2. Choisir la première cible de démonstration.
3. Créer l'espace de travail Rust.
4. Construire la boucle d'exécution de la Phase 1.
5. Rendre le premier objet visible dès que possible.