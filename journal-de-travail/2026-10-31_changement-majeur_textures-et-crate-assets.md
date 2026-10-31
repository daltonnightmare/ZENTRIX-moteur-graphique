# 2026-10-31 — Changement majeur : textures et crate `zentrix-assets`

## Résumé

Le moteur sait plaquer des images sur les maillages : chargement PNG/JPEG, textures GPU avec
mipmaps, filtrage anisotrope. La démo affiche des caisses en bois (texture chargée depuis le
disque) et un sol en damier (texture générée). Étape F de la
[feuille de route](../documentation/guide-debutant/07-feuille-de-route.md).

## Pourquoi

Les couleurs par sommet ne suffisent pas à rendre des surfaces détaillées : les textures sont la
base de tout matériau (et plus tard de l'éclairage : normal maps, etc.).

## Détail

### `zentrix-core`

| Fichier | Changement |
|---------|------------|
| `src/image.rs` *(nouveau)* | `Image` RGBA8 sRGB : `new`, `solid`, `checkerboard`, `from_fn`, `pixel`, `downsample` (moyenne 2×2), `mip_chain` ; 4 tests |
| `src/color.rs` | `Color::to_srgba8()` : conversion linéaire → octets sRGB (courbe officielle) ; 1 test |

### `zentrix-assets` *(nouvelle crate)*

- `load_image(path)` et `decode_image(bytes)` : PNG/JPEG → `Image` (toujours RGBA8).
- `AssetError` : `Io` (fichier illisible) ou `Decode` (contenu invalide), avec le chemin.
- `asset_dir()` : trouve le dossier `assets/` (variable `ZENTRIX_ASSETS`, dossier courant, à
  côté de l'exécutable, puis racine du workspace).
- 4 tests (décodage, fichier absent, contenu invalide, dossier trouvé).
- Dépendance `image 0.25` limitée aux formats PNG et JPEG (`default-features = false`).

### `zentrix-render`

| Fichier | Changement |
|---------|------------|
| `src/texture.rs` *(nouveau)* | `TextureHandle`, `Texture` (bind group), layout du groupe n°1 (texture + sampler), sampler commun (`Repeat`, `Linear`, anisotropie 16), envoi de toute la chaîne de mipmaps ; réduction automatique si l'image dépasse la limite du GPU |
| `src/vertex.rs` | `Vertex` passe à 32 octets avec `uv: [f32; 2]` (location 2) ; instances déplacées en locations 3 à 6 |
| `src/mesh.rs` | UV sur `triangle`, `plane`, `cube` ; `with_color()` et `with_uv_scale()` |
| `src/renderer.rs` | `add_texture()`, texture blanche par défaut (handle 0), `DrawCommand.texture`, regroupement par (texture, maillage) et `set_bind_group(1, …)` seulement quand la texture change |
| `src/pipeline.rs` | Deuxième bind group layout |
| `src/shaders/mesh.wgsl` | `textureSample(...) * couleur du sommet` |

### Ressources et démo

- `assets/textures/caisse.png` *(nouveau)* : caisse en bois 256×256 générée pour la démo.
- `zentrix-app` : sol en damier répété 8×8 fois, une case sur deux est une caisse texturée ;
  si `caisse.png` est introuvable, avertissement puis damier magenta de remplacement.

## Décisions techniques

- **`Image` dans le noyau** : `zentrix-render` ne dépend pas de `zentrix-assets`, les deux
  partagent seulement ce type (règle « les dépendances vont vers le bas »).
- **Mipmaps calculées sur le CPU** : simple et portable. Limite connue : la moyenne est faite
  sur les valeurs sRGB et non en linéaire (écart minime).
- **Texture blanche par défaut** : un seul pipeline pour les objets texturés ou non.
- **Format `Rgba8UnormSrgb`** : le GPU convertit sRGB → linéaire à la lecture ; associé à
  `to_srgba8()`, une couleur a le même rendu en texture et en couleur de sommet.

## Vérification

- `cargo build` et `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 31 tests réussis (9 nouveaux).
- Lancement sous Vulkan (Intel Iris Xe) : caisses texturées correctement orientées, damier net
  au loin (pas de moiré), aucune erreur de validation.
- Test du cas d'erreur avec `ZENTRIX_ASSETS` pointant vers un dossier inexistant :
  avertissement dans les logs, application toujours en marche.

## Impact / suite

- Prochaine étape : **G — rendu de debug** (lignes : grille, axes), fin de la Phase 2.
