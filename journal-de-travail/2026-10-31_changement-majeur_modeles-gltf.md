# 2026-10-31 — Changement majeur : modèles 3D glTF

## Résumé

Le moteur charge des modèles 3D au format glTF 2.0 (`.gltf` / `.glb`) : géométrie, hiérarchie
de nœuds, couleur de base et texture des matériaux. La démo place 7 arbres (un modèle généré par
un script du dépôt) autour de la grille de cubes. Étape K de la
[feuille de route](../documentation/guide-debutant/07-feuille-de-route.md).

## Pourquoi

Jusqu'ici, toute la géométrie était écrite dans le code (cube, plan). glTF, exporté par Blender
et la plupart des outils, ouvre la porte à n'importe quel contenu 3D.

## Détail

### `zentrix-assets`

| Élément | Rôle |
|---------|------|
| `src/model.rs` *(nouveau)* | `Model { parts, images }`, `ModelPart { name, mesh, transform, texture }`, `load_gltf(path)`, `decode_gltf(octets)` |
| `AssetError::Gltf` | Nouvelle variante (fichier, JSON, buffer ou image invalide) |
| `Cargo.toml` | Dépendance `gltf` 1.4 (fonctionnalités `import`, `utils`, `names` ; même `image` 0.25 que le reste) |

Comportement du chargeur :

- parcours de la scène par défaut (sinon la première) ; la hiérarchie des nœuds est
  **aplatie** (transformation de chaque primitive relative à la racine du modèle) ;
- valeurs par défaut quand un attribut manque : UV (0, 0), couleur blanche, indices 0, 1, 2…,
  normales calculées (`compute_normals`) ;
- `baseColorFactor` multiplié dans la couleur des sommets ; `baseColorTexture` → indice d'image ;
- images converties en RGBA 8 bits (R, RG, RGB, RGBA ; formats 16 bits / flottants remplacés
  par du blanc avec un avertissement) ;
- primitives non triangulaires ignorées avec un avertissement.

3 tests : glTF minimal intégré (base64) avec valeurs par défaut, JSON invalide, chargement de
`arbre.glb`.

### `zentrix-core`

- `Transform::from_matrix(Mat4)` (inverse de `matrix()`) ; 1 test.
- `Vertex` et `MeshData` déplacés ici (voir l'[entrée dédiée](2026-10-31_modification_geometrie-dans-le-noyau.md)).

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `src/model.rs` *(nouveau)* | `upload(renderer, gpu, &Model) -> ModelHandles` (une fois par fichier) et `spawn(world, &ModelHandles, nom, transform) -> Entity` (une fois par exemplaire : racine + un enfant `Parent` par morceau) ; 1 test |
| `src/demo.rs` | `DemoAssets::tree`, `spawn_forest` : 7 arbres (positions, tailles et orientations variées) |
| `src/main.rs` | `load_tree_model()` : si `arbre.glb` manque, avertissement et démo sans arbres |

### Ressources et outils

- `tools/generer_arbre.py` *(nouveau)* : génère `assets/models/arbre.glb` avec la seule
  bibliothèque standard Python (`python -I tools/generer_arbre.py`). Le modèle exerce exprès
  plusieurs cas : nœuds imbriqués, deux matériaux, texture PNG intégrée, un maillage complet
  (normales + UV + indices) et un maillage réduit aux positions.
- `assets/models/arbre.glb` *(nouveau, 6 Ko)*.

## Décisions techniques

- **Modèle généré plutôt que téléchargé** : pas de question de licence, fichier minuscule,
  reproductible, et il teste précisément les cas voulus.
- **Upload séparé du spawn** : 7 arbres partagent 2 maillages et 1 texture sur le GPU.
- **Hiérarchie aplatie au chargement** puis recréée avec `Parent` au spawn : chaque morceau
  reste une entité manipulable (on pourrait retrouver « Arbre 3 / Feuillage » par son nom).

## Vérification

- `cargo build` et `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 64 tests réussis.
- Lancement sous Vulkan : « Modèle chargé : … arbre.glb (2 morceaux, 1 images) »,
  « Scène créée : 52 entités », troncs texturés, feuillage éclairé, 60 FPS.

## Limites connues

Pas d'animations, de squelettes, de transparence ni de propriétés PBR (métal, rugosité, normal
maps) ; les modes de répétition des samplers glTF sont ignorés (toujours `Repeat`).

## Impact / suite

- Prochaine étape proposée : **L — ombres portées (shadow mapping)**.
