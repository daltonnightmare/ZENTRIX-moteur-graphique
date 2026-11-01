# 2026-10-31 — Changement majeur : normal maps et tone mapping

## Résumé

Suite des [matériaux PBR](2026-10-31_changement-majeur_materiaux-pbr.md).

- **Normal maps** : une texture peut ajouter du relief à une surface sans ajouter de
  triangles. Les sommets ont maintenant une tangente.
- **Tone mapping** : la méthode Khronos PBR Neutral ramène les couleurs trop lumineuses dans
  ce que l'écran peut afficher, au lieu de les écrêter. Elle est active par défaut et se
  bascule avec `F6`.
- **Démo** : le sol est en dalles biseautées, et les sphères en plastique sont alvéolées comme
  des balles de golf.

## Pourquoi

- **Normal maps** : un sol ou une balle de golf vraiment en relief demanderaient des milliers
  de triangles. Une normal map ne change que la normale utilisée pour l'éclairage, pixel par
  pixel : le relief paraît vrai pour un coût minime. Presque tous les modèles glTF en
  fournissent une.
- **Tone mapping** : avec le PBR, les reflets dépassent souvent 1. Sans tone mapping, un
  reflet doré devient un aplat jaune-blanc sans nuance.

## Détail

### `zentrix-core`

| Élément | Changement |
|---------|------------|
| `Vertex::tangent: [f32; 4]` | xyz = direction des `u` croissants, w = ±1 (texture en miroir). `[0; 4]` = inconnue : le shader ignore alors la normal map. Voir l'[entrée dédiée](2026-10-31_modification_vertex-tangente-et-params.md) |
| `MeshData::compute_tangents()` | Pour chaque triangle, on cherche les directions 3D correspondant à « `u` augmente » et « `v` diminue », à partir des différences de positions et d'UV. On accumule le résultat par sommet, puis on rend la tangente perpendiculaire à la normale (Gram-Schmidt) et on en déduit le signe w. Appelée par `plane`, `cube` et `sphere` |
| `Image::normal_map_from_heights(w, h, force, hauteur)` | Calcule une normal map à partir d'une carte de hauteurs, en différences centrées. L'image est supposée répétée. Convention OpenGL (celle de glTF et de Blender) : le vert pointe vers le haut de l'image |
| `Material::normal_scale` | Intensité du relief (1 par défaut). Lu dans `normalTexture.scale` pour les glTF |

### `zentrix-render`

| Élément | Changement |
|---------|------------|
| `Renderer::add_normal_map` | Envoie la texture au format `Rgba8Unorm`, **sans conversion sRGB** : ce sont des directions, pas des couleurs. Interne : `TextureKind { Color, Data }` |
| Normal map plate par défaut | La texture n°1, un pixel (128, 128, 255), est utilisée quand `normal_map` vaut `None` |
| Groupe de ressources n°3 | La normal map. Il a la même forme que le n°1, donc une seule mise en page (*layout*) sert aux deux |
| `DrawCommand::normal_map` | Nouveau champ. La clé de lot devient (texture, normal map, maillage) |
| `Renderer::tone_mapping` | `true` par défaut ; transmis dans `params.z` |
| `mesh.wgsl` | `surface_normal` construit la matrice TBN (tangente, bitangente, normale) et y lit la normal map ; `tone_map_pbr_neutral` applique le tone mapping. La normal map est lue **avant** tout `if`, à cause de la règle d'uniformité de WGSL |

### `zentrix-scene`

| Élément | Changement |
|---------|------------|
| `MeshRenderer` | Champ `normal_map` et méthode `with_normal_map` |
| `extract_draws` | Recopie `normal_map` dans la `DrawCommand` |
| Sérialisation | `normal_map` est écrite par nom et omise si elle est absente. Les anciens fichiers restent compatibles |

### `zentrix-assets` (glTF)

| Élément | Changement |
|---------|------------|
| `ModelPart::normal_map` | Indice de l'image `normalTexture` |
| Tangentes | Lues dans l'attribut `TANGENT` du fichier ; calculées par `compute_tangents` s'il est absent |

### `zentrix-app`

| Élément | Changement |
|---------|------------|
| `model.rs` | Les images utilisées comme normal map sont envoyées avec `add_normal_map` |
| `demo.rs` | `tiles_normal_map` (dalles biseautées, alignées sur le damier du sol) et `dimples_normal_map` (alvéoles, deux fois plus de colonnes que de lignes pour qu'elles restent rondes sur une sphère). Le sol et les 5 sphères en plastique utilisent ces normal maps |
| `main.rs` | Normal maps générées au démarrage (256² pour le sol, 512 × 256 pour les sphères), noms `dalles_relief` et `alveoles_relief`. Touche `F6` pour le tone mapping |
| `inspector.rs` | Case « Tone mapping (F6) », curseur « relief » dans la section matériau |

## Vérification

- `cargo build`, `cargo clippy --workspace --all-targets`, `cargo fmt --check` : propres.
- `cargo test --workspace` : 84 tests réussis. Nouveaux tests :
  - tangentes du cube : unitaires, perpendiculaires aux normales, valeur exacte sur la face avant ;
  - texture en miroir → w = -1 ;
  - normal map d'une surface plate et de rampes horizontale et verticale (convention du vert) ;
  - normal maps de la démo (biseaux, alvéoles) ;
  - le sol et les 5 sphères en plastique ont une normal map ;
  - aller-retour de sauvegarde avec une normal map.

  Le shader est validé par naga, y compris la règle d'uniformité.
- Démo lancée sous Vulkan (Intel Iris Xe), avec une capture d'écran :
  - les alvéoles sont nettement visibles sur les sphères rouges, et le reflet suit les creux ;
  - les biseaux des dalles du sol accrochent la lumière ;
  - les reflets de l'or ne sont plus écrêtés.

## Limites connues

- **Modèles glTF** : la lecture des normal maps et des tangentes n'a été testée sur aucun
  fichier réel qui en contient ; l'arbre de la démo n'en a pas.
- **Bascule `F6`** et **curseur « relief »** : vérifiés à la compilation seulement, pas encore
  essayés à la main.
- **Mipmaps des normal maps** : on fait la moyenne des valeurs encodées. Au loin, le relief
  s'aplatit un peu, ce qui reste acceptable.
- **Toujours absents** : les textures métal/rugosité, l'occlusion, l'émission et la carte
  d'environnement (IBL).
