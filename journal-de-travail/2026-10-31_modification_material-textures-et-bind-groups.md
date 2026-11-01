# 2026-10-31 — Modification : `MaterialTextures`, bind groups réorganisés

## Résumé

Ces changements d'API et de format accompagnent les
[textures de matériau et l'environnement](2026-10-31_changement-majeur_textures-materiau-et-environnement.md).

## Détail

| Élément | Avant | Après |
|---------|-------|-------|
| Textures d'un objet | `texture` et `normal_map` (champs de `DrawCommand` et de `MeshRenderer`) | **`textures: MaterialTextures`** : `base_color`, `normal`, `metallic_roughness`, `occlusion`, `emissive`, tous facultatifs. `MaterialTextures::color(t)` raccourcit le cas d'une simple texture de couleur |
| `MeshRenderer` | `with_texture` et `with_normal_map` | Ces deux méthodes restent ; + `with_textures(MaterialTextures)` |
| `ModelPart` (assets) | `texture` et `normal_map` | `textures: PartTextures` (indices d'image) ; `is_color_image` |
| `Texture` (interne) | Gardait un bind group par texture | Garde sa **vue** ; les bind groups de matériau sont créés à la demande, dans `HashMap<[TextureHandle; 5], BindGroup>` |
| Clé de lot | (texture, normal map, maillage) | ([5 textures], maillage) |
| `Renderer` | — | `add_data_texture` (sans sRGB ; `add_normal_map` l'appelle) ; `set_sky(&Sky)` ; `environment_enabled` ; `environment_intensity` |
| `FrameUniform::new` | Deux `bool` (ombres, tone mapping) | Deux tableaux `params` et `environment` (`[f32; 4]`) calculés par le `Renderer` |
| `FrameUniform` | 208 octets | **288 octets** (+ `environment`, + `inv_view_proj`) |
| `InstanceRaw` | 96 octets | **112 octets** (+ `emissive` : rgb + intensité de l'occlusion, location 11) |
| `Material` | — | + `emissive`, + `occlusion_strength`. `dielectric` et `metal` complètent avec `..Self::DEFAULT` |

## Impact

- **Code** : partout où on lisait `draw.texture`, il faut écrire `draw.textures.base_color`.
  Les tests de la démo, de la scène et des modèles ont été mis à jour.
- **Fichiers de scène** : compatibles. Les noms `texture` et `normal_map` sont conservés dans le
  format RON.
- **Correction au passage** : un commentaire de `vertex.rs` affichait « relief, relief ». Une
  relance d'un script de patch précédent (non idempotent) l'avait doublé.
