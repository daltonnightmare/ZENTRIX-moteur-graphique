# 2026-11-01 — Changement majeur : ombres en cascade (CSM)

## Résumé

Les ombres ne sont plus limitées à une zone fixe de 23 m autour du centre de la scène. Elles
**suivent la caméra** jusqu'à 50 m (réglable). La vue est découpée en **3 tranches de
profondeur**, chacune avec sa propre shadow map de 2048² :

| Tranche | Profondeur (réglages par défaut) |
|---------|----------------------------------|
| 0 | 0,1 → 4,8 m |
| 1 | 4,8 → 13 m |
| 2 | 13 → 50 m |

La première tranche couvre une petite zone, donc ses ombres sont très nettes ; la dernière
couvre une grande zone, mais elle est vue de loin.

`ShadowArea` est **supprimée**, remplacée par `ShadowSettings`.

## Détail

### `zentrix-render`

| Élément | Changement |
|---------|------------|
| `cascades.rs` (nouveau) | Voir le détail sous le tableau |
| `shadow.rs` | Shadow map à **couches** (`D2Array`, une couche par tranche), une vue de rendu par couche. Nouveau buffer des matrices des tranches, lu avec un **décalage dynamique** (une case alignée sur `min_uniform_buffer_offset_alignment` par tranche) et son layout `create_cascade_layout` |
| `shadow.wgsl` | Ne lit plus `FrameUniform`, seulement la matrice de la tranche en cours (`Cascade`) |
| `pipeline.rs` | Les pipelines d'ombre utilisent le layout des tranches |
| `FrameUniform` | Réorganisée, de 288 à 464 octets : `cascades: [mat4; 3]`, `camera_forward`, `cascade_far`, `cascade_texel`. `params.z` = tranches teintées (debug), `params.w` réservé au nombre de lumières. Même struct dans mesh, debug et sky |
| `mesh.wgsl` | Voir le détail sous le tableau |
| `renderer.rs` | Voir le détail sous le tableau |
| `lighting.rs` | `ShadowArea` et ses tests supprimés |

Détail de `cascades.rs` :
- `CASCADE_COUNT = 3`.
- `ShadowSettings { distance (50 m), split_lambda (0,75) }`.
- `split_distances` : répartition « pratique » de GPU Gems 3, un mélange de linéaire et de
  logarithmique.
- `compute_cascades` :
  - chaque tranche est entourée d'une **sphère**, dont la taille ne dépend pas de
    l'orientation de la caméra ;
  - le rayon est arrondi au 1/16 de mètre ;
  - la caméra de la lumière est reculée de 40 m (`CASTER_MARGIN`) pour inclure les objets
    hors champ qui projettent une ombre ;
  - **calage sur la grille des texels** : l'origine du monde tombe pile sur un texel ;
  - chaque tranche indique sa fin (`far`) et la taille d'un texel en mètres.

Détail de `mesh.wgsl` :
- `cascade_index(profondeur)` choisit la tranche ;
- `cascade_shadow` : décalage le long de la normale (*normal offset*) de 1,5 texel, puis PCF
  3 × 3 dans la couche de la tranche ;
- `shadow_factor` : **fondu** avec la tranche suivante sur les 10 derniers % de chaque
  tranche, et les ombres s'effacent après la dernière ;
- les tranches sont teintées en rouge, vert et bleu si l'option de debug est active ;
- la position vue depuis la lumière n'est plus transmise par le vertex shader : elle est
  calculée par pixel, selon la tranche.

Détail de `renderer.rs` :
- champs publics `shadow_settings` et `show_cascades`, accesseur `cascades()` ;
- les tranches sont calculées à chaque image ;
- une passe d'ombre par tranche, dans sa couche, avec le décalage de sa matrice ;
- le culling des ombres utilise la plus grande tranche.

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `main.rs` | Plus de `ShadowArea`. Avec l'option de debug, les boîtes des 3 tranches sont dessinées de la couleur de leur tranche, à la place de la boîte unique dessinée avec `F1` |
| `inspector.rs` | Curseur de distance des ombres (10 à 150 m) et case « Tranches » |

## Vérification

- `cargo test --workspace` : **141 tests** réussis. Tests des cascades :
  - les tranches s'allongent, et sont égales avec lambda = 0 ;
  - les points au fond de chaque tranche, au centre et dans les coins de l'écran, sont dans sa
    boîte ;
  - les texels grandissent d'une tranche à l'autre ;
  - la taille ne change pas quand la caméra tourne ;
  - l'origine du monde reste sur un texel quand la caméra se déplace un peu.
- `cargo clippy --workspace --all-targets` : propre.
- Démo lancée (Vulkan) :
  - les ombres sont présentes, sans acné visible ;
  - 30 appels de dessin d'ombre (10 objets × 3 tranches) ;
  - avec « Tranches », le premier plan est teinté en vert et le fond en bleu, et les boîtes
    des tranches sont dessinées.

## Limites connues

- **Coût CPU** : chaque objet qui projette une ombre est dessiné **3 fois**, même s'il n'est
  que dans une tranche. Le culling des ombres se fait contre la plus grande tranche
  seulement.
- **Résolution** : la première tranche est minuscule (4,8 m). Avec une caméra à 11 m de la
  scène, elle ne sert presque pas. Une répartition plus linéaire (`split_lambda` plus petit)
  peut mieux convenir à une scène vue de loin.
- **Mémoire** : 48 Mo de mémoire GPU pour les 3 couches.
- **Scintillement en tournant** : il est réduit par la sphère et le calage, mais une rotation
  de la lumière (le soleil qui tourne) déplace encore légèrement les bords d'ombre.
