//! Les maillages (« meshes ») : la géométrie des objets.
//!
//! Deux types, deux mondes :
//!
//! - [`MeshData`] vit en **mémoire CPU** (de simples `Vec`). On peut le créer,
//!   le modifier, le tester sans GPU.
//! - [`Mesh`] vit en **mémoire GPU** (des buffers). Il est créé à partir d'un
//!   `MeshData` puis rangé dans le `Renderer`, qui rend en échange un
//!   [`MeshHandle`].

use zentrix_rhi::wgpu;
use zentrix_rhi::wgpu::util::DeviceExt;

use crate::vertex::Vertex;

/// Une géométrie en mémoire CPU : des sommets et des indices.
///
/// # Pourquoi des indices ?
///
/// Un cube a 8 coins mais 12 triangles, soit 36 sommets si on les écrit tous.
/// Avec un **index buffer**, chaque sommet n'est stocké qu'une fois et les
/// triangles le désignent par son numéro :
///
/// ```text
///   3 ──── 2      sommets : [0, 1, 2, 3]
///   │   ╱  │      indices : [0, 1, 2,   0, 2, 3]
///   │  ╱   │                 └ triangle ┘ └ triangle ┘
///   0 ──── 1
/// ```
///
/// (Notre cube a quand même 24 sommets et non 8 : chaque face a sa propre
/// couleur, donc un coin partagé par 3 faces existe en 3 exemplaires.)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    /// Des `u32` (jusqu'à 4 milliards de sommets) plutôt que des `u16`
    /// (65 535 au maximum) : les modèles chargés plus tard dépasseront vite
    /// cette limite. Le coût : 2 octets de plus par indice.
    pub indices: Vec<u32>,
}

/// Les UV des 4 coins d'une face rectangulaire, dans l'ordre où nos faces
/// listent leurs coins : bas-gauche, bas-droite, haut-droite, haut-gauche.
/// (v = 0 est le HAUT de l'image, d'où le `1.0` pour les coins du bas.)
const QUAD_UVS: [[f32; 2]; 4] = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

impl MeshData {
    /// Le triangle historique de ZENTRIX (rouge, vert, bleu), dans le plan XY.
    pub fn triangle() -> Self {
        Self {
            vertices: vec![
                Vertex::new([-0.5, -0.5, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0]),
                Vertex::new([0.5, -0.5, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
                Vertex::new([0.0, 0.5, 0.0], [0.0, 0.0, 1.0], [0.5, 0.0]),
            ],
            indices: vec![0, 1, 2],
        }
    }

    /// Un carré de côté 1 couché au sol (plan XZ), face visible vers le haut (+Y).
    /// La texture le recouvre une fois (UV de 0 à 1).
    pub fn plane(color: [f32; 3]) -> Self {
        let corners = [
            [-0.5, 0.0, 0.5],
            [0.5, 0.0, 0.5],
            [0.5, 0.0, -0.5],
            [-0.5, 0.0, -0.5],
        ];
        Self {
            // `zip` avance dans deux tableaux à la fois : (coin 0, uv 0), (coin 1, uv 1)...
            vertices: corners
                .into_iter()
                .zip(QUAD_UVS)
                .map(|(position, uv)| Vertex::new(position, color, uv))
                .collect(),
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }

    /// Un cube de côté 1 centré sur l'origine, avec une couleur par face.
    /// Chaque face reçoit la texture entière.
    pub fn cube() -> Self {
        // Chaque face : sa couleur et ses 4 coins, listés dans le sens
        // anti-horaire QUAND ON REGARDE LA FACE DE L'EXTÉRIEUR (FrontFace::Ccw).
        // `0.5` et `-0.5` sont abrégés en `P` et `N` pour rester lisible.
        const P: f32 = 0.5;
        const N: f32 = -0.5;
        #[rustfmt::skip]
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            ([0.90, 0.30, 0.30], [[N, N, P], [P, N, P], [P, P, P], [N, P, P]]), // +Z avant   : rouge
            ([0.30, 0.80, 0.80], [[P, N, N], [N, N, N], [N, P, N], [P, P, N]]), // -Z arrière : cyan
            ([0.30, 0.85, 0.40], [[P, N, P], [P, N, N], [P, P, N], [P, P, P]]), // +X droite  : vert
            ([0.85, 0.40, 0.85], [[N, N, N], [N, N, P], [N, P, P], [N, P, N]]), // -X gauche  : magenta
            ([0.35, 0.45, 0.95], [[N, P, P], [P, P, P], [P, P, N], [N, P, N]]), // +Y dessus  : bleu
            ([0.95, 0.80, 0.30], [[N, N, N], [P, N, N], [P, N, P], [N, N, P]]), // -Y dessous : jaune
        ];

        let mut mesh = MeshData::default();
        for (color, corners) in faces {
            // Numéro du premier coin de cette face dans `vertices`.
            let base = mesh.vertices.len() as u32;
            mesh.vertices.extend(
                corners
                    .into_iter()
                    .zip(QUAD_UVS)
                    .map(|(position, uv)| Vertex::new(position, color, uv)),
            );
            // Deux triangles par face : (0, 1, 2) et (0, 2, 3).
            mesh.indices.extend([0, 1, 2, 0, 2, 3].map(|i| base + i));
        }
        mesh
    }

    /// Variante « builder » : donne la même couleur à tous les sommets.
    /// Avec du blanc, la texture apparaît avec ses vraies couleurs.
    pub fn with_color(mut self, color: [f32; 3]) -> Self {
        for vertex in &mut self.vertices {
            vertex.color = color;
        }
        self
    }

    /// Variante « builder » : multiplie les UV, pour **répéter** la texture.
    /// `with_uv_scale(8.0)` sur un sol : la texture y apparaît 8 × 8 fois.
    pub fn with_uv_scale(mut self, scale: f32) -> Self {
        for vertex in &mut self.vertices {
            vertex.uv = vertex.uv.map(|c| c * scale);
        }
        self
    }
}

/// Une référence légère vers un maillage stocké dans le `Renderer`.
///
/// C'est un simple numéro (`usize`), donc `Copy` et gratuit à dupliquer :
/// 1000 cubes peuvent partager le même `MeshHandle`, la géométrie n'existe
/// qu'une fois sur le GPU. Le champ est privé : impossible de fabriquer un
/// handle invalide depuis l'extérieur de la crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshHandle(pub(crate) usize);

/// Un maillage chargé sur le GPU.
pub(crate) struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    /// Copie un [`MeshData`] dans deux buffers GPU.
    pub fn upload(device: &wgpu::Device, data: &MeshData) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&data.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&data.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            index_count: data.indices.len() as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zentrix_core::math::Vec3;

    #[test]
    fn cube_24_sommets_36_indices() {
        let cube = MeshData::cube();
        assert_eq!(cube.vertices.len(), 24);
        assert_eq!(cube.indices.len(), 36);
        assert!(
            cube.indices
                .iter()
                .all(|&i| (i as usize) < cube.vertices.len())
        );
    }

    #[test]
    fn toutes_les_faces_du_cube_regardent_vers_l_exterieur() {
        // Si un triangle est dans le mauvais sens, le « backface culling » le
        // cache et on voit un trou dans le cube. Ce test l'attrape sans GPU.
        let cube = MeshData::cube();
        let pos = |i: u32| Vec3::from(cube.vertices[i as usize].position);
        for tri in cube.indices.chunks(3) {
            let (a, b, c) = (pos(tri[0]), pos(tri[1]), pos(tri[2]));
            // Le produit vectoriel donne la normale du triangle (règle de la main droite).
            let normal = (b - a).cross(c - a);
            let center = (a + b + c) / 3.0;
            assert!(normal.dot(center) > 0.0, "triangle {tri:?} à l'envers");
        }
    }
}
