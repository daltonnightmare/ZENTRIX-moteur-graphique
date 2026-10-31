//! # zentrix-render
//!
//! Le rendu **haut niveau** : il sait *quoi* dessiner (des maillages vus par
//! une caméra, texturés, bientôt éclairés) et s'appuie sur
//! `zentrix-rhi` pour *parler* au GPU.
//!
//! ## Utilisation
//!
//! ```text
//!  au démarrage :  let cube = renderer.add_mesh(&gpu, &MeshData::cube());
//!                  let bois = renderer.add_texture(&gpu, &image);
//!  chaque image :  renderer.render(&mut gpu, &camera, &[
//!                      DrawCommand { mesh: cube, texture: Some(bois), transform: t.matrix() },
//!                  ])?;
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
//!    ├─ 4. queue.submit(encoder.finish())
//!    │                                 → on envoie le carnet au GPU
//!    └─ 5. frame.present()              → l'image apparaît à l'écran
//! ```
//!
//! Important : les étapes 2 et 3 n'exécutent **rien** sur le GPU. Elles
//! *enregistrent* des commandes. Le GPU ne travaille qu'au `submit`.

mod camera;
mod depth;
mod mesh;
mod pipeline;
mod renderer;
mod texture;
mod vertex;

pub use camera::Camera;
pub use mesh::{MeshData, MeshHandle};
pub use renderer::{DrawCommand, Renderer};
pub use texture::TextureHandle;
pub use vertex::{InstanceRaw, Vertex};
