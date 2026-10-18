//! # zentrix-render
//!
//! Le rendu **haut niveau** : il sait *quoi* dessiner (des triangles, des
//! couleurs, bientôt des maillages, des caméras et des lumières) et s'appuie
//! sur `zentrix-rhi` pour *parler* au GPU.
//!
//! ## Le trajet d'une image
//!
//! ```text
//!  Renderer::render()
//!    │
//!    ├─ 1. gpu.acquire_frame()        → on obtient une image vierge de la fenêtre
//!    ├─ 2. device.create_command_encoder()
//!    │                                 → un « carnet » où l'on note les commandes
//!    ├─ 3. encoder.begin_render_pass()  → on efface l'image (clear color)
//!    │      ├─ set_pipeline(...)        → « voici comment dessiner »
//!    │      ├─ set_vertex_buffer(...)   → « voici les sommets »
//!    │      └─ draw(0..3, 0..1)         → « dessine 3 sommets, 1 fois »
//!    ├─ 4. queue.submit(encoder.finish())
//!    │                                 → on envoie le carnet au GPU
//!    └─ 5. frame.present()              → l'image apparaît à l'écran
//! ```
//!
//! Important : les étapes 2 et 3 n'exécutent **rien** sur le GPU. Elles
//! *enregistrent* des commandes. Le GPU ne travaille qu'au `submit`.

mod pipeline;
mod renderer;
mod vertex;

pub use renderer::Renderer;
pub use vertex::Vertex;
