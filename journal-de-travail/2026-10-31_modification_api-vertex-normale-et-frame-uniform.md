# 2026-10-31 — Modification : `Vertex` avec normale, `FrameUniform`, lumière de la scène

## Résumé

Changements d'API liés à l'[éclairage](2026-10-31_changement-majeur_eclairage.md).

## Détail

| Élément | Avant | Après |
|---------|-------|-------|
| `Vertex::new` | `new(position, color, uv)` | `new(position, color, uv, normal)` (passer `[0.0; 3]` puis `MeshData::compute_normals()` si inconnue) |
| Taille de `Vertex` | 32 octets | 44 octets |
| Locations des instances (shader) | 3 à 6 | 4 à 7 |
| Uniform du groupe 0 | `CameraUniform { view_proj }`, vertex shader seulement | `FrameUniform { view_proj, camera_position, light_direction, light_color, ambient }`, vertex + fragment |
| Noms internes du `Renderer` | `camera_buffer`, `camera_bind_group`, `camera_layout` | `frame_buffer`, `frame_bind_group`, `frame_layout` |
| `Renderer` | — | champ public `lighting: Lighting` |
| `Scene` | — | champ public `ambient_light: Color` ; `Default` écrit à la main (l'ambiante par défaut n'est pas `Color::default()`) |

## Impact

- Les couleurs de la démo sont plus sombres sur les faces à l'ombre : c'est voulu.
- Un `MeshData` construit à la main avec des normales nulles (`[0.0; 3]`) s'affiche mal tant
  qu'on n'appelle pas `compute_normals()` : le shader normalise un vecteur nul, ce qui donne
  un résultat indéfini (souvent noir, parfois des pixels aberrants selon le GPU).
