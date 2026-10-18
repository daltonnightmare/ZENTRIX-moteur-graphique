//! Le [`Renderer`] : point d'entrée du rendu pour l'application.

use zentrix_core::Color;
use zentrix_rhi::wgpu::util::DeviceExt; // apporte la méthode `create_buffer_init`
use zentrix_rhi::{GpuContext, RhiError, wgpu};

use crate::pipeline::create_triangle_pipeline;
use crate::vertex::Vertex;

/// Les 3 sommets de notre triangle de démonstration.
///
/// Une `const` de tableau : stockée directement dans l'exécutable.
/// L'ordre (bas-gauche → bas-droite → haut) est anti-horaire, ce qui
/// correspond à `FrontFace::Ccw` dans le pipeline. Inversez deux sommets et
/// le triangle disparaîtra (il sera considéré comme « vu de dos ») !
const TRIANGLE: [Vertex; 3] = [
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    }, // rouge
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    }, // vert
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    }, // bleu
];

/// Le moteur de rendu. Il **possède** les ressources GPU (pipeline, buffers)
/// et dessine une image à chaque appel de [`Renderer::render`].
///
/// Quand le `Renderer` est détruit (sort de portée), Rust appelle
/// automatiquement le « destructeur » (`Drop`) de chaque champ : les buffers
/// et le pipeline sont libérés sur le GPU. Pas de `free` ou de `delete` à
/// écrire, et impossible de l'oublier.
pub struct Renderer {
    /// Couleur de fond utilisée pour effacer l'écran à chaque image.
    pub clear_color: Color,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

impl Renderer {
    /// Crée toutes les ressources GPU nécessaires au rendu.
    ///
    /// `gpu: &GpuContext` : on **emprunte** le contexte GPU le temps de la
    /// fonction ; on n'en devient pas propriétaire.
    pub fn new(gpu: &GpuContext) -> Self {
        let pipeline = create_triangle_pipeline(gpu.device(), gpu.surface_format());

        // Un BUFFER est un bloc de mémoire sur le GPU. On y copie nos sommets.
        // `bytemuck::cast_slice` voit notre `[Vertex; 3]` comme une suite d'octets
        // (`&[u8]`), sans copie et sans `unsafe`, grâce au trait `Pod`.
        let vertex_buffer = gpu
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Triangle Vertex Buffer"),
                contents: bytemuck::cast_slice(&TRIANGLE),
                // Ce buffer sera lu comme source de sommets.
                usage: wgpu::BufferUsages::VERTEX,
            });

        log::info!("Renderer initialisé");

        Self {
            clear_color: Color::default(),
            pipeline,
            vertex_buffer,
            vertex_count: TRIANGLE.len() as u32,
        }
    }

    /// Dessine une image complète et l'affiche.
    ///
    /// `&mut GpuContext` car acquérir une image peut nécessiter de
    /// reconfigurer la surface (donc de modifier le contexte).
    pub fn render(&self, gpu: &mut GpuContext) -> Result<(), RhiError> {
        // 1. Obtenir l'image de la fenêtre. `let ... else` : si le motif ne
        //    correspond pas (ici `None`), on exécute le bloc `else`, qui doit
        //    obligatoirement sortir de la fonction.
        let Some(frame) = gpu.acquire_frame()? else {
            return Ok(()); // pas d'image disponible : on saute cette frame
        };

        // 2. Le « command encoder » : un carnet où l'on enregistre des commandes.
        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });

        // 3. Une « render pass » : une séquence de dessins vers une même image.
        //
        // Les accolades `{ ... }` créent une PORTÉE. La `render_pass` emprunte
        // `encoder` de façon exclusive ; elle doit donc être détruite (à la `}`)
        // avant qu'on puisse appeler `encoder.finish()`. Le borrow checker de
        // Rust vérifie cela pour nous.
        {
            let c = self.clear_color;
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Au début de la passe : remplir l'image avec la couleur de fond.
                        // (wgpu attend des f64, d'où les conversions `as f64`.)
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: c.r as f64,
                            g: c.g as f64,
                            b: c.b as f64,
                            a: c.a as f64,
                        }),
                        // À la fin : conserver le résultat (on veut l'afficher !).
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            // `slice(..)` : utiliser le buffer en entier (`..` = plage complète).
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // Dessiner les sommets 0, 1, 2 (`0..3`), pour 1 instance (`0..1`).
            render_pass.draw(0..self.vertex_count, 0..1);
        } // <- `render_pass` est détruite ici, l'emprunt de `encoder` prend fin.

        // 4. Fermer le carnet et l'envoyer au GPU. C'est seulement maintenant
        //    que le GPU commence réellement à travailler.
        gpu.queue().submit(std::iter::once(encoder.finish()));

        // 5. Afficher l'image. `present` consomme `frame` : on ne peut plus l'utiliser.
        gpu.present(frame);
        Ok(())
    }
}
