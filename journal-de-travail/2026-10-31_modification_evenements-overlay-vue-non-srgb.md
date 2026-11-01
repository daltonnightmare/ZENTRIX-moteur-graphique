# 2026-10-31 — Modification : événements bruts, overlay de rendu, vue non-sRGB

## Résumé

Trois ajouts aux crates du moteur, nécessaires à l'[interface egui](2026-10-31_changement-majeur_interface-egui-inspecteur.md)
mais indépendants d'elle (aucune de ces crates ne connaît egui).

## Détail

### `zentrix-platform`

| Élément | Changement |
|---------|------------|
| `Application::on_window_event(&WindowEvent) -> bool` | Nouvelle méthode (par défaut : `false`), appelée pour chaque événement brut avant tout le reste. `true` = consommé : l'**appui** (touche, bouton, molette) n'est pas transmis à `Input` ni à `on_key` ; les **relâchements** le sont toujours (pas de touche bloquée) |
| Ré-exports | `WindowEvent`, et la crate `winit` elle-même (même version pour egui-winit) |

### `zentrix-render`

| Élément | Changement |
|---------|------------|
| Trait `RenderOverlay` | `prepare(device, queue, encoder, taille) -> Vec<CommandBuffer>` puis `draw(&mut RenderPass<'static>)` |
| `Renderer::render_with_overlay(..., Option<&mut dyn RenderOverlay>)` | Passe « Overlay » après la passe principale (`LoadOp::Load`, sans depth buffer) ; `render` l'appelle avec `None` |
| Étapes numérotées de `render` | 1 à 9 (l'overlay est l'étape 7) |

### `zentrix-rhi`

| Élément | Changement |
|---------|------------|
| `SurfaceConfiguration::view_formats` | Autorise le format non-sRGB de la surface |
| `Frame::overlay_view` | La même image, vue au format non-sRGB (`Bgra8Unorm` au lieu de `Bgra8UnormSrgb`) |
| `GpuContext::overlay_format()` | Ce format |

### `zentrix-core`

`math` ré-exporte aussi `EulerRot` (rotations affichées en degrés dans l'inspecteur).

## Pourquoi une vue non-sRGB ?

La scène écrit des couleurs linéaires dans une image sRGB : le GPU les convertit. egui, lui,
produit déjà des couleurs « prêtes pour l'écran » : convertir une seconde fois les
délaverait. En dessinant l'interface dans une vue non-sRGB de la même image, aucune
conversion n'est faite pour elle.
