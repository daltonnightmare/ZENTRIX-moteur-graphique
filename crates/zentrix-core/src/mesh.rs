//! La géométrie en mémoire CPU : [`Vertex`] et [`MeshData`].
//!
//! Ce sont de simples données : `zentrix-assets` en fabrique (modèles glTF),
//! `zentrix-render` les envoie au GPU. Comme pour `Image`, elles vivent dans
//! le noyau pour que ces deux crates n'aient pas à dépendre l'une de l'autre.
//! La description du format pour le GPU (le « vertex layout » wgpu) reste
//! dans `zentrix-render`.

use crate::bounds::Aabb;
use crate::math::Vec3;

/// Un sommet : une position 3D, une couleur, une coordonnée de texture et une normale.
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
/// En mémoire, un `Vertex` occupe 60 octets :
///
/// ```text
///  octet : 0          12         24      32         44            60
///          ├──────────┼──────────┼───────┼──────────┼─────────────┤
///          │ x  y  z  │ r  g  b  │ u  v  │ nx ny nz │ tx ty tz tw │
///          └──────────┴──────────┴───────┴──────────┴─────────────┘
///           position   color      uv      normal     tangent
///           (loc. 0)   (loc. 1)   (loc.2) (loc. 3)   (loc. 10)
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
    /// La **normale** : un vecteur de longueur 1, perpendiculaire à la
    /// surface, qui indique vers où elle « regarde ». C'est elle qui permet de
    /// calculer l'éclairage : une face tournée vers la lumière est claire,
    /// une face qui lui tourne le dos est sombre.
    ///
    /// ```text
    ///          ▲ normale
    ///          │
    ///   ───────┴───────  surface
    /// ```
    pub normal: [f32; 3],
    /// La **tangente** (xyz) : la direction, sur la surface, dans laquelle
    /// `u` augmente. Avec la normale, elle oriente une **normal map** (voir
    /// [`MeshData::compute_tangents`]). `w` vaut 1 ou -1 : il dit dans quel
    /// sens va la troisième direction (la « bitangente » = normale × tangente
    /// × w), car une texture peut être plaquée en miroir.
    ///
    /// `[0.0; 4]` = inconnue : le shader ignore alors la normal map.
    pub tangent: [f32; 4],
}

impl Vertex {
    /// Construit un sommet. `const fn` : utilisable dans des constantes.
    ///
    /// Si on ne connaît pas encore la normale, on peut passer `[0.0; 3]` puis
    /// appeler `MeshData::compute_normals` une fois le maillage complet. La
    /// tangente, elle, est calculée par `MeshData::compute_tangents`.
    pub const fn new(position: [f32; 3], color: [f32; 3], uv: [f32; 2], normal: [f32; 3]) -> Self {
        Self {
            position,
            color,
            uv,
            normal,
            tangent: [0.0; 4],
        }
    }
}

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
                Vertex::new(
                    [-0.5, -0.5, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0],
                    [0.0, 0.0, 1.0],
                ),
                Vertex::new(
                    [0.5, -0.5, 0.0],
                    [0.0, 1.0, 0.0],
                    [1.0, 1.0],
                    [0.0, 0.0, 1.0],
                ),
                Vertex::new(
                    [0.0, 0.5, 0.0],
                    [0.0, 0.0, 1.0],
                    [0.5, 0.0],
                    [0.0, 0.0, 1.0],
                ),
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
        let mut mesh = Self {
            // `zip` avance dans deux tableaux à la fois : (coin 0, uv 0), (coin 1, uv 1)...
            vertices: corners
                .into_iter()
                .zip(QUAD_UVS)
                .map(|(position, uv)| Vertex::new(position, color, uv, [0.0, 1.0, 0.0]))
                .collect(),
            indices: vec![0, 1, 2, 0, 2, 3],
        };
        mesh.compute_tangents();
        mesh
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
                    .map(|(position, uv)| Vertex::new(position, color, uv, [0.0; 3])),
            );
            // Deux triangles par face : (0, 1, 2) et (0, 2, 3).
            mesh.indices.extend([0, 1, 2, 0, 2, 3].map(|i| base + i));
        }
        // Chaque face a ses propres sommets : la moyenne des normales autour
        // d'un sommet est donc exactement la normale de sa face.
        mesh.compute_normals();
        mesh.compute_tangents();
        mesh
    }

    /// Une sphère blanche de rayon 0,5 centrée sur l'origine (« sphère UV »,
    /// comme un globe terrestre) : `rings` tranches de latitude du pôle nord
    /// au pôle sud, `segments` quartiers de longitude.
    ///
    /// Une surface courbe montre tout l'éventail des angles entre la normale
    /// et la lumière : c'est l'objet idéal pour juger un matériau.
    pub fn sphere(segments: u32, rings: u32) -> Self {
        use std::f32::consts::{PI, TAU};
        let (segments, rings) = (segments.max(3), rings.max(2));

        let mut mesh = MeshData::default();
        // Une colonne de sommets de plus que de quartiers : la première et la
        // dernière sont au même endroit, mais avec u = 0 et u = 1 (sans quoi
        // la texture ferait tout le tour à l'envers dans le dernier quartier).
        for i in 0..=rings {
            let theta = PI * i as f32 / rings as f32; // 0 au pôle nord
            for j in 0..=segments {
                let phi = TAU * j as f32 / segments as f32;
                let normal = [
                    theta.sin() * phi.sin(),
                    theta.cos(),
                    theta.sin() * phi.cos(),
                ];
                mesh.vertices.push(Vertex::new(
                    normal.map(|c| c * 0.5),
                    [1.0; 3],
                    [j as f32 / segments as f32, i as f32 / rings as f32],
                    // Sur une sphère centrée, la normale est la direction du sommet.
                    normal,
                ));
            }
        }

        let row = segments + 1;
        for i in 0..rings {
            for j in 0..segments {
                // Le quadrilatère (i, j) vu de l'extérieur :
                //   a ─ d     a = haut gauche, d = haut droite
                //   │ ╲ │
                //   b ─ c     (ligne i + 1, plus au sud)
                let a = i * row + j;
                let (b, c, d) = (a + row, a + row + 1, a + 1);
                // Aux pôles, deux coins sont confondus : un seul triangle.
                if i != rings - 1 {
                    mesh.indices.extend([a, b, c]);
                }
                if i != 0 {
                    mesh.indices.extend([a, c, d]);
                }
            }
        }
        mesh.compute_tangents();
        mesh
    }

    /// Calcule la normale de chaque sommet à partir des triangles.
    ///
    /// Pour chaque triangle, le produit vectoriel de deux de ses côtés donne
    /// un vecteur perpendiculaire (règle de la main droite : il sort de la
    /// face avant, celle dont les sommets tournent dans le sens anti-horaire).
    /// Sa longueur vaut le double de l'aire du triangle : en additionnant ces
    /// vecteurs sans les normaliser, les grands triangles comptent davantage.
    ///
    /// Un sommet partagé par plusieurs triangles reçoit la moyenne de leurs
    /// normales : la surface paraît **lisse** (une sphère). Pour des arêtes
    /// **vives** (un cube), chaque face doit avoir ses propres sommets.
    pub fn compute_normals(&mut self) {
        let mut normals = vec![Vec3::ZERO; self.vertices.len()];
        // `as_chunks::<3>()` découpe les indices en tableaux `[u32; 3]` (un par
        // triangle) ; `.0` ignore un éventuel reste incomplet.
        for &[a, b, c] in self.indices.as_chunks::<3>().0 {
            let [a, b, c] = [a, b, c].map(|i| i as usize);
            let position = |i: usize| Vec3::from(self.vertices[i].position);
            let face = (position(b) - position(a)).cross(position(c) - position(a));
            for i in [a, b, c] {
                normals[i] += face;
            }
        }
        for (vertex, normal) in self.vertices.iter_mut().zip(normals) {
            // `normalize_or_zero` : un sommet isolé (sans triangle) garde (0, 0, 0).
            vertex.normal = normal.normalize_or_zero().to_array();
        }
    }

    /// Calcule la tangente de chaque sommet à partir des positions, des UV
    /// et des normales (à appeler après `compute_normals` si besoin).
    ///
    /// # À quoi sert une tangente ?
    ///
    /// Une **normal map** est une image dont chaque pixel stocke une normale
    /// « en relief » : rouge = vers la droite de l'image, vert = vers son
    /// haut, bleu = hors de la surface. Pour l'utiliser, le shader doit savoir
    /// où sont « la droite » et « le haut » de l'image **sur l'objet** :
    ///
    /// ```text
    ///        N (normale)
    ///        ▲   ▲ B (bitangente : le haut de l'image, v qui diminue)
    ///        │  ╱
    ///        │ ╱
    ///        └──────▶ T (tangente : la droite de l'image, u qui augmente)
    /// ```
    ///
    /// Pour chaque triangle, on cherche les directions 3D qui correspondent
    /// à « u augmente » et « v diminue », en résolvant un petit système à
    /// partir des différences de positions et d'UV. Comme pour les normales,
    /// chaque sommet reçoit la somme des triangles qui le touchent ; on rend
    /// enfin la tangente perpendiculaire à la normale (Gram-Schmidt).
    pub fn compute_tangents(&mut self) {
        let mut tangents = vec![Vec3::ZERO; self.vertices.len()];
        let mut bitangents = vec![Vec3::ZERO; self.vertices.len()];
        for &[a, b, c] in self.indices.as_chunks::<3>().0 {
            let [va, vb, vc] = [a, b, c].map(|i| self.vertices[i as usize]);
            let e1 = Vec3::from(vb.position) - Vec3::from(va.position);
            let e2 = Vec3::from(vc.position) - Vec3::from(va.position);
            let (du1, dv1) = (vb.uv[0] - va.uv[0], vb.uv[1] - va.uv[1]);
            let (du2, dv2) = (vc.uv[0] - va.uv[0], vc.uv[1] - va.uv[1]);
            // e1 = du1·T + dv1·V et e2 = du2·T + dv2·V, où V suit les v
            // croissants. Le déterminant est nul si les UV du triangle sont
            // alignés (texture « écrasée ») : rien à en tirer.
            let det = du1 * dv2 - du2 * dv1;
            if det.abs() < 1e-12 {
                continue;
            }
            let t = (e1 * dv2 - e2 * dv1) / det;
            let v = (e2 * du1 - e1 * du2) / det;
            for i in [a, b, c] {
                tangents[i as usize] += t;
                // Le haut de l'image est du côté des v DÉCROISSANTS.
                bitangents[i as usize] -= v;
            }
        }
        for ((vertex, t), b) in self.vertices.iter_mut().zip(tangents).zip(bitangents) {
            let n = Vec3::from(vertex.normal);
            // Retirer de `t` sa part dans la direction de la normale.
            let t_perp = (t - n * n.dot(t)).normalize_or_zero();
            // La bitangente « naturelle » (n × t) va-t-elle dans le même sens
            // que celle des UV ? Sinon la texture est en miroir : w = -1.
            let w = if n.cross(t_perp).dot(b) < 0.0 {
                -1.0
            } else {
                1.0
            };
            vertex.tangent = t_perp.extend(w).to_array();
        }
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

    /// La boîte englobante des sommets, dans le repère de l'objet. Sert au
    /// *culling* : le rendu la calcule une fois, à l'envoi du maillage.
    /// Un maillage vide donne une boîte réduite à l'origine.
    pub fn bounds(&self) -> Aabb {
        Aabb::from_points(self.vertices.iter().map(|v| Vec3::from(v.position)))
            .unwrap_or(Aabb::new(Vec3::ZERO, Vec3::ZERO))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boite_englobante() {
        let cube = MeshData::cube().bounds();
        assert_eq!(cube.min, Vec3::splat(-0.5));
        assert_eq!(cube.max, Vec3::splat(0.5));
        // Le sol est plat : une boîte d'épaisseur nulle.
        let plane = MeshData::plane([1.0; 3]).bounds();
        assert_eq!(plane.min.y, 0.0);
        assert_eq!(plane.max.y, 0.0);
    }

    #[test]
    fn taille_du_vertex() {
        // 15 flottants de 4 octets = 60 octets. Si ce test casse, le shader
        // (mesh.wgsl) et le layout de zentrix-render ne correspondent plus !
        assert_eq!(std::mem::size_of::<Vertex>(), 60);
    }

    #[test]
    fn les_triangles_de_la_sphere_regardent_vers_l_exterieur() {
        let sphere = MeshData::sphere(16, 8);
        // Pôles : 1 triangle par quartier ; ailleurs : 2.
        assert_eq!(sphere.indices.len(), 3 * 16 * (2 * 8 - 2));
        for &[a, b, c] in sphere.indices.as_chunks::<3>().0 {
            let [a, b, c] = [a, b, c].map(|i| Vec3::from(sphere.vertices[i as usize].position));
            // Sens anti-horaire vu de l'extérieur : la normale géométrique
            // pointe dans le même sens que la position (vers l'extérieur).
            let face_normal = (b - a).cross(c - a);
            assert!(face_normal.dot(a + b + c) > 0.0, "triangle à l'envers");
        }
        for v in &sphere.vertices {
            assert!((Vec3::from(v.position).length() - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn tangentes_du_cube() {
        let cube = MeshData::cube();
        for v in &cube.vertices {
            let (n, t) = (Vec3::from(v.normal), Vec3::from_slice(&v.tangent[..3]));
            assert!((t.length() - 1.0).abs() < 1e-5, "tangente unitaire");
            assert!(
                n.dot(t).abs() < 1e-5,
                "tangente perpendiculaire à la normale"
            );
            assert_eq!(v.tangent[3].abs(), 1.0);
        }
        // Face avant (+Z) : u augmente vers +X, le haut de l'image est +Y.
        let avant = &cube.vertices[0];
        assert_eq!(avant.tangent, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn une_texture_en_miroir_donne_w_negatif() {
        let mut quad = MeshData::plane([1.0; 3]);
        for v in &mut quad.vertices {
            v.uv[0] = 1.0 - v.uv[0]; // u inversé
        }
        quad.compute_tangents();
        assert!(quad.vertices.iter().all(|v| v.tangent[3] == -1.0));
    }

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
    fn les_normales_du_cube_sortent_des_faces() {
        let cube = MeshData::cube();
        for vertex in &cube.vertices {
            let normal = Vec3::from(vertex.normal);
            assert!(
                (normal.length() - 1.0).abs() < 1e-6,
                "normale de longueur 1"
            );
            // La normale d'une face de cube est l'un des 6 axes, et pointe
            // du même côté que le sommet (vers l'extérieur).
            assert!(normal.dot(Vec3::from(vertex.position)) > 0.0);
            assert_eq!(normal.abs().max_element(), 1.0, "{normal}");
        }
    }

    #[test]
    fn normales_lissees_sur_un_sommet_partage() {
        // Deux triangles en « toit » partagent l'arête 0-1 : la normale des
        // sommets partagés est la moyenne des deux pentes, donc verticale.
        let mut toit = MeshData {
            vertices: [
                [0.0, 1.0, 0.0],
                [0.0, 1.0, -1.0],
                [-1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            ]
            .map(|p| Vertex::new(p, [1.0; 3], [0.0; 2], [0.0; 3]))
            .to_vec(),
            // Sens anti-horaire vu du dessus : pente gauche (0, 1, 2), pente droite (0, 3, 1).
            indices: vec![0, 1, 2, 0, 3, 1],
        };
        toit.compute_normals();
        let partage = Vec3::from(toit.vertices[0].normal);
        assert!(partage.abs_diff_eq(Vec3::Y, 1e-6), "{partage}");
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
