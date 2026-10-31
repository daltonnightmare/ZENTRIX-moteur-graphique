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
| **Phase 3** — Scène et ECS | 🟡 **en cours** : `zentrix-scene` (hecs), composants `Transform` / `MeshRenderer` / `Camera3d` / `Name`, extraction vers le rendu ✅ · hiérarchie, sérialisation ⏳ |
| Phase 4+ — éclairage, modèles 3D, audio, physique... | ⏳ à venir |

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

### Étape I — Hiérarchie de transformations ⭐⭐⭐ ← prochaine étape

- Composants `Parent` / `Children` et un `GlobalTransform` calculé : un objet suit son parent
  (une roue suit la voiture, une arme suit la main).
- Un système de propagation qui parcourt l'arbre de la racine vers les feuilles, avant l'extraction.

**Démo** : un « système solaire » (une lune qui tourne autour d'une planète qui tourne autour du soleil).

### Étape J — Éclairage simple ⭐⭐⭐

- Ajouter les **normales** dans `Vertex`, une lumière directionnelle + ambiante dans un uniform.
- Éclairage de Lambert (diffus) dans `mesh.wgsl`, puis Blinn-Phong (reflets).

**Démo** : des cubes qui ont une face éclairée et une face dans l'ombre.

### Étape K — Modèles 3D (glTF) ⭐⭐⭐⭐

- Dans `zentrix-assets` : charger des fichiers `.gltf` / `.glb` (crate `gltf`) en `MeshData` + textures.

**Démo** : un modèle 3D téléchargé (licence libre) au milieu de la scène.

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

[nanites_et_lumen.md](../nanites_et_lumen.md) décrit des techniques de pointe (géométrie virtualisée, éclairage global dynamique). Elles demandent de maîtriser au préalable : les compute shaders, les storage buffers, le *culling* sur GPU, les *render graphs*, le *deferred shading*... Gardez-les comme horizon lointain : chaque étape ci-dessus vous en rapproche.

---

➡️ **Chapitre suivant** : [Glossaire](08-glossaire.md)
