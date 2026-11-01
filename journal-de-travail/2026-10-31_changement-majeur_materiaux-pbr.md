# 2026-10-31 — Changement majeur : matériaux PBR (métal / rugosité)

## Résumé

Chaque objet a désormais un **matériau** (`Material` : couleur de base, métal, rugosité) et
l'éclairage suit le modèle **PBR** de Cook-Torrance, le même que glTF, Blender, Unreal ou
Godot. Il remplace l'éclairage Blinn-Phong (Lambert seul pour la partie diffuse, avec une
brillance fixe de 32). Une **vitrine de sphères** dans la démo montre l'effet de la rugosité sur
un métal (or) et sur un isolant (plastique rouge). L'inspecteur règle le matériau de l'entité
sélectionnée.

## Pourquoi

Avec Blinn-Phong, tous les objets avaient le même reflet, et aucun ne pouvait ressembler à du
métal. Le modèle métal / rugosité décrit la matière avec deux nombres qui ont un sens
physique : un matériau réglé dans Blender a le même aspect dans ZENTRIX.

## Détail

### `zentrix-core`

| Élément | Changement |
|---------|------------|
| `src/material.rs` *(nouveau)* | `Material { base_color, metallic, roughness }` (serde, `#[serde(default)]`), `DEFAULT` (blanc, 0, 0,5), `dielectric(couleur, rugosité)`, `metal(couleur, rugosité)` |
| `MeshData::sphere(segments, rings)` | Sphère UV de rayon 0,5, normales exactes, couture dupliquée pour les UV, un seul triangle par quartier aux pôles |
| `Cargo.toml` | `ron` en dépendance de test |

### `zentrix-render`

| Élément | Changement |
|---------|------------|
| `InstanceRaw` | 64 → **96 octets** : `base_color` (location 8) et `material` = (métal, rugosité, 0, 0) (location 9) ; `InstanceRaw::new(matrice, &matériau)` |
| `DrawCommand` | Nouveau champ `material`. Le matériau voyage avec l'instance : il **ne coupe pas les lots** (25 cubes de matériaux différents = toujours un appel de dessin) |
| `shaders/mesh.wgsl` | `fs_main` réécrit : GGX (D), Smith / Schlick-GGX (G), Fresnel-Schlick (F), diffuse de Lambert pondérée par (1 − F)(1 − métal). Matériau transmis au fragment shader en `@interpolate(flat)` |
| `shaders/shadow.wgsl` | Inchangé (lit seulement les locations 0 et 4 à 7) ; commentaire mis à jour |
| `lighting.rs` | Documentation du modèle d'éclairage |
| `lib.rs` | Ré-exporte `Material` |

Conventions choisies dans le shader :

- **Intensité** : une lumière d'intensité 1 rend une surface blanche mate, vue de face,
  exactement blanche. La radiance est donc multipliée par π (qui compense le 1/π de Lambert) :
  les scènes existantes gardent à peu près la même luminosité.
- **Rugosité** bornée à 0,04 au minimum (un reflet infiniment petit scintillerait).
- **Ambiante** : `ambiante × mix(albédo, f0 × 0,5, métal)`. C'est une approximation : sans
  elle, les métaux, qui n'ont pas de couleur diffuse, seraient noirs dans l'ombre.

### `zentrix-scene`

| Élément | Changement |
|---------|------------|
| `MeshRenderer` | Champ `material`, `with_material(...)`. N'implémente plus `Eq` (des `f32`) |
| `extract_draws` | Recopie le matériau dans la `DrawCommand` |
| Sérialisation | `MeshRef` gagne `material`, omis quand il vaut `Material::DEFAULT`. **Les fichiers de scène existants se chargent sans changement**, et la version du format reste 1 |

### `zentrix-assets`

| Élément | Changement |
|---------|------------|
| `ModelPart::material` | Lit `baseColorFactor`, `metallicFactor` et `roughnessFactor` du glTF |
| Couleur de base | N'est plus recopiée dans la couleur des sommets : elle passe par le matériau |

### `zentrix-app`

| Élément | Changement |
|---------|------------|
| `demo.rs` | Vitrine (`spawn_material_showcase`) : 5 sphères en or et 5 en plastique rouge, rugosité 0,1 → 0,9. Matériaux pour le sol (0,9), les caisses (0,85), les cubes colorés (plus lisses vers le fond), le soleil, la planète (métal) et la lune. 41 entités (+ 7 arbres) |
| `main.rs` | Maillage `sphere` (48 × 24), nom `"sphere"` dans l'annuaire |
| `model.rs` | Les morceaux d'un modèle gardent leur matériau glTF |
| `inspector.rs` | Section matériau : couleur de base, curseurs métal et rugosité |

## Vérification

- `cargo build`, `cargo clippy --workspace --all-targets`, `cargo fmt --check` : propres.
- `cargo test --workspace` : tous les tests passent. Nouveaux tests :
  - un matériau incomplet se relit en RON avec les valeurs par défaut ;
  - les triangles de la sphère sont tournés vers l'extérieur ;
  - aller-retour de sauvegarde avec un matériau ;
  - un fichier de scène sans matériau donne le matériau par défaut ;
  - matériau d'un glTF (valeurs par défaut de la norme : métal 1, rugosité 1) et de l'arbre ;
  - la vitrine contient 5 sphères en or de rugosité croissante.

  `mesh.wgsl` est validé par naga.
- Démo lancée sous Vulkan (Intel Iris Xe), avec une capture d'écran : le reflet passe d'un point
  vif (rugosité 0,1) à une tache large et terne (0,9). Les reflets de l'or sont dorés, ceux du
  plastique sont blancs.
- La démo était encore ouverte de votre côté pendant les essais : l'exécutable a été compilé
  dans `target/verif`.

## Limites connues

- **Pas de reflets de l'environnement** : un métal ne reflète que le soleil, et paraît sombre
  ailleurs. Il faudrait une carte d'environnement (*IBL*, « image-based lighting »).
- **Seuls les facteurs glTF sont lus** : les textures métal/rugosité, les normal maps (qui
  demandent des tangentes dans `Vertex`), l'occlusion et l'émission sont ignorées.
- **Pas de *tone mapping*** : les reflets intenses sont simplement écrêtés à 1.
- **Normales et échelle** : une échelle non uniforme fausse toujours légèrement les normales
  (limite déjà connue), ce qui se voit davantage avec des reflets précis.
- **Section matériau de l'inspecteur** : vérifiée à la compilation seulement, pas encore
  essayée à la souris.
