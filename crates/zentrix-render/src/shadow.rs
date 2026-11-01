//! La **shadow map** : la profondeur de la scène vue depuis la lumière.
//!
//! ```text
//!  passe 1 (ombre)  : on dessine la scène depuis la lumière, profondeur seule
//!                     → pour chaque « rayon », la distance du premier objet touché
//!  passe 2 (couleur): pour chaque pixel, on calcule sa distance à la lumière
//!                     et on la compare à la shadow map :
//!                       plus loin que l'objet enregistré → un objet le cache → OMBRE
//! ```
//!
//! Avec les ombres en cascade (cascades.rs), il y a une shadow map par
//! tranche : une **texture à couches** (*texture array*), une couche par
//! tranche. La passe d'ombre est faite une fois par couche.

use zentrix_core::math::Mat4;
use zentrix_rhi::wgpu;

use crate::cascades::CASCADE_COUNT;
use crate::depth::DEPTH_FORMAT;

/// Résolution de chaque couche de la shadow map (carrée). 2048² × 4 octets
/// = 16 Mo par couche, 48 Mo pour les trois.
pub(crate) const SHADOW_MAP_SIZE: u32 = 2048;

/// La shadow map et ce qu'il faut pour l'écrire et la lire.
pub(crate) struct ShadowMap {
    /// Une vue par couche : la cible de la passe d'ombre de chaque tranche.
    pub layer_views: Vec<wgpu::TextureView>,
    /// Le bind group n°2 du pipeline des maillages : toutes les couches + le sampler.
    pub bind_group: wgpu::BindGroup,
    /// Les matrices des tranches, une par « case » de `cascade_stride` octets.
    cascade_buffer: wgpu::Buffer,
    cascade_stride: u32,
    /// Le bind group n°0 des pipelines d'ombre : la matrice d'UNE tranche,
    /// choisie par un décalage dans `cascade_buffer` (voir `cascade_offset`).
    pub cascade_bind_group: wgpu::BindGroup,
}

/// Le layout du bind group n°2 (`@group(2)` dans mesh.wgsl).
pub(crate) fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Shadow Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    // Une texture de PROFONDEUR à couches
                    // (`texture_depth_2d_array` en WGSL).
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // Un sampler de COMPARAISON (`sampler_comparison` en WGSL).
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
    })
}

/// Le layout du bind group n°0 des pipelines d'ombre : un uniform buffer à
/// **décalage dynamique**. Le même bind group sert aux trois passes ; on
/// donne à chacune le décalage de sa tranche dans le buffer
/// (`set_bind_group(0, groupe, &[décalage])`).
pub(crate) fn create_cascade_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Cascade Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: wgpu::BufferSize::new(64),
            },
            count: None,
        }],
    })
}

impl ShadowMap {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        cascade_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: CASCADE_COUNT as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            // Écrite par la passe d'ombre, puis LUE par le shader des maillages.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let layer_views = (0..CASCADE_COUNT as u32)
            .map(|layer| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: layer,
                    array_layer_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
        let array_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        // Un sampler de comparaison ne renvoie pas une profondeur, mais le
        // RÉSULTAT d'un test : 1.0 si `profondeur du pixel <= shadow map`
        // (éclairé), 0.0 sinon (dans l'ombre). Avec le filtrage `Linear`, le
        // GPU fait ce test sur 4 texels voisins et mélange les résultats :
        // les bords d'ombre sont adoucis gratuitement.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&array_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Un décalage dynamique doit être un multiple d'un alignement imposé
        // par le GPU (souvent 256 octets) : chaque matrice (64 octets)
        // occupe donc une case de cette taille.
        let cascade_stride = device.limits().min_uniform_buffer_offset_alignment.max(64);
        let cascade_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Cascade Buffer"),
            size: u64::from(cascade_stride) * CASCADE_COUNT as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let cascade_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cascade Bind Group"),
            layout: cascade_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &cascade_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(64),
                }),
            }],
        });

        Self {
            layer_views,
            bind_group,
            cascade_buffer,
            cascade_stride,
            cascade_bind_group,
        }
    }

    /// Envoie les matrices des tranches au GPU.
    pub fn write_cascades(&self, queue: &wgpu::Queue, matrices: &[Mat4; CASCADE_COUNT]) {
        for (i, matrix) in matrices.iter().enumerate() {
            queue.write_buffer(
                &self.cascade_buffer,
                u64::from(self.cascade_offset(i)),
                bytemuck::cast_slice(&matrix.to_cols_array()),
            );
        }
    }

    /// Le décalage de la tranche `i` dans le buffer des matrices.
    pub fn cascade_offset(&self, i: usize) -> u32 {
        self.cascade_stride * i as u32
    }
}
