# 2026-10-31 — Changement majeur : rendu 3D (maillages, caméra, profondeur, instanciation)

## Résumé

ZENTRIX passe d'un triangle 2D codé en dur à une vraie scène 3D : une grille de 5×5 cubes qui
tournent sur un sol, vue en perspective par une caméra en orbite. Étapes A à D de la
[feuille de route](../documentation/guide-debutant/07-feuille-de-route.md).

## Pourquoi

Suite logique de la Phase 2 (noyau de rendu) : sans maillages, caméra ni depth buffer, aucune
scène 3D n'est possible.

## Détail

### `zentrix-core`

| Fichier | Changement |
|---------|------------|
| `Cargo.toml` | Ajout de la dépendance `glam` (la seule du noyau) |
| `src/math.rs` *(nouveau)* | Ré-export de `Vec2/3/4`, `Mat4`, `Quat` et `glam::camera` ; conventions du moteur (main droite, Y en haut) |
| `src/transform.rs` *(nouveau)* | `Transform { translation, rotation, scale }` → `matrix()` ; 2 tests + 1 doc-test |
| `src/lib.rs` | Déclare `math` et `transform`, exporte `Transform` |

### `zentrix-render`

| Fichier | Changement |
|---------|------------|
| `src/mesh.rs` *(nouveau)* | `MeshData` (CPU : `triangle()`, `plane()`, `cube()`), `Mesh` (buffers GPU, indices `u32`), `MeshHandle` ; 2 tests (dont orientation des faces du cube) |
| `src/camera.rs` *(nouveau)* | `Camera` : `view()`, `projection(aspect)`, `view_projection(aspect)` ; 2 tests |
| `src/depth.rs` *(nouveau)* | `DepthTexture` en `Depth32Float` |
| `src/vertex.rs` | `Vertex::new` (`const fn`), `PartialEq` ; nouveau `InstanceRaw` (matrice modèle, locations 2 à 5, `VertexStepMode::Instance`) ; 1 test |
| `src/pipeline.rs` | `create_triangle_pipeline` → `create_mesh_pipeline` : bind group de la caméra, instance buffer, test de profondeur `Less` |
| `src/shaders/mesh.wgsl` | Renommé depuis `triangle.wgsl` ; uniform caméra + matrice d'instance |
| `src/renderer.rs` | Réécrit : `add_mesh()`, `render(gpu, camera, &[DrawCommand])`, uniform buffer caméra, instance buffer extensible, depth buffer recréé au redimensionnement, tri par maillage et un `draw_indexed` instancié par maillage |
| `src/lib.rs` | Exporte `Camera`, `MeshData`, `MeshHandle`, `DrawCommand`, `InstanceRaw` |

### `zentrix-app`

`src/main.rs` : scène de démo (sol + 25 cubes), caméra en orbite, liste de `DrawCommand`
reconstruite à chaque image dans `on_update`.

## Décisions techniques

- **`DrawCommand { mesh, transform }`** : le rendu ne connaît pas la logique de jeu ; plus tard,
  l'ECS produira cette liste.
- **Instanciation** : 26 objets dessinés en 2 appels de dessin.
- **Projection `glam::camera::rh::proj::directx`**, même sur Vulkan : wgpu impose la
  convention WebGPU (Y vers le haut, profondeur 0..1). La variante `vulkan` inverserait l'image.
- **glam 0.34** a supprimé `Mat4::look_at_rh` / `perspective_rh` : on passe par `glam::camera`.

## Vérification

- `cargo build` et `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 16 tests réussis.
- Lancement avec `WGPU_BACKEND=vulkan` sur Intel Iris Xe : scène correcte (perspective,
  occlusion entre cubes), aucune erreur de validation dans les logs.

## Impact / suite

- API de `Renderer::render` modifiée (nouveaux paramètres `camera` et `draws`).
- Couleurs plus claires à l'écran : la surface sRGB interprète les couleurs comme linéaires.
- Prochaine étape : **E — entrées et caméra libre** (struct `Input`, ZQSD + souris).
