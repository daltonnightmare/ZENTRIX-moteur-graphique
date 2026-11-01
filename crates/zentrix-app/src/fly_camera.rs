//! [`FlyCamera`] : un composant qui pilote une entité au clavier et à la souris.
//!
//! Il vit dans `zentrix-app` car c'est la seule crate qui connaît à la fois
//! les entrées (`zentrix-platform`) et la scène (`zentrix-scene`) : les
//! couches du moteur restent indépendantes les unes des autres.
//!
//! Dans la scène, l'entité caméra porte trois composants :
//! `Transform` (où elle est), `Camera3d` (son objectif) et `FlyCamera` (qui
//! la pilote). Le système [`FlyCamera::update`] modifie le `Transform`.

use zentrix_core::Transform;
use zentrix_core::math::{Quat, Vec3};
use zentrix_platform::{Input, KeyCode};

/// Limite du regard vers le haut ou le bas : un peu moins de 90°. À 90° pile,
/// la direction regardée serait parallèle au « haut » de la caméra, et la
/// matrice de vue deviendrait indéfinie (l'image se retournerait).
const MAX_PITCH: f32 = 89f32.to_radians();

/// Un contrôleur « fantôme » : l'entité vole librement, sans gravité ni collision.
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
///
/// La position, elle, est dans le `Transform` de l'entité : le contrôleur ne
/// stocke que ce qui lui est propre.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FlyCamera {
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
    /// Crée un contrôleur et le `Transform` d'une entité placée en
    /// `position` et tournée vers `target`.
    pub fn looking_at(position: Vec3, target: Vec3) -> (Self, Transform) {
        let dir = (target - position).normalize();
        let controller = Self {
            // On inverse les formules de `forward()` : `asin` retrouve
            // l'angle à partir de la hauteur, `atan2` à partir de x et z.
            yaw: dir.x.atan2(-dir.z),
            pitch: dir.y.asin().clamp(-MAX_PITCH, MAX_PITCH),
            speed: 5.0,
            sensitivity: 0.002,
        };
        let transform = Transform::from_translation(position).with_rotation(controller.rotation());
        (controller, transform)
    }

    /// L'orientation sous forme de quaternion, pour le `Transform`.
    ///
    /// D'abord le tangage (autour de X : lever la tête), puis le lacet
    /// (autour de Y : tourner la tête). Le lacet est négatif car une rotation
    /// positive autour de Y tourne vers la GAUCHE (règle de la main droite).
    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(-self.yaw) * Quat::from_rotation_x(self.pitch)
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

    /// Le **système** : applique les entrées de cette image au `Transform`
    /// de l'entité pilotée.
    ///
    /// - `look` : la souris doit-elle tourner la caméra ? (vrai quand le
    ///   curseur est capturé, sinon bouger la souris vers un autre programme
    ///   ferait tourner la vue) ;
    /// - contrôles **physiques** : `KeyW/A/S/D` = ZQSD en AZERTY, WASD en QWERTY.
    pub fn update(&mut self, transform: &mut Transform, input: &Input, dt: f32, look: bool) {
        self.look(transform, input, look);

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
        transform.translation += direction.normalize_or_zero() * self.speed * boost * dt;
    }

    /// L'orientation seule : la souris tourne la tête. Utilisée aussi en
    /// mode marche (voir player.rs), où c'est le personnage qui se déplace.
    pub fn look(&mut self, transform: &mut Transform, input: &Input, look: bool) {
        if look {
            let delta = input.mouse_delta();
            self.yaw += delta.x * self.sensitivity;
            // La souris vers le bas donne un y positif : on soustrait pour
            // que pousser la souris vers l'avant lève le regard.
            self.pitch = (self.pitch - delta.y * self.sensitivity).clamp(-MAX_PITCH, MAX_PITCH);
        }
        transform.rotation = self.rotation();
    }

    /// « Devant », à plat : la direction de la marche. Regarder ses pieds
    /// ne doit pas ralentir le personnage.
    pub fn forward_flat(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        Vec3::new(sin_yaw, 0.0, -cos_yaw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_nuls_regardent_vers_moins_z() {
        let (cam, _) = FlyCamera::looking_at(Vec3::ZERO, -Vec3::Z);
        assert!(cam.yaw.abs() < 1e-6 && cam.pitch.abs() < 1e-6);
        assert!(cam.forward().abs_diff_eq(-Vec3::Z, 1e-6));
        assert!(cam.right().abs_diff_eq(Vec3::X, 1e-6));
    }

    #[test]
    fn looking_at_retrouve_la_direction() {
        let position = Vec3::new(0.0, 4.0, 11.0);
        let target = Vec3::new(2.0, 0.5, -3.0);
        let (cam, transform) = FlyCamera::looking_at(position, target);
        assert_eq!(transform.translation, position);
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
        let (cam, _) = FlyCamera::looking_at(Vec3::ZERO, Vec3::Y);
        assert!(cam.pitch <= MAX_PITCH);
        assert!(cam.right().is_finite());
    }

    #[test]
    fn la_rotation_regarde_dans_la_direction_forward() {
        // Le Transform (utilisé par la scène pour construire la caméra) et
        // `forward()` (utilisé pour se déplacer) doivent être d'accord.
        let (cam, transform) =
            FlyCamera::looking_at(Vec3::new(1.0, 3.0, 2.0), Vec3::new(-4.0, 0.0, -6.0));
        let regard = transform.rotation * Vec3::NEG_Z;
        assert!(
            regard.abs_diff_eq(cam.forward(), 1e-5),
            "{regard} != {}",
            cam.forward()
        );
    }
}
