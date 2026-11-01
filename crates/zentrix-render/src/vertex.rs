//! Les formats de données lus par le vertex shader : la description des
//! sommets (`Vertex`, défini dans `zentrix-core`) et les instances
//! ([`InstanceRaw`]).

use zentrix_core::math::Mat4;
use zentrix_core::{AlphaMode, Material, Vertex};
use zentrix_rhi::wgpu;

/// Les attributs d'un [`Vertex`], dans le même ordre que les `@location(n)` du shader.
///
/// `vertex_attr_array!` est une **macro** (reconnaissable au `!`) : du code
/// qui écrit du code. Elle calcule automatiquement les décalages (offsets)
/// de chaque attribut dans la struct.
///
/// La tangente prend la location 10 : les locations 4 à 9 étaient déjà
/// celles des instances. Les numéros n'ont pas besoin de se suivre.
const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
    0 => Float32x3, // position -> @location(0) dans le shader
    1 => Float32x3, // color    -> @location(1)
    2 => Float32x2, // uv       -> @location(2)
    3 => Float32x3, // normal   -> @location(3)
    10 => Float32x4, // tangent -> @location(10)
];

/// Décrit au GPU comment lire un tableau de [`Vertex`] dans un buffer.
///
/// La durée de vie `'static` indique que la description renvoyée ne
/// référence que des données qui existent pendant tout le programme
/// (notre constante `VERTEX_ATTRIBUTES`).
pub(crate) fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        // Combien d'octets sauter pour passer au sommet suivant.
        // `size_of` est calculé à la compilation (60 ici).
        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        // Avancer d'un élément à chaque SOMMET (l'alternative `Instance`
        // sert au rendu instancié : dessiner 1000 arbres en un appel).
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &VERTEX_ATTRIBUTES,
    }
}

/// Les données propres à **une instance** (un exemplaire d'un maillage) :
/// sa matrice modèle et son matériau, 28 `f32` = 112 octets.
///
/// # Le rendu instancié
///
/// Pour dessiner 25 cubes, on pourrait faire 25 appels `draw`. À la place, on
/// range les 25 matrices dans un seul buffer et on fait **un** appel
/// `draw_indexed(.., 0..25)`. Le GPU lit alors ce buffer avec
/// `VertexStepMode::Instance` : il avance d'une matrice à chaque nouvelle
/// instance (et non à chaque sommet).
///
/// Un attribut de sommet fait au plus 4 composantes (`Float32x4`) : la matrice
/// 4×4 est donc découpée en ses 4 colonnes, recomposées dans le shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    /// Les 4 colonnes de la matrice (glam stocke les matrices colonne par
    /// colonne, exactement comme WGSL).
    pub model: [[f32; 4]; 4],
    /// `Material::base_color` (r, g, b, a).
    pub base_color: [f32; 4],
    /// (métal, rugosité, intensité du relief, mode alpha). Le mode alpha
    /// est codé en un nombre : 0 = opaque, un seuil > 0 = découpe sous ce
    /// seuil, -1 = mélange (voir `fs_main` dans mesh.wgsl).
    pub material: [f32; 4],
    /// `Material::emissive` (r, g, b), puis l'intensité de l'occlusion.
    pub emissive: [f32; 4],
}

impl InstanceRaw {
    /// Les locations 4 à 9 suivent celles du `Vertex` (0 à 3) ; la 10 est la
    /// tangente du `Vertex`, d'où 11 pour l'émission.
    const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
        4 => Float32x4, // colonne 0
        5 => Float32x4, // colonne 1
        6 => Float32x4, // colonne 2
        7 => Float32x4, // colonne 3 (contient la translation)
        8 => Float32x4, // couleur de base
        9 => Float32x4, // métal, rugosité, relief
        11 => Float32x4, // émission, occlusion
    ];

    /// Une instance placée par `model`, avec le matériau `material`.
    pub fn new(model: Mat4, material: &Material) -> Self {
        let c = material.base_color;
        Self {
            model: model.to_cols_array_2d(),
            base_color: [c.r, c.g, c.b, c.a],
            material: [
                material.metallic,
                material.roughness,
                material.normal_scale,
                match material.alpha_mode {
                    AlphaMode::Opaque => 0.0,
                    // Un seuil nul ne découperait rien : autant dire opaque.
                    AlphaMode::Mask { cutoff } => cutoff.max(0.0),
                    AlphaMode::Blend => -1.0,
                },
            ],
            emissive: [
                material.emissive.r,
                material.emissive.g,
                material.emissive.b,
                material.occlusion_strength,
            ],
        }
    }

    /// Décrit au GPU comment lire un tableau d'`InstanceRaw`.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// Une instance avec le matériau par défaut.
impl From<Mat4> for InstanceRaw {
    fn from(model: Mat4) -> Self {
        Self::new(model, &Material::DEFAULT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taille_de_l_instance() {
        assert_eq!(std::mem::size_of::<InstanceRaw>(), 112);
    }
}
