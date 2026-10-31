//! Les formats de données lus par le vertex shader : les sommets ([`Vertex`])
//! et les instances ([`InstanceRaw`]).

use zentrix_core::math::Mat4;
use zentrix_rhi::wgpu;

/// Un sommet : une position 3D, une couleur et une coordonnée de texture.
///
/// # Les attributs magiques expliqués
///
/// - `#[repr(C)]` : par défaut, Rust peut réordonner les champs d'une struct
///   pour gagner de la place. Le GPU, lui, attend un ordre **exact**.
///   `repr(C)` force Rust à garder l'ordre écrit, comme en langage C.
///
/// - `bytemuck::Pod` (« Plain Old Data ») : garantit que la struct n'est qu'un
///   paquet d'octets (pas de pointeur, pas de trou). On peut alors la convertir
///   en `&[u8]` **sans `unsafe`** avec `bytemuck::cast_slice`.
///
/// - `bytemuck::Zeroable` : une struct remplie de zéros est valide.
///
/// En mémoire, un `Vertex` occupe 32 octets :
///
/// ```text
///  octet : 0      4      8      12     16     20     24     28     32
///          ├──────┼──────┼──────┼──────┼──────┼──────┼──────┼──────┤
///          │  x   │  y   │  z   │  r   │  g   │  b   │  u   │  v   │
///          └──────┴──────┴──────┴──────┴──────┴──────┴──────┴──────┘
///           position (loc. 0)    color (loc. 1)       uv (loc. 2)
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// Position dans le repère **de l'objet** (« object space ») : le cube va
    /// de -0.5 à 0.5 quel que soit l'endroit où il est placé. C'est la matrice
    /// modèle de chaque instance, puis la caméra, qui le placent à l'écran.
    pub position: [f32; 3],
    /// Couleur RGB (linéaire) entre 0 et 1. Elle **multiplie** la couleur de
    /// la texture : blanc = texture inchangée, rouge = texture teintée de rouge.
    pub color: [f32; 3],
    /// Coordonnée de texture (« UV ») : quel point de l'image se plaque sur
    /// ce sommet.
    ///
    /// ```text
    ///  (0,0) ───── u ────▶ (1,0)      u : de gauche à droite de l'image
    ///    │       image       │        v : de HAUT en bas (convention wgpu)
    ///    v                   │
    ///    ▼                   │        Au-delà de 1, l'image se répète
    ///  (0,1) ───────────── (1,1)      (sampler en mode `Repeat`).
    /// ```
    pub uv: [f32; 2],
}

impl Vertex {
    /// Construit un sommet. `const fn` : utilisable dans des constantes.
    pub const fn new(position: [f32; 3], color: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            color,
            uv,
        }
    }

    /// La liste des attributs, dans le même ordre que les `@location(n)` du shader.
    ///
    /// `vertex_attr_array!` est une **macro** (reconnaissable au `!`) : du code
    /// qui écrit du code. Elle calcule automatiquement les décalages (offsets)
    /// de chaque attribut dans la struct.
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3, // position -> @location(0) dans le shader
        1 => Float32x3, // color    -> @location(1)
        2 => Float32x2, // uv       -> @location(2)
    ];

    /// Décrit au GPU comment lire un tableau de `Vertex` dans un buffer.
    ///
    /// La durée de vie `'static` indique que la description renvoyée ne
    /// référence que des données qui existent pendant tout le programme
    /// (notre constante `ATTRIBUTES`).
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            // Combien d'octets sauter pour passer au sommet suivant.
            // `size_of` est calculé à la compilation (32 ici).
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            // Avancer d'un élément à chaque SOMMET (l'alternative `Instance`
            // sert au rendu instancié : dessiner 1000 arbres en un appel).
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// Les données propres à **une instance** (un exemplaire d'un maillage) :
/// sa matrice modèle, 16 `f32` = 64 octets.
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
}

impl InstanceRaw {
    /// Les locations 3 à 6 suivent celles du `Vertex` (0 à 2).
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        3 => Float32x4, // colonne 0
        4 => Float32x4, // colonne 1
        5 => Float32x4, // colonne 2
        6 => Float32x4, // colonne 3 (contient la translation)
    ];

    /// Décrit au GPU comment lire un tableau d'`InstanceRaw`.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

impl From<Mat4> for InstanceRaw {
    fn from(model: Mat4) -> Self {
        Self {
            model: model.to_cols_array_2d(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taille_de_l_instance() {
        assert_eq!(std::mem::size_of::<InstanceRaw>(), 64);
    }

    #[test]
    fn taille_du_vertex() {
        // 8 flottants de 4 octets = 32 octets. Si ce test casse, le shader
        // et le layout ne correspondent plus à la struct !
        assert_eq!(std::mem::size_of::<Vertex>(), 32);
    }
}
