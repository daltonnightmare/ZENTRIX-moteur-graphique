# 2026-10-31 — Changement majeur : scène et ECS (`zentrix-scene`)

## Résumé

Tout ce qui existe dans le monde du jeu (caméra, sol, cubes) est maintenant une **entité** d'un
ECS (*Entity Component System*, bibliothèque `hecs`). Le jeu définit ses propres composants et
systèmes ; la scène produit les listes simples que le rendu attend. Étape H de la
[feuille de route](../documentation/guide-debutant/07-feuille-de-route.md) : début de la Phase 3.

## Pourquoi

Jusqu'ici, la démo construisait ses `DrawCommand` à la main dans une double boucle. Un ECS donne
une structure qui passe à l'échelle : ajouter un comportement = ajouter un composant et un
système, sans hiérarchie de classes. C'est la base de tout le gameplay à venir.

## Détail

### `zentrix-scene` *(nouvelle crate)*

| Élément | Rôle |
|---------|------|
| `Scene { pub world: hecs::World }` | Le monde ; l'API `hecs` (`spawn`, `query`, `query_mut`...) est utilisée directement |
| `Scene::extract_draws(&mut Vec<DrawCommand>)` | Une commande par entité ayant `Transform` + `MeshRenderer` |
| `Scene::camera() -> Option<Camera>` | Première entité `Transform` + `Camera3d` ; regarde vers le -Z de son `Transform` |
| `MeshRenderer { mesh, texture }` | Composant « visible » |
| `Camera3d { fov_y, near, far }` | Réglages de l'objectif (la position vient du `Transform`) |
| `Name` | Nom lisible d'une entité |
| Ré-exports | `hecs`, `Entity` |

3 tests + 1 doc-test (un composant et un système définis par le jeu).

### `zentrix-render`

`MeshHandle::from_index` et `TextureHandle::from_index` (`#[doc(hidden)]`) : fabriquer des
handles sans GPU, pour les tests et les outils.

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `src/demo.rs` *(nouveau)* | `DemoAssets` (handles chargés), composant de jeu `Spin`, `spawn()` (caméra + sol + 25 cubes = 27 entités), systèmes `spin_system`, `fly_camera_system`, `bounds_system` ; 2 tests |
| `src/fly_camera.rs` | `FlyCamera` devient un **composant** : plus de champ `position`, `update(&mut Transform, …)` écrit position et rotation, `looking_at()` renvoie `(FlyCamera, Transform)`, nouvelle méthode `rotation()` ; 1 test de cohérence ajouté |
| `src/main.rs` | L'application possède une `Scene` ; `on_update` = systèmes puis extraction ; la caméra de rendu vient de `scene.camera()` |

## Décisions techniques

- **`hecs`** plutôt que `bevy_ecs` : minimal, sans ordonnanceur ni macros, facile à comprendre.
  Le choix reste réversible : le rendu ne voit jamais l'ECS.
- **`zentrix-scene` dépend de `zentrix-render`** (pour les types `MeshHandle`, `DrawCommand`,
  `Camera`). C'est un écart avec le schéma initial de la feuille de route, mis à jour ; la règle
  essentielle tient : le rendu ne dépend jamais de la scène.
- **La caméra est une entité** : position et orientation dans son `Transform`, objectif dans
  `Camera3d`. On pourra l'attacher à un objet (étape I : hiérarchie).
- **Rotation incrémentale** dans `spin_system` (`step * rotation`, puis `normalize()` contre la
  dérive des arrondis), au lieu d'un angle calculé depuis le temps total.

## Vérification

- `cargo build` et `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 45 tests réussis.
- Lancement sous Vulkan : « Scène créée : 27 entités », rendu identique à l'étape précédente
  (caisses, cubes, sol, grille, boîtes avec F2), 60 FPS.
- Test de contrôle : caméra immobile au repos et après un simple clic.

## Impact / suite

- Prochaine étape : **I — hiérarchie de transformations** (`Parent`, `GlobalTransform`).
