# 2026-10-31 — Changement majeur : ombres portées (shadow mapping)

## Résumé

Les objets projettent leur ombre sur le sol et les uns sur les autres. L'ombre suit la lumière,
qui tourne lentement dans la démo. `F3` active / désactive les ombres pour comparer. Étape L de
la [feuille de route](../documentation/guide-debutant/07-feuille-de-route.md).

## Pourquoi

Sans ombre, les objets semblent flotter au-dessus du sol : on évalue mal leur position. Les
ombres sont l'un des indices de profondeur les plus importants pour l'œil.

## Principe

1. **Passe d'ombre** : la scène est dessinée depuis la lumière, profondeur seule, dans une
   texture de 2048 × 2048 (la *shadow map*).
2. **Passe principale** : chaque pixel calcule sa position vue depuis la lumière et la compare
   à la shadow map ; s'il est plus loin que ce qui y est enregistré, un objet le cache : il est
   dans l'ombre. L'ombre retire la lumière directe (diffuse + reflet), pas l'ambiante.

## Détail

### `zentrix-render`

| Fichier | Changement |
|---------|------------|
| `src/lighting.rs` | `ShadowArea { center, radius }` (public) et `light_view_projection(direction)` : projection **orthographique** (rayons parallèles) qui entoure la zone ; 2 tests |
| `src/shadow.rs` *(nouveau)* | `ShadowMap` : texture `Depth32Float` 2048², sampler de **comparaison** (`LessEqual`, filtrage linéaire), bind group n°2 |
| `src/shaders/shadow.wgsl` *(nouveau)* | Vertex shader seul : position × matrice de la lumière |
| `src/pipeline.rs` | `create_shadow_pipeline` (sans fragment shader, biais de profondeur constant 2 / pente 2.0) ; groupe n°2 ajouté au pipeline des maillages ; test de validation de `shadow.wgsl` |
| `src/shaders/mesh.wgsl` | `light_clip` transmis au fragment shader ; `shadow_factor` : PCF 3 × 3 avec `textureSampleCompareLevel` ; zone hors shadow map = éclairée |
| `src/renderer.rs` | Passe d'ombre avant la passe principale ; champs publics `shadows_enabled` et `shadow_area` ; `draw_batches` partagé par les deux passes |

### `zentrix-app`

- `F3` : ombres activées / désactivées (message dans les logs).
- `shadow_area` réglée sur la scène (centre (0, 0.5, 0), rayon 11,5 : tout le sol).
- `F1` affiche aussi, en gris, la boîte couverte par la shadow map (inverse de la matrice de
  la lumière appliqué au pavé NDC).

## Décisions techniques

- **Biais de profondeur dans le pipeline** (et non dans le shader) contre l'« acné d'ombre ».
- **`textureSampleCompareLevel`** plutôt que `textureSampleCompare` : pas de dérivées, donc
  utilisable dans une boucle et après un `if` (règles d'uniformité de WGSL).
- **Double filtrage** : sampler de comparaison linéaire (4 tests matériels) × PCF 3 × 3 =
  36 tests par pixel, pour des bords doux.
- **Zone d'ombre fixe** choisie par l'application : simple et stable. Les techniques pour les
  grands mondes (cascades de shadow maps, alignement sur les texels contre le scintillement)
  viendront plus tard.

## Vérification

- `cargo build`, `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 67 tests réussis (dont la validation des trois shaders).
- Lancement sous Vulkan : ombres sous les cubes, les caisses et les arbres, aucune acné
  visible sur le sol ; disparaissent avec `F3` ; 60 FPS ; aucune erreur de validation.

## Limites connues

- Une seule lumière projette des ombres (la directionnelle).
- Hors de la zone (`ShadowArea`), tout est éclairé.
- Coût : une deuxième passe de géométrie par image + 16 Mo de mémoire GPU.
