# 2026-10-31 — Changement majeur : textures de matériau complètes et environnement (IBL)

## Résumé

Cette étape termine la piste PBR (suite des [normal maps](2026-10-31_changement-majeur_normal-maps-et-tone-mapping.md)).

- **Textures de matériau.** Un matériau accepte maintenant les **5 textures** de glTF :
  - couleur ;
  - relief (normal map) ;
  - métal/rugosité ;
  - **occlusion** ;
  - **émission**.

  Le matériau gagne aussi une couleur d'émission et une intensité d'occlusion.
- **Environnement (IBL, *Image-Based Lighting*).** Un ciel procédural est précalculé au
  démarrage dans deux *cube maps*. Il éclaire la scène (lumière diffuse) et se reflète dans les
  objets, avec un reflet plus ou moins flou selon la rugosité. Le ciel est aussi dessiné en
  arrière-plan. La touche `F7` l'active ou le désactive.
- **Démo.**
  - Le soleil est **émissif**.
  - Deux **caisses « sci-fi »** utilisent les 5 textures : panneaux de métal poli et de
    peinture, rainures creusées et assombries, néons cyan.
  - Les sphères en or reflètent le ciel.

## Pourquoi

Sans environnement, un métal ne reflète que le soleil et paraît noir partout ailleurs : c'était
la principale limite de l'étape PBR. Quant aux textures, ce sont elles qui donnent leur aspect
aux modèles glTF réels : sans elles, un modèle importé de Blender perd ses détails de
matière.

## Détail

### Organisation des ressources du shader

wgpu ne garantit que **4 groupes de ressources** (*bind groups*) par pipeline, et ils étaient
tous utilisés. Ils sont donc réorganisés :

| Groupe | Avant | Après |
|--------|-------|-------|
| 0 | caméra, lumière | inchangé (le `FrameUniform` grandit) |
| 1 | texture de couleur | **les 5 textures du matériau + sampler** |
| 2 | shadow map | inchangé |
| 3 | normal map | **l'environnement** (2 cube maps + sampler) |

Un bind group de matériau est créé **à la demande**, pour chaque combinaison de textures
rencontrée, puis gardé en cache. Voir l'[entrée dédiée](2026-10-31_modification_material-textures-et-bind-groups.md).

### Environnement : `zentrix-render/src/environment.rs` *(nouveau)*

| Élément | Rôle |
|---------|------|
| `Sky { zenith, horizon, ground }` | Ciel en dégradé, sans soleil. Le soleil de la scène tourne, et son reflet est déjà donné par l'éclairage direct |
| `face_direction` | Convention des faces d'une cube map (Vulkan / D3D / GL) ; testée par aller-retour |
| `bake_specular` | Cube map des reflets de 64² avec 7 niveaux de mipmap : niveau k = rugosité k/6. Préfiltrage GGX par échantillonnage d'importance, avec 48 échantillons de Hammersley |
| `bake_irradiance` | Cube map de lumière diffuse de 16², calculée avec 256 échantillons pondérés par le cosinus |
| `Environment` | Envoi au GPU, couche par couche, au format `Rgba8UnormSrgb` (valeurs ≤ 1, pas besoin de HDR) |
| `Renderer::set_sky` | Remplace le ciel ; le précalcul est relancé |

Le précalcul dure environ **160 ms en debug** au démarrage.

### Shaders

- **`mesh.wgsl`**
  - Les 5 textures sont lues **avant** tout `if`, à cause de la règle d'uniformité.
  - Métal et rugosité valent facteur × texture, selon la convention glTF : canal B pour le
    métal, canal G pour la rugosité.
  - Occlusion : `mix(1, texture, intensité)`. Elle n'atténue **que** la lumière ambiante.
  - L'émission est ajoutée à la fin, avant le tone mapping.
  - L'IBL suit l'approximation *split sum* : `irradiance × albédo × kd` +
    `préfiltré(reflet, rugosité × lod_max) × env_brdf`. `env_brdf` est l'approximation
    analytique de Karis, ce qui évite une texture de table (*BRDF LUT*).
  - L'ancienne lumière ambiante uniforme sert de secours quand l'environnement est désactivé.
- **`sky.wgsl`** *(nouveau)*
  - Un triangle couvre tout l'écran ; les sommets sont calculés à partir de `vertex_index`.
  - La direction de vue est reconstruite avec `inv_view_proj`.
  - Il est dessiné après les objets, à la profondeur 1, avec le test `LessEqual` et sans
    écriture de profondeur.
  - Il applique le même tone mapping que les objets.
- **`FrameUniform`** : 208 → **288 octets**. Deux champs s'ajoutent : `environment` (activé,
  intensité, lod max) et `inv_view_proj`. Les 4 shaders le déclarent.

### `zentrix-core`

`Material` gagne deux champs :
- `emissive: Color` (noir par défaut) ;
- `occlusion_strength: f32` (1 par défaut).

Ils sont transmis dans `InstanceRaw`, à la location 11.

### `zentrix-assets` (glTF)

- `ModelPart::textures: PartTextures` (5 indices d'image) remplace `texture` et `normal_map`.
- `convert_material` lit aussi `metallicRoughnessTexture`, `occlusionTexture` (avec sa
  `strength`), `emissiveTexture` et `emissiveFactor`.
- `PartTextures::is_color_image` décide du format : seules la couleur et l'émission sont en
  sRGB.
- Nouveau test : un glTF dont le matériau utilise toutes les textures. Les images sont un PNG
  rangé dans le buffer binaire ; le crate `gltf` refuse les images en `data:` dans un import en
  mémoire.

### `zentrix-scene`

- `MeshRenderer::textures: MaterialTextures` remplace `texture` et `normal_map`.
- Nouvelle méthode `with_textures`. `with_texture` et `with_normal_map` existent toujours.
- Sérialisation : trois champs facultatifs s'ajoutent, `metallic_roughness`, `occlusion` et
  `emissive`. Les champs `texture` et `normal_map` gardent leur nom, donc les anciens fichiers
  sont compatibles.
- Deux fonctions utilitaires sont factorisées : `texture_name` et `texture_handle`.

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `demo.rs` | `panel_images` génère les 5 images d'une plaque « sci-fi » ; `spawn_panel_crates` place 2 caisses qui tournent ; soleil émissif ; `panel_texture_names` |
| `main.rs` | Envoi des 5 textures de la plaque (couleur et émission en sRGB, les autres en données) ; touche `F7` |
| `model.rs` | Chaque image d'un glTF est envoyée en couleur ou en données selon son usage |
| `inspector.rs` | Cases et curseur « Environnement (F7) » et « ciel » ; dans la section matériau, curseur d'occlusion et valeurs d'émission (r, v, b), qui peuvent dépasser 1 |

## Vérification

- `cargo build`, `cargo clippy --workspace --all-targets`, `cargo fmt --check` : propres.
- `cargo test --workspace` : **91 tests** réussis. Nouveaux tests :
  - faces de cube map (aller-retour) ;
  - un ciel uniforme reste uniforme après préfiltrage et calcul de l'irradiance ;
  - la lumière diffuse vient du côté de la normale ;
  - la chaîne de mipmaps des reflets a 7 niveaux ;
  - matériau glTF complet ;
  - aller-retour de sauvegarde avec 3 textures ;
  - plaque sci-fi (métal, peinture, rainure, néon) ;
  - les 2 caisses utilisent les 5 textures.

  Naga valide `mesh.wgsl` et `sky.wgsl`.
- Démo lancée sous Vulkan (Intel Iris Xe), avec captures d'écran :
  - le ciel est en dégradé ;
  - les sphères en or reflètent le ciel et l'horizon, et ne sont plus sombres hors du reflet du
    soleil ;
  - les néons des caisses brillent et le soleil est émissif ;
  - avec `F7` désactivé, on retrouve le fond uni et la lumière ambiante uniforme.

  La démo tourne à 60 à 69 FPS (vsync).
- Lors du second essai, la caméra était tournée vers le ciel dès la première capture. C'est
  probablement une interaction avec la fenêtre pendant le test : aucun chargement de scène n'a
  lieu au démarrage, et la caméra ne tourne que souris capturée.

## Limites connues

- **Le ciel est procédural** (un simple dégradé). Charger une image HDR (`.hdr`, `.exr`)
  demanderait un format flottant (`Rgba16Float`) et une conversion équirectangulaire → cube.
- **Préfiltrage simplifié** (vue = normale) : les reflets vus très en biais sont un peu trop
  ronds. `env_brdf` est une approximation analytique.
- **Ombres et environnement** : l'environnement ne connaît pas les ombres. Un objet sous un autre
  reçoit toute la lumière du ciel ; seule la texture d'occlusion l'atténue.
- **Émission** : elle n'éclaire pas les autres objets, et il n'y a pas de halo (*bloom*).
- **Non essayés à la main** : la bascule `F7` depuis l'inspecteur, les curseurs « ciel »,
  « occlusion » et les valeurs d'émission. Seule la touche `F7` a été essayée.
