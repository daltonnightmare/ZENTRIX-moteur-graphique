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

## Phase 4 — Nouvelles pistes

Toutes les pistes ci-dessus sont réalisées. Les suivantes sont regroupées par ce qu'elles
apportent, dans l'ordre conseillé à l'intérieur de chaque groupe.

### Ce qui se voit à l'écran

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Transparence** ✅ | Verre, eau, feuillage découpé : `alpha` du matériau, textures avec canal alpha, *alpha test* (découpe) et *alpha blending* (mélange). Les objets transparents sont dessinés **après** les opaques, triés de l'arrière vers l'avant, sans écrire dans le depth buffer — réalisée : `AlphaMode` (opaque, découpe, mélange) et `double_sided` dans `Material` (lus depuis glTF), alpha prémultiplié (le verre garde ses reflets), 4 variantes du pipeline des maillages, tri par distance ([culling.rs](../../crates/zentrix-render/src/culling.rs)), ombres trouées pour la découpe ; démo : vitre, cubes de verre, lanterne grillagée. Pistes suivantes : transparence indépendante de l'ordre (OIT), réfraction, ombres colorées du verre | ⭐⭐ |
| **Post-traitement** ✅ | Dessiner d'abord dans une texture HDR, puis des passes plein écran : **bloom** (les zones très lumineuses « bavent »), **anti-aliasing** (FXAA ou MSAA : bords sans escaliers), tone mapping déplacé dans cette passe — réalisé : image HDR `Rgba16Float`, bloom à la Call of Duty (seuil doux, 6 niveaux de réduction puis remontée additive), tone mapping et FXAA en passes finales ([postprocess.rs](../../crates/zentrix-render/src/postprocess.rs), [post.wgsl](../../crates/zentrix-render/src/shaders/post.wgsl)) ; réglages dans l'inspecteur. Pistes suivantes : MSAA, exposition automatique, profondeur de champ, flou de mouvement, SSAO | ⭐⭐⭐ |
| **Ombres en cascade (CSM)** ✅ | Plusieurs shadow maps de plus en plus grandes selon la distance : ombres nettes près de la caméra, présentes jusqu'au loin (aujourd'hui, une seule zone de 23 m) — réalisées : 3 tranches jusqu'à 50 m ([cascades.rs](../../crates/zentrix-render/src/cascades.rs)), sphères englobantes et calage sur les texels contre le scintillement, fondu entre tranches, décalage selon la normale, debug « Tranches ». Pistes suivantes : culling par tranche, ombres douces (PCSS), mise à jour moins fréquente des tranches lointaines | ⭐⭐⭐ |
| **Lumières ponctuelles et projecteurs** ✅ | Lampes et spots colorés, avec atténuation selon la distance ; plusieurs lumières par image (storage buffer) ; les néons éclairent vraiment la scène — réalisées : composants `PointLight` et `SpotLight` (sauvegardés), `LocalLight` côté rendu, jusqu'à 256 lumières par image dans un storage buffer, atténuation en 1/d² avec fenêtre, cône à bord doux, culling des lumières hors de la vue ; démo : flamme de la lanterne, néons, manège de lampes colorées, projecteur, touche `L` (nuit). Pistes suivantes : ombres des lampes (cube maps), *clustered shading* pour des centaines de lampes, lumières surfaciques | ⭐⭐⭐ |

### Ce qui rend le moteur « jouable »

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Contrôleur de personnage** | Marcher sur le sol, sauter, monter des marches, glisser le long des murs (`KinematicCharacterController` de rapier), caméra à la première ou à la troisième personne | ⭐⭐ |
| **Petit gameplay** | Un mini-jeu dans le moteur (ramasser des objets, toucher des cibles avec les balles, score à l'écran) : le meilleur test de l'API du moteur | ⭐⭐ |
| **Animation de modèles** | Animations glTF : images clés (position, rotation), puis squelette et *skinning* pour des personnages | ⭐⭐⭐⭐ |

### Ce qui en fait un vrai outil

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Éditeur** | Sélection par clic dans la scène (lancer de rayon), *gizmos* pour déplacer / tourner / agrandir à la souris, création et suppression d'entités, annuler / refaire (Ctrl+Z) | ⭐⭐⭐ |
| **Rechargement à chaud** | Modifier un shader `.wgsl`, une texture ou une scène et voir le résultat sans relancer (surveillance des fichiers avec la crate `notify`) | ⭐⭐ |
| **Mesures GPU** | *Timestamp queries* : le temps de chaque passe sur la carte graphique, affiché à côté du profileur CPU | ⭐⭐ |

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
