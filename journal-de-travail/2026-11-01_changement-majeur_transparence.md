# 2026-11-01 — Changement majeur : transparence (découpe, mélange, double face)

## Résumé

Les matériaux peuvent maintenant être **transparents**. Les trois modes alpha de glTF sont
pris en charge :

| Mode | Usage | Rendu |
|------|-------|-------|
| `Opaque` | Le cas courant | Alpha ignoré |
| `Mask { cutoff }` (découpe) | Feuillage, grillage | Les pixels sous le seuil sont abandonnés (`discard`). L'objet reste un opaque : pas de tri, et ses **ombres sont trouées** |
| `Blend` (mélange) | Verre, eau | Dessiné après les opaques et le ciel, du plus loin au plus proche, sans écrire dans le depth buffer |

L'option **`double_sided`** dessine aussi l'arrière des triangles : on voit l'intérieur d'une
cage ou d'un cube de verre.

Démo :
- une vitre et trois cubes de verre colorés, à gauche de la grille ;
- une lanterne grillagée avec une flamme lumineuse, à droite.

## Détail

### `zentrix-core`

- `AlphaMode`, sérialisable : `Opaque`, `Mask(cutoff: …)`, `Blend`.
- Nouveaux champs de `Material` : `alpha_mode` et `double_sided`. Les anciennes scènes se
  relisent grâce à `serde(default)`.
- `Material::glass(couleur, alpha)` : lisse, en mélange, double face.
- `Material::is_transparent()`.

### `zentrix-assets`

`convert_material` lit `alphaMode`, `alphaCutoff` (0,5 par défaut, comme le veut glTF) et
`doubleSided`.

### `zentrix-render`

| Élément | Changement |
|---------|------------|
| `InstanceRaw.material.w` | Le mode alpha codé en un nombre : 0 = opaque, > 0 = seuil de découpe, -1 = mélange |
| `mesh.wgsl` | Voir le détail sous le tableau |
| `shadow.wgsl` | Nouvelles entrées `vs_masked` / `fs_masked` : lisent l'alpha de la texture de couleur et abandonnent les pixels transparents |
| `pipeline.rs` | `MeshVariant { blend, double_sided }` : 4 pipelines des maillages, même shader. La variante mélange utilise `PREMULTIPLIED_ALPHA_BLENDING`, sans écriture de profondeur. Deux pipelines d'ombre : plein, et découpe (avec le bind group des textures). Plus aucune face éliminée dans la passe d'ombre, pour que les objets double face projettent une ombre |
| `culling.rs` | `DrawLists` : `opaque` (trié par lot), `transparent` (trié du plus loin au plus proche, par la distance de l'origine de l'objet à l'œil), `shadow`. Le verre ne projette pas d'ombre. La clé de lot commence par le numéro du pipeline |
| `renderer.rs` | `BatchKey = (pipeline, textures, maillage)`. `draw_batches` change de pipeline quand la clé l'exige. Ordre de la passe principale : opaques, ciel, transparents, lignes de debug. Buffer d'instances : opaques, transparents, ombres. `RenderStats::transparent` |

Détail des changements de `mesh.wgsl` :
- `alpha` = alpha de la texture × alpha du matériau ;
- `discard` sous le seuil en mode découpe ;
- `@builtin(front_facing)` retourne la normale quand on voit l'arrière d'un triangle ;
- **alpha prémultiplié** : seule la lumière diffuse est multipliée par l'alpha. Les reflets
  (spéculaire direct et environnement) et l'émission restent entiers. Une vitre presque
  invisible reflète donc encore le ciel.

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `demo.rs` | `spawn_glass` : vitre (alpha 0,12) et cubes de verre cyan, ambre, magenta qui tournent, avec collision. `spawn_lantern` : cube grillagé (métal, découpe, double face) et sphère émissive à l'intérieur. `lattice_image` : texture de grillage (barreaux pleins, trous à alpha 0). `DemoAssets::lattice_texture` |
| `main.rs` | Texture « grillage » chargée et nommée pour la sauvegarde |
| `inspector.rs` | Choix du mode alpha (opaque, découpe, mélange), seuil, opacité, case « Double face » ; nombre d'objets transparents dans le panneau Performance |

## Vérification

- `cargo test --workspace` : **136 tests** réussis, dont les nouveaux :
  - matériau : modes alpha en RON (aller-retour) ;
  - tri : le verre est trié du plus loin au plus proche et ne projette pas d'ombre ; la
    découpe reste opaque mais garde ses textures pour l'ombre ; plusieurs threads = un thread,
    avec du verre ;
  - démo : 4 objets en verre, une découpe double face, la texture de grillage a des trous ;
  - comptes de la démo et de la physique mis à jour : 59 entités, 30 corps cinématiques,
    13 fixes.
- Les shaders sont validés par naga.
- `cargo clippy --workspace --all-targets` est propre.
- Démo lancée (Vulkan) et vérifiée sur captures d'écran :
  - les cubes de verre teintent ce qui est derrière eux et montrent leurs faces arrière ;
  - la lanterne laisse voir l'intérieur de la cage et la flamme ;
  - son **ombre au sol est trouée**.

## Limites connues

- **Tri par objet** : le tri se fait objet par objet, sur l'origine de chaque objet. Deux
  objets transparents qui s'interpénètrent, ou un grand objet près d'un petit, peuvent être
  dessinés dans le mauvais ordre. Les faces d'un même objet double face ne sont pas triées
  entre elles ; c'est peu visible sur un cube.
- **Ombres du verre** : le verre ne projette aucune ombre, alors qu'il devrait faire une ombre
  légère ou colorée.
- **Découpe vue de loin** : les mipmaps font la moyenne de l'alpha. De loin, un grillage fin
  peut s'éclaircir puis disparaître sous le seuil (une solution : l'*alpha to coverage*, avec
  le MSAA).
- **Mélange et tone mapping** : le mélange se fait après le tone mapping, sur l'image finale,
  et non en HDR. Ce sera à revoir avec la piste « Post-traitement ».
