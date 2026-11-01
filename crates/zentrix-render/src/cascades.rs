//! Les **ombres en cascade** (*Cascaded Shadow Maps*, CSM).
//!
//! Une seule shadow map pour toute la scène pose un dilemme : petite zone =
//! ombres nettes mais qui s'arrêtent vite ; grande zone = ombres partout mais
//! floues. Or, près de la caméra, on voit les détails ; au loin, un pixel de
//! l'écran couvre beaucoup de terrain. On découpe donc la vue en **tranches
//! de profondeur**, chacune avec sa propre shadow map :
//!
//! ```text
//!   vue de dessus          tranche 0     tranche 1         tranche 2
//!                         (0,1 → 5 m)   (5 → 13 m)        (13 → 50 m)
//!          👁 ──────────────┬──────────────┬──────────────────────────┐
//!             ╲   petite    │   moyenne    │        grande            │
//!               ╲  zone     │    zone      │        zone              │
//!                 ╲ nette   │              │      (plus floue)        │
//! ```
//!
//! Les trois shadow maps ont la même résolution : la première, minuscule,
//! est très précise ; la dernière, immense, l'est moins, mais elle est vue
//! de loin. Le shader (mesh.wgsl) choisit la tranche selon la profondeur du
//! pixel.
//!
//! # Deux astuces contre le scintillement
//!
//! - Chaque tranche est entourée d'une **sphère** : sa taille ne dépend pas
//!   de l'orientation de la caméra. Une boîte ajustée changerait de taille
//!   en tournant la tête, et les bords d'ombre « grouilleraient ».
//! - La zone ne se déplace que par **pas d'un texel** de la shadow map :
//!   quand la caméra avance, chaque texel reste au même endroit du monde.

use zentrix_core::math::camera::rh::{proj, view};
use zentrix_core::math::{Mat4, Vec3, Vec4};

use crate::camera::Camera;

/// Le nombre de tranches. 3 est un bon compromis : chaque tranche coûte une
/// passe d'ombre de plus.
pub const CASCADE_COUNT: usize = 3;

/// Au-delà de la sphère d'une tranche, à quelle distance « en amont » de la
/// lumière un objet peut-il encore projeter une ombre dedans (un arbre
/// hors du champ, une montagne...) ?
const CASTER_MARGIN: f32 = 40.0;

/// Les réglages des ombres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowSettings {
    /// Jusqu'à quelle distance de la caméra il y a des ombres.
    pub distance: f32,
    /// Comment répartir les tranches : 0 = tranches de même épaisseur, 1 =
    /// tranches de plus en plus épaisses (« logarithmiques »), ce qui donne
    /// plus de précision près de l'œil.
    pub split_lambda: f32,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            distance: 50.0,
            split_lambda: 0.75,
        }
    }
}

/// Une tranche, prête pour le GPU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cascade {
    /// La matrice « vue depuis la lumière » de la tranche.
    pub view_projection: Mat4,
    /// La profondeur (distance devant la caméra) où la tranche se termine.
    pub far: f32,
    /// La taille d'un texel de sa shadow map, en mètres.
    pub texel_size: f32,
}

/// Où se terminent les tranches, de la plus proche à la plus lointaine.
///
/// La répartition « pratique » (GPU Gems 3) mélange une répartition
/// linéaire et une logarithmique selon `lambda`.
pub fn split_distances(near: f32, far: f32, lambda: f32) -> [f32; CASCADE_COUNT] {
    std::array::from_fn(|i| {
        let t = (i + 1) as f32 / CASCADE_COUNT as f32;
        let logarithmic = near * (far / near).powf(t);
        let linear = near + (far - near) * t;
        lambda * logarithmic + (1.0 - lambda) * linear
    })
}

/// Calcule les tranches pour cette caméra et cette lumière.
///
/// `aspect` : largeur / hauteur de l'image ; `map_size` : la résolution de
/// chaque shadow map, en texels.
pub fn compute_cascades(
    camera: &Camera,
    aspect: f32,
    light_direction: Vec3,
    settings: &ShadowSettings,
    map_size: u32,
) -> [Cascade; CASCADE_COUNT] {
    let direction = light_direction.normalize_or(Vec3::NEG_Y);
    let near = camera.near.max(0.01);
    let far = settings.distance.max(near * 2.0);
    let splits = split_distances(near, far, settings.split_lambda.clamp(0.0, 1.0));

    // Le repère de la caméra : vers l'avant, la droite, le haut.
    let forward = (camera.target - camera.position).normalize_or(Vec3::NEG_Z);
    let tan_v = (camera.fov_y * 0.5).tan();
    let tan_h = tan_v * aspect;

    std::array::from_fn(|i| {
        let slice_near = if i == 0 { near } else { splits[i - 1] };
        let slice_far = splits[i];

        // La sphère qui entoure la tranche. Son centre est sur l'axe de
        // vue ; on le place là où les coins proches et lointains sont à la
        // même distance (la plus petite sphère centrée sur l'axe). Les coins
        // sont à (± tan_h·d, ± tan_v·d) de l'axe, à la profondeur d.
        let k = tan_h * tan_h + tan_v * tan_v; // (écart² au bord) / d²
        let center_depth = ((slice_near + slice_far) * 0.5 * (1.0 + k)).min(slice_far);
        let radius_at = |d: f32| ((d - center_depth).powi(2) + k * d * d).sqrt();
        let radius = radius_at(slice_near).max(radius_at(slice_far));
        // Arrondi au 1/16 de mètre supérieur : une valeur qui ne « tremble »
        // pas d'une image à l'autre à cause des arrondis de calcul.
        let radius = (radius * 16.0).ceil() / 16.0;
        let center = camera.position + forward * center_depth;

        // La caméra de la lumière : en recul de la sphère, de quoi voir
        // aussi les objets qui projettent une ombre de loin.
        let up = if direction.y.abs() > 0.99 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        let distance = radius + CASTER_MARGIN;
        let view = view::look_at_mat4(center - direction * distance, center, up);
        let mut projection =
            proj::directx::orthographic(-radius, radius, -radius, radius, 0.0, distance + radius);

        // Le calage sur la grille des texels : on regarde où tombe
        // l'origine du monde dans la shadow map, et on décale la projection
        // pour qu'elle tombe pile sur un texel.
        let half = map_size as f32 * 0.5;
        let origin = (projection * view) * Vec4::W;
        let texel = origin.truncate().truncate() * half;
        let offset = (texel.round() - texel) / half;
        projection.w_axis.x += offset.x;
        projection.w_axis.y += offset.y;

        Cascade {
            view_projection: projection * view,
            far: slice_far,
            texel_size: 2.0 * radius / map_size as f32,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera() -> Camera {
        Camera {
            position: Vec3::new(0.0, 4.0, 11.0),
            target: Vec3::new(0.0, 0.5, 0.0),
            ..Camera::default()
        }
    }

    const SUN: Vec3 = Vec3::new(-0.4, -1.0, -0.3);

    #[test]
    fn les_tranches_s_allongent() {
        let s = split_distances(0.1, 50.0, 0.75);
        assert!(s[0] < s[1] && s[1] < s[2]);
        assert!((s[2] - 50.0).abs() < 1e-3);
        // Plus épaisses en s'éloignant.
        assert!(s[1] - s[0] > s[0]);
        // lambda = 0 : des tranches égales.
        let lin = split_distances(0.0001, 30.0, 0.0);
        assert!((lin[0] - 10.0).abs() < 1e-2 && (lin[1] - 20.0).abs() < 1e-2);
    }

    #[test]
    fn chaque_tranche_voit_les_points_de_sa_tranche() {
        let camera = camera();
        let cascades = compute_cascades(&camera, 16.0 / 9.0, SUN, &ShadowSettings::default(), 2048);
        let forward = (camera.target - camera.position).normalize();
        let mut previous = camera.near;
        for cascade in &cascades {
            // Des points au fond de la tranche, au centre et dans les coins de l'écran.
            let d = cascade.far * 0.999;
            let right = forward.cross(Vec3::Y).normalize();
            let up = right.cross(forward);
            let tan_v = (camera.fov_y * 0.5).tan();
            for (sx, sy) in [(0.0, 0.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
                let p = camera.position
                    + forward * d
                    + right * sx * tan_v * 16.0 / 9.0 * d
                    + up * sy * tan_v * d;
                let ndc = cascade.view_projection.project_point3(p);
                assert!(ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0, "{ndc}");
                assert!((0.0..=1.0).contains(&ndc.z), "{ndc}");
            }
            assert!(cascade.far > previous);
            previous = cascade.far;
        }
        // Les texels s'agrandissent avec la distance.
        assert!(cascades[0].texel_size < cascades[1].texel_size);
        assert!(cascades[1].texel_size < cascades[2].texel_size);
    }

    #[test]
    fn la_taille_ne_change_pas_quand_on_tourne_la_tete() {
        let a = camera();
        let b = Camera {
            target: a.position + Vec3::new(5.0, -1.0, 2.0),
            ..a
        };
        let settings = ShadowSettings::default();
        let ca = compute_cascades(&a, 1.5, SUN, &settings, 2048);
        let cb = compute_cascades(&b, 1.5, SUN, &settings, 2048);
        for (x, y) in ca.iter().zip(&cb) {
            assert_eq!(x.texel_size, y.texel_size);
        }
    }

    #[test]
    fn la_grille_des_texels_ne_glisse_pas() {
        // Deux caméras décalées d'un rien : l'origine du monde tombe dans
        // les deux cas pile sur un texel (même partie fractionnaire nulle).
        let settings = ShadowSettings::default();
        for dx in [0.0, 0.013, 0.37] {
            let camera = Camera {
                position: camera().position + Vec3::X * dx,
                target: camera().target + Vec3::X * dx,
                ..camera()
            };
            let cascade = compute_cascades(&camera, 1.5, SUN, &settings, 2048)[1];
            let texel = cascade.view_projection.project_point3(Vec3::ZERO) * 1024.0;
            assert!((texel.x - texel.x.round()).abs() < 1e-2, "{texel}");
            assert!((texel.y - texel.y.round()).abs() < 1e-2, "{texel}");
        }
    }
}
