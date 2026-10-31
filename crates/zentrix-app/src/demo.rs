//! La scène de démonstration : ses entités, ses composants et ses systèmes.
//!
//! C'est le code « du jeu ». Il montre comment on utilise l'ECS :
//!
//! - le moteur fournit des composants génériques (`Transform`, `MeshRenderer`,
//!   `Camera3d`, `Name`) ;
//! - le jeu définit les siens ([`Spin`], [`FlyCamera`]) — de simples structs ;
//! - le composant `Parent` attache une entité à une autre (le système solaire) ;
//! - les **systèmes** sont de simples fonctions qui parcourent les entités
//!   ayant certains composants (une « requête »).

use zentrix_core::math::{Quat, Vec3};
use zentrix_core::{Color, Transform};
use zentrix_platform::Input;
use zentrix_render::{DebugLines, MeshHandle, TextureHandle};
use zentrix_scene::{
    Camera3d, DirectionalLight, Entity, GlobalTransform, MeshRenderer, Name, Parent, Scene, hecs,
};

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
    /// Les cubes colorés d'une seule teinte du système solaire.
    pub sun: MeshHandle,
    pub planet: MeshHandle,
    pub moon: MeshHandle,
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

    // La lumière : une entité sans maillage. Ses rayons partent vers le -Z
    // de son Transform : on la penche de 50° vers le bas, puis on la fait
    // tourner lentement autour de la verticale (le même `Spin` que les
    // cubes !) pour que l'éclairage change sous nos yeux.
    world.spawn((
        Name::new("Lumière du soleil"),
        Transform::IDENTITY
            .with_rotation(Quat::from_rotation_y(0.6) * Quat::from_rotation_x(-50f32.to_radians())),
        DirectionalLight {
            color: Color::rgb(1.0, 0.95, 0.85),
            intensity: 1.1,
        },
        Spin {
            axis: Vec3::Y,
            speed: 0.15,
        },
    ));

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

    spawn_solar_system(world, assets);

    camera
}

/// Un « système solaire » au-dessus des cubes, pour montrer la hiérarchie :
///
/// ```text
///   Soleil  (tourne sur lui-même)
///     └── Planète  (à 3 unités du soleil, tourne sur elle-même)
///           └── Lune  (à 2 unités de la planète... dans le repère de la planète)
/// ```
///
/// Aucune orbite n'est programmée : chaque entité tourne seulement sur
/// elle-même (`Spin`), et ses enfants, fixes dans son repère, sont emportés.
fn spawn_solar_system(world: &mut hecs::World, assets: &DemoAssets) {
    let sun = world.spawn((
        Name::new("Soleil"),
        Transform::from_translation(Vec3::new(0.0, 3.5, 0.0)),
        MeshRenderer::new(assets.sun),
        Spin {
            axis: Vec3::Y,
            speed: 0.6,
        },
    ));
    let planet = world.spawn((
        Name::new("Planète"),
        // Position RELATIVE au soleil. L'échelle 0,5 s'applique aussi à tout
        // ce qui est attaché à la planète (ses enfants sont deux fois plus
        // petits et deux fois plus proches qu'écrit).
        Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)).with_uniform_scale(0.5),
        MeshRenderer::new(assets.planet),
        Parent(sun),
        Spin {
            axis: Vec3::Y,
            speed: 2.0,
        },
    ));
    world.spawn((
        Name::new("Lune"),
        // 2 unités dans le repère de la planète = 1 unité dans le monde.
        Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)).with_uniform_scale(0.5),
        MeshRenderer::new(assets.moon),
        Parent(planet),
    ));
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

/// Système de debug : dessine la boîte englobante de chaque objet visible,
/// et un trait entre chaque enfant et son parent.
///
/// On lit le `GlobalTransform` (position dans le monde) et non le
/// `Transform` (position relative au parent) : sinon la boîte de la lune
/// serait dessinée à 2 unités de l'ORIGINE au lieu de 2 unités de sa planète.
pub fn bounds_system(world: &hecs::World, debug: &mut DebugLines) {
    // `With<_, &MeshRenderer>` : filtre « seulement les entités visibles »,
    // sans avoir besoin de lire ce composant.
    for global in world
        .query::<hecs::With<&GlobalTransform, &MeshRenderer>>()
        .iter()
    {
        // La même matrice que l'objet : la boîte tourne avec lui. On saute
        // le sol (un plan plat de 16 unités) dont la boîte n'apporte rien.
        if global.0.x_axis.length() < 10.0 {
            debug.wire_box(global.0, Color::rgb(1.0, 0.85, 0.2));
        }
    }

    for (global, parent) in world.query::<(&GlobalTransform, &Parent)>().iter() {
        if let Ok(parent_global) = world.get::<&GlobalTransform>(parent.0) {
            debug.line(
                parent_global.translation(),
                global.translation(),
                Color::rgb(0.2, 0.9, 1.0),
            );
        }
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
            sun: MeshHandle::from_index(3),
            planet: MeshHandle::from_index(4),
            moon: MeshHandle::from_index(5),
        }
    }

    #[test]
    fn la_scene_contient_camera_sol_et_cubes() {
        let mut scene = Scene::default();
        let camera = spawn(&mut scene, &assets());
        // 1 caméra + 1 lumière + 1 sol + 25 cubes + soleil, planète, lune.
        assert_eq!(scene.entity_count(), 31);
        assert!(scene.world.get::<&Camera3d>(camera).is_ok());

        let mut draws = Vec::new();
        scene.extract_draws(&mut draws);
        assert_eq!(
            draws.len(),
            29,
            "ni la caméra ni la lumière ne sont dessinées"
        );
        assert!(scene.lighting().intensity > 0.0, "la scène a un soleil");
        // 13 caisses texturées (cases paires du damier 5×5).
        let caisses = draws
            .iter()
            .filter(|d| d.texture == Some(assets().crate_texture))
            .count();
        assert_eq!(caisses, 13);
    }

    #[test]
    fn la_lune_suit_la_planete_qui_suit_le_soleil() {
        let mut scene = Scene::default();
        spawn(&mut scene, &assets());
        scene.update_transforms();
        let position_de = |nom: &str| {
            scene
                .world
                .query::<(&Name, &GlobalTransform)>()
                .iter()
                .find(|(name, _)| name.0 == nom)
                .map(|(_, global)| global.translation())
                .expect("entité trouvée")
        };
        // Soleil en (0, 3.5, 0) ; planète 3 unités plus loin en X ; lune à
        // 2 × 0,5 = 1 unité de la planète.
        assert!(position_de("Planète").abs_diff_eq(Vec3::new(3.0, 3.5, 0.0), 1e-5));
        assert!(position_de("Lune").abs_diff_eq(Vec3::new(4.0, 3.5, 0.0), 1e-5));
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
