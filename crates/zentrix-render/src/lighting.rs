//! L'éclairage de la scène : une lumière directionnelle et une lumière ambiante.

use zentrix_core::Color;
use zentrix_core::math::Vec3;

/// Comment la scène est éclairée.
///
/// # Le modèle d'éclairage (dans mesh.wgsl)
///
/// La couleur d'un pixel est la somme de trois termes, dosés par le
/// `Material` de l'objet (métal, rugosité) :
///
/// ```text
///  ambiante  : lumière venue « de partout » (le ciel, les rebonds)
///              → évite que les faces à l'ombre soient noires
///  diffuse   : (Lambert) proportionnelle au cosinus de l'angle entre la
///              normale et la direction de la lumière
///              → face de profil = sombre, face de face = claire
///              (aucune pour un métal)
///  spéculaire : (Cook-Torrance, PBR) le reflet, qui dépend aussi de la
///              position de l'œil : petit et vif sur un objet lisse,
///              large et terne sur un objet rugueux
///
///          lumière         normale
///             ╲    θ   ▲
///              ╲  ╭──╮ │      diffuse = max(cos θ, 0)
///               ╲ │  │ │
///   ─────────────╲┴──┴─┴──────  surface
/// ```
///
/// Une **lumière directionnelle** est infiniment lointaine (le soleil) :
/// tous ses rayons sont parallèles, seule sa direction compte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lighting {
    /// Direction dans laquelle **voyagent** les rayons (du soleil vers le sol).
    /// Normalisée automatiquement par le `Renderer`.
    pub direction: Vec3,
    /// Couleur de la lumière (linéaire).
    pub color: Color,
    /// Intensité : multiplie la couleur. 1.0 = normal.
    pub intensity: f32,
    /// Lumière ambiante, ajoutée partout.
    pub ambient: Color,
}

impl Default for Lighting {
    /// Un soleil de milieu de journée, un peu de biais, et un ambiant bleuté
    /// discret (la couleur du ciel).
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.4, -1.0, -0.3),
            color: Color::rgb(1.0, 0.97, 0.9),
            intensity: 1.0,
            ambient: Color::rgb(0.16, 0.18, 0.24),
        }
    }
}

/// Une lumière **locale** : une lampe qui éclaire autour d'elle (lumière
/// ponctuelle), ou dans un cône (projecteur, si `spot` est donné).
///
/// Contrairement au soleil, elle a une **position**, et son éclairage
/// diminue avec la distance : en `1 / distance²`, comme une vraie lampe (la
/// même lumière s'étale sur une sphère 4 fois plus grande quand la distance
/// double). Au-delà de `range`, elle n'éclaire plus rien : le shader peut
/// l'ignorer, et le culling l'écarter si sa sphère est hors de la vue.
///
/// ```text
///   lumière ponctuelle            projecteur
///         \ | /                       ●
///       ── ● ──                      /│\   angle intérieur : pleine lumière
///         / | \                     / │ \  angle extérieur : plus rien
///    (dans toutes les              /  │  \ entre les deux : fondu
///     directions)
/// ```
///
/// Pas d'ombres portées pour ces lumières (seul le soleil en a).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalLight {
    pub position: Vec3,
    /// Couleur (linéaire).
    pub color: Color,
    /// L'éclairage reçu à 1 m de la lampe, comme `Lighting::intensity` pour
    /// le soleil : 1 = une surface blanche face à la lampe, à 1 m, est blanche.
    pub intensity: f32,
    /// La portée, en mètres.
    pub range: f32,
    /// `Some` pour un projecteur.
    pub spot: Option<SpotCone>,
}

/// Le cône d'un projecteur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpotCone {
    /// La direction éclairée (normalisée par le rendu).
    pub direction: Vec3,
    /// Le demi-angle du cône en pleine lumière, en radians.
    pub inner_angle: f32,
    /// Le demi-angle au-delà duquel il n'y a plus de lumière, en radians.
    pub outer_angle: f32,
}

/// Une lumière locale telle que le shader la lit (`LocalLight` dans
/// mesh.wgsl) : 4 × vec4 = 64 octets.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct LightRaw {
    /// xyz : position ; w : portée.
    pub position_range: [f32; 4],
    /// rgb : couleur × intensité ; w : 1 pour un projecteur, 0 sinon.
    pub color_kind: [f32; 4],
    /// xyz : direction du projecteur ; w : cosinus de l'angle extérieur.
    pub direction_cos_outer: [f32; 4],
    /// x : cosinus de l'angle intérieur ; y, z, w : inutilisés.
    pub cos_inner: [f32; 4],
}

impl From<&LocalLight> for LightRaw {
    fn from(light: &LocalLight) -> Self {
        let c = light.color;
        let k = light.intensity;
        let (kind, direction, cos_outer, cos_inner) = match light.spot {
            // Le shader compare des COSINUS (un produit scalaire) : on les
            // calcule une fois ici plutôt qu'à chaque pixel. L'angle
            // intérieur ne doit pas dépasser l'extérieur.
            Some(spot) => {
                let outer = spot.outer_angle.clamp(0.001, std::f32::consts::FRAC_PI_2);
                let inner = spot.inner_angle.clamp(0.0, outer - 0.0005);
                (
                    1.0,
                    spot.direction.normalize_or(Vec3::NEG_Y),
                    outer.cos(),
                    inner.cos(),
                )
            }
            None => (0.0, Vec3::NEG_Y, -1.0, -1.0),
        };
        Self {
            position_range: light.position.extend(light.range.max(0.01)).to_array(),
            color_kind: [c.r * k, c.g * k, c.b * k, kind],
            direction_cos_outer: direction.extend(cos_outer).to_array(),
            cos_inner: [cos_inner, 0.0, 0.0, 0.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taille_d_une_lumiere_locale() {
        assert_eq!(std::mem::size_of::<LightRaw>(), 64);
    }

    #[test]
    fn un_projecteur_compare_des_cosinus() {
        let spot = LocalLight {
            position: Vec3::Y,
            color: Color::WHITE,
            intensity: 2.0,
            range: 10.0,
            spot: Some(SpotCone {
                direction: Vec3::new(0.0, -2.0, 0.0),
                inner_angle: 0.6,
                // Un angle intérieur plus grand que l'extérieur est corrigé.
                outer_angle: 0.4,
            }),
        };
        let raw = LightRaw::from(&spot);
        assert_eq!(raw.color_kind, [2.0, 2.0, 2.0, 1.0]);
        assert_eq!(&raw.direction_cos_outer[..3], &[0.0, -1.0, 0.0]);
        assert!(raw.cos_inner[0] > raw.direction_cos_outer[3]);
        let point = LightRaw::from(&LocalLight { spot: None, ..spot });
        assert_eq!(point.color_kind[3], 0.0);
    }
}
