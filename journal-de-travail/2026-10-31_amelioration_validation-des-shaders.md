# 2026-10-31 — Amélioration : validation des shaders WGSL dans les tests

## Résumé

Les shaders `mesh.wgsl` et `debug.wgsl` sont compilés et validés par `cargo test`, sans carte
graphique.

## Pourquoi

Jusqu'ici, une erreur dans un `.wgsl` n'apparaissait qu'au lancement de l'application (panic
à la création du pipeline). Avec l'éclairage, le shader des maillages devient plus long : il
fallait un filet de sécurité.

## Détail

- `naga` (le compilateur de shaders utilisé par wgpu, même version 30) ajouté au catalogue du
  workspace avec la fonctionnalité `wgsl-in`, et en `[dev-dependencies]` de `zentrix-render`
  (il n'alourdit pas le programme final).
- `pipeline.rs` : 2 tests qui analysent puis valident chaque shader ; en cas d'erreur, le
  message de naga (avec la ligne fautive) est affiché.
- `renderer.rs` : test de la taille de `FrameUniform` (128 octets, doit correspondre aux
  shaders).

## Au passage

`MeshData::compute_normals` utilise `as_chunks::<3>()` (suggestion de clippy) au lieu de
`chunks_exact(3)` : chaque triangle arrive directement sous forme de tableau `[u32; 3]`.
