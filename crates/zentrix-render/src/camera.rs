//! La [`Camera`] : l'œil virtuel qui regarde la scène.

use zentrix_core::math::camera::rh::{proj, view};
use zentrix_core::math::{Mat4, Vec3};

/// Une caméra en **perspective** : les objets lointains paraissent plus petits.
///
/// Elle produit deux matrices, combinées en une seule pour le shader :
///
/// ```text
///  coordonnées      model        monde       view       caméra    projection     clip
///  de l'objet  ──────────────▶  (scène) ──────────────▶ (œil) ──────────────▶  (écran)
///               Transform::matrix()      Camera::view()       Camera::projection()
/// ```
///
/// - **view** : place le monde « devant » la caméra (la caméra est à l'origine
///   et regarde vers -Z) ;
/// - **projection** : applique la perspective et ramène le champ de vision dans
///   le cube standard du GPU (x, y entre -1 et 1, profondeur entre 0 et 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// Où se trouve l'œil.
    pub position: Vec3,
    /// Le point regardé.
    pub target: Vec3,
    /// Quelle direction est « le haut » (presque toujours `Vec3::Y`).
    pub up: Vec3,
    /// Champ de vision **vertical**, en radians. 60° à 75° donne un rendu naturel.
    pub fov_y: f32,
    /// Distance du plan de coupe proche : rien n'est dessiné plus près.
    /// Ne jamais mettre 0 (la précision de profondeur s'effondrerait).
    pub near: f32,
    /// Distance du plan de coupe lointain : rien n'est dessiné plus loin.
    pub far: f32,
}

impl Camera {
    /// La matrice de **vue** : du repère du monde vers le repère de la caméra.
    ///
    /// `rh` = « right-handed », la convention de ZENTRIX (voir `zentrix_core::math`).
    pub fn view(&self) -> Mat4 {
        view::look_at_mat4(self.position, self.target, self.up)
    }

    /// La matrice de **projection**.
    ///
    /// `aspect` = largeur / hauteur de l'image. Sans lui, l'image serait
    /// étirée dès que la fenêtre n'est pas carrée (le défaut du triangle de la v1).
    ///
    /// # Pourquoi `proj::directx` alors qu'on tourne sur Vulkan ?
    ///
    /// Chaque API a sa convention pour l'écran (le « NDC ») : OpenGL met la
    /// profondeur entre -1 et 1, Vulkan pointe l'axe Y vers le **bas**... wgpu
    /// cache ces différences et impose partout la convention WebGPU :
    /// Y vers le haut et profondeur entre 0 et 1, soit exactement celle de
    /// DirectX. Avec `proj::vulkan`, l'image serait à l'envers !
    pub fn projection(&self, aspect: f32) -> Mat4 {
        proj::directx::perspective(self.fov_y, aspect, self.near, self.far)
    }

    /// `projection × view` : la matrice envoyée au vertex shader.
    ///
    /// L'ordre compte ! Les matrices s'appliquent de **droite à gauche** :
    /// d'abord `view`, puis `projection`.
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        self.projection(aspect) * self.view()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 2.0, 6.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov_y: 60f32.to_radians(),
            near: 0.1,
            far: 200.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_cible_est_au_centre_de_l_ecran() {
        let camera = Camera::default();
        // `project_point3` applique la matrice ET la division par w (perspective).
        let ndc = camera
            .view_projection(16.0 / 9.0)
            .project_point3(camera.target);
        assert!(ndc.x.abs() < 1e-5 && ndc.y.abs() < 1e-5, "obtenu : {ndc}");
        // Profondeur entre 0 (plan proche) et 1 (plan lointain).
        assert!(ndc.z > 0.0 && ndc.z < 1.0, "profondeur : {}", ndc.z);
    }

    #[test]
    fn plus_loin_signifie_profondeur_plus_grande() {
        let camera = Camera::default();
        let vp = camera.view_projection(1.0);
        let proche = vp.project_point3(Vec3::new(0.0, 0.0, 2.0));
        let loin = vp.project_point3(Vec3::new(0.0, 0.0, -20.0));
        // C'est exactement ce que teste le depth buffer avec `CompareFunction::Less`.
        assert!(proche.z < loin.z);
    }
}
