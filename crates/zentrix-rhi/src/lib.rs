//! # zentrix-rhi — Render Hardware Interface
//!
//! La couche la plus **basse** du rendu : elle parle directement au GPU via
//! [`wgpu`]. Elle sait :
//!
//! - trouver une carte graphique et s'y connecter ([`GpuContext::new`]) ;
//! - relier le GPU à la fenêtre (la *surface*) ;
//! - fournir l'image sur laquelle dessiner à chaque frame ([`GpuContext::acquire_frame`]) ;
//! - s'adapter quand la fenêtre change de taille ([`GpuContext::resize`]).
//!
//! Elle ne sait **pas** ce qu'est un triangle, une caméra ou un matériau :
//! c'est le travail de `zentrix-render`, la couche du dessus.
//!
//! ## Vocabulaire wgpu (à lire une fois !)
//!
//! | Objet wgpu  | Analogie                                    | Rôle |
//! |-------------|---------------------------------------------|------|
//! | `Instance`  | L'annuaire                                  | Point d'entrée : liste les GPU et les API disponibles (Vulkan, DX12, Metal...). |
//! | `Adapter`   | Une carte graphique précise                 | Le GPU physique choisi (ex. « NVIDIA RTX 3060 »). |
//! | `Device`    | Une ligne ouverte avec ce GPU               | Sert à **créer** des ressources : buffers, textures, pipelines, shaders. |
//! | `Queue`     | La boîte aux lettres du GPU                 | Sert à **envoyer** du travail (commandes, données) au GPU. |
//! | `Surface`   | La toile accrochée à la fenêtre             | L'endroit où les images finales sont affichées. |
//! | `SurfaceConfiguration` | Le format de la toile            | Taille, format des pixels, synchronisation verticale (V-Sync). |

mod context;
mod error;

pub use context::{Frame, GpuContext};
pub use error::RhiError;

// Ré-export de wgpu : les crates du dessus (zentrix-render) utiliseront
// `zentrix_rhi::wgpu` et seront ainsi garanties d'avoir EXACTEMENT la même version.
pub use wgpu;
