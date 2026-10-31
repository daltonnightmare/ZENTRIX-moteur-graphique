//! Le **rendu de debug** : des lignes de couleur pour visualiser ce qui est
//! normalement invisible (axes du monde, grille, boîtes englobantes,
//! directions, trajectoires...).
//!
//! C'est un outil de **développement** : pas d'épaisseur, pas d'éclairage,
//! juste des segments. Mais c'est souvent le moyen le plus rapide de
//! comprendre un bug de position ou d'orientation.

use zentrix_core::Color;
use zentrix_core::math::{Mat4, Vec3};
use zentrix_rhi::wgpu;

/// Un sommet de ligne : position dans le **monde** et couleur. 24 octets.
///
/// Plus simple qu'un `Vertex` de maillage : pas de coordonnée de texture, et
/// pas de matrice d'instance (les positions sont déjà dans le monde).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LineVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl LineVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3, // position -> @location(0) dans debug.wgsl
        1 => Float32x3, // color    -> @location(1)
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// La liste des lignes de debug à dessiner pour l'image en cours.
///
/// L'application la remplit (souvent dans `on_update`), la passe à
/// `Renderer::render`, puis la vide avec [`DebugLines::clear`] à l'image
/// suivante. C'est un dessin en **mode immédiat** : une ligne n'existe que
/// pendant l'image où on la demande.
///
/// ```
/// use zentrix_core::Color;
/// use zentrix_core::math::Vec3;
/// use zentrix_render::DebugLines;
///
/// let mut debug = DebugLines::default();
/// debug.line(Vec3::ZERO, Vec3::Y, Color::WHITE);
/// debug.axes(Vec3::ZERO, 1.0);
/// assert_eq!(debug.line_count(), 4);
/// ```
#[derive(Debug, Default, Clone)]
pub struct DebugLines {
    /// Deux sommets par ligne (topologie `LineList`).
    vertices: Vec<LineVertex>,
}

impl DebugLines {
    /// Un segment de `start` à `end`.
    pub fn line(&mut self, start: Vec3, end: Vec3, color: Color) {
        let color = [color.r, color.g, color.b];
        self.vertices.extend([
            LineVertex {
                position: start.to_array(),
                color,
            },
            LineVertex {
                position: end.to_array(),
                color,
            },
        ]);
    }

    /// Une grille horizontale (plan XZ) de `size × size` unités, centrée sur
    /// `center`, découpée en `divisions` cases par côté.
    pub fn grid(&mut self, center: Vec3, size: f32, divisions: u32, color: Color) {
        let divisions = divisions.max(1);
        let half = size / 2.0;
        let step = size / divisions as f32;
        // `..=` : plage INCLUSIVE. 4 cases demandent 5 lignes dans chaque sens.
        for i in 0..=divisions {
            let offset = -half + i as f32 * step;
            // Une ligne parallèle à Z, puis une parallèle à X.
            self.line(
                center + Vec3::new(offset, 0.0, -half),
                center + Vec3::new(offset, 0.0, half),
                color,
            );
            self.line(
                center + Vec3::new(-half, 0.0, offset),
                center + Vec3::new(half, 0.0, offset),
                color,
            );
        }
    }

    /// Les trois axes du repère à partir de `origin`, selon la convention
    /// universelle : **X rouge, Y vert, Z bleu** (« RGB = XYZ »).
    pub fn axes(&mut self, origin: Vec3, length: f32) {
        self.line(origin, origin + Vec3::X * length, Color::RED);
        self.line(origin, origin + Vec3::Y * length, Color::GREEN);
        self.line(origin, origin + Vec3::Z * length, Color::BLUE);
    }

    /// Les 12 arêtes d'un cube de côté 1 (comme `MeshData::cube`), placé par
    /// la matrice `transform`. Avec la même matrice que l'objet, on obtient
    /// le contour de sa **boîte englobante orientée**.
    pub fn wire_box(&mut self, transform: Mat4, color: Color) {
        // Les 8 coins : chaque bit de `i` choisit -0.5 ou +0.5 sur un axe.
        //   bit 0 → x, bit 1 → y, bit 2 → z
        let corner = |i: usize| {
            let pick = |bit: usize| if i & (1 << bit) != 0 { 0.5 } else { -0.5 };
            transform.transform_point3(Vec3::new(pick(0), pick(1), pick(2)))
        };
        // Deux coins sont reliés par une arête s'ils ne diffèrent que d'UN
        // bit, c'est-à-dire sur un seul axe. Pour chaque coin et chaque axe,
        // on ne trace l'arête que depuis le coin où ce bit vaut 0 (sinon
        // chaque arête serait tracée deux fois).
        for i in 0..8 {
            for bit in 0..3 {
                if i & (1 << bit) == 0 {
                    self.line(corner(i), corner(i | (1 << bit)), color);
                }
            }
        }
    }

    /// Supprime toutes les lignes (en gardant la mémoire pour l'image suivante).
    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    /// Nombre de lignes enregistrées.
    pub fn line_count(&self) -> usize {
        self.vertices.len() / 2
    }

    /// Les sommets, à envoyer au GPU.
    pub(crate) fn vertices(&self) -> &[LineVertex] {
        &self.vertices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taille_du_sommet_de_ligne() {
        assert_eq!(std::mem::size_of::<LineVertex>(), 24);
    }

    #[test]
    fn grille_de_4_cases() {
        let mut debug = DebugLines::default();
        debug.grid(Vec3::ZERO, 4.0, 4, Color::WHITE);
        // 5 lignes dans chaque direction.
        assert_eq!(debug.line_count(), 10);
        // La première ligne longe le bord x = -2.
        assert_eq!(debug.vertices()[0].position, [-2.0, 0.0, -2.0]);
    }

    #[test]
    fn axes_en_rouge_vert_bleu() {
        let mut debug = DebugLines::default();
        debug.axes(Vec3::ZERO, 2.0);
        let v = debug.vertices();
        assert_eq!(v[1].position, [2.0, 0.0, 0.0]);
        assert_eq!(v[1].color, [1.0, 0.0, 0.0]);
        assert_eq!(v[3].color, [0.0, 1.0, 0.0]);
        assert_eq!(v[5].color, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn boite_a_12_aretes_de_longueur_1() {
        let mut debug = DebugLines::default();
        debug.wire_box(Mat4::IDENTITY, Color::WHITE);
        assert_eq!(debug.line_count(), 12);
        for edge in debug.vertices().chunks(2) {
            let length = Vec3::from(edge[0].position).distance(Vec3::from(edge[1].position));
            assert!((length - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn clear_vide_tout() {
        let mut debug = DebugLines::default();
        debug.axes(Vec3::ZERO, 1.0);
        debug.clear();
        assert_eq!(debug.line_count(), 0);
    }
}
