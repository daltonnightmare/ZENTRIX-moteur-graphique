# 2026-11-01 — Amélioration : propagation des transforms et liste des entités

## Résumé

Le nouveau profileur a tout de suite montré deux points chauds avec le champ de stress (4 170
entités, build debug). Ce n'était pas le rendu :

| Étape | Avant | Après |
|-------|-------|-------|
| transforms (`Scene::update_transforms`) | 6,86 ms | 0,69 ms |
| interface (inspecteur) | 5,53 ms | 1,1 ms |

## Détail

### `zentrix-scene/src/hierarchy.rs` — `propagate`

- **Avant** : toutes les entités passaient par deux `HashMap` (matrices locales, matrices
  globales), puis par un `world.get` chacune.
- **Après** :
  - les entités sans `GlobalTransform` en reçoivent un d'abord. En régime normal, il n'y en a
    aucune ;
  - les **racines** (sans `Parent`), soit presque toutes, sont calculées par une requête
    directe (`query_mut` + `Without<_, &Parent>`) : global = local, sans aucune table ;
  - seuls les **enfants** passent par les tables et la récursion mémorisée. Le global d'un
    parent racine est lu dans le monde, à condition qu'il ait encore un `Transform`.
- Nouveau test : un parent qui perd son `Transform` ne transmet pas son ancien
  `GlobalTransform` à l'enfant.

### `zentrix-app/src/inspector.rs` — liste des entités

- `ScrollArea::show_rows` : seules les lignes visibles sont mises en page.
- La liste (un texte par entité, filtrée et triée) est **gardée d'une image à l'autre**. Elle
  est refaite seulement :
  - si le nombre d'entités change ;
  - si le filtre change ;
  - ou au bout de 30 images, pour voir les entités remplacées à nombre égal (une balle qui en
    chasse une autre).

## Vérification

- Tests de la hiérarchie : tous réussis (enfant, orbite, orphelin, cycle, branche supprimée,
  parent sans transform).
- Mesures dans la démo avant et après (tableau ci-dessus). En release, transforms coûte 0,07 ms.
