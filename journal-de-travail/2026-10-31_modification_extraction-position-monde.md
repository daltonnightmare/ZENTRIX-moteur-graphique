# 2026-10-31 — Modification : extraction en position monde, boîtes de debug généralisées

## Résumé

Conséquences de la [hiérarchie de transformations](2026-10-31_changement-majeur_hierarchie-de-transformations.md)
sur le code existant.

## Détail

| Élément | Avant | Après |
|---------|-------|-------|
| `Scene::extract_draws` | utilisait `Transform::matrix()` | utilise le `GlobalTransform` s'il existe, sinon le `Transform` |
| `Scene::camera` | position et direction tirées du `Transform` | tirées de la matrice monde (point d'origine, -Z et +Y transformés) ; fonctionne pour une caméra enfant d'un objet |
| `Scene` | un seul champ public `world` | + un champ privé (cache de propagation) : on la crée avec `Scene::default()` |
| `bounds_system` (démo, F2) | boîtes des entités ayant un `Spin`, à partir du `Transform` | boîtes de **toutes** les entités visibles (sauf le sol), à partir du `GlobalTransform`, + un trait cyan entre chaque enfant et son parent |

## Point d'attention

Il faut appeler `scene.update_transforms()` à chaque image avant l'extraction : sinon
`extract_draws` et `camera` utilisent les `GlobalTransform` de l'image précédente.
