# 2026-10-31 — Suppression : caméra en orbite automatique

## Résumé

La caméra qui tournait automatiquement autour de la scène (cercle de rayon 11, à 6 unités de
haut) est retirée de `ZentrixApp::on_update`.

## Pourquoi

Elle est remplacée par la caméra libre pilotée par l'utilisateur
(voir [entrées et caméra libre](2026-10-31_changement-majeur_entrees-et-camera-libre.md)).
La caméra démarre maintenant en `(0, 4, 11)`, tournée vers le centre de la grille.

## Récupération

Le calcul supprimé tenait en trois lignes :

```rust
let angle = self.time * 0.25;
self.camera.position = Vec3::new(angle.cos() * 11.0, 6.0, angle.sin() * 11.0);
self.camera.target = Vec3::new(0.0, 0.5, 0.0);
```
