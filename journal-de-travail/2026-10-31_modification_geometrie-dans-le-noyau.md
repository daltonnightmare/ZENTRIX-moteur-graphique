# 2026-10-31 — Modification : `Vertex` et `MeshData` déplacés dans `zentrix-core`

## Résumé

Les données de géométrie (`Vertex`, `MeshData` et ses méthodes `cube`, `plane`, `triangle`,
`compute_normals`, `with_color`, `with_uv_scale`) quittent `zentrix-render` pour `zentrix-core`.

## Pourquoi

Le chargeur glTF (`zentrix-assets`) doit produire des maillages, mais la règle du projet veut
que `zentrix-assets` ne dépende que du noyau. Même raisonnement que pour `Image` : les données
vont dans le noyau, ce qui touche au GPU reste dans le rendu.

## Détail

| Élément | Avant | Après |
|---------|-------|-------|
| `Vertex`, `MeshData` | `zentrix-render/src/vertex.rs`, `mesh.rs` | `zentrix-core/src/mesh.rs` (avec leurs 5 tests) |
| Description GPU des sommets | `Vertex::layout()` | fonction interne `vertex_layout()` de `zentrix-render` (on ne peut pas ajouter de méthode à un type d'une autre crate) |
| `zentrix-render/src/mesh.rs` | données + GPU | seulement `MeshHandle` et `Mesh` (buffers) |
| `zentrix-core/Cargo.toml` | `glam` | `glam` + `bytemuck` (pour que `Vertex` reste `Pod`) |

## Compatibilité

`zentrix-render` ré-exporte `MeshData` et `Vertex` : `zentrix_render::MeshData` fonctionne
toujours, aucun appel n'a dû changer dans l'application. Seul `Vertex::layout()` (qui était
public) a disparu de l'API.
