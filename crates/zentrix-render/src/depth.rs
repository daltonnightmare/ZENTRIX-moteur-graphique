//! Le **depth buffer** (tampon de profondeur).
//!
//! Sans lui, le GPU dessine les triangles dans l'ordre où on les envoie : un
//! cube lointain dessiné en dernier recouvrirait un cube proche. Le depth
//! buffer est une image invisible, de la taille de la fenêtre, qui mémorise
//! pour chaque pixel la profondeur de l'objet le plus proche déjà dessiné.
//! Un nouveau pixel n'est écrit que s'il est **plus proche** (`CompareFunction::Less`).

use zentrix_rhi::wgpu;

/// Le format des pixels de profondeur : un `f32` par pixel.
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// La texture de profondeur et la taille pour laquelle elle a été créée.
pub(crate) struct DepthTexture {
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
}

impl DepthTexture {
    /// Crée une texture de profondeur de la taille donnée.
    ///
    /// Elle doit toujours avoir **exactement** la taille de l'image de la
    /// fenêtre : le `Renderer` la recrée quand la fenêtre change de taille.
    pub fn new(device: &wgpu::Device, (width, height): (u32, u32)) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            // Elle ne sert que de cible de rendu (pour l'instant : plus tard, on
            // pourra la lire dans un shader pour des effets comme le SSAO).
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        // La vue garde la texture en vie (wgpu compte les références) :
        // pas besoin de stocker `texture` à part.
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            view,
            size: (width, height),
        }
    }
}
