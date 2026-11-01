//! [`Transform`] : la position, l'orientation et la taille d'un objet.

use crate::math::{Mat4, Quat, Vec3};

/// Où se trouve un objet dans le monde, comment il est tourné, et sa taille.
///
/// On stocke les trois composantes **séparément** (et non une matrice) car
/// c'est bien plus pratique à modifier : `transform.translation.x += 1.0`.
/// La matrice n'est calculée qu'au moment du rendu, avec [`Transform::matrix`].
///
/// # Exemple
///
/// ```
/// use zentrix_core::Transform;
/// use zentrix_core::math::Vec3;
///
/// let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
/// // Le point d'origine de l'objet se retrouve en (1, 2, 3) dans le monde.
/// assert_eq!(t.matrix().transform_point3(Vec3::ZERO), Vec3::new(1.0, 2.0, 3.0));
/// ```
// `Serialize` / `Deserialize` (crate `serde`) : le type peut être converti
// en texte (RON, JSON...) et reconstruit, pour sauvegarder les scènes.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transform {
    /// Position dans le monde.
    pub translation: Vec3,
    /// Orientation (un quaternion unitaire).
    pub rotation: Quat,
    /// Facteur de taille sur chaque axe (`Vec3::ONE` = taille d'origine).
    pub scale: Vec3,
}

impl Transform {
    /// La transformation « neutre » : à l'origine, sans rotation, taille 1.
    pub const IDENTITY: Transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Une transformation qui ne fait que déplacer l'objet.
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    /// Décompose une matrice en position, rotation et taille (l'inverse de
    /// [`Transform::matrix`]). Utile pour les transformations venues d'un
    /// fichier (glTF), souvent stockées sous forme de matrice.
    ///
    /// Une matrice avec du **cisaillement** (déformation en biais, rare) ne
    /// peut pas s'écrire ainsi : le résultat n'en est qu'une approximation.
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Variante « builder » : renvoie une copie avec la rotation donnée.
    /// Permet d'enchaîner : `Transform::from_translation(p).with_rotation(r)`.
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Variante « builder » : renvoie une copie avec un facteur de taille
    /// identique sur les trois axes.
    pub fn with_uniform_scale(mut self, scale: f32) -> Self {
        self.scale = Vec3::splat(scale);
        self
    }

    /// La **matrice modèle** (« model matrix ») : elle fait passer un point des
    /// coordonnées de l'objet aux coordonnées du monde.
    ///
    /// L'ordre est fixe : d'abord la taille, puis la rotation, puis le
    /// déplacement (« Scale → Rotate → Translate »). Dans l'autre ordre, l'objet
    /// tournerait autour de l'origine du monde au lieu de tourner sur lui-même.
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_matrix_est_l_inverse_de_matrix() {
        let t = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))
            .with_rotation(Quat::from_rotation_z(0.7))
            .with_uniform_scale(1.5);
        let retour = Transform::from_matrix(t.matrix());
        assert!(retour.translation.abs_diff_eq(t.translation, 1e-5));
        assert!(retour.scale.abs_diff_eq(t.scale, 1e-5));
        assert!(retour.rotation.abs_diff_eq(t.rotation, 1e-5));
    }

    #[test]
    fn identite_ne_change_rien() {
        let p = Vec3::new(1.0, -2.0, 3.0);
        assert_eq!(Transform::IDENTITY.matrix().transform_point3(p), p);
    }

    #[test]
    fn echelle_puis_rotation_puis_translation() {
        // Un point en (1, 0, 0), agrandi ×2, tourné de 90° autour de Y, puis
        // déplacé de (0, 0, -5). Rotation de +90° autour de Y : X → -Z.
        let t = Transform::from_translation(Vec3::new(0.0, 0.0, -5.0))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
            .with_uniform_scale(2.0);
        let p = t.matrix().transform_point3(Vec3::X);
        assert!(
            p.abs_diff_eq(Vec3::new(0.0, 0.0, -7.0), 1e-5),
            "obtenu : {p}"
        );
    }
}
