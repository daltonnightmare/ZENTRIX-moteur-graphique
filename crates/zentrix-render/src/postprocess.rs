//! Le **post-traitement** : des effets appliqués à l'image entière, après
//! le dessin de la scène (voir shaders/post.wgsl).
//!
//! ```text
//!  passe principale ──▶ image HDR (Rgba16Float, valeurs > 1 permises)
//!                            │
//!            ┌───────────────┴─ bloom (si actif) ──────────────────────┐
//!            │  prefilter : HDR → niveau 0 (½ taille), seuil          │
//!            │  downsample : niveau 0 → 1 → … → N-1 (½ à chaque fois)  │
//!            │  upsample : N-1 → … → 0, ADDITIONNÉS en remontant       │
//!            └───────────────┬─────────────────────────────────────────┘
//!                            ▼
//!  composite : HDR + bloom, tone mapping ──▶ image LDR (si FXAA) ou fenêtre
//!                            ▼
//!  FXAA : bords adoucis ──▶ fenêtre
//! ```
//!
//! Les images intermédiaires dépendent de la taille de la fenêtre : elles
//! sont recréées quand elle change ([`PostProcess::ensure_size`]).

use zentrix_rhi::wgpu;
use zentrix_rhi::wgpu::util::DeviceExt;

/// Le format de l'image HDR : 4 flottants de 16 bits par pixel. Des valeurs
/// jusqu'à 65 504, avec une bonne précision dans les sombres. Deux fois plus
/// lourd qu'une image ordinaire (8 octets par pixel au lieu de 4).
pub(crate) const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Le nombre maximal de niveaux du bloom. Le dernier mesure 1/64 de l'image :
/// assez pour un halo large, sans niveaux minuscules inutiles.
const BLOOM_MAX_LEVELS: u32 = 6;

/// Les réglages du post-traitement pour une image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PostSettings {
    /// 0 = pas de bloom.
    pub bloom_intensity: f32,
    /// La luminosité à partir de laquelle un pixel « bave ».
    pub bloom_threshold: f32,
    pub tone_mapping: bool,
    pub fxaa: bool,
}

/// Les réglages tels que le shader les lit (`PostUniform` dans post.wgsl).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniform {
    bloom: [f32; 4],
    options: [f32; 4],
}

/// Les images qui dépendent de la taille de la fenêtre, et les bind groups
/// qui les lisent.
struct Targets {
    size: (u32, u32),
    hdr_view: wgpu::TextureView,
    /// L'image après tone mapping, lue par le FXAA.
    ldr_view: wgpu::TextureView,
    /// Une vue par niveau du bloom (chacune ne voit qu'un niveau de mipmap).
    bloom_views: Vec<wgpu::TextureView>,
    /// Lit l'image HDR (prefilter).
    prefilter_group: wgpu::BindGroup,
    /// `down_groups[i]` lit le niveau i, pour écrire le niveau i + 1.
    down_groups: Vec<wgpu::BindGroup>,
    /// `up_groups[i]` lit le niveau i + 1, pour l'ajouter au niveau i.
    up_groups: Vec<wgpu::BindGroup>,
    /// Lit l'image HDR et le niveau 0 du bloom.
    composite_group: wgpu::BindGroup,
    /// Lit l'image LDR.
    fxaa_group: wgpu::BindGroup,
}

/// Ce dont les bind groups des images ont besoin.
struct Shared {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniform: wgpu::Buffer,
    /// Une texture de 1 pixel, pour remplir la 2e image des passes qui n'en
    /// lisent qu'une (un bind group doit être complet).
    dummy_view: wgpu::TextureView,
    output_format: wgpu::TextureFormat,
}

/// Les pipelines, les ressources et les images du post-traitement.
pub(crate) struct PostProcess {
    shared: Shared,
    prefilter: wgpu::RenderPipeline,
    downsample: wgpu::RenderPipeline,
    upsample: wgpu::RenderPipeline,
    composite: wgpu::RenderPipeline,
    fxaa: wgpu::RenderPipeline,
    targets: Targets,
}

impl PostProcess {
    /// `output_format` : celui de la fenêtre (sRGB).
    pub fn new(
        device: &wgpu::Device,
        output_format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post Bind Group Layout"),
            entries: &[
                texture_entry(0),
                texture_entry(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        // Linéaire : lire « entre » deux pixels donne leur moyenne, ce que
        // les filtres du bloom utilisent pour lire 4 pixels en une fois.
        // `ClampToEdge` : au bord de l'image, on répète le dernier pixel.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Post Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Post Uniform"),
            contents: bytemuck::bytes_of(&PostUniform {
                bloom: [0.0; 4],
                options: [0.0; 4],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let dummy_view = create_texture(device, "Post Dummy", (1, 1), HDR_FORMAT, 1)
            .create_view(&wgpu::TextureViewDescriptor::default());

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Post Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/post.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post Pipeline Layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry: &str, format, blend| {
            create_fullscreen_pipeline(device, &pipeline_layout, &shader, entry, format, blend)
        };
        // L'agrandissement AJOUTE au niveau existant : couleur finale =
        // nouvelle × 1 + ancienne × 1.
        let additive = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::REPLACE,
        };
        let replace = wgpu::BlendState::REPLACE;

        let shared = Shared {
            layout,
            sampler,
            uniform,
            dummy_view,
            output_format,
        };
        Self {
            prefilter: pipeline("fs_prefilter", HDR_FORMAT, replace),
            downsample: pipeline("fs_downsample", HDR_FORMAT, replace),
            upsample: pipeline("fs_upsample", HDR_FORMAT, additive),
            composite: pipeline("fs_composite", output_format, replace),
            fxaa: pipeline("fs_fxaa", output_format, replace),
            targets: Targets::new(device, &shared, size),
            shared,
        }
    }

    /// L'image HDR, cible de la passe principale.
    pub fn hdr_view(&self) -> &wgpu::TextureView {
        &self.targets.hdr_view
    }

    /// Recrée les images si la fenêtre a changé de taille.
    pub fn ensure_size(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        if self.targets.size != size {
            self.targets = Targets::new(device, &self.shared, size);
        }
    }

    /// Enregistre toutes les passes du post-traitement ; le résultat est
    /// écrit dans `output` (l'image de la fenêtre).
    pub fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        settings: &PostSettings,
        output: &wgpu::TextureView,
    ) {
        let t = &self.targets;
        let levels = t.bloom_views.len();
        let bloom = settings.bloom_intensity > 0.0;
        let uniform = PostUniform {
            bloom: [
                settings.bloom_threshold,
                // Le genou : la moitié du seuil (une valeur courante).
                settings.bloom_threshold * 0.5,
                if bloom { settings.bloom_intensity } else { 0.0 },
                levels as f32,
            ],
            options: [f32::from(u8::from(settings.tone_mapping)), 0.0, 0.0, 0.0],
        };
        queue.write_buffer(&self.shared.uniform, 0, bytemuck::bytes_of(&uniform));

        if bloom {
            pass(
                encoder,
                "Bloom Prefilter",
                &t.bloom_views[0],
                false,
                &self.prefilter,
                &t.prefilter_group,
            );
            for i in 1..levels {
                pass(
                    encoder,
                    "Bloom Downsample",
                    &t.bloom_views[i],
                    false,
                    &self.downsample,
                    &t.down_groups[i - 1],
                );
            }
            // En remontant : chaque niveau reçoit le niveau plus petit,
            // étalé, PAR-DESSUS son propre contenu (`load = true`).
            for i in (0..levels - 1).rev() {
                pass(
                    encoder,
                    "Bloom Upsample",
                    &t.bloom_views[i],
                    true,
                    &self.upsample,
                    &t.up_groups[i],
                );
            }
        }

        if settings.fxaa {
            pass(
                encoder,
                "Composite",
                &t.ldr_view,
                false,
                &self.composite,
                &t.composite_group,
            );
            pass(encoder, "FXAA", output, false, &self.fxaa, &t.fxaa_group);
        } else {
            pass(
                encoder,
                "Composite",
                output,
                false,
                &self.composite,
                &t.composite_group,
            );
        }
    }
}

impl Targets {
    fn new(device: &wgpu::Device, shared: &Shared, (width, height): (u32, u32)) -> Self {
        let size = (width.max(1), height.max(1));
        let hdr_view = create_texture(device, "HDR Target", size, HDR_FORMAT, 1)
            .create_view(&wgpu::TextureViewDescriptor::default());
        let ldr_view = create_texture(device, "LDR Target", size, shared.output_format, 1)
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Le bloom : une texture à la moitié de la taille, avec ses niveaux.
        let bloom_size = ((size.0 / 2).max(1), (size.1 / 2).max(1));
        let levels = bloom_levels(bloom_size);
        let bloom = create_texture(device, "Bloom", bloom_size, HDR_FORMAT, levels);
        let bloom_views: Vec<_> = (0..levels)
            .map(|level| {
                bloom.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        let group = |label, source: &wgpu::TextureView, second: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &shared.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(second),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&shared.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: shared.uniform.as_entire_binding(),
                    },
                ],
            })
        };
        let dummy = &shared.dummy_view;
        let n = bloom_views.len();
        Targets {
            prefilter_group: group("Bloom Prefilter", &hdr_view, dummy),
            down_groups: (0..n - 1)
                .map(|i| group("Bloom Downsample", &bloom_views[i], dummy))
                .collect(),
            up_groups: (0..n - 1)
                .map(|i| group("Bloom Upsample", &bloom_views[i + 1], dummy))
                .collect(),
            composite_group: group("Composite", &hdr_view, &bloom_views[0]),
            fxaa_group: group("FXAA", &ldr_view, dummy),
            size: (width, height),
            hdr_view,
            ldr_view,
            bloom_views,
        }
    }
}

/// Le nombre de niveaux du bloom pour une image de taille `size` : on
/// divise par 2 tant que le niveau garde au moins 2 pixels de côté.
fn bloom_levels((width, height): (u32, u32)) -> u32 {
    let mut levels = 1;
    let mut side = width.min(height);
    while levels < BLOOM_MAX_LEVELS && side >= 4 {
        side /= 2;
        levels += 1;
    }
    levels
}

/// Une texture qui peut être une cible de rendu ET être lue par un shader.
fn create_texture(
    device: &wgpu::Device,
    label: &str,
    (width, height): (u32, u32),
    format: wgpu::TextureFormat,
    mip_level_count: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

/// Un pipeline « plein écran » : le triangle de `vs_main`, sans vertex
/// buffer ni depth buffer.
fn create_fullscreen_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    entry_point: &str,
    format: wgpu::TextureFormat,
    blend: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(entry_point),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(entry_point),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// Une passe plein écran : `pipeline` lit `group` et écrit dans `target`.
/// `load` : garder le contenu de la cible (pour y ajouter) au lieu de l'effacer.
fn pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &str,
    target: &wgpu::TextureView,
    load: bool,
    pipeline: &wgpu::RenderPipeline,
    group: &wgpu::BindGroup,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: if load {
                    wgpu::LoadOp::Load
                } else {
                    wgpu::LoadOp::Clear(wgpu::Color::BLACK)
                },
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, group, &[]);
    pass.draw(0..3, 0..1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn niveaux_du_bloom() {
        // 960 × 540 (la moitié du 1080p) : 6 niveaux, jusqu'à 30 × 16.
        assert_eq!(bloom_levels((960, 540)), BLOOM_MAX_LEVELS);
        // Une toute petite fenêtre : moins de niveaux, jamais zéro.
        assert_eq!(bloom_levels((8, 8)), 3);
        assert_eq!(bloom_levels((1, 1)), 1);
    }

    #[test]
    fn taille_du_uniform() {
        assert_eq!(std::mem::size_of::<PostUniform>(), 32);
    }
}
