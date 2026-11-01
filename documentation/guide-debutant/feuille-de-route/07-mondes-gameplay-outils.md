# Feuille de route : Phase 7 — Mondes, gameplay, outils et plateformes

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phases 5 et 6 — Nanite et Lumen](05-06-nanite-et-lumen.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phase 8 — Lancer de rayons logiciel ➡️](08-lancer-de-rayons.md)

---

## Phase 7 — Mondes, gameplay, outils et plateformes

Les phases précédentes portent surtout sur le rendu. Celle-ci rassemble ce qui manque pour
passer d'un **moteur de démonstration** à un moteur avec lequel on **fait des jeux**. Les
pistes sont indépendantes : on les pioche selon le projet.

**Par où commencer** :
- l'**intégration continue** (petite, et elle protège tout le reste) ;
- le **render graph** (avant les Phases 5 et 6) ;
- les **particules** et le **ciel physique** (spectaculaires, ils réutilisent l'existant) ;
- la **version web** (la démo devient partageable).

### Rendu : des mondes plus grands

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Terrain** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **Végétation en masse** | Herbe instanciée sur le GPU, qui ondule au vent (animation dans le vertex shader), densité selon la distance ; forêts entières grâce au culling sur GPU (Phase 5) | ⭐⭐⭐ |
| **Eau** | Surface animée (somme de vagues, ou FFT pour l'océan), réflexion (SSR ou ciel), réfraction du fond, couleur et transparence selon la profondeur, écume sur les bords | ⭐⭐⭐⭐ |
| **Ciel physique et cycle jour/nuit** | Diffusion de la lumière dans l'atmosphère (Rayleigh et Mie, méthode de Hillaire) : ciel bleu, couchers de soleil orange. L'environnement (reflets, lumière ambiante) est recalculé quand le soleil bouge, au lieu du dégradé fixe actuel | ⭐⭐⭐⭐ |
| **Nuages volumétriques** | De vrais nuages en volume (bruit 3D, marche de rayons à basse résolution, accumulation temporelle), éclairés par le soleil et qui projettent leur ombre sur le sol ; ils suivent la météo de la Phase 19. Un gros levier visuel pour le ciel | ⭐⭐⭐⭐ |
| **Brouillard volumétrique** | Une grille 3D calée sur la caméra (*froxels*) où la lumière du soleil et des lampes se diffuse, en tenant compte des ombres : rayons de lumière visibles (*god rays*) | ⭐⭐⭐⭐ |
| **Particules GPU** | Des milliers de particules (fumée, étincelles, feu, poussière) mises à jour par un compute shader ; émetteurs comme composants de scène ; mélange additif pour le feu, avec le bloom. Les effets complets (foudre, explosions, impacts) sont détaillés dans la **Phase 21** | ⭐⭐⭐ |
| **Decals** | Images projetées sur les surfaces existantes (impacts, taches, marquages) par une boîte de projection ; utilisées par les impacts de la **Phase 21** | ⭐⭐⭐ |

Détail du **terrain** :
- une heightmap (une image de hauteurs) qui devient un maillage, découpé en dalles avec des
  niveaux de détail (*clipmaps* ou *quadtree*) ;
- un mélange de textures selon la hauteur et la pente (herbe, roche, sable) ;
- une forme de collision *heightfield* dans rapier.

### Moteur et gameplay

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Scripting** | La logique de jeu dans un langage léger (Rhai, Lua avec `mlua`, ou WASM), rechargée sans recompiler le moteur. Les scripts sont des composants qui accèdent à la scène par une API sûre | ⭐⭐⭐⭐ |
| **Prefabs** | Un objet modèle (une caisse avec son collider, son son et sa lampe) sauvegardé dans un fichier, instancié plusieurs fois ; le modifier met à jour toutes ses copies | ⭐⭐⭐ |
| **Entrées avancées** | Manettes, actions nommées, touches configurables : détaillées dans la **Phase 15** | ⭐⭐ |
| **Physique avancée** | Articulations (charnières, ressorts), ragdolls, véhicules (roues, suspension), **déclencheurs** (*triggers*) qui signalent l'entrée d'un objet dans une zone | ⭐⭐⭐ |
| **Navigation et IA** | Un **navmesh** généré depuis la géométrie de la scène, la recherche de chemin (A*), des agents qui suivent le chemin et s'évitent ; premières machines à états pour l'IA. Les PNJ complets sont détaillés dans la **Phase 20** | ⭐⭐⭐⭐ |
| **Audio avancé** | Musique en streaming (lue depuis le disque), réverbération selon les zones (pièce, grotte, extérieur), effet Doppler, sons d'impact selon la matière | ⭐⭐⭐ |
| **Réseau** | Multijoueur client-serveur : synchronisation des entités, prédiction côté client, compensation de la latence | ⭐⭐⭐⭐⭐ |

### Outils et pipeline

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Pipeline d'assets** | Chargement **asynchrone** en tâche de fond (pas de gel pendant le chargement), compression des textures (KTX2 / Basis Universal), **cache** des données converties (meshlets, champs de distance des Phases 5 et 6) | ⭐⭐⭐⭐ |
| **Interface de jeu** | Menus, HUD, barres de vie, textes à l'écran, dessinés par le moteur et distincts de l'inspecteur egui (qui reste un outil de debug) : détaillée dans la **Phase 16** | ⭐⭐⭐ |
| **Console de commandes** | Une console en jeu (touche `²`) : on tape `r.shadows.distance 80` ou `physics.gravity 0 -3 0` pour changer un réglage, comme les variables de console d'Unreal. Les réglages sont déclarés par les crates, et lisibles dans un fichier de configuration | ⭐⭐ |
| **Streaming des mips de textures** | Ne charger en haute résolution que les textures proches de la caméra, et les mips grossiers pour le reste : essentiel avec peu de mémoire GPU | ⭐⭐⭐⭐ |
| **Render graph** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |

Détail du **render graph** :
- les passes sont décrites avec ce qu'elles lisent et écrivent, et le moteur en déduit
  l'ordre, les textures intermédiaires et leur réutilisation ;
- il remplace l'enchaînement écrit à la main dans `renderer.rs`, qui devient lourd avec le
  post-traitement, les cascades, et bientôt SSAO et SSR.

### Plateformes et qualité

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Intégration continue** ✅ | GitHub Actions : `cargo test`, `cargo clippy` et `cargo fmt --check` à chaque push, sur Windows et Linux — réalisée : [.github/workflows/ci.yml](../../../.github/workflows/ci.yml) (formatage, puis clippy et tests sous Windows et Linux, avertissements traités comme des erreurs, cache des dépendances) | ⭐ |
| **Tests d'images** | Rendre des scènes de référence sans fenêtre (*headless*) et les comparer pixel par pixel à des images validées : une régression visuelle est détectée automatiquement | ⭐⭐⭐ |
| **Version web** | Compiler en WASM et tourner dans le navigateur via WebGPU (wgpu le permet presque directement) ; adapter le chargement des fichiers et l'audio | ⭐⭐⭐ |
| **Linux et macOS** | Vérifier la compilation et le rendu (Vulkan, Metal), puis packager une version téléchargeable pour chaque système | ⭐⭐ |
| **Benchmarks** | Des scènes de mesure fixes (champ de stress, centaines de lampes, beaucoup de physique), lancées automatiquement, pour suivre les performances d'une version à l'autre | ⭐⭐ |

---

[⬅️ Phases 5 et 6 — Nanite et Lumen](05-06-nanite-et-lumen.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phase 8 — Lancer de rayons logiciel ➡️](08-lancer-de-rayons.md)
