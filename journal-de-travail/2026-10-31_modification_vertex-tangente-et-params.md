# 2026-10-31 — Modification : `Vertex` à 60 octets, `params` dans `FrameUniform`

## Résumé

Deux changements de format GPU, faits pour les
[normal maps et le tone mapping](2026-10-31_changement-majeur_normal-maps-et-tone-mapping.md).

## Détail

| Élément | Avant | Après |
|---------|-------|-------|
| `Vertex` (`zentrix-core`) | 44 octets : position, couleur, uv, normale | **60 octets** : + `tangent: [f32; 4]` |
| Layout des sommets (`zentrix-render/vertex.rs`) | locations 0 à 3 | + **location 10** (`Float32x4`). Les locations 4 à 9 restent celles des instances, et les numéros n'ont pas besoin de se suivre |
| `Vertex::new` | — | Signature inchangée ; la tangente vaut `[0; 4]` jusqu'à `compute_tangents` |
| `FrameUniform::shadow_params` | x = ombres, y = taille d'un texel | Renommé **`params`** (Rust et les 3 shaders) ; z = tone mapping. Taille inchangée (208 octets) |
| `InstanceRaw::material` | (métal, rugosité, 0, 0) | (métal, rugosité, **intensité du relief**, 0) |
| Textures | Toujours en `Rgba8UnormSrgb` | `Rgba8UnormSrgb` (couleurs) ou `Rgba8Unorm` (données : normal maps) |
| Numérotation des textures | n°0 = blanche | n°0 = blanche, **n°1 = normal map plate**. Les textures de l'application commencent donc à 2 |

## Impact

- Le code qui fabrique des `Vertex` avec `Vertex::new` n'a pas à changer.
- Le code qui écrit `Vertex { ... }` à la main doit ajouter `tangent`.
- Les fichiers de scène ne sont pas touchés : les textures y sont désignées par leur nom.
