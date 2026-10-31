# 2026-10-31 — Suppression : triangle codé en dur et fond animé

## Résumé

Éléments de la démo v1 retirés ou renommés lors du passage au rendu 3D
(voir [changement majeur du jour](2026-10-31_changement-majeur_rendu-3d-maillages-camera-profondeur.md)).

## Détail

| Supprimé | Remplacé par |
|----------|--------------|
| `const TRIANGLE` dans `zentrix-render/src/renderer.rs` | `MeshData::triangle()` dans `mesh.rs` |
| Champs `vertex_buffer` / `vertex_count` du `Renderer` | `Vec<Mesh>` + `MeshHandle` |
| `create_triangle_pipeline` | `create_mesh_pipeline` |
| `shaders/triangle.wgsl` | `shaders/mesh.wgsl` (renommé avec `git mv`) |
| Fond animé (interpolation de `clear_color` dans `ZentrixApp::on_update`) | Couleur fixe `Color::ZENTRIX_NIGHT` |

## Pourquoi

Le triangle en dur et le fond animé servaient à valider la Phase 1 ; la démo montre désormais
la scène 3D. La touche Espace met toujours l'animation en pause (rotation des cubes et caméra).

## Récupération

`git show a80ea0b:crates/zentrix-render/src/renderer.rs` et
`git show a80ea0b:crates/zentrix-render/src/shaders/triangle.wgsl`.
