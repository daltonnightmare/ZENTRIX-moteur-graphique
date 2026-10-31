//! Les textures : des images stockées sur le GPU, plaquées sur les maillages.

use zentrix_core::Image;
use zentrix_rhi::wgpu;

/// Le format des textures de couleur. `Srgb` : le GPU convertit
/// automatiquement les octets sRGB du fichier en couleurs linéaires au moment
/// de la lecture dans le shader (voir `Color::to_srgba8`).
const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Une référence légère vers une texture stockée dans le `Renderer`
/// (même principe que `MeshHandle`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureHandle(pub(crate) usize);

impl TextureHandle {
    /// Fabrique un handle à partir d'un numéro, **sans vérification**.
    /// Réservé aux tests et aux outils (voir `MeshHandle::from_index`).
    #[doc(hidden)]
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }
}

/// Une texture prête à l'emploi : on ne garde que son **bind group**, qui
/// référence la vue de la texture et le sampler (et les maintient en vie).
pub(crate) struct Texture {
    pub bind_group: wgpu::BindGroup,
}

/// Le layout du bind group n°1 (`@group(1)` dans mesh.wgsl) : une texture 2D
/// et un sampler, lus par le fragment shader.
pub(crate) fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Texture Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    // `Float { filterable: true }` : des couleurs que le sampler
                    // peut mélanger (filtrage linéaire).
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Le **sampler** : la façon de lire une texture.
///
/// - `Repeat` : au-delà de UV = 1, l'image se répète (carrelage) ;
/// - filtrage `Linear` : mélange les pixels voisins au lieu de montrer des
///   gros carrés quand on s'approche ;
/// - mipmaps `Linear` : mélange aussi deux niveaux de mipmap, sans saut visible ;
/// - `anisotropy_clamp: 16` : garde la texture nette quand on la voit de
///   biais (un sol au loin). Exige que les trois filtres soient `Linear`.
///
/// Un seul sampler suffit pour toutes les textures du moteur.
pub(crate) fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Default Sampler"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: 16,
        ..Default::default()
    })
}

impl Texture {
    /// Envoie une image sur le GPU, avec toute sa chaîne de mipmaps.
    pub fn upload(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        image: &Image,
        label: &str,
    ) -> Self {
        // Les mipmaps sont calculées sur le CPU (voir `Image::mip_chain`).
        // Si l'image dépasse la taille maximale acceptée par le GPU, on saute
        // simplement les premiers niveaux : la texture sera moins détaillée,
        // mais le programme fonctionne.
        let max_size = device.limits().max_texture_dimension_2d;
        let chain: Vec<Image> = image
            .mip_chain()
            .into_iter()
            .skip_while(|mip| mip.width > max_size || mip.height > max_size)
            .collect();
        if chain[0].width != image.width {
            log::warn!(
                "Texture « {label} » réduite de {}x{} à {}x{} (limite du GPU : {max_size})",
                image.width,
                image.height,
                chain[0].width,
                chain[0].height
            );
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: chain[0].width,
                height: chain[0].height,
                depth_or_array_layers: 1,
            },
            mip_level_count: chain.len() as u32,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: COLOR_FORMAT,
            // Lue par les shaders, et remplie par une copie depuis le CPU.
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // `enumerate` numérote les éléments : (0, image pleine), (1, moitié)...
        for (level, mip) in chain.iter().enumerate() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level as u32,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &mip.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    // 4 octets par pixel (RGBA).
                    bytes_per_row: Some(4 * mip.width),
                    rows_per_image: Some(mip.height),
                },
                wgpu::Extent3d {
                    width: mip.width,
                    height: mip.height,
                    depth_or_array_layers: 1,
                },
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        Self { bind_group }
    }
}
