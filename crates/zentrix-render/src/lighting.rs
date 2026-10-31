//! L'éclairage de la scène : une lumière directionnelle et une lumière ambiante.

use zentrix_core::Color;
use zentrix_core::math::Vec3;

/// Comment la scène est éclairée.
///
/// # Le modèle d'éclairage (dans mesh.wgsl)
///
/// La couleur d'un pixel est la somme de trois termes :
///
/// ```text
///  ambiante  : lumière venue « de partout » (le ciel, les rebonds)
///              → évite que les faces à l'ombre soient noires
///  diffuse   : (Lambert) proportionnelle au cosinus de l'angle entre la
///              normale et la direction de la lumière
///              → face de profil = sombre, face de face = claire
///  spéculaire : (Blinn-Phong) le petit reflet brillant, qui dépend
///              aussi de la position de l'œil
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
