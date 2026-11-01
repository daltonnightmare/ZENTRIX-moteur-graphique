//! Le mode « marche » : un personnage à la première personne (touche `C`).
//!
//! Le joueur est une entité à part, invisible, dont le `Transform` est la
//! position des **pieds** :
//!
//! ```text
//!   « Joueur » : Transform (pieds) + RigidBody::kinematic + Collider (capsule)
//!                + CharacterController (zentrix-physics) + Player
//!   « Caméra » : ses yeux, placés à EYE_HEIGHT au-dessus des pieds
//! ```
//!
//! Chaque image :
//! 1. [`input_system`] : la souris tourne la tête (le `FlyCamera` de la
//!    caméra), les touches donnent la vitesse voulue et le saut ;
//! 2. `Physics::step` déplace le personnage (murs, marches, gravité) ;
//! 3. [`camera_follow_system`] : la caméra rejoint les yeux.
//!
//! Le mode vol (la caméra libre) reste disponible : `C` passe de l'un à
//! l'autre. Le joueur n'existe que pendant la marche.

use zentrix_core::Transform;
use zentrix_core::math::Vec3;
use zentrix_physics::{CharacterController, Collider, Physics, RigidBody};
use zentrix_platform::{Input, KeyCode};
use zentrix_scene::{Entity, Name, hecs};

use crate::demo::GROUND_Y;
use crate::fly_camera::FlyCamera;

/// Hauteur des yeux au-dessus des pieds.
pub const EYE_HEIGHT: f32 = 1.6;
/// Vitesses de marche et de course (`Maj`), en mètres par seconde.
pub const WALK_SPEED: f32 = 4.0;
pub const RUN_SPEED: f32 = 7.0;
/// La capsule : 2 × (0,6 + 0,3) = 1,8 m de haut, 60 cm de large.
const CAPSULE_HALF_HEIGHT: f32 = 0.6;
const CAPSULE_RADIUS: f32 = 0.3;
/// Plus bas que ça, le joueur est tombé du monde : il revient au départ.
const FALL_LIMIT: f32 = -15.0;
/// Le point de départ : sur le sol (qui va de -8 à 8), juste devant la
/// vitrine de sphères (dont la dernière rangée s'arrête en z = 7), face à la
/// grille, dans l'allée entre deux colonnes de sphères (en x = 0 et 1,8) :
/// avancer y mène tout droit. Quelques centimètres au-dessus du sol : jamais dedans.
const RESPAWN: Vec3 = Vec3::new(0.9, GROUND_Y + 0.02, 7.5);

/// Composant : « cette entité est le personnage du joueur ».
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Player;

/// Le personnage du joueur, s'il existe (mode marche).
pub fn find(world: &hecs::World) -> Option<Entity> {
    world.query::<hecs::With<Entity, &Player>>().iter().next()
}

/// Crée le personnage, les pieds en `feet`.
pub fn spawn(world: &mut hecs::World, feet: Vec3) -> Entity {
    let half_total = CAPSULE_HALF_HEIGHT + CAPSULE_RADIUS;
    world.spawn((
        Name::new("Joueur"),
        Transform::from_translation(feet),
        RigidBody::kinematic(),
        // La capsule est centrée sur l'origine de l'entité : on la remonte de
        // sa demi-hauteur pour que son bas touche les pieds.
        Collider::capsule(CAPSULE_HALF_HEIGHT, CAPSULE_RADIUS).with_offset(Vec3::Y * half_total),
        CharacterController::default(),
        Player,
    ))
}

/// `C` : passe du vol à la marche, ou l'inverse. Renvoie `true` si l'on
/// marche maintenant.
///
/// En passant à la marche, le personnage apparaît sous la caméra, posé sur
/// ce qu'un rayon vertical trouve en dessous (le sol, le dessus d'un cube) ;
/// au-dessus du vide, il revient au point de départ.
pub fn toggle(world: &mut hecs::World, physics: &Physics) -> bool {
    if let Some(player) = find(world) {
        let _ = world.despawn(player);
        return false;
    }
    let eye = world
        .query::<hecs::With<&Transform, &FlyCamera>>()
        .iter()
        .next()
        .map(|t| t.translation);
    let feet = eye
        .and_then(|eye| physics.cast_ray(eye, Vec3::NEG_Y, 100.0))
        // Quelques centimètres au-dessus : jamais dans le sol.
        .map(|hit| hit.point + Vec3::Y * 0.02)
        .unwrap_or(RESPAWN);
    spawn(world, feet);
    true
}

/// La vitesse voulue d'après les touches (`is_down`) et l'orientation de la
/// tête : `Z` `Q` `S` `D` (touches physiques W A S D), `Maj` pour courir.
///
/// Séparée de [`input_system`] pour être testée sans clavier.
pub fn desired_velocity(fly: &FlyCamera, is_down: impl Fn(KeyCode) -> bool) -> Vec3 {
    let forward = fly.forward_flat();
    let right = forward.cross(Vec3::Y);
    let mut direction = Vec3::ZERO;
    for (key, axis) in [
        (KeyCode::KeyW, forward),
        (KeyCode::KeyS, -forward),
        (KeyCode::KeyD, right),
        (KeyCode::KeyA, -right),
    ] {
        if is_down(key) {
            direction += axis;
        }
    }
    let speed = if is_down(KeyCode::ShiftLeft) {
        RUN_SPEED
    } else {
        WALK_SPEED
    };
    // Normalisé : la diagonale ne va pas plus vite que la ligne droite.
    direction.normalize_or_zero() * speed
}

/// Système : la souris tourne la tête, les touches dirigent le personnage.
/// `Espace` (maintenu) saute dès que le personnage touche le sol.
pub fn input_system(world: &mut hecs::World, input: &Input, look: bool) {
    let Some(fly) = world
        .query_mut::<(&mut Transform, &mut FlyCamera)>()
        .into_iter()
        .next()
        .map(|(transform, fly)| {
            fly.look(transform, input, look);
            fly.clone()
        })
    else {
        return;
    };
    let velocity = desired_velocity(&fly, |key| input.is_key_down(key));
    let jump = input.is_key_down(KeyCode::Space);

    for (transform, character) in
        world.query_mut::<hecs::With<(&mut Transform, &mut CharacterController), &Player>>()
    {
        character.desired_velocity = velocity;
        character.jump = jump;
        // Tombé du monde : retour au départ, sans garder sa vitesse de chute.
        if transform.translation.y < FALL_LIMIT {
            transform.translation = RESPAWN;
            character.vertical_speed = 0.0;
        }
    }
}

/// Système, APRÈS la physique : la caméra rejoint les yeux du personnage.
pub fn camera_follow_system(world: &mut hecs::World) {
    let Some(feet) = world
        .query::<hecs::With<&Transform, &Player>>()
        .iter()
        .next()
        .map(|t| t.translation)
    else {
        return;
    };
    for (transform, _) in world.query_mut::<(&mut Transform, &FlyCamera)>() {
        transform.translation = feet + Vec3::Y * EYE_HEIGHT;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zentrix_physics::TIME_STEP;
    use zentrix_scene::Scene;

    /// Le sol de la démo (dessus en GROUND_Y) et une caméra à `eye`.
    fn scene_avec_camera(eye: Vec3) -> (Scene, Physics) {
        let mut scene = Scene::default();
        scene.world.spawn((
            Transform::from_translation(Vec3::new(0.0, GROUND_Y - 0.5, 0.0)),
            RigidBody::fixed(),
            Collider::cuboid(Vec3::new(8.0, 0.5, 8.0)),
        ));
        let (fly, transform) = FlyCamera::looking_at(eye, eye - Vec3::Z);
        scene.world.spawn((transform, fly));
        let mut physics = Physics::new();
        physics.step(&mut scene, TIME_STEP);
        (scene, physics)
    }

    fn eye(scene: &Scene) -> Vec3 {
        scene
            .world
            .query::<hecs::With<&Transform, &FlyCamera>>()
            .iter()
            .next()
            .unwrap()
            .translation
    }

    #[test]
    fn c_pose_le_personnage_sous_la_camera_puis_le_retire() {
        let (mut scene, physics) = scene_avec_camera(Vec3::new(2.0, 5.0, 3.0));
        assert!(toggle(&mut scene.world, &physics));
        let player = find(&scene.world).expect("un joueur");
        let feet = scene.world.get::<&Transform>(player).unwrap().translation;
        assert!(
            (feet - Vec3::new(2.0, GROUND_Y, 3.0)).length() < 0.05,
            "{feet}"
        );

        assert!(!toggle(&mut scene.world, &physics));
        assert!(find(&scene.world).is_none());
    }

    /// Régression : passer en marche dans la vraie scène de démo, quelques
    /// secondes après le lancement, faisait paniquer rapier (« touching pair
    /// not linked in the persistent islands »).
    #[test]
    fn marcher_dans_la_scene_de_demo() {
        let mut scene = Scene::default();
        crate::demo::spawn(&mut scene, &crate::demo::tests::assets());
        let mut physics = Physics::new();
        // Des images de durées irrégulières, comme dans l'application : selon
        // l'image, la physique simule 0, 1 ou 2 pas.
        let durees = [0.011, 0.016, 0.023, 0.007, 0.019];
        let avancer = |scene: &mut Scene, physics: &mut Physics, secondes: f32| {
            let mut t = 0.0;
            let mut i = 0;
            while t < secondes {
                let dt = durees[i % durees.len()];
                crate::demo::spin_system(&mut scene.world, dt);
                physics.step(scene, dt);
                camera_follow_system(&mut scene.world);
                t += dt;
                i += 1;
            }
        };
        avancer(&mut scene, &mut physics, 5.0);
        assert!(toggle(&mut scene.world, &physics));
        let player = find(&scene.world).unwrap();
        scene
            .world
            .get::<&mut CharacterController>(player)
            .unwrap()
            .desired_velocity = Vec3::NEG_Z * WALK_SPEED;
        avancer(&mut scene, &mut physics, 3.0);
        assert!(
            scene
                .world
                .get::<&CharacterController>(player)
                .unwrap()
                .grounded
        );
    }

    #[test]
    fn le_point_de_depart_est_sur_le_sol_de_la_demo() {
        // Le sol de la démo : 16 × 16, centré sur l'origine.
        assert!(RESPAWN.x.abs() < 8.0 - CAPSULE_RADIUS && RESPAWN.z.abs() < 8.0 - CAPSULE_RADIUS);
        // Constantes : vérifié dès la compilation.
        const { assert!(RESPAWN.y > GROUND_Y) };
    }

    #[test]
    fn au_dessus_du_vide_le_personnage_revient_au_depart() {
        let (mut scene, physics) = scene_avec_camera(Vec3::new(30.0, 5.0, 0.0));
        toggle(&mut scene.world, &physics);
        let player = find(&scene.world).unwrap();
        assert_eq!(
            scene.world.get::<&Transform>(player).unwrap().translation,
            RESPAWN
        );
    }

    #[test]
    fn marcher_amene_la_camera_a_hauteur_des_yeux() {
        let (mut scene, mut physics) = scene_avec_camera(Vec3::new(0.0, 5.0, 0.0));
        toggle(&mut scene.world, &physics);
        let player = find(&scene.world).unwrap();
        // L'image où son corps est créé, le personnage attend (voir
        // `Physics::move_characters`).
        physics.step(&mut scene, TIME_STEP);
        // On avance d'un mètre vers -Z (la caméra regarde vers -Z).
        let fly = scene
            .world
            .query::<&FlyCamera>()
            .iter()
            .next()
            .unwrap()
            .clone();
        let velocity = desired_velocity(&fly, |k| k == KeyCode::KeyW);
        scene
            .world
            .get::<&mut CharacterController>(player)
            .unwrap()
            .desired_velocity = velocity;
        for _ in 0..15 {
            physics.step(&mut scene, TIME_STEP);
            camera_follow_system(&mut scene.world);
        }
        let e = eye(&scene);
        assert!((e.y - (GROUND_Y + EYE_HEIGHT)).abs() < 0.05, "{e}");
        assert!((e.z - -1.0).abs() < 0.05, "4 m/s × 0,25 s = 1 m : {e}");
    }

    #[test]
    fn la_vitesse_suit_le_regard_mais_reste_horizontale() {
        // Regard vers +X, la tête baissée.
        let (fly, _) = FlyCamera::looking_at(Vec3::ZERO, Vec3::new(1.0, -1.0, 0.0));
        let avant = desired_velocity(&fly, |k| k == KeyCode::KeyW);
        assert!(avant.abs_diff_eq(Vec3::X * WALK_SPEED, 1e-4), "{avant}");

        // Diagonale avant-droite en courant : la même vitesse qu'en ligne droite.
        let diagonale = desired_velocity(&fly, |k| {
            matches!(k, KeyCode::KeyW | KeyCode::KeyD | KeyCode::ShiftLeft)
        });
        assert!((diagonale.length() - RUN_SPEED).abs() < 1e-4);
        assert!(diagonale.z > 0.0, "à droite de +X, c'est +Z : {diagonale}");

        // Avant et arrière ensemble s'annulent.
        let rien = desired_velocity(&fly, |k| matches!(k, KeyCode::KeyW | KeyCode::KeyS));
        assert_eq!(rien, Vec3::ZERO);
    }
}
