//! Les types mathématiques de ZENTRIX (ré-exportés depuis [`glam`]).
//!
//! On n'écrit pas notre propre bibliothèque de maths : `glam` est rapide
//! (instructions SIMD), bien testée et utilisée par Bevy. En passant par ce
//! module, toutes les crates écrivent `use zentrix_core::math::Vec3;` et
//! utilisent donc **la même version** de glam.
//!
//! | Type   | Contenu                 | Sert à...                                   |
//! |--------|-------------------------|---------------------------------------------|
//! | `Vec3` | 3 `f32` (x, y, z)       | positions, directions, échelles             |
//! | `Vec4` | 4 `f32`                 | coordonnées homogènes (le `w` des shaders)  |
//! | `Mat4` | 16 `f32` (4×4)          | transformations : déplacer, tourner, projeter |
//! | `Quat` | 4 `f32` (quaternion)    | rotations (sans le « blocage de cardan »)   |
//!
//! ## Convention de ZENTRIX
//!
//! - Repère **main droite**, **Y vers le haut** : X à droite, Y en haut,
//!   Z vers l'observateur (la caméra regarde vers -Z par défaut).
//! - Angles en **radians** (`std::f32::consts::PI` = 180°).
//! - Profondeur NDC de 0 à 1 (convention wgpu / Vulkan / DirectX).

pub use glam::{EulerRot, Mat4, Quat, Vec2, Vec3, Vec4};

/// Les fonctions de caméra de glam (matrices de vue et de projection),
/// rangées par convention : `camera::rh` (main droite) ou `camera::lh`.
pub use glam::camera;
