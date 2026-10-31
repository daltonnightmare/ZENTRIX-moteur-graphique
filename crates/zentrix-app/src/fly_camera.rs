//! [`FlyCamera`] : une caméra libre pilotée au clavier et à la souris.
//!
//! Elle vit dans `zentrix-app` car c'est la seule crate qui connaît à la fois
//! les entrées (`zentrix-platform`) et la caméra (`zentrix-render`) : les
//! couches du moteur restent indépendantes les unes des autres.

use zentrix_core::math::Vec3;
use zentrix_platform::{Input, KeyCode};
use zentrix_render::Camera;

/// Limite du regard vers le haut ou le bas : un peu moins de 90°. À 90° pile,
/// la direction regardée serait parallèle au « haut » de la caméra, et la
/// matrice de vue deviendrait indéfinie (l'image se retournerait).
const MAX_PITCH: f32 = 89f32.to_radians();

/// Une caméra « fantôme » : elle vole librement, sans gravité ni collision.
///
/// # L'orientation : lacet et tangage
///
/// Plutôt que de stocker une direction, on stocke deux angles, faciles à
/// modifier avec la souris :
///
/// ```text
///   yaw (lacet) : tourner la tête à gauche / à droite   ← souris horizontale
///   pitch (tangage) : lever / baisser la tête           ← souris verticale
/// ```
///
/// Avec `yaw = 0` et `pitch = 0`, la caméra regarde vers -Z (convention de ZENTRIX).
#[derive(Debug, Clone)]
pub struct FlyCamera {
    pub position: Vec3,
    /// Lacet en radians. Positif = tourné vers la droite (+X).
    pub yaw: f32,
    /// Tangage en radians. Positif = vers le haut.
    pub pitch: f32,
    /// Vitesse de déplacement en unités par seconde.
    pub speed: f32,
    /// Radians de rotation par unité de déplacement brut de la souris.
    pub sensitivity: f32,
}

impl FlyCamera {
    /// Crée une caméra placée en `position` et tournée vers `target`.
    pub fn looking_at(position: Vec3, target: Vec3) -> Self {
        let dir = (target - position).normalize();
        Self {
            position,
            // On inverse les formules de `forward()` : `asin` retrouve
            // l'angle à partir de la hauteur, `atan2` à partir de x et z.
            yaw: dir.x.atan2(-dir.z),
            pitch: dir.y.asin().clamp(-MAX_PITCH, MAX_PITCH),
            speed: 5.0,
            sensitivity: 0.002,
        }
    }

    /// La direction regardée (vecteur de longueur 1).
    ///
    /// Trigonométrie : `cos(pitch)` est la part horizontale de la direction,
    /// `sin(pitch)` sa part verticale. La part horizontale est ensuite
    /// répartie entre X et -Z selon le lacet.
    pub fn forward(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        Vec3::new(cos_pitch * sin_yaw, sin_pitch, -cos_pitch * cos_yaw)
    }

    /// La direction « à droite » de la caméra, toujours horizontale.
    ///
    /// Le produit vectoriel de deux vecteurs donne un vecteur perpendiculaire
    /// aux deux : ici perpendiculaire à « devant » et à « en haut ».
    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    /// Applique les entrées de cette image.
    ///
    /// - `look` : la souris doit-elle tourner la caméra ? (vrai quand le
    ///   curseur est capturé, sinon bouger la souris vers un autre programme
    ///   ferait tourner la vue) ;
    /// - contrôles **physiques** : `KeyW/A/S/D` = ZQSD en AZERTY, WASD en QWERTY.
    pub fn update(&mut self, input: &Input, dt: f32, look: bool) {
        // --- Orientation ------------------------------------------------------
        if look {
            let delta = input.mouse_delta();
            self.yaw += delta.x * self.sensitivity;
            // La souris vers le bas donne un y positif : on soustrait pour
            // que pousser la souris vers l'avant lève le regard.
            self.pitch = (self.pitch - delta.y * self.sensitivity).clamp(-MAX_PITCH, MAX_PITCH);
        }

        // --- Vitesse : la molette la multiplie ou la divise par 1,2 par cran --
        let scroll = input.scroll_delta();
        if scroll != 0.0 {
            self.speed = (self.speed * 1.2f32.powf(scroll)).clamp(0.5, 100.0);
            log::info!("Vitesse de la caméra : {:.1} unités/s", self.speed);
        }

        // --- Déplacement ------------------------------------------------------
        // On additionne les directions demandées, puis on normalise : sans
        // cela, avancer en diagonale (Z + D) irait 1,41 fois plus vite.
        let mut direction = Vec3::ZERO;
        let axes = [
            (KeyCode::KeyW, self.forward()),  // Z en AZERTY
            (KeyCode::KeyS, -self.forward()), // S
            (KeyCode::KeyD, self.right()),    // D
            (KeyCode::KeyA, -self.right()),   // Q en AZERTY
            (KeyCode::Space, Vec3::Y),        // monter
            (KeyCode::ControlLeft, -Vec3::Y), // descendre
        ];
        for (key, axis) in axes {
            if input.is_key_down(key) {
                direction += axis;
            }
        }

        // Maj gauche : sprint.
        let boost = if input.is_key_down(KeyCode::ShiftLeft) {
            3.0
        } else {
            1.0
        };
        // `normalize_or_zero` : un vecteur nul (aucune touche, ou Z + S) ne
        // peut pas être normalisé ; on obtient alors simplement zéro.
        // Multiplier par `dt` rend la vitesse indépendante du nombre d'images par seconde.
        self.position += direction.normalize_or_zero() * self.speed * boost * dt;
    }

    /// Recopie la position et l'orientation dans la caméra de rendu.
    pub fn apply_to(&self, camera: &mut Camera) {
        camera.position = self.position;
        camera.target = self.position + self.forward();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_nuls_regardent_vers_moins_z() {
        let cam = FlyCamera::looking_at(Vec3::ZERO, -Vec3::Z);
        assert!(cam.yaw.abs() < 1e-6 && cam.pitch.abs() < 1e-6);
        assert!(cam.forward().abs_diff_eq(-Vec3::Z, 1e-6));
        assert!(cam.right().abs_diff_eq(Vec3::X, 1e-6));
    }

    #[test]
    fn looking_at_retrouve_la_direction() {
        let position = Vec3::new(0.0, 4.0, 11.0);
        let target = Vec3::new(2.0, 0.5, -3.0);
        let cam = FlyCamera::looking_at(position, target);
        let attendu = (target - position).normalize();
        assert!(
            cam.forward().abs_diff_eq(attendu, 1e-5),
            "{}",
            cam.forward()
        );
    }

    #[test]
    fn le_tangage_est_borne() {
        // Regarder pile vers le haut est ramené juste sous la verticale.
        let cam = FlyCamera::looking_at(Vec3::ZERO, Vec3::Y);
        assert!(cam.pitch <= MAX_PITCH);
        assert!(cam.right().is_finite());
    }
}
