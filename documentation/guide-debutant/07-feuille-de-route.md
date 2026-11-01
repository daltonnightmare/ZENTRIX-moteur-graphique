# Chapitre 7 — Feuille de route : la suite de ZENTRIX

**Objectif** : savoir quoi construire ensuite, dans quel ordre, et où le placer dans l'architecture.

Ce chapitre relie la base actuelle aux documents de conception : [GAME_ENGINE_DEVELOPMENT_PHASES.md](../GAME_ENGINE_DEVELOPMENT_PHASES.md) (les phases) et [crates.md](../crates.md) (le découpage en crates).

---

## Où en est-on ?

| Phase (voir GAME_ENGINE_DEVELOPMENT_PHASES.md) | État |
|------------------------------------------------|------|
| **Phase 0** — Vision, périmètre | ✅ Rust + wgpu, bureau d'abord, workspace multi-crates |
| **Phase 1** — Fondation et boucle d'exécution | ✅ fenêtre, événements, delta time, FPS, logs, arrêt propre · ⏳ fichier de configuration |
| **Phase 2** — Noyau de rendu | ✅ **terminée** : GPU + surface + pipeline · maillages, uniforms, caméra 3D, depth buffer, instanciation · entrées + caméra libre · textures · rendu de debug |
| **Phase 3** — Scène et ECS | 🟡 **en cours** : `zentrix-scene` (hecs), composants `Transform` / `MeshRenderer` / `Camera3d` / `Name`, extraction vers le rendu ✅ · hiérarchie (`Parent`, `GlobalTransform`) ✅ · sauvegarde / chargement RON ✅ |
| Phase 4+ — éclairage, modèles 3D, audio, physique... | 🟡 éclairage simple (Lambert + Blinn-Phong) ✅ puis PBR complet (Cook-Torrance, textures, environnement) ✅ · modèles 3D glTF ✅ · ombres portées ✅ · physique ✅ · audio ✅ |

---

## Les prochaines étapes, dans l'ordre conseillé

Chaque étape produit quelque chose de **visible**, ce qui permet de valider qu'elle fonctionne. C'est le principe « valider chaque phase avec une démo fonctionnelle ».

> ✅ **Étapes A à D réalisées** (voir `mesh.rs`, `camera.rs`, `depth.rs`, `renderer.rs` dans
> `zentrix-render`, et `transform.rs` dans `zentrix-core`). Les exercices 4 et 7 restent
> intéressants à refaire de votre côté : comparez ensuite votre solution au code du moteur.
>
> Choix réalisés au passage : les objets sont décrits par des `DrawCommand { mesh, transform }`
> que l'application fournit à chaque image ; le `Renderer` les trie par maillage et dessine
> chaque groupe en **un seul** appel instancié (`draw_indexed(.., 0..N)`). La matrice de chaque
> instance passe par un *instance buffer* (`InstanceRaw`), la caméra par un *uniform buffer*.

### Étape A — Maillages (`Mesh`) ⭐⭐ ✅

- Faire l'[exercice 4](06-exercices.md#exercice-4--un-carré-avec-un-index-buffer) (index buffer).
- Créer `crates/zentrix-render/src/mesh.rs` avec une struct `Mesh` (vertex buffer + index buffer + nombre d'indices).
- Le `Renderer` dessine une **liste** de maillages au lieu d'un triangle codé en dur.

**Démo** : un triangle et un carré côte à côte.

### Étape B — Uniforms et temps ⭐⭐⭐ ✅

- Faire l'[exercice 7](06-exercices.md#exercice-7--faire-tourner-le-triangle-uniform-buffer).
- Généraliser : une struct `Globals` envoyée une fois par image (temps, taille d'écran).

**Démo** : le triangle tourne.

### Étape C — Caméra 3D ⭐⭐⭐ ✅

- Ajouter `glam` à `zentrix-render` (déjà déclaré dans le catalogue du workspace).
- Créer `camera.rs` : position, cible, champ de vision → matrice `view_projection` (voir l'exemple de [moteurGraphique.md §1.7](../moteurGraphique.md)).
- Envoyer la matrice via l'uniform de l'étape B, la multiplier dans le vertex shader.
- Corriger le rapport largeur/hauteur au redimensionnement : fini le triangle étiré.

**Démo** : un cube 3D, vu en perspective.

### Étape D — Profondeur (depth buffer) ⭐⭐ ✅

- Créer une texture de profondeur (`Depth32Float`), à recréer à chaque redimensionnement.
- Activer `depth_stencil` dans le pipeline.

**Démo** : plusieurs cubes qui se cachent correctement les uns les autres.

### Étape E — Entrées et caméra libre ⭐⭐ ✅

> ✅ Réalisée : `Input` dans [input.rs](../../crates/zentrix-platform/src/input.rs) (transmis à
> `on_update`), caméra libre `FlyCamera` dans [fly_camera.rs](../../crates/zentrix-app/src/fly_camera.rs).
> Les contrôles utilisent des **touches physiques** : `KeyCode::KeyW` est « Z » en AZERTY.


- Dans `zentrix-platform`, créer une struct `Input` qui mémorise les touches enfoncées et le mouvement de la souris (voir Phase 5).
- Déplacer la caméra avec `ZQSD` + souris, en multipliant par `dt`.

**Démo** : se promener dans une scène de cubes.

### Étape F — Textures ⭐⭐⭐ ✅

> ✅ Réalisée : `Image` dans [image.rs](../../crates/zentrix-core/src/image.rs) (avec mipmaps),
> crate [zentrix-assets](../../crates/zentrix-assets/src/lib.rs) (PNG/JPEG via `image`),
> [texture.rs](../../crates/zentrix-render/src/texture.rs) côté GPU. `zentrix-render` ne dépend
> **pas** de `zentrix-assets` : les deux partagent seulement le type `Image` du noyau.


- Créer la crate **`zentrix-assets`** (voir [chapitre 3 §3.6](03-cargo-et-workspace.md#36-ajouter-une-nouvelle-crate-au-moteur)) qui charge les images avec la crate `image`.
- Côté rendu : texture GPU + *sampler* + coordonnées UV dans `Vertex` + bind group.

**Démo** : un cube texturé.

### Étape G — Rendu de debug ⭐⭐ ✅

> ✅ Réalisée : `DebugLines` dans [debug.rs](../../crates/zentrix-render/src/debug.rs)
> (`line`, `grid`, `axes`, `wire_box`), second pipeline `LineList` avec test de profondeur sans
> écriture. L'API retenue diffère légèrement de `renderer.debug_line(...)` : l'application
> remplit un `DebugLines` et le passe à `render`, comme les `DrawCommand`.

- Une passe qui dessine des lignes (`PrimitiveTopology::LineList`) : grille au sol, axes X/Y/Z.
- API : `renderer.debug_line(debut, fin, couleur)` (voir [moteurGraphique.md §2.6](../moteurGraphique.md)).

**Démo** : grille et repère visibles dans la scène. **Fin de la Phase 2.**

### Étape H — Scène et ECS (Phase 3) ⭐⭐⭐⭐ ✅

- Créer la crate **`zentrix-scene`** avec un ECS (`hecs` est le plus simple pour commencer, `bevy_ecs` le plus complet).
- Composants `Transform`, `MeshRenderer`, `Camera`.
- Le `Renderer` reçoit une liste de « commandes de dessin » extraite de la scène — il ne connaît jamais l'ECS directement (voir [moteurGraphique.md §2.1–2.3](../moteurGraphique.md)).

> ✅ Réalisée : [zentrix-scene](../../crates/zentrix-scene/src/lib.rs) avec `hecs` 0.11. La caméra est
> une entité (`Transform` + `Camera3d`) : elle regarde vers le -Z de son `Transform`.
> `Scene::extract_draws` et `Scene::camera` produisent ce que le rendu attend. Côté jeu,
> [demo.rs](../../crates/zentrix-app/src/demo.rs) définit ses propres composants (`Spin`,
> `FlyCamera`) et ses systèmes (`spin_system`, `fly_camera_system`, `bounds_system`).

### Étape I — Hiérarchie de transformations ⭐⭐⭐ ✅

> ✅ Réalisée : [hierarchy.rs](../../crates/zentrix-scene/src/hierarchy.rs). Seul `Parent` est
> nécessaire (pas de composant `Children`) : `Scene::update_transforms` calcule les
> `GlobalTransform` avec mémorisation et une profondeur maximale (protection contre les
> cycles). `Scene::despawn_recursive` supprime une branche entière.

- Composants `Parent` / `Children` et un `GlobalTransform` calculé : un objet suit son parent
  (une roue suit la voiture, une arme suit la main).
- Un système de propagation qui parcourt l'arbre de la racine vers les feuilles, avant l'extraction.

**Démo** : un « système solaire » (une lune qui tourne autour d'une planète qui tourne autour du soleil).

### Étape J — Éclairage simple ⭐⭐⭐ ✅

> ✅ Réalisée : normales dans `Vertex` (`MeshData::compute_normals`), `Lighting` dans
> [lighting.rs](../../crates/zentrix-render/src/lighting.rs), uniform `FrameUniform` (caméra +
> lumière), diffuse + spéculaire dans `mesh.wgsl`. Côté scène : composant `DirectionalLight`
> (direction = -Z de son `Transform`) et `Scene::ambient_light`. Les shaders sont désormais
> validés par `cargo test` (naga), sans GPU.

- Ajouter les **normales** dans `Vertex`, une lumière directionnelle + ambiante dans un uniform.
- Éclairage de Lambert (diffus) dans `mesh.wgsl`, puis Blinn-Phong (reflets).

**Démo** : des cubes qui ont une face éclairée et une face dans l'ombre.

### Étape K — Modèles 3D (glTF) ⭐⭐⭐⭐ ✅

- Dans `zentrix-assets` : charger des fichiers `.gltf` / `.glb` (crate `gltf`) en `MeshData` + textures.

**Démo** : un modèle 3D téléchargé (licence libre) au milieu de la scène.

> ✅ Réalisée : `load_gltf` dans [model.rs](../../crates/zentrix-assets/src/model.rs). Pour que
> `zentrix-assets` ne dépende que du noyau, `Vertex` et `MeshData` ont été déplacés dans
> `zentrix-core` (ré-exportés par `zentrix-render`). Le modèle de la démo n'est pas téléchargé
> mais **généré** par [tools/generer_arbre.py](../../tools/generer_arbre.py) (pas de question de
> licence) : 7 arbres sont placés autour des cubes.

### Étape L — Ombres portées (shadow mapping) ⭐⭐⭐⭐ ✅

> ✅ Réalisée : [shadow.rs](../../crates/zentrix-render/src/shadow.rs) (shadow map 2048²,
> sampler de comparaison), `ShadowArea::light_view_projection` dans
> [lighting.rs](../../crates/zentrix-render/src/lighting.rs) (projection orthographique),
> passe d'ombre avec biais de profondeur ([shadow.wgsl](../../crates/zentrix-render/src/shaders/shadow.wgsl)),
> PCF 3×3 dans `mesh.wgsl`. `F3` active / désactive les ombres.

- Une première passe dessine la scène **vue depuis la lumière** dans une texture de profondeur
  (la *shadow map*).
- Dans `mesh.wgsl`, chaque pixel compare sa distance à la lumière avec la shadow map : plus
  loin = dans l'ombre. Filtrage PCF pour adoucir les bords.

**Démo** : les cubes et les arbres projettent leur ombre sur le sol, qui tourne avec le soleil.

### Étape M — Sauvegarde et chargement de scènes ⭐⭐⭐ ✅

- Sérialiser les entités et leurs composants (crate `serde` + format RON ou JSON).
- Recharger une scène au démarrage : premier pas vers un éditeur.

> ✅ Réalisée : [serialization.rs](../../crates/zentrix-scene/src/serialization.rs). Les handles
> sont remplacés par des noms (`AssetNames`), les liens `Parent` sont renumérotés, et le jeu
> déclare ses propres composants avec `SceneSerializer::register`. Chargement « tout ou
> rien ». Dans la démo : `F5` / `F9`.

## Et ensuite ? Pistes pour la suite

La Phase 3 (scène) est bien avancée. Pistes possibles, à choisir selon le projet :

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Interface de debug** ✅ | `egui` dans la fenêtre : FPS, liste des entités, édition des `Transform` en direct (premier éditeur) — réalisée : crate [zentrix-ui](../../crates/zentrix-ui/src/lib.rs), [inspecteur](../../crates/zentrix-app/src/inspector.rs) (`F4`) | ⭐⭐⭐ |
| **Matériaux PBR** ✅ | métal / rugosité, normal maps, lumière plus réaliste (glTF fournit déjà ces données) — réalisé : `Material` (couleur de base, métal, rugosité) par instance, Cook-Torrance dans [mesh.wgsl](../../crates/zentrix-render/src/shaders/mesh.wgsl), facteurs glTF, vitrine de sphères, réglage dans l'inspecteur ; normal maps (tangentes dans `Vertex`, `add_normal_map`, glTF `normalTexture`), tone mapping Khronos PBR Neutral (`F6`) ; textures métal/rugosité, occlusion, émission (`MaterialTextures`) ; environnement (IBL) : ciel procédural précalculé en cube maps, reflets et lumière diffuse, ciel en arrière-plan (`F7`). Pistes suivantes : ciel HDR chargé depuis une image, BRDF LUT exacte, transparence | ⭐⭐⭐⭐ |
| **Audio** ✅ | crate `kira` ou `rodio` : sons positionnés dans la scène — réalisé : crate [zentrix-audio](../../crates/zentrix-audio/src/lib.rs) (kira 0.12 : `AudioListener`, `SoundEmitter` en boucle, `play_at` ponctuel, sons WAV/OGG ou synthétisés, mode muet sans carte son) ; chocs de la physique (`Physics::impacts`) → sons d'impact ; démo : chocs, lancers, néons qui bourdonnent, volume et `F8`. Pistes suivantes : musique en streaming, effets (réverbération), effet Doppler | ⭐⭐ |
| **Physique** ✅ | crate `rapier3d` : gravité, collisions, objets qui tombent — réalisée : crate [zentrix-physics](../../crates/zentrix-physics/src/lib.rs) (composants `RigidBody` / `Collider`, pas fixe de 1/60 s, corps dynamiques / fixes / cinématiques, lancer de rayon, impulsions, dessin des formes avec `F2`) ; démo : pyramide de caisses, balles (`B`), clic droit pour pousser. Pistes suivantes : événements de collision, articulations, contrôleur de personnage, interpolation | ⭐⭐⭐ |
| **Performance** ✅ | culling (ne pas dessiner hors de l'écran), mesures avec `tracy`, plusieurs threads — réalisée : culling par la pyramide de vision (caméra) et par la zone d'ombre (lumière), sur plusieurs threads au-delà de 1 024 objets ([culling.rs](../../crates/zentrix-render/src/culling.rs)) ; profileur d'image intégré ([profiler.rs](../../crates/zentrix-core/src/profiler.rs)) et panneau « Performance » de l'inspecteur ; zones Tracy (`--features tracy`) ; V-Sync désactivable ; champ de stress de 4 096 cubes (`N`). Pistes suivantes : mesures GPU (*timestamp queries*), culling sur GPU, niveaux de détail (LOD), physique sur plusieurs threads | ⭐⭐⭐⭐ |

## Les phases suivantes

La suite est découpée en petits fichiers, un par groupe de phases, dans le dossier
[feuille-de-route/](feuille-de-route/). Les phases se renvoient l'une à l'autre par leur
numéro (« étape 11.5 », « Phase 18 ») : ce tableau indique où trouver chacune.

| Fichier | Contenu |
|---------|---------|
| [Phase 4 — Nouvelles pistes](feuille-de-route/04-nouvelles-pistes.md) | transparence, post-traitement, ombres en cascade, lampes, gameplay, outils |
| [Phases 5 et 6 — Nanite et Lumen](feuille-de-route/05-06-nanite-et-lumen.md) | géométrie virtualisée, éclairage global |
| [Phase 7 — Mondes, gameplay, outils et plateformes](feuille-de-route/07-mondes-gameplay-outils.md) | terrain, eau, ciel, particules, scripting, réseau, render graph, CI, web |
| [Phase 8 — Lancer de rayons logiciel](feuille-de-route/08-lancer-de-rayons.md) | BVH, ombres douces, AO et reflets tracés, adaptés aux petits GPU |
| [Phases 9 et 10 — Beau sur petit matériel, contenu et production](feuille-de-route/09-10-beau-sur-petit-materiel.md) | le cap, upscaling, lightmapper, caméra cinéma, scalabilité, matériaux, éditeur |
| [Phases 11 et 12 — Animation et cinématiques](feuille-de-route/11-12-animation-et-cinematiques.md) | skinning, machines à états, IK, séquences, caméras de cinéma |
| [Phases 13 et 14 — Types de mondes, 2D et 3D](feuille-de-route/13-14-mondes-et-2d-3d.md) | mondes fermés, semi-ouverts et ouverts ; sprites, tilemaps, HD-2D |
| [Phases 15 à 17 — Entrées, interface et Blender](feuille-de-route/15-17-entrees-interface-blender.md) | actions et manettes ; fenêtres, design system, icônes ; Blender et rigs |
| [Phases 18 à 20 — Masse et gravité, saisons, PNJ](feuille-de-route/18-20-physique-saisons-pnj.md) | gravité modifiable, météo et saisons, comportements et animations des PNJ |
| [Phases 21 à 23 — Effets, apprentissage automatique, destruction](feuille-de-route/21-23-effets-ia-destruction.md) | foudre, explosions, impacts ; ML pour les PNJ ; états et destruction |
| [Phases 24 à 27 — Systèmes de jeu, réseau, audio, plateformes](feuille-de-route/24-27-jeu-reseau-audio-plateformes.md) | caméras de jeu, sauvegarde de partie, quêtes ; multijoueur ; musique adaptative, occlusion ; Steam Deck, mobile, mods, VR |

---

## Architecture cible

```text
                         zentrix-app
          ┌──────────┬──────┴─────┬─────────────┐
          ▼          ▼            ▼             ▼
  zentrix-platform  zentrix-scene  zentrix-assets  zentrix-render
     (+ Input)       (ECS)          (images,         (Mesh, Camera,
          │            │             modèles)         passes, debug)
          │            └───────────────│────────────────▶ │
          │                            │            zentrix-rhi
          │                            │                 │
          └──────────────▶ zentrix-core ◀────────────────┘
                         (+ maths glam, Image)
```

> `zentrix-scene` dépend de `zentrix-render` pour connaître les **types** à produire
> (`MeshHandle`, `TextureHandle`, `DrawCommand`, `Camera`). L'inverse reste interdit : le rendu
> ne connaît jamais l'ECS.

Règles à conserver en grandissant :

1. **Les dépendances vont vers le bas.** `zentrix-render` ne dépend jamais de `zentrix-scene`.
2. **Le rendu ne possède pas la logique de jeu**, et le jeu ne manipule jamais directement des objets wgpu.
3. **Pas d'abstraction prématurée** : on ne crée un système de plugins, un render graph, etc. que lorsqu'on a au moins deux cas d'utilisation réels (principe de [GAME_ENGINE_DEVELOPMENT_PHASES.md](../GAME_ENGINE_DEVELOPMENT_PHASES.md)).
4. **Chaque étape a sa démo** et ses tests là où c'est possible (`zentrix-core` en premier).

---

## Et Nanite / Lumen ?

[nanites_et_lumen.md](../nanites_et_lumen.md) décrit des techniques de pointe (géométrie virtualisée, éclairage global dynamique). Elles demandent de maîtriser au préalable : les compute shaders, les storage buffers, le *culling* sur GPU, les *render graphs*, le *deferred shading*... Le chemin est maintenant détaillé dans [Nanite et Lumen](feuille-de-route/05-06-nanite-et-lumen.md) : la **Phase 5** pour Nanite, la **Phase 6** pour Lumen.

---

➡️ **Chapitre suivant** : [Glossaire](08-glossaire.md)
