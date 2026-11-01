//! Les textures : des images stockées sur le GPU, plaquées sur les maillages.

use zentrix_core::Image;
use zentrix_rhi::wgpu;

/// Ce que contient une texture, qui décide de son format sur le GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextureKind {
    /// Des couleurs, en sRGB : le GPU convertit automatiquement les octets
    /// du fichier en couleurs linéaires au moment de la lecture dans le
    /// shader (voir `Color::to_srgba8`).
    Color,
    /// Des données (normales...) : les octets sont lus tels quels, de 0 à 1.
    /// Une conversion sRGB fausserait les directions d'une normal map.
    Data,
}

impl TextureKind {
    fn format(self) -> wgpu::TextureFormat {
        match self {
            TextureKind::Color => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureKind::Data => wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

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

/// Les textures d'un **matériau**, toutes facultatives. Une texture absente
/// est remplacée par une texture neutre : blanche (qui ne change rien à
/// une multiplication), ou « plate » pour la normal map.
///
/// | Texture              | Contenu (convention glTF)                     | Format |
/// |----------------------|-----------------------------------------------|--------|
/// | `base_color`         | la couleur                                    | sRGB   |
/// | `normal`             | le relief                                     | données |
/// | `metallic_roughness` | vert = rugosité, bleu = métal                 | données |
/// | `occlusion`          | rouge = part de lumière ambiante reçue        | données |
/// | `emissive`           | la couleur de la lumière émise                | sRGB   |
///
/// « Données » : créée avec `Renderer::add_normal_map` (ou
/// `add_data_texture`), sans conversion sRGB.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialTextures {
    pub base_color: Option<TextureHandle>,
    pub normal: Option<TextureHandle>,
    pub metallic_roughness: Option<TextureHandle>,
    pub occlusion: Option<TextureHandle>,
    pub emissive: Option<TextureHandle>,
}

impl MaterialTextures {
    /// Seulement une texture de couleur.
    pub fn color(texture: TextureHandle) -> Self {
        Self {
            base_color: Some(texture),
            ..Self::default()
        }
    }
}

/// Une texture prête à l'emploi : on garde sa **vue** (la façon dont les
/// shaders la voient). Les bind groups qui la référencent sont créés à la
/// demande par le `Renderer`, un par combinaison de textures de matériau.
pub(crate) struct Texture {
    pub view: wgpu::TextureView,
}

/// Le layout du bind group n°1 (`@group(1)` dans mesh.wgsl) : les 5
/// textures d'un matériau (dans l'ordre de [`MaterialTextures`]) et un
/// sampler, lus par le fragment shader.
pub(crate) fn create_material_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let texture = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            // `Float { filterable: true }` : des valeurs que le sampler
            // peut mélanger (filtrage linéaire).
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Material Bind Group Layout"),
        entries: &[
            texture(0),
            texture(1),
            texture(2),
            texture(3),
            texture(4),
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Le bind group d'un matériau : ses 5 textures (déjà remplacées par les
/// textures neutres si besoin) et le sampler commun.
pub(crate) fn create_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    views: [&wgpu::TextureView; 5],
) -> wgpu::BindGroup {
    let mut entries: Vec<wgpu::BindGroupEntry> = views
        .iter()
        .zip(0u32..)
        .map(|(view, binding)| wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(view),
        })
        .collect();
    entries.push(wgpu::BindGroupEntry {
        binding: 5,
        resource: wgpu::BindingResource::Sampler(sampler),
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Material Bind Group"),
        layout,
        entries: &entries,
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
        image: &Image,
        kind: TextureKind,
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
            format: kind.format(),
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
        Self { view }
    }
}
