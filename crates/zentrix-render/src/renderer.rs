//! Le [`Renderer`] : point d'entrée du rendu pour l'application.

use zentrix_core::Color;
use zentrix_core::math::Mat4;
use zentrix_rhi::wgpu::util::DeviceExt; // apporte la méthode `create_buffer_init`
use zentrix_rhi::{GpuContext, RhiError, wgpu};

use crate::camera::Camera;
use crate::depth::DepthTexture;
use crate::mesh::{Mesh, MeshData, MeshHandle};
use crate::pipeline::create_mesh_pipeline;
use crate::vertex::InstanceRaw;

/// « Dessine ce maillage, à cet endroit. »
///
/// C'est **tout** ce que le rendu sait de la scène. Le jeu (ou plus tard
/// l'ECS) construit une liste de `DrawCommand` à chaque image et la donne à
/// [`Renderer::render`]. Ainsi le rendu ne dépend jamais de la logique de jeu.
#[derive(Debug, Clone, Copy)]
pub struct DrawCommand {
    pub mesh: MeshHandle,
    /// La matrice modèle, généralement obtenue avec `Transform::matrix()`.
    pub transform: Mat4,
}

/// Les données de la caméra telles que le shader les lit (`CameraUniform`
/// dans mesh.wgsl).
///
/// Attention à l'alignement : WGSL impose des règles strictes sur la taille
/// des uniforms (multiples de 16 octets). Une `mat4x4<f32>` fait 64 octets,
/// tout va bien ; si on ajoute un jour un `f32` seul, il faudra du remplissage.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

/// Nombre d'instances que le buffer peut contenir au départ. Il grandit
/// automatiquement si la scène en demande plus.
const INITIAL_INSTANCE_CAPACITY: usize = 256;

/// Le moteur de rendu. Il **possède** les ressources GPU (pipeline, buffers,
/// maillages) et dessine une image à chaque appel de [`Renderer::render`].
///
/// Quand le `Renderer` est détruit (sort de portée), Rust appelle
/// automatiquement le « destructeur » (`Drop`) de chaque champ : les buffers
/// et le pipeline sont libérés sur le GPU. Pas de `free` ou de `delete` à
/// écrire, et impossible de l'oublier.
pub struct Renderer {
    /// Couleur de fond utilisée pour effacer l'écran à chaque image.
    pub clear_color: Color,
    pipeline: wgpu::RenderPipeline,

    /// Tous les maillages chargés. Un `MeshHandle` est un indice dans ce `Vec`.
    meshes: Vec<Mesh>,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    /// Les matrices de toutes les instances de l'image, à la suite.
    instance_buffer: wgpu::Buffer,
    /// Nombre de matrices que `instance_buffer` peut contenir.
    instance_capacity: usize,
    /// Tableaux de travail réutilisés d'une image à l'autre : `clear()` vide
    /// un `Vec` sans libérer sa mémoire, on évite donc une allocation par image.
    /// `instances` sert au tri, `instance_data` au format attendu par le GPU.
    instances: Vec<(MeshHandle, InstanceRaw)>,
    instance_data: Vec<InstanceRaw>,

    depth: DepthTexture,
}

impl Renderer {
    /// Crée toutes les ressources GPU nécessaires au rendu.
    ///
    /// `gpu: &GpuContext` : on **emprunte** le contexte GPU le temps de la
    /// fonction ; on n'en devient pas propriétaire.
    pub fn new(gpu: &GpuContext) -> Self {
        let device = gpu.device();

        // --- La caméra : un uniform buffer + son bind group -------------------
        // Le LAYOUT décrit la forme (« un uniform buffer, lu par le vertex
        // shader ») ; le BIND GROUP relie cette forme à un buffer précis.
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0, // `@binding(0)` dans le shader
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // COPY_DST : on pourra y écrire depuis le CPU à chaque image (`write_buffer`).
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::bytes_of(&CameraUniform {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline = create_mesh_pipeline(device, gpu.surface_format(), &camera_layout);

        log::info!("Renderer initialisé");

        Self {
            clear_color: Color::default(),
            pipeline,
            meshes: Vec::new(),
            camera_buffer,
            camera_bind_group,
            instance_buffer: create_instance_buffer(device, INITIAL_INSTANCE_CAPACITY),
            instance_capacity: INITIAL_INSTANCE_CAPACITY,
            instances: Vec::new(),
            instance_data: Vec::new(),
            depth: DepthTexture::new(device, gpu.size()),
        }
    }

    /// Envoie un maillage sur le GPU et renvoie le handle qui permet de le dessiner.
    pub fn add_mesh(&mut self, gpu: &GpuContext, data: &MeshData) -> MeshHandle {
        self.meshes.push(Mesh::upload(gpu.device(), data));
        MeshHandle(self.meshes.len() - 1)
    }

    /// Dessine une image complète, vue par `camera`, et l'affiche.
    ///
    /// `&mut GpuContext` car acquérir une image peut nécessiter de
    /// reconfigurer la surface (donc de modifier le contexte).
    pub fn render(
        &mut self,
        gpu: &mut GpuContext,
        camera: &Camera,
        draws: &[DrawCommand],
    ) -> Result<(), RhiError> {
        // 1. Obtenir l'image de la fenêtre. `let ... else` : si le motif ne
        //    correspond pas (ici `None`), on exécute le bloc `else`, qui doit
        //    obligatoirement sortir de la fonction.
        let Some(frame) = gpu.acquire_frame()? else {
            return Ok(()); // pas d'image disponible : on saute cette frame
        };

        // 2. Le depth buffer doit avoir la taille de la fenêtre : on le recrée
        //    si elle a changé (redimensionnement).
        if self.depth.size != gpu.size() {
            self.depth = DepthTexture::new(gpu.device(), gpu.size());
        }

        // 3. Mettre à jour les données de l'image sur le GPU.
        let (width, height) = gpu.size();
        let aspect = width as f32 / height as f32;
        let camera_uniform = CameraUniform {
            view_proj: camera.view_projection(aspect).to_cols_array_2d(),
        };
        gpu.queue()
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&camera_uniform));
        self.upload_instances(gpu, draws);

        // 4. Le « command encoder » : un carnet où l'on enregistre des commandes.
        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });

        // 5. Une « render pass » : une séquence de dessins vers une même image.
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        // 1.0 = « infiniment loin » : le premier objet dessiné
                        // sur un pixel passe toujours le test de profondeur.
                        load: wgpu::LoadOp::Clear(1.0),
                        // Conservée pour de futures passes (ex. lignes de debug).
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None, // notre format n'a pas de stencil
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            // `self.instances` est trié par maillage : les instances d'un même
            // maillage se suivent. `chunk_by` découpe le tableau en groupes
            // consécutifs de même maillage, et chaque groupe = UN appel de dessin.
            let mut first = 0u32;
            for group in self.instances.chunk_by(|a, b| a.0 == b.0) {
                let mesh = &self.meshes[group[0].0.0];
                let count = group.len() as u32;
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                // Tous les indices du maillage, pour les instances first..first+count.
                render_pass.draw_indexed(0..mesh.index_count, 0, first..first + count);
                first += count;
            }
        } // <- `render_pass` est détruite ici, l'emprunt de `encoder` prend fin.

        // 6. Fermer le carnet et l'envoyer au GPU. C'est seulement maintenant
        //    que le GPU commence réellement à travailler.
        gpu.queue().submit(std::iter::once(encoder.finish()));

        // 7. Afficher l'image. `present` consomme `frame` : on ne peut plus l'utiliser.
        gpu.present(frame);
        Ok(())
    }

    /// Trie les commandes par maillage et copie leurs matrices dans
    /// `instance_buffer`, en l'agrandissant si nécessaire.
    fn upload_instances(&mut self, gpu: &GpuContext, draws: &[DrawCommand]) {
        self.instances.clear();
        self.instances.extend(
            draws
                .iter()
                .map(|d| (d.mesh, InstanceRaw::from(d.transform))),
        );
        // `sort_by_key` est stable : l'ordre des commandes d'un même maillage
        // est conservé. (Le test de profondeur rend l'ordre sans importance
        // pour des objets opaques, mais ce sera utile pour la transparence.)
        self.instances.sort_by_key(|(mesh, _)| *mesh);

        if self.instances.len() > self.instance_capacity {
            // On double (au minimum) la capacité pour ne pas recréer le buffer
            // à chaque nouvel objet.
            self.instance_capacity = self.instances.len().next_power_of_two();
            self.instance_buffer = create_instance_buffer(gpu.device(), self.instance_capacity);
            log::debug!(
                "Instance buffer agrandi à {} instances",
                self.instance_capacity
            );
        }

        // On n'envoie que les matrices (pas les handles), à la suite.
        self.instance_data.clear();
        self.instance_data
            .extend(self.instances.iter().map(|(_, raw)| *raw));
        if !self.instance_data.is_empty() {
            gpu.queue().write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&self.instance_data),
            );
        }
    }
}

/// Crée un buffer vide pouvant contenir `capacity` instances.
fn create_instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Instance Buffer"),
        size: (capacity * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}
