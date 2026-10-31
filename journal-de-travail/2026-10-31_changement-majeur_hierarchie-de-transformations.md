# 2026-10-31 — Changement majeur : hiérarchie de transformations

## Résumé

Une entité peut être attachée à une autre avec le composant `Parent` : son `Transform` devient
relatif à son parent, et le moteur calcule sa position dans le monde (`GlobalTransform`). La
démo affiche un « système solaire » (soleil → planète → lune) au-dessus des cubes. Étape I de
la [feuille de route](../documentation/guide-debutant/07-feuille-de-route.md).

## Pourquoi

Sans hiérarchie, tout objet composé (une voiture et ses roues, un personnage et son arme, une
caméra embarquée) oblige à recalculer les positions à la main. C'est une brique de base de
toute scène de jeu.

## Détail

### `zentrix-scene`

| Élément | Rôle |
|---------|------|
| `src/hierarchy.rs` *(nouveau)* | `Parent(Entity)`, `GlobalTransform(Mat4)` (+ `translation()`), propagation avec mémorisation, recherche des descendants |
| `Scene::update_transforms()` | Calcule le `GlobalTransform` de toute entité qui a un `Transform` (l'ajoute s'il manque) |
| `Scene::despawn_recursive(entity)` | Supprime une entité et toute sa descendance ; renvoie le nombre supprimé |
| `Cargo.toml` | Ajout de `log` (avertissement en cas de cycle) |

6 nouveaux tests : enfant qui suit son parent, orbite induite par la rotation du parent
(soleil → planète → lune), orphelin qui redevient racine, cycle sans blocage, suppression
récursive, caméra embarquée.

### `zentrix-app`

- `demo.rs` : `spawn_solar_system` (soleil jaune en (0, 3.5, 0), planète bleue à 3 unités,
  lune grise) ; `DemoAssets` gagne trois maillages unis ; 1 test de positions monde.
- `main.rs` : appel de `scene.update_transforms()` après les systèmes, avant l'extraction.

## Décisions techniques

- **Seulement `Parent`, pas de `Children`** : une seule source de vérité, rien à
  synchroniser. Le prix est la recherche des descendants (parcours des liens), utilisée
  seulement par `despawn_recursive`.
- **Propagation mémorisée** : chaque entité est calculée une fois, quel que soit le nombre de
  ses descendants. Coût : deux `HashMap` réutilisées d'une image à l'autre.
- **Profondeur maximale de 64** : un cycle (A parent de B, B parent de A) produit un
  avertissement dans les logs au lieu d'une boucle infinie.
- **Parent disparu = racine** : l'enfant n'est pas supprimé automatiquement ; utiliser
  `despawn_recursive` pour supprimer une branche entière.
- **L'échelle du parent s'applique aux enfants** (positions et tailles), comme dans tous les
  moteurs : commenté dans la démo, où la lune est à « 2 unités » de la planète mais à 1 unité
  dans le monde.

## Vérification

- `cargo build` et `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 52 tests réussis.
- Lancement sous Vulkan : « Scène créée : 30 entités », planète et lune en orbite, liens
  parent → enfant (cyan) et boîtes visibles avec F2, 60 FPS.

## Impact / suite

- Prochaine étape : **J — éclairage simple** (normales, lumière directionnelle, Lambert).
