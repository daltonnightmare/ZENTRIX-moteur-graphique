# 2026-10-31 — Amélioration : buffer GPU extensible commun, FPS dans le titre

## Résumé

Factorisation du code des buffers réécrits à chaque image, et affichage des images par seconde
dans le titre de la fenêtre.

## Détail

### `GrowableBuffer` (`zentrix-render/src/buffer.rs`, nouveau)

Le buffer des instances gérait lui-même sa capacité (`instance_capacity`,
`create_instance_buffer`). Les lignes de debug avaient besoin du même mécanisme : il est
maintenant regroupé dans `GrowableBuffer::write(device, queue, octets)`, qui agrandit le buffer
à la puissance de deux supérieure quand les données ne tiennent plus. Utilisé par le buffer des
instances et par celui des lignes.

Supprimé de `renderer.rs` : le champ `instance_capacity` et la fonction
`create_instance_buffer`.

### FPS dans le titre (`zentrix-app`)

`FpsCounter` compte les images sur 0,5 s et met à jour le titre :
« ZENTRIX — *GPU* (*API*) — 60 FPS — clic : piloter, Échap : libérer ». 1 test.

## Modification d'API associée

`Renderer::render` prend un quatrième paramètre `debug: &DebugLines` (passer
`&DebugLines::default()` pour ne rien dessiner).
