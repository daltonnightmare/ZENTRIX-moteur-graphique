//! Le format d'un sommet (« vertex ») tel que le GPU le lit.

use zentrix_rhi::wgpu;

/// Un sommet : une position 3D et une couleur.
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
/// En mémoire, un `Vertex` occupe 24 octets :
///
/// ```text
///  octet : 0        4        8        12       16       20       24
///          ├────────┼────────┼────────┼────────┼────────┼────────┤
///          │  x     │  y     │  z     │  r     │  g     │  b     │
///          └────────┴────────┴────────┴────────┴────────┴────────┘
///           position (location 0)      color (location 1)
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// Position en « coordonnées normalisées » (NDC) pour l'instant :
    /// x et y vont de -1 (gauche/bas) à +1 (droite/haut), quelle que soit la
    /// taille de la fenêtre. Une caméra viendra transformer ça plus tard.
    pub position: [f32; 3],
    /// Couleur RGB entre 0 et 1.
    pub color: [f32; 3],
}

impl Vertex {
    /// La liste des attributs, dans le même ordre que les `@location(n)` du shader.
    ///
    /// `vertex_attr_array!` est une **macro** (reconnaissable au `!`) : du code
    /// qui écrit du code. Elle calcule automatiquement les décalages (offsets)
    /// de chaque attribut dans la struct.
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3, // position -> @location(0) dans le shader
        1 => Float32x3, // color    -> @location(1)
    ];

    /// Décrit au GPU comment lire un tableau de `Vertex` dans un buffer.
    ///
    /// La durée de vie `'static` indique que la description renvoyée ne
    /// référence que des données qui existent pendant tout le programme
    /// (notre constante `ATTRIBUTES`).
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            // Combien d'octets sauter pour passer au sommet suivant.
            // `size_of` est calculé à la compilation (24 ici).
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            // Avancer d'un élément à chaque SOMMET (l'alternative `Instance`
            // sert au rendu instancié : dessiner 1000 arbres en un appel).
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taille_du_vertex() {
        // 6 flottants de 4 octets = 24 octets. Si ce test casse, le shader
        // et le layout ne correspondent plus à la struct !
        assert_eq!(std::mem::size_of::<Vertex>(), 24);
    }
}
