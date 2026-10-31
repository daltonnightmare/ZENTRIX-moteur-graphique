# 2026-10-31 — Suppression : état « manuel » de la démo

## Résumé

Avec le passage à l'ECS (voir [scène et ECS](2026-10-31_changement-majeur_scene-et-ecs.md)),
l'état que `ZentrixApp` gérait à la main est retiré.

## Détail

| Supprimé de `zentrix-app` | Remplacé par |
|---------------------------|--------------|
| Champs `time`, `camera`, `fly_camera` de `ZentrixApp` | Entité caméra (`Transform` + `Camera3d` + `FlyCamera`) dans la `Scene` |
| Champs `cube`, `white_cube`, `ground`, `crate_texture`, `ground_texture` de `GraphicsState` | `DemoAssets`, utilisé une seule fois pour créer les entités |
| Double boucle qui construisait les `DrawCommand` à chaque image | `Scene::extract_draws` |
| Constantes `GRID_SIZE`, `GRID_SPACING` dans `main.rs` | Déplacées dans `demo.rs` (avec `GROUND_Y`) |
| `FlyCamera::position` et `FlyCamera::apply_to(&mut Camera)` | Le `Transform` de l'entité et `Scene::camera()` |

## Récupération

Version précédente : `git show e3e56fe:crates/zentrix-app/src/main.rs` (étape textures) ;
l'étape G (debug) n'était pas encore commitée au moment de ce changement.
