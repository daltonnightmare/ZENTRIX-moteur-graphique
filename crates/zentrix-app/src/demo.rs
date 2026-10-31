//! La scène de démonstration : ses entités, ses composants et ses systèmes.
//!
//! C'est le code « du jeu ». Il montre comment on utilise l'ECS :
//!
//! - le moteur fournit des composants génériques (`Transform`, `MeshRenderer`,
//!   `Camera3d`, `Name`) ;
//! - le jeu définit les siens ([`Spin`], [`FlyCamera`]) — de simples structs ;
//! - les **systèmes** sont de simples fonctions qui parcourent les entités
//!   ayant certains composants (une « requête »).

use zentrix_core::math::{Quat, Vec3};
use zentrix_core::{Color, Transform};
use zentrix_platform::Input;
use zentrix_render::{DebugLines, MeshHandle, TextureHandle};
use zentrix_scene::{Camera3d, Entity, MeshRenderer, Name, Scene, hecs};

use crate::fly_camera::FlyCamera;

/// Nombre de cubes par côté de la grille (la scène en contient le carré).
const GRID_SIZE: i32 = 5;
/// Distance entre deux cubes voisins.
const GRID_SPACING: f32 = 1.8;
/// Hauteur du sol.
pub const GROUND_Y: f32 = -0.5;

/// Les ressources GPU dont la démo a besoin, déjà chargées dans le `Renderer`.
#[derive(Debug, Clone, Copy)]
pub struct DemoAssets {
    pub cube: MeshHandle,
    /// Le même cube, mais tout blanc : la texture y garde ses vraies couleurs.
    pub white_cube: MeshHandle,
    pub ground: MeshHandle,
    pub crate_texture: TextureHandle,
    pub ground_texture: TextureHandle,
}

/// Composant du jeu : « cette entité tourne sur elle-même ».
#[derive(Debug, Clone, Copy)]
pub struct Spin {
    /// Axe de rotation (de longueur 1).
    pub axis: Vec3,
    /// Vitesse en radians par seconde.
    pub speed: f32,
}

/// Crée toutes les entités de la démo. Renvoie l'entité caméra.
pub fn spawn(scene: &mut Scene, assets: &DemoAssets) -> Entity {
    let world = &mut scene.world;

    // La caméra : une entité comme les autres, avec un Transform.
    let (fly, transform) =
        FlyCamera::looking_at(Vec3::new(0.0, 4.0, 11.0), Vec3::new(0.0, 0.5, 0.0));
    let camera = world.spawn((Name::new("Caméra"), transform, Camera3d::default(), fly));

    // Le sol : un plan de 1×1 agrandi 16 fois.
    world.spawn((
        Name::new("Sol"),
        Transform::from_translation(Vec3::new(0.0, GROUND_Y, 0.0)).with_uniform_scale(16.0),
        MeshRenderer::new(assets.ground).with_texture(assets.ground_texture),
    ));

    // Une grille de cubes centrée sur l'origine. `half` décale la grille
    // pour que son centre soit en (0, 0) plutôt que son coin.
    let half = (GRID_SIZE - 1) as f32 * GRID_SPACING / 2.0;
    let axis = Vec3::new(1.0, 1.0, 0.3).normalize();
    for x in 0..GRID_SIZE {
        for z in 0..GRID_SIZE {
            let position = Vec3::new(
                x as f32 * GRID_SPACING - half,
                0.3,
                z as f32 * GRID_SPACING - half,
            );
            // Une case sur deux (comme un damier) : une caisse en bois.
            let renderer = if (x + z) % 2 == 0 {
                MeshRenderer::new(assets.white_cube).with_texture(assets.crate_texture)
            } else {
                MeshRenderer::new(assets.cube)
            };
            world.spawn((
                Name::new(format!("Cube ({x}, {z})")),
                Transform::from_translation(position),
                renderer,
                // Chaque cube tourne à sa propre vitesse.
                Spin {
                    axis,
                    speed: 0.5 + 0.15 * (x + z) as f32,
                },
            ));
        }
    }

    camera
}

/// Système : fait tourner toutes les entités qui ont un [`Spin`].
///
/// `query_mut::<(&mut Transform, &Spin)>()` : « toutes les entités qui ont
/// un `Transform` (que je vais modifier) ET un `Spin` (que je lis) ». Le sol
/// et la caméra, sans `Spin`, ne sont pas visités.
pub fn spin_system(world: &mut hecs::World, dt: f32) {
    for (transform, spin) in world.query_mut::<(&mut Transform, &Spin)>() {
        // On COMPOSE une petite rotation avec l'orientation actuelle.
        // `normalize` corrige les minuscules erreurs d'arrondi qui, image
        // après image, finiraient par déformer l'objet.
        let step = Quat::from_axis_angle(spin.axis, spin.speed * dt);
        transform.rotation = (step * transform.rotation).normalize();
    }
}

/// Système : applique les entrées à toutes les entités pilotées par une [`FlyCamera`].
pub fn fly_camera_system(world: &mut hecs::World, input: &Input, dt: f32, look: bool) {
    for (transform, fly) in world.query_mut::<(&mut Transform, &mut FlyCamera)>() {
        fly.update(transform, input, dt, look);
    }
}

/// Système de debug : dessine la boîte englobante de chaque cube qui tourne.
pub fn bounds_system(world: &hecs::World, debug: &mut DebugLines) {
    // `With<_, &Spin>` : filtre « seulement les entités qui ont un Spin »,
    // sans avoir besoin de lire ce composant.
    for transform in world.query::<hecs::With<&Transform, &Spin>>().iter() {
        // La même matrice que le cube : la boîte tourne avec lui.
        debug.wire_box(transform.matrix(), Color::rgb(1.0, 0.85, 0.2));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets() -> DemoAssets {
        DemoAssets {
            cube: MeshHandle::from_index(0),
            white_cube: MeshHandle::from_index(1),
            ground: MeshHandle::from_index(2),
            crate_texture: TextureHandle::from_index(1),
            ground_texture: TextureHandle::from_index(2),
        }
    }

    #[test]
    fn la_scene_contient_camera_sol_et_cubes() {
        let mut scene = Scene::default();
        let camera = spawn(&mut scene, &assets());
        // 1 caméra + 1 sol + 25 cubes.
        assert_eq!(scene.entity_count(), 27);
        assert!(scene.world.get::<&Camera3d>(camera).is_ok());

        let mut draws = Vec::new();
        scene.extract_draws(&mut draws);
        assert_eq!(draws.len(), 26, "la caméra n'est pas dessinée");
        // 13 caisses texturées (cases paires du damier 5×5).
        let caisses = draws
            .iter()
            .filter(|d| d.texture == Some(assets().crate_texture))
            .count();
        assert_eq!(caisses, 13);
    }

    #[test]
    fn seules_les_entites_spin_tournent() {
        let mut scene = Scene::default();
        let sol = scene.world.spawn((Transform::IDENTITY,));
        let cube = scene.world.spawn((
            Transform::IDENTITY,
            Spin {
                axis: Vec3::Y,
                speed: std::f32::consts::PI, // un demi-tour par seconde
            },
        ));

        spin_system(&mut scene.world, 0.5); // un quart de tour

        let rotation_sol = scene.world.get::<&Transform>(sol).unwrap().rotation;
        let rotation_cube = scene.world.get::<&Transform>(cube).unwrap().rotation;
        assert_eq!(rotation_sol, Quat::IDENTITY);
        let x = rotation_cube * Vec3::X;
        // Un quart de tour autour de +Y envoie X sur -Z.
        assert!(x.abs_diff_eq(Vec3::NEG_Z, 1e-5), "{x}");
    }
}
