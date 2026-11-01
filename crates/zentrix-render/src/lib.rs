//! # zentrix-render
//!
//! Le rendu **haut niveau** : il sait *quoi* dessiner (des maillages vus par
//! une caméra, texturés, éclairés et ombrés) et s'appuie sur
//! `zentrix-rhi` pour *parler* au GPU.
//!
//! ## Utilisation
//!
//! ```text
//!  au démarrage :  let cube = renderer.add_mesh(&gpu, &MeshData::cube());
//!                  let bois = renderer.add_texture(&gpu, &image);
//!  chaque image :  debug.clear();
//!                  debug.axes(Vec3::ZERO, 1.0);           // lignes de debug (facultatif)
//!                  renderer.render(&mut gpu, &camera, &[
//!                      DrawCommand { mesh: cube, textures: MaterialTextures::color(bois),
//!                                    transform: t.matrix(), material: Material::DEFAULT },
//!                  ], &debug)?;
//! ```
//!
//! ## Le trajet d'une image
//!
//! ```text
//!  Renderer::render()
//!    │
//!    ├─ 1. gpu.acquire_frame()        → on obtient une image vierge de la fenêtre
//!    ├─    queue.write_buffer(...)    → on envoie la caméra et les matrices des objets
//!    ├─ 2. device.create_command_encoder()
//!    │                                 → un « carnet » où l'on note les commandes
//!    ├─ 3. encoder.begin_render_pass()  → on efface l'image et le depth buffer
//!    │      ├─ set_pipeline(...)        → « voici comment dessiner »
//!    │      ├─ set_bind_group(0, ...)   → « voici la caméra »
//!    │      └─ pour chaque maillage :
//!    │           ├─ set_vertex_buffer / set_index_buffer → « voici la géométrie »
//!    │           └─ draw_indexed(.., instances)          → « dessine-la N fois »
//!    │      └─ set_pipeline(lignes) + draw(...)            → les lignes de debug
//!    ├─ 4. queue.submit(encoder.finish())
//!    │                                 → on envoie le carnet au GPU
//!    └─ 5. frame.present()              → l'image apparaît à l'écran
//! ```
//!
//! Important : les étapes 2 et 3 n'exécutent **rien** sur le GPU. Elles
//! *enregistrent* des commandes. Le GPU ne travaille qu'au `submit`.

mod buffer;
mod camera;
mod cascades;
mod culling;
mod debug;
mod depth;
mod environment;
mod gpu_timer;
mod lighting;
mod mesh;
mod pipeline;
mod postprocess;
mod renderer;
mod shadow;
mod texture;
mod vertex;

pub use camera::Camera;
pub use cascades::{CASCADE_COUNT, Cascade, ShadowSettings};
pub use debug::DebugLines;
pub use environment::Sky;
pub use gpu_timer::{GPU_SECTIONS, GpuSection, GpuTimings};
pub use lighting::{Lighting, LocalLight, SpotCone};
pub use mesh::MeshHandle;
pub use renderer::{DrawCommand, MAX_LOCAL_LIGHTS, RenderOverlay, RenderStats, Renderer};
pub use texture::{MaterialTextures, TextureHandle};
pub use vertex::InstanceRaw;
// Les données de géométrie vivent dans le noyau ; on les ré-exporte pour que
// `zentrix_render::MeshData` continue de fonctionner.
pub use zentrix_core::{Material, MeshData, Vertex};
