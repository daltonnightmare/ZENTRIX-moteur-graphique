//! Les **volumes englobants** et la **pyramide de vision** : de quoi savoir,
//! sans rien dessiner, si un objet peut être vu.
//!
//! ```text
//!   vue de dessus           plan lointain
//!                      ┌─────────────────────┐
//!                       \     ■ visible     /
//!            ■ caché     \                 /      ■ caché
//!                         \       ■ ──────/───  à cheval : gardé
//!                          \             /
//!                           └─────┬─────┘  plan proche
//!                                 👁 caméra
//! ```
//!
//! Le *frustum culling* (« élagage par la pyramide de vision ») écarte les
//! objets entièrement hors de cette pyramide : le GPU ne recevra même pas
//! leurs triangles. Le test est **prudent** : un objet douteux est gardé.
//! On peut dessiner un objet invisible pour rien, jamais oublier un objet
//! visible.

use crate::math::{Mat4, Vec3, Vec4};

/// Une boîte alignée sur les axes (*Axis-Aligned Bounding Box*) : la plus
/// petite boîte « droite » qui contient un objet. Deux coins suffisent à la
/// décrire.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub const fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// La boîte qui contient tous ces points. `None` s'il n'y en a aucun.
    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        let mut points = points.into_iter();
        let first = points.next()?;
        Some(points.fold(Self::new(first, first), |aabb, p| Self {
            min: aabb.min.min(p),
            max: aabb.max.max(p),
        }))
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// La demi-taille : du centre jusqu'au coin `max`.
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// La boîte, une fois l'objet déplacé, tourné et agrandi par `matrix`.
    ///
    /// Tourner une boîte donne une boîte penchée, plus « droite ». On prend
    /// donc la boîte droite qui contient la boîte penchée (un peu plus
    /// grande, ce qui reste prudent). La méthode de Jim Arvo évite de
    /// transformer les 8 coins : la demi-taille sur chaque axe est la somme
    /// des demi-tailles d'origine, pondérées par la valeur **absolue** de la
    /// rotation.
    pub fn transformed(&self, matrix: &Mat4) -> Self {
        let center = matrix.transform_point3(self.center());
        let half = self.half_extents();
        let extent = matrix.x_axis.truncate().abs() * half.x
            + matrix.y_axis.truncate().abs() * half.y
            + matrix.z_axis.truncate().abs() * half.z;
        Self::new(center - extent, center + extent)
    }
}

/// La **pyramide de vision** (*frustum*) d'une caméra : 6 plans (gauche,
/// droite, bas, haut, proche, lointain) dont les normales pointent vers
/// l'intérieur.
///
/// Un plan est rangé dans un `Vec4` (a, b, c, d) : un point `p` est du bon
/// côté si `a·x + b·y + c·z + d >= 0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frustum {
    pub planes: [Vec4; 6],
}

impl Frustum {
    /// Extrait les 6 plans d'une matrice `projection × vue` (méthode de
    /// Gribb et Hartmann).
    ///
    /// Un point est visible si, une fois multiplié par la matrice, ses
    /// coordonnées (x, y, z, w) vérifient `-w <= x <= w`, `-w <= y <= w` et
    /// `0 <= z <= w` (la profondeur va de 0 à 1 avec wgpu). Chaque inégalité
    /// est un plan : `w + x >= 0` donne le plan de gauche, `w - x >= 0` celui
    /// de droite... Avec les lignes `r0..r3` de la matrice, `x = r0·p` et
    /// `w = r3·p` : le plan de gauche est donc simplement `r3 + r0`.
    ///
    /// Marche aussi pour une projection orthographique (la lumière du soleil).
    pub fn from_view_projection(m: &Mat4) -> Self {
        let (r0, r1, r2, r3) = (m.row(0), m.row(1), m.row(2), m.row(3));
        let planes = [r3 + r0, r3 - r0, r3 + r1, r3 - r1, r2, r3 - r2].map(|plane| {
            // Normaliser n'est pas nécessaire au test, mais donne des
            // distances en vraies unités (utile pour déboguer).
            let length = plane.truncate().length();
            if length > 0.0 { plane / length } else { plane }
        });
        Self { planes }
    }

    /// La boîte touche-t-elle (au moins en partie) la pyramide ?
    ///
    /// Pour chaque plan, on prend le coin de la boîte le plus « en avant »
    /// selon la normale du plan. S'il est déjà derrière le plan, toute la
    /// boîte l'est : elle est invisible. Prudent : une boîte près d'un coin
    /// de la pyramide peut être gardée sans être vraiment visible.
    pub fn intersects(&self, aabb: &Aabb) -> bool {
        self.planes.iter().all(|plane| {
            let normal = plane.truncate();
            let corner = Vec3::select(normal.cmpge(Vec3::ZERO), aabb.max, aabb.min);
            normal.dot(corner) + plane.w >= 0.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Quat;
    use crate::math::camera::rh::{proj, view};

    fn camera() -> Frustum {
        // L'œil en (0, 0, 5), qui regarde l'origine (donc vers -Z).
        let view = view::look_at_mat4(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        let proj = proj::directx::perspective(60f32.to_radians(), 1.0, 0.1, 100.0);
        Frustum::from_view_projection(&(proj * view))
    }

    fn cube_at(center: Vec3) -> Aabb {
        Aabb::new(center - 0.5, center + 0.5)
    }

    #[test]
    fn boite_de_points() {
        let aabb = Aabb::from_points([
            Vec3::new(1.0, -2.0, 0.0),
            Vec3::new(-1.0, 3.0, 0.5),
            Vec3::ZERO,
        ])
        .unwrap();
        assert_eq!(aabb.min, Vec3::new(-1.0, -2.0, 0.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 3.0, 0.5));
        assert_eq!(Aabb::from_points([]), None);
    }

    #[test]
    fn transformer_une_boite() {
        let unit = cube_at(Vec3::ZERO);
        // Déplacée et agrandie : facile.
        let moved = unit.transformed(&Mat4::from_scale_rotation_translation(
            Vec3::new(2.0, 1.0, 1.0),
            Quat::IDENTITY,
            Vec3::new(10.0, 0.0, 0.0),
        ));
        assert!((moved.min - Vec3::new(9.0, -0.5, -0.5)).length() < 1e-5);
        assert!((moved.max - Vec3::new(11.0, 0.5, 0.5)).length() < 1e-5);
        // Tournée de 45° autour de Y : la boîte droite qui la contient est
        // plus large, de demi-taille √2 / 2 × 2 = 0,707.
        let turned = unit.transformed(&Mat4::from_rotation_y(45f32.to_radians()));
        assert!(
            (turned.max.x - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-5,
            "{}",
            turned.max.x
        );
        assert!((turned.max.y - 0.5).abs() < 1e-5);
    }

    #[test]
    fn devant_la_camera_est_visible() {
        let frustum = camera();
        assert!(frustum.intersects(&cube_at(Vec3::ZERO)));
        assert!(frustum.intersects(&cube_at(Vec3::new(0.0, 0.0, -50.0))));
    }

    #[test]
    fn derriere_a_cote_et_trop_loin_sont_caches() {
        let frustum = camera();
        assert!(
            !frustum.intersects(&cube_at(Vec3::new(0.0, 0.0, 10.0))),
            "derrière"
        );
        assert!(
            !frustum.intersects(&cube_at(Vec3::new(30.0, 0.0, 0.0))),
            "à droite"
        );
        assert!(
            !frustum.intersects(&cube_at(Vec3::new(0.0, -30.0, 0.0))),
            "en bas"
        );
        assert!(
            !frustum.intersects(&cube_at(Vec3::new(0.0, 0.0, -200.0))),
            "trop loin"
        );
    }

    #[test]
    fn un_objet_a_cheval_sur_le_bord_est_garde() {
        let frustum = camera();
        // La face arrière du cube est à 5,5 m de l'œil, où le demi-champ
        // vertical de 30° monte jusqu'à tan(30°) × 5,5 = 3,18 m. Centré à
        // 3,2 m, le bas du cube (2,7 m) entre dans le champ : il est gardé.
        assert!(frustum.intersects(&cube_at(Vec3::new(0.0, 3.2, 0.0))));
        // Centré à 4 m, son bas (3,5 m) reste au-dessus : caché.
        assert!(!frustum.intersects(&cube_at(Vec3::new(0.0, 4.0, 0.0))));
    }

    #[test]
    fn projection_orthographique() {
        // Une boîte de -10 à 10 en x et y, profondeur de 0 à 20 devant l'œil.
        let view = view::look_at_mat4(Vec3::new(0.0, 10.0, 0.0), Vec3::ZERO, Vec3::Z);
        let proj = proj::directx::orthographic(-10.0, 10.0, -10.0, 10.0, 0.0, 20.0);
        let frustum = Frustum::from_view_projection(&(proj * view));
        assert!(frustum.intersects(&cube_at(Vec3::new(9.0, 0.0, 9.0))));
        assert!(!frustum.intersects(&cube_at(Vec3::new(12.0, 0.0, 0.0))));
        assert!(
            !frustum.intersects(&cube_at(Vec3::new(0.0, 12.0, 0.0))),
            "derrière l'œil"
        );
    }
}
