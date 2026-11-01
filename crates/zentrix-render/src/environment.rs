//! L'**environnement** : le ciel qui entoure la scène, son éclairage et ses
//! reflets (« IBL », *Image-Based Lighting*).
//!
//! # Pourquoi précalculer ?
//!
//! Pour éclairer un point, il faudrait additionner la lumière venue de
//! **toutes** les directions du ciel : des milliers de lectures par pixel. On
//! fait ce calcul une seule fois, au démarrage, et on range les résultats
//! dans deux **cube maps** (des textures à 6 faces, lues avec une direction) :
//!
//! | Cube map     | Contenu, pour chaque direction `d`                        | Utilisée pour |
//! |--------------|-----------------------------------------------------------|---------------|
//! | `specular`   | le ciel vu dans la direction `d`, de plus en plus **flou** à chaque niveau de mipmap | les reflets (niveau choisi selon la rugosité) |
//! | `irradiance` | la moyenne du ciel vu par une surface tournée vers `d`   | la lumière diffuse |
//!
//! Le flou des reflets suit la distribution GGX du shader (« préfiltrage ») :
//! une surface de rugosité 0,5 reflète le niveau de mipmap du milieu.
//!
//! # Les faces d'une cube map
//!
//! ```text
//!          ┌────┐
//!          │ +Y │            Ordre des couches sur le GPU :
//!     ┌────┼────┼────┬────┐  +X, -X, +Y, -Y, +Z, -Z
//!     │ -X │ +Z │ +X │ -Z │
//!     └────┼────┼────┴────┘  Chaque face est une image carrée ; le GPU
//!          │ -Y │            choisit la face et le pixel à partir de la
//!          └────┘            direction (voir `face_direction`).
//! ```
//!
//! Le ciel actuel est **procédural** ([`Sky`] : un dégradé, sans soleil, car
//! celui de la scène tourne et se reflète déjà par l'éclairage direct). Une
//! image HDR de ciel réel pourrait le remplacer plus tard.

use std::f32::consts::TAU;

use zentrix_core::math::{Vec2, Vec3};
use zentrix_core::{Color, Image};
use zentrix_rhi::wgpu;

/// Un ciel en dégradé : le **zénith** au-dessus, l'**horizon**, puis le **sol**
/// (vu sous l'horizon). Couleurs linéaires, entre 0 et 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sky {
    pub zenith: Color,
    pub horizon: Color,
    pub ground: Color,
}

impl Default for Sky {
    /// Un ciel de fin d'après-midi, assez sombre pour ne pas éblouir la scène.
    fn default() -> Self {
        Self {
            zenith: Color::rgb(0.08, 0.16, 0.38),
            horizon: Color::rgb(0.42, 0.45, 0.52),
            ground: Color::rgb(0.09, 0.085, 0.08),
        }
    }
}

impl Sky {
    /// La couleur du ciel vue dans la direction `dir` (de longueur 1).
    pub fn radiance(&self, dir: Vec3) -> Vec3 {
        let v = |c: Color| Vec3::new(c.r, c.g, c.b);
        if dir.y >= 0.0 {
            // `sqrt` : le dégradé change vite près de l'horizon, puis lentement.
            v(self.horizon).lerp(v(self.zenith), dir.y.sqrt())
        } else {
            // Sous l'horizon, le sol arrive vite (une bande floue d'environ 10°).
            v(self.horizon).lerp(v(self.ground), (-dir.y * 6.0).min(1.0))
        }
    }
}

/// La direction (de longueur 1) qui correspond au point `(u, v)` (de 0 à 1,
/// `v` vers le bas) de la face `face` (0 à 5 : +X, -X, +Y, -Y, +Z, -Z).
///
/// C'est la convention de Vulkan, Direct3D et OpenGL : le GPU fait le
/// calcul inverse quand le shader lit la cube map avec une direction.
pub fn face_direction(face: usize, u: f32, v: f32) -> Vec3 {
    let (s, t) = (2.0 * u - 1.0, 2.0 * v - 1.0);
    let d = match face {
        0 => Vec3::new(1.0, -t, -s),
        1 => Vec3::new(-1.0, -t, s),
        2 => Vec3::new(s, 1.0, t),
        3 => Vec3::new(s, -1.0, -t),
        4 => Vec3::new(s, -t, 1.0),
        _ => Vec3::new(-s, -t, -1.0),
    };
    d.normalize()
}

/// Les points de la suite de **Hammersley** : `n` points répartis très
/// régulièrement dans le carré [0, 1]², bien mieux que des tirages au hasard
/// (moins d'échantillons pour le même résultat, et aucun bruit qui varie).
fn hammersley(i: u32, n: u32) -> Vec2 {
    // `reverse_bits` : le « miroir binaire » de i (0,1 → 0,1000...).
    Vec2::new(i as f32 / n as f32, i.reverse_bits() as f32 / 2f32.powi(32))
}

/// Deux directions perpendiculaires à `n` (et entre elles) : un repère local
/// dans lequel `n` joue le rôle de l'axe Z.
fn tangent_frame(n: Vec3) -> (Vec3, Vec3) {
    let up = if n.z.abs() < 0.999 { Vec3::Z } else { Vec3::X };
    let tx = up.cross(n).normalize();
    (tx, n.cross(tx))
}

/// Un échantillon pour le reflet : une orientation de micro-facette `h`,
/// tirée selon la distribution GGX autour de `n` (les orientations les plus
/// probables sont les plus souvent choisies : « importance sampling »).
fn importance_sample_ggx(xi: Vec2, n: Vec3, roughness: f32) -> Vec3 {
    let a = roughness * roughness;
    let phi = TAU * xi.x;
    let cos_theta = ((1.0 - xi.y) / (1.0 + (a * a - 1.0) * xi.y)).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
    let (tx, ty) = tangent_frame(n);
    (tx * (sin_theta * phi.cos()) + ty * (sin_theta * phi.sin()) + n * cos_theta).normalize()
}

/// Le ciel « vu dans un miroir de rugosité `roughness` » dans la direction `n`.
///
/// Simplification classique : on suppose que l'œil regarde la surface de
/// face (vue = normale = direction du reflet). Elle évite une dimension de
/// plus dans la texture, au prix de reflets un peu trop ronds vus de biais.
fn prefilter(sky: &Sky, n: Vec3, roughness: f32, samples: u32) -> Vec3 {
    if roughness < 1e-3 {
        return sky.radiance(n); // un miroir parfait : le ciel tel quel
    }
    let (mut sum, mut weight) = (Vec3::ZERO, 0.0);
    for i in 0..samples {
        let h = importance_sample_ggx(hammersley(i, samples), n, roughness);
        // La lumière qui, réfléchie par la micro-facette h, part vers n.
        let l = 2.0 * n.dot(h) * h - n;
        let n_dot_l = n.dot(l);
        if n_dot_l > 0.0 {
            sum += sky.radiance(l) * n_dot_l;
            weight += n_dot_l;
        }
    }
    if weight > 0.0 {
        sum / weight
    } else {
        sky.radiance(n)
    }
}

/// La lumière diffuse reçue par une surface tournée vers `n` : la moyenne du
/// ciel sur la demi-sphère, pondérée par le cosinus (Lambert). Les
/// directions sont tirées directement selon ce cosinus : une simple moyenne
/// suffit alors.
fn irradiance(sky: &Sky, n: Vec3, samples: u32) -> Vec3 {
    let (tx, ty) = tangent_frame(n);
    let mut sum = Vec3::ZERO;
    for i in 0..samples {
        let xi = hammersley(i, samples);
        let phi = TAU * xi.x;
        let (cos_theta, sin_theta) = ((1.0 - xi.y).sqrt(), xi.y.sqrt());
        let l = tx * (sin_theta * phi.cos()) + ty * (sin_theta * phi.sin()) + n * cos_theta;
        sum += sky.radiance(l.normalize());
    }
    sum / samples as f32
}

/// Les 6 faces `size × size` obtenues en appelant `f` pour chaque direction.
fn bake_faces(size: u32, f: impl Fn(Vec3) -> Vec3) -> [Image; 6] {
    std::array::from_fn(|face| {
        Image::from_fn(size, size, |x, y| {
            let u = (x as f32 + 0.5) / size as f32;
            let v = (y as f32 + 0.5) / size as f32;
            let c = f(face_direction(face, u, v));
            Color::rgb(c.x, c.y, c.z).to_srgba8()
        })
    })
}

/// Côté d'une face de la cube map des reflets (niveau 0).
const SPECULAR_SIZE: u32 = 64;
/// Côté d'une face de la cube map de lumière diffuse (très floue : petite).
const IRRADIANCE_SIZE: u32 = 16;
const SPECULAR_SAMPLES: u32 = 48;
const IRRADIANCE_SAMPLES: u32 = 256;

/// Les niveaux de mipmap de la cube map des reflets : niveau `k` sur `n`
/// = rugosité `k / (n - 1)`, de 64 × 64 (miroir) à 1 × 1 (rugosité 1).
pub fn bake_specular(sky: &Sky) -> Vec<[Image; 6]> {
    let levels = SPECULAR_SIZE.ilog2() + 1;
    (0..levels)
        .map(|level| {
            let roughness = level as f32 / (levels - 1) as f32;
            bake_faces(SPECULAR_SIZE >> level, |n| {
                prefilter(sky, n, roughness, SPECULAR_SAMPLES)
            })
        })
        .collect()
}

/// La cube map de lumière diffuse.
pub fn bake_irradiance(sky: &Sky) -> [Image; 6] {
    bake_faces(IRRADIANCE_SIZE, |n| irradiance(sky, n, IRRADIANCE_SAMPLES))
}

// ---------------------------------------------------------------------------
// Côté GPU
// ---------------------------------------------------------------------------

/// L'environnement envoyé au GPU : le bind group des deux cube maps.
pub(crate) struct Environment {
    pub bind_group: wgpu::BindGroup,
    /// Numéro du dernier niveau de mipmap des reflets (rugosité 1).
    pub max_lod: f32,
}

/// Le layout du bind group de l'environnement (`@group(3)` dans mesh.wgsl,
/// `@group(1)` dans sky.wgsl) : deux cube maps et un sampler.
pub(crate) fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let cube = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::Cube,
            multisampled: false,
        },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Environment Bind Group Layout"),
        entries: &[
            cube(0),
            cube(1),
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// Envoie une cube map (une liste de niveaux de mipmap de 6 faces) sur le
/// GPU et renvoie sa vue « cube ».
fn upload_cube(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    levels: &[[Image; 6]],
    label: &str,
) -> wgpu::TextureView {
    let size = levels[0][0].width;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        // Une cube map est une texture 2D à 6 couches.
        size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6,
        },
        mip_level_count: levels.len() as u32,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Couleurs du ciel entre 0 et 1 : 8 bits sRGB suffisent.
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    for (level, faces) in levels.iter().enumerate() {
        for (face, image) in faces.iter().enumerate() {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: level as u32,
                    // `z` : la couche, donc la face.
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &image.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image.width),
                    rows_per_image: Some(image.height),
                },
                wgpu::Extent3d {
                    width: image.width,
                    height: image.height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
    texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(label),
        // Les 6 couches vues ensemble comme un cube.
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    })
}

impl Environment {
    /// Précalcule l'environnement de `sky` (quelques dizaines de
    /// millisecondes) et l'envoie au GPU.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sky: &Sky,
    ) -> Self {
        let start = std::time::Instant::now();
        let specular = bake_specular(sky);
        let irradiance = [bake_irradiance(sky)];
        log::info!(
            "Environnement précalculé en {:.0} ms",
            start.elapsed().as_secs_f32() * 1000.0
        );

        let specular_view = upload_cube(device, queue, &specular, "Environment Specular");
        let irradiance_view = upload_cube(device, queue, &irradiance, "Environment Irradiance");
        // `ClampToEdge` : pas de répétition entre les faces d'un cube.
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Environment Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Environment Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&specular_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&irradiance_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Self {
            bind_group,
            max_lod: (specular.len() - 1) as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// L'inverse de `face_direction`, tel que le calcule le GPU.
    fn face_uv(d: Vec3) -> (usize, f32, f32) {
        let a = d.abs();
        let (face, ma, s, t) = if a.x >= a.y && a.x >= a.z {
            if d.x > 0.0 {
                (0, a.x, -d.z, -d.y)
            } else {
                (1, a.x, d.z, -d.y)
            }
        } else if a.y >= a.z {
            if d.y > 0.0 {
                (2, a.y, d.x, d.z)
            } else {
                (3, a.y, d.x, -d.z)
            }
        } else if d.z > 0.0 {
            (4, a.z, d.x, -d.y)
        } else {
            (5, a.z, -d.x, -d.y)
        };
        (face, (s / ma + 1.0) / 2.0, (t / ma + 1.0) / 2.0)
    }

    #[test]
    fn faces_du_cube_aller_retour() {
        for face in 0..6 {
            for (u, v) in [(0.5, 0.5), (0.2, 0.7), (0.9, 0.1)] {
                let (f, u2, v2) = face_uv(face_direction(face, u, v));
                assert_eq!(f, face);
                assert!((u - u2).abs() < 1e-5 && (v - v2).abs() < 1e-5);
            }
        }
        // Le centre de la face +Y regarde vers le haut.
        assert!(face_direction(2, 0.5, 0.5).abs_diff_eq(Vec3::Y, 1e-6));
    }

    #[test]
    fn un_ciel_uniforme_reste_uniforme() {
        let gris = Color::rgb(0.3, 0.3, 0.3);
        let sky = Sky {
            zenith: gris,
            horizon: gris,
            ground: gris,
        };
        let attendu = Vec3::splat(0.3);
        for n in [Vec3::Y, Vec3::NEG_Y, Vec3::new(1.0, 1.0, 0.0).normalize()] {
            assert!(irradiance(&sky, n, 64).abs_diff_eq(attendu, 1e-4));
            assert!(prefilter(&sky, n, 0.6, 32).abs_diff_eq(attendu, 1e-4));
        }
    }

    #[test]
    fn la_lumiere_diffuse_vient_surtout_du_cote_de_la_normale() {
        let sky = Sky::default();
        let haut = irradiance(&sky, Vec3::Y, 256);
        let bas = irradiance(&sky, Vec3::NEG_Y, 256);
        // Une surface tournée vers le ciel reçoit plus de bleu que celle
        // tournée vers le sol.
        assert!(haut.z > bas.z);
        // Le flou ne crée pas de lumière : on reste entre les extrêmes.
        assert!(haut.z <= sky.zenith.b.max(sky.horizon.b) + 1e-4);
    }

    #[test]
    fn chaine_des_reflets() {
        let levels = bake_specular(&Sky::default());
        assert_eq!(levels.len(), 7, "64, 32, 16, 8, 4, 2, 1");
        assert_eq!(levels[6][0].width, 1);
    }
}
