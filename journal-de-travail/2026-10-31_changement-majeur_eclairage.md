# 2026-10-31 — Changement majeur : éclairage (normales, lumière directionnelle, Lambert + Blinn-Phong)

## Résumé

Les objets sont maintenant éclairés : une lumière directionnelle (un soleil) éclaire les faces
qui lui font face, les autres restent dans une pénombre ambiante, et un reflet brillant apparaît
selon l'angle de vue. Dans la démo, la lumière tourne lentement : l'éclairage évolue en direct.
Étape J de la [feuille de route](../documentation/guide-debutant/07-feuille-de-route.md).

## Pourquoi

Sans éclairage, toutes les faces d'un objet d'une même couleur se confondent : on perçoit mal
les volumes. L'éclairage est aussi la base des étapes suivantes (ombres, matériaux PBR).

## Détail

### `zentrix-render`

| Fichier | Changement |
|---------|------------|
| `src/vertex.rs` | `Vertex.normal` (location 3), 44 octets ; instances en locations 4 à 7 |
| `src/mesh.rs` | `MeshData::compute_normals()` (moyenne pondérée par l'aire des triangles) ; normales du triangle, du plan et du cube ; 2 tests |
| `src/lighting.rs` *(nouveau)* | `Lighting { direction, color, intensity, ambient }` (public), valeur par défaut « soleil de midi » |
| `src/renderer.rs` | `CameraUniform` → `FrameUniform` (128 octets : matrice + position de l'œil + lumière + ambiante), visible des deux shaders ; champ public `Renderer::lighting` ; 2 tests |
| `src/shaders/mesh.wgsl` | Normale et position monde transmises au fragment shader ; ambiante + diffuse (Lambert) + spéculaire (Blinn-Phong, exposant 32, force 0,25) |
| `src/shaders/debug.wgsl` | Même déclaration `FrameUniform` (seule `view_proj` est utilisée) |

### `zentrix-scene`

- Composant `DirectionalLight { color, intensity }` : direction des rayons = -Z de la matrice
  monde de l'entité (comme `Camera3d`).
- `Scene::ambient_light` (public) et `Scene::lighting() -> Lighting`. Sans lumière
  directionnelle : intensité 0, seule l'ambiante éclaire.
- 1 test (direction, couleur, intensité, ambiante).

### `zentrix-app`

- Entité « Lumière du soleil » penchée de 50°, qui tourne autour de la verticale grâce au
  composant `Spin` déjà existant.
- `F1` affiche aussi la direction de la lumière (flèche jaune depuis le ciel).
- `renderer.lighting = scene.lighting()` avant chaque rendu.

## Décisions techniques

- **Uniquement des `vec4` dans l'uniform** : évite les pièges d'alignement des `vec3` en WGSL.
- **Normales transformées par la matrice modèle** (w = 0) : exact pour une échelle uniforme
  seulement. Une échelle non uniforme demandera la « matrice normale » (inverse transposée) —
  limite documentée dans le shader.
- **Pas de reflet sur les faces à l'ombre** (`select` sur la diffuse) : évite des reflets
  impossibles sur l'arrière des objets.
- **La lumière est une entité** : elle profite de tout l'ECS (hiérarchie, `Spin`...).

## Vérification

- `cargo build` et `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 59 tests réussis (dont la validation des deux shaders).
- Lancement sous Vulkan : faces éclairées / dans l'ombre, reflet visible sur les caisses,
  flèche de la lumière avec F1, 60 FPS, aucune erreur de validation.

## Impact / suite

- Prochaine étape : **K — modèles 3D (glTF)**. Les ombres portées (shadow mapping) viendront
  ensuite.
