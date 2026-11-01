# 2026-11-01 — Changement majeur : lampes et projecteurs (`PointLight`, `SpotLight`)

## Résumé

Jusqu'ici, seul le soleil éclairait la scène. Elle peut maintenant contenir des **lumières
locales** :
- des **lampes** (`PointLight`), qui éclairent dans toutes les directions ;
- des **projecteurs** (`SpotLight`), qui éclairent dans un cône.

Leur éclairage diminue en 1 / distance² jusqu'à une portée. Jusqu'à **256** lumières locales
par image sont envoyées au shader (celles qui touchent la vue).

Dans la démo :
- la flamme de la lanterne éclaire en orange ;
- les caisses sci-fi éclairent le sol en cyan ;
- trois lampes colorées (rouge, vert, bleu) tournent au-dessus de la grille ;
- un projecteur éclaire la vitrine de sphères ;
- la nouvelle touche **`L`** passe à la **nuit** : soleil, ciel et lumière ambiante presque
  éteints, seules les lampes éclairent.

## Détail

### `zentrix-render`

| Élément | Changement |
|---------|------------|
| `lighting.rs` | `LocalLight { position, color, intensity, range, spot: Option<SpotCone> }`, `SpotCone { direction, inner_angle, outer_angle }`. `LightRaw` (64 octets, interne) : les cosinus des angles sont calculés côté CPU, et l'angle intérieur est borné par l'extérieur |
| `renderer.rs` | Champ public `lights: Vec<LocalLight>`, constante `MAX_LOCAL_LIGHTS = 256`. **Storage buffer** des lumières en binding 1 du groupe 0. `upload_lights` écarte les lumières dont la sphère de portée est hors de la vue. `params.w` = nombre de lumières. `RenderStats::lights` |
| `mesh.wgsl` | Voir le détail sous le tableau |

Détail de `mesh.wgsl` :
- **Factorisation** : `Surface` et `direct_light(surface, l, radiance)` contiennent le calcul
  Cook-Torrance, partagé par le soleil et les lampes.
- `local_light` calcule la lumière reçue d'une lampe :
  - une atténuation en `1 / d²` (bornée à 10 cm de la lampe) ;
  - multipliée par une fenêtre `(1 - (d/portée)⁴)²`, qui l'amène doucement à zéro à la
    portée ;
  - pour un projecteur, un `smoothstep` entre les cosinus des deux angles.
- Une boucle parcourt toutes les lumières de l'image ; elle saute celles hors de portée.

### `zentrix-scene`

- Composants `PointLight { color, intensity, range }` et
  `SpotLight { color, intensity, range, inner_angle, outer_angle }`, tous deux avec
  `serde(default)`. Ils sont enregistrés dans le sérialiseur par défaut.
- `Scene::extract_lights` : la position vient de la matrice monde ; la direction d'un
  projecteur est le -Z de son repère, comme pour `DirectionalLight`.

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `demo.rs` | `PointLight` sur la flamme de la lanterne et sur les caisses sci-fi. `spawn_lights` : un pivot qui tourne (« Manège de lampes ») avec 3 petites sphères émissives qui portent chacune une `PointLight`, et un projecteur à 4,5 m au-dessus de la vitrine |
| `main.rs` | Les lumières sont extraites à chaque image. Avec `F2`, chaque lampe est marquée par un petit cube de sa couleur, et chaque projecteur par un trait dans sa direction. Touche `L` : `toggle_night`, qui garde les réglages du jour pour les rétablir |
| `inspector.rs` | Réglages de la lampe ou du projecteur sélectionné (couleur, intensité, portée, angles en degrés). Nombre de lumières dans le panneau Performance |

## Vérification

- `cargo test --workspace` : **144 tests** réussis. Nouveaux tests :
  - taille de `LightRaw` ;
  - un projecteur compare des cosinus, et un angle intérieur trop grand est corrigé ;
  - extraction : position et direction d'un projecteur tourné vers le bas ;
  - démo : 64 entités, 6 lampes et 1 projecteur retrouvés après sauvegarde et rechargement.
- `cargo clippy --workspace --all-targets` : propre.
- Démo lancée (Vulkan), captures de jour et de nuit :
  - de nuit, les lampes colorées teintent les cubes et le sol, et se déplacent entre deux
    captures ;
  - le sol est cyan autour des caisses sci-fi, la lanterne diffuse une lueur orangée ;
  - le projecteur éclaire le dessus des sphères ;
  - le bloom fait briller les sources.

## Limites connues

- **Pas d'ombres** pour les lampes : la lumière traverse les objets (la lanterne éclaire à
  travers ses barreaux sans projeter leur image). Il faudrait une shadow map en cube par lampe.
- **Coût** : chaque pixel parcourt **toutes** les lumières visibles. Cela suffit pour quelques
  dizaines ; pour des centaines, il faudrait les répartir par zone de l'écran (*clustered
  shading*).
- **Unités** : l'intensité est relative (1 = blanc à 1 m), pas en lumens ou en candelas.
- **Nuit** : la touche `L` ne sauvegarde rien. Les réglages du jour sont rétablis par un
  second appui, pas par un rechargement de scène.
