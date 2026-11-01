//! La scène de démonstration : ses entités, ses composants et ses systèmes.
//!
//! C'est le code « du jeu ». Il montre comment on utilise l'ECS :
//!
//! - le moteur fournit des composants génériques (`Transform`, `MeshRenderer`,
//!   `Camera3d`, `Name`) ;
//! - le jeu définit les siens ([`Spin`], [`FlyCamera`]) — de simples structs ;
//! - le composant `Parent` attache une entité à une autre (le système solaire) ;
//! - le `Material` de chaque `MeshRenderer` dit de quoi l'objet est fait
//!   (la vitrine de sphères) ;
//! - `RigidBody` et `Collider` (zentrix-physics) confient l'objet à la
//!   physique : la pile de caisses, les balles lancées avec `B` ;
//! - `AudioListener` et `SoundEmitter` (zentrix-audio) : la caméra écoute,
//!   les caisses sci-fi bourdonnent ;
//! - la **transparence** (`AlphaMode`) : une vitre et des cubes de verre,
//!   une lanterne grillagée (découpe) ;
//! - des **lampes** (`PointLight`) et un **projecteur** (`SpotLight`) : la
//!   flamme de la lanterne, les néons, trois lampes colorées qui tournent ;
//! - les **systèmes** sont de simples fonctions qui parcourent les entités
//!   ayant certains composants (une « requête »).

use zentrix_audio::{AudioListener, SoundEmitter};
use zentrix_core::math::{Quat, Vec3};
use zentrix_core::{AlphaMode, Color, Image, Material, Transform};
use zentrix_physics::{Collider, RigidBody};
use zentrix_platform::Input;
use zentrix_render::{DebugLines, MaterialTextures, MeshHandle, TextureHandle};
use zentrix_scene::{
    Camera3d, DirectionalLight, Entity, GlobalTransform, MeshRenderer, Name, Parent, PointLight,
    Scene, SpotLight, hecs,
};

use crate::fly_camera::FlyCamera;
use crate::model::{self, ModelHandles};

/// Nombre de cubes par côté de la grille (la scène en contient le carré).
const GRID_SIZE: i32 = 5;
/// Distance entre deux cubes voisins.
const GRID_SPACING: f32 = 1.8;
/// Hauteur du sol.
pub const GROUND_Y: f32 = -0.5;

/// Les ressources GPU dont la démo a besoin, déjà chargées dans le `Renderer`.
#[derive(Debug, Clone)]
pub struct DemoAssets {
    pub cube: MeshHandle,
    /// Le même cube, mais tout blanc : la texture y garde ses vraies couleurs.
    pub white_cube: MeshHandle,
    pub ground: MeshHandle,
    pub crate_texture: TextureHandle,
    /// Un grillage : des barreaux opaques et des trous transparents (alpha 0).
    pub lattice_texture: TextureHandle,
    pub ground_texture: TextureHandle,
    /// Le relief du sol : des dalles séparées par des joints (normal map).
    pub ground_normal_map: TextureHandle,
    /// Des alvéoles, comme une balle de golf (normal map).
    pub dimples_normal_map: TextureHandle,
    /// Les 5 textures des caisses « sci-fi » (voir [`panel_images`]).
    pub panel: MaterialTextures,
    /// Les cubes colorés d'une seule teinte du système solaire.
    pub sun: MeshHandle,
    pub planet: MeshHandle,
    pub moon: MeshHandle,
    /// Une sphère blanche, pour la vitrine de matériaux.
    pub sphere: MeshHandle,
    /// Le modèle d'arbre (glTF), s'il a pu être chargé.
    pub tree: Option<ModelHandles>,
}

/// Composant du jeu : « cette entité est une balle lancée par le joueur ».
/// Le numéro sert à retrouver les plus anciennes, pour les supprimer.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Ball {
    pub serial: u32,
}

/// Les noms des sons de la démo (voir `main.rs`, où ils sont créés).
pub const IMPACT_SOUND: &str = "choc";
pub const LAUNCH_SOUND: &str = "lancer";
pub const HUM_SOUND: &str = "bourdonnement";

/// Nombre maximal de balles dans la scène : au-delà, les plus anciennes
/// disparaissent.
pub const MAX_BALLS: usize = 40;
/// Vitesse d'une balle au lancer, en unités par seconde.
const BALL_SPEED: f32 = 16.0;

/// Composant du jeu : « cette entité tourne sur elle-même ».
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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
    // La caméra est aussi l'oreille : les sons sont entendus depuis elle.
    let camera = world.spawn((
        Name::new("Caméra"),
        transform,
        Camera3d::default(),
        fly,
        AudioListener,
    ));

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
        // Échelle (16, 1, 16) : le plan n'a pas d'épaisseur, mais sa forme de
        // collision en a une, qu'on ne veut pas multiplier par 16.
        Transform {
            scale: Vec3::new(16.0, 1.0, 16.0),
            ..Transform::from_translation(Vec3::new(0.0, GROUND_Y, 0.0))
        },
        MeshRenderer::new(assets.ground)
            .with_texture(assets.ground_texture)
            .with_normal_map(assets.ground_normal_map)
            .with_material(Material::dielectric(Color::WHITE, 0.6)),
        // Une dalle de 20 cm d'épaisseur, dont le DESSUS est le plan du sol.
        RigidBody::fixed(),
        Collider::cuboid(Vec3::new(0.5, 0.1, 0.5)).with_offset(Vec3::new(0.0, -0.1, 0.0)),
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
            // Une case sur deux (comme un damier) : une caisse en bois, mate.
            // Les autres cubes, en plastique, sont de plus en plus lisses
            // vers le fond de la grille.
            let renderer = if (x + z) % 2 == 0 {
                MeshRenderer::new(assets.white_cube)
                    .with_texture(assets.crate_texture)
                    .with_material(Material::dielectric(Color::WHITE, 0.85))
            } else {
                let roughness = 0.2 + 0.15 * z as f32;
                MeshRenderer::new(assets.cube)
                    .with_material(Material::dielectric(Color::WHITE, roughness))
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
                // Cinématique : c'est `Spin` qui le fait tourner, et il
                // repousse les balles qui le touchent.
                RigidBody::kinematic(),
                Collider::cuboid(Vec3::splat(0.5)),
            ));
        }
    }

    spawn_solar_system(world, assets);
    spawn_material_showcase(world, assets, half);
    spawn_panel_crates(world, assets, half);
    spawn_crate_pile(world, assets, half);
    spawn_glass(world, assets, half);
    spawn_lantern(world, assets, half);
    spawn_lights(world, assets, half);
    if let Some(tree) = &assets.tree {
        spawn_forest(world, tree);
    }

    camera
}

/// Quelques arbres (modèle glTF) autour de la grille de cubes. Chaque arbre
/// est un exemplaire du même modèle : mêmes maillages sur le GPU, entités
/// différentes (position, orientation, taille).
fn spawn_forest(world: &mut hecs::World, tree: &ModelHandles) {
    // (x, z, taille, rotation en radians autour de la verticale)
    let placements = [
        (-6.5, -6.5, 1.3, 0.0),
        (6.3, -6.8, 1.0, 1.1),
        (-6.8, 6.2, 1.1, 2.3),
        (6.6, 6.4, 1.4, 0.7),
        (-7.2, -0.5, 0.9, 1.9),
        (7.0, 1.2, 1.2, 2.8),
        (0.8, -7.2, 1.0, 0.4),
    ];
    for (i, (x, z, size, angle)) in placements.into_iter().enumerate() {
        model::spawn(
            world,
            tree,
            &format!("Arbre {}", i + 1),
            // La base du tronc (y = 0 dans le modèle) est posée sur le sol.
            Transform::from_translation(Vec3::new(x, GROUND_Y, z))
                .with_rotation(Quat::from_rotation_y(angle))
                .with_uniform_scale(size),
        );
    }
}

/// Une vitrine de matériaux devant la grille : deux rangées de sphères, de
/// la plus lisse (à gauche) à la plus rugueuse (à droite). Devant : de l'or
/// (un métal) ; derrière : du plastique rouge (un isolant), alvéolé comme
/// une balle de golf par une normal map.
///
/// Comparer les deux rangées montre ce que change `metallic` : le reflet de
/// l'or est doré et il n'a presque pas de couleur « diffuse » ; celui du
/// plastique est blanc, posé sur une couleur rouge visible sous tous les angles.
fn spawn_material_showcase(world: &mut hecs::World, assets: &DemoAssets, half: f32) {
    const COUNT: i32 = 5;
    let gold = Color::rgb(1.0, 0.77, 0.34);
    let red = Color::rgb(0.8, 0.08, 0.06);
    let rows = [
        ("or", Material::metal(gold, 0.0), None, half + 2.9),
        (
            "plastique",
            Material::dielectric(red, 0.0),
            Some(assets.dimples_normal_map),
            half + 1.7,
        ),
    ];
    for (label, material, normal_map, z) in rows {
        for i in 0..COUNT {
            let roughness = 0.1 + 0.2 * i as f32; // 0,1 · 0,3 · 0,5 · 0,7 · 0,9
            world.spawn((
                Name::new(format!("Sphère {label} (rugosité {roughness:.1})")),
                // Rayon 0,5 : posée sur le sol.
                Transform::from_translation(Vec3::new(
                    i as f32 * GRID_SPACING - half,
                    GROUND_Y + 0.5,
                    z,
                )),
                MeshRenderer::new(assets.sphere)
                    .with_textures(MaterialTextures {
                        normal: normal_map,
                        ..MaterialTextures::default()
                    })
                    .with_material(Material {
                        roughness,
                        ..material
                    }),
                // Immobiles, mais les balles rebondissent dessus.
                RigidBody::fixed(),
                Collider::ball(0.5),
            ));
        }
    }
}

/// Une pyramide de caisses **dynamiques** derrière la grille : 4 + 3 + 2 + 1.
/// Elles tiennent en équilibre... jusqu'à ce qu'une balle les touche.
fn spawn_crate_pile(world: &mut hecs::World, assets: &DemoAssets, half: f32) {
    const SIZE: f32 = 0.7;
    let mut n = 0;
    for row in 0..4 {
        let count = 4 - row;
        for i in 0..count {
            n += 1;
            // Chaque rangée est décalée d'une demi-caisse : une vraie pyramide.
            let x = (i as f32 - (count - 1) as f32 / 2.0) * (SIZE + 0.02);
            let y = GROUND_Y + SIZE / 2.0 + row as f32 * SIZE;
            world.spawn((
                Name::new(format!("Caisse empilée {n}")),
                Transform::from_translation(Vec3::new(x, y, -half - 1.8)).with_uniform_scale(SIZE),
                MeshRenderer::new(assets.white_cube)
                    .with_texture(assets.crate_texture)
                    .with_material(Material::dielectric(Color::WHITE, 0.85)),
                RigidBody::dynamic(),
                // Densité du bois : environ la moitié de celle de l'eau.
                Collider::cuboid(Vec3::splat(0.5)).with_density(0.6),
            ));
        }
    }
}

/// À gauche de la grille : une grande vitre et, derrière elle, trois cubes
/// de verre colorés. Regardés à travers la vitre, ils montrent que les
/// objets transparents sont bien dessinés du plus loin au plus proche :
/// chaque couche teinte celles qui sont derrière elle.
fn spawn_glass(world: &mut hecs::World, assets: &DemoAssets, half: f32) {
    world.spawn((
        Name::new("Vitre"),
        // Un cube aplati : une plaque de 6 cm d'épaisseur.
        Transform {
            scale: Vec3::new(0.06, 1.4, 5.0),
            ..Transform::from_translation(Vec3::new(-half - 1.1, GROUND_Y + 0.7, 0.0))
        },
        MeshRenderer::new(assets.white_cube)
            .with_material(Material::glass(Color::rgb(0.85, 0.95, 1.0), 0.12)),
        RigidBody::fixed(),
        Collider::cuboid(Vec3::splat(0.5)),
    ));
    let cubes = [
        ("cyan", Color::rgb(0.2, 0.8, 0.9), 0.35, -2.0),
        ("ambre", Color::rgb(1.0, 0.6, 0.1), 0.45, 0.0),
        ("magenta", Color::rgb(0.9, 0.2, 0.7), 0.3, 2.0),
    ];
    for (i, (label, color, alpha, z)) in cubes.into_iter().enumerate() {
        world.spawn((
            Name::new(format!("Cube de verre {label}")),
            Transform::from_translation(Vec3::new(-half - 2.4, GROUND_Y + 0.6, z))
                .with_uniform_scale(0.9),
            MeshRenderer::new(assets.white_cube).with_material(Material::glass(color, alpha)),
            Spin {
                axis: Vec3::Y,
                speed: 0.3 + 0.1 * i as f32,
            },
            RigidBody::kinematic(),
            Collider::cuboid(Vec3::splat(0.5)),
        ));
    }
}

/// À droite de la grille : une **lanterne** grillagée. Le grillage est une
/// texture dont les trous ont un alpha nul, en mode découpe et double face :
/// on voit l'intérieur de la cage à travers les trous, ses faces du fond
/// comprises, et une petite sphère lumineuse au centre. Au soleil, la cage
/// projette une ombre trouée.
fn spawn_lantern(world: &mut hecs::World, assets: &DemoAssets, half: f32) {
    let center = Vec3::new(half + 2.0, GROUND_Y + 0.7, 0.0);
    world.spawn((
        Name::new("Lanterne grillagée"),
        Transform::from_translation(center).with_uniform_scale(1.4),
        MeshRenderer::new(assets.white_cube)
            .with_texture(assets.lattice_texture)
            .with_material(Material {
                alpha_mode: AlphaMode::Mask { cutoff: 0.5 },
                double_sided: true,
                ..Material::metal(Color::rgb(0.75, 0.55, 0.35), 0.4)
            }),
        RigidBody::fixed(),
        Collider::cuboid(Vec3::splat(0.5)),
    ));
    world.spawn((
        Name::new("Flamme de la lanterne"),
        Transform::from_translation(center).with_uniform_scale(0.5),
        MeshRenderer::new(assets.sphere).with_material(Material {
            emissive: Color::rgb(4.0, 1.8, 0.5),
            ..Material::dielectric(Color::rgb(1.0, 0.6, 0.2), 0.5)
        }),
        // La flamme éclaire vraiment autour d'elle. (Sans ombres : la
        // lumière traverse les barreaux sans en projeter l'image.)
        PointLight {
            color: Color::rgb(1.0, 0.55, 0.2),
            intensity: 3.0,
            range: 6.0,
        },
    ));
}

/// Trois lampes colorées (rouge, vert, bleu) qui tournent au-dessus de la
/// grille, accrochées à un pivot invisible qui tourne (la même astuce que
/// le système solaire). Chacune est une petite sphère lumineuse avec une
/// `PointLight` : on voit leurs couleurs glisser sur les cubes et le sol.
///
/// Et un projecteur au-dessus de la vitrine de sphères, tourné vers le bas.
fn spawn_lights(world: &mut hecs::World, assets: &DemoAssets, half: f32) {
    let pivot = world.spawn((
        Name::new("Manège de lampes"),
        Transform::from_translation(Vec3::new(0.0, GROUND_Y + 1.6, 0.0)),
        Spin {
            axis: Vec3::Y,
            speed: 0.5,
        },
    ));
    let lamps = [
        ("rouge", Color::rgb(1.0, 0.15, 0.1)),
        ("verte", Color::rgb(0.2, 1.0, 0.3)),
        ("bleue", Color::rgb(0.2, 0.4, 1.0)),
    ];
    for (i, (label, color)) in lamps.into_iter().enumerate() {
        let angle = i as f32 * std::f32::consts::TAU / 3.0;
        world.spawn((
            Name::new(format!("Lampe {label}")),
            Transform::from_translation(Vec3::new(angle.cos(), 0.0, angle.sin()) * 3.2)
                .with_uniform_scale(0.18),
            Parent(pivot),
            MeshRenderer::new(assets.sphere).with_material(Material {
                emissive: Color::rgb(color.r * 6.0, color.g * 6.0, color.b * 6.0),
                ..Material::dielectric(color, 0.5)
            }),
            PointLight {
                color,
                intensity: 2.5,
                range: 7.0,
            },
        ));
    }

    // Un projecteur à 4 m de haut, au-dessus de la vitrine, braqué vers le
    // bas : son -Z pointe vers le sol après un quart de tour autour de X.
    world.spawn((
        Name::new("Projecteur"),
        Transform::from_translation(Vec3::new(0.0, GROUND_Y + 4.5, half + 2.3))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        SpotLight {
            color: Color::rgb(1.0, 0.92, 0.75),
            intensity: 10.0,
            range: 9.0,
            inner_angle: 18f32.to_radians(),
            outer_angle: 30f32.to_radians(),
        },
    ));
}

/// Une texture de **grillage** : des barreaux pleins (alpha 255) qui se
/// croisent, et des trous (alpha 0). `bars` barreaux dans chaque sens.
pub fn lattice_image(size: u32, bars: u32) -> Image {
    let cell = size / bars;
    // Le barreau occupe un tiers de chaque cellule, centré.
    let is_bar = |c: u32| {
        let t = c % cell;
        t >= cell / 3 && t < cell - cell / 3
    };
    Image::from_fn(size, size, |x, y| {
        if is_bar(x) || is_bar(y) {
            // Un peu plus clair au croisement des barreaux (les soudures).
            let shade = if is_bar(x) && is_bar(y) { 255 } else { 215 };
            [shade, shade, shade, 255]
        } else {
            [0, 0, 0, 0]
        }
    })
}

/// Lance une balle depuis `from` (la caméra), droit devant elle.
pub fn launch_ball(world: &mut hecs::World, assets: &DemoAssets, from: &Transform, serial: u32) {
    let forward = from.rotation * Vec3::NEG_Z;
    world.spawn((
        Name::new(format!("Balle {serial}")),
        // Un peu devant l'œil, pour ne pas apparaître dans la caméra.
        Transform::from_translation(from.translation + forward * 0.8).with_uniform_scale(0.5),
        MeshRenderer::new(assets.sphere)
            .with_material(Material::dielectric(Color::rgb(1.0, 0.35, 0.05), 0.35)),
        RigidBody::dynamic().with_velocity(forward * BALL_SPEED),
        // Rayon 0,5 × échelle 0,5 = 25 cm. Dense et un peu rebondissante.
        Collider::ball(0.5).with_density(3.0).with_restitution(0.5),
        Ball { serial },
    ));
}

/// Système : supprime ce qui est tombé du monde (plus bas que y = -20) et
/// les balles en trop (les plus anciennes d'abord).
pub fn cleanup_system(world: &mut hecs::World) {
    let mut doomed: Vec<Entity> = world
        .query::<(Entity, &Transform, &RigidBody)>()
        .iter()
        .filter(|(_, t, _)| t.translation.y < -20.0)
        .map(|(e, _, _)| e)
        .collect();

    let mut balls: Vec<(u32, Entity)> = world
        .query::<(Entity, &Ball)>()
        .iter()
        .map(|(e, b)| (b.serial, e))
        .collect();
    if balls.len() > MAX_BALLS {
        balls.sort_unstable();
        let excess = balls.len() - MAX_BALLS;
        doomed.extend(balls[..excess].iter().map(|&(_, e)| e));
    }
    for entity in doomed {
        // Déjà supprimée (comptée deux fois) : sans importance.
        let _ = world.despawn(entity);
    }
}

/// Composant du jeu : « ce cube fait partie du champ de stress ». Il permet
/// de retirer le champ d'un coup.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct StressCube;

/// Le champ de stress compte `STRESS_SIDE²` cubes.
pub const STRESS_SIDE: i32 = 64;
const STRESS_SPACING: f32 = 2.5;

/// Le **champ de stress** (touche `N`) : 4 096 cubes qui tournent, en nappe
/// ondulée au-dessus et autour de la scène, sur 160 m de côté. De quoi
/// mettre le moteur à l'épreuve et voir le culling travailler : la caméra
/// n'en voit jamais qu'une partie, et seuls ceux qui survolent le sol
/// projettent une ombre dans la zone d'ombre.
pub fn spawn_stress_field(world: &mut hecs::World, assets: &DemoAssets) {
    let half = STRESS_SIDE as f32 * STRESS_SPACING / 2.0;
    let cold = Color::rgb(0.15, 0.45, 0.95);
    let warm = Color::rgb(0.95, 0.35, 0.25);
    // `spawn_batch` crée toutes les entités d'un coup, plus vite qu'une à une.
    world.spawn_batch((0..STRESS_SIDE * STRESS_SIDE).map(|i| {
        let (column, row) = ((i % STRESS_SIDE) as f32, (i / STRESS_SIDE) as f32);
        let x = column * STRESS_SPACING - half;
        let z = row * STRESS_SPACING - half;
        let y = 6.0 + 2.0 * (x * 0.15).sin() * (z * 0.15).cos();
        // La couleur passe du bleu (bas) au rouge (haut) avec la hauteur.
        let color = cold.lerp(warm, (y - 4.0) / 4.0);
        (
            Transform::from_translation(Vec3::new(x, y, z)).with_uniform_scale(0.8),
            MeshRenderer::new(assets.white_cube).with_material(Material::dielectric(color, 0.5)),
            Spin {
                axis: Vec3::new(column.sin(), 1.0, row.cos()).normalize(),
                speed: 0.6 + (i % 7) as f32 * 0.1,
            },
            StressCube,
        )
    }));
}

/// Retire le champ de stress. Renvoie le nombre de cubes retirés.
pub fn remove_stress_field(world: &mut hecs::World) -> usize {
    let cubes: Vec<Entity> = world
        .query::<hecs::With<Entity, &StressCube>>()
        .iter()
        .collect();
    for &entity in &cubes {
        let _ = world.despawn(entity);
    }
    cubes.len()
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
        // Le soleil BRILLE : il émet sa propre lumière (sans éclairer les
        // autres objets pour autant), visible même dans l'ombre.
        MeshRenderer::new(assets.sun).with_material(Material {
            emissive: Color::rgb(1.2, 0.7, 0.2),
            ..Material::dielectric(Color::WHITE, 0.3)
        }),
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
        // Une planète en métal bleuté (la couleur vient des sommets).
        MeshRenderer::new(assets.planet).with_material(Material::metal(Color::WHITE, 0.35)),
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
        MeshRenderer::new(assets.moon).with_material(Material::dielectric(Color::WHITE, 0.95)),
        Parent(planet),
    ));
}

/// Deux caisses « sci-fi » de part et d'autre de la vitrine, qui utilisent
/// toutes les textures d'un matériau (voir [`panel_images`]). Elles tournent
/// lentement : on voit le ciel glisser sur leurs plaques de métal.
fn spawn_panel_crates(world: &mut hecs::World, assets: &DemoAssets, half: f32) {
    let material = Material {
        // Métal et rugosité viennent de la texture (multipliée par 1).
        metallic: 1.0,
        roughness: 1.0,
        // Multiplie la texture d'émission : des néons bien lumineux.
        emissive: Color::rgb(2.0, 2.0, 2.0),
        ..Material::DEFAULT
    };
    for (i, side) in [-1.0f32, 1.0].into_iter().enumerate() {
        world.spawn((
            Name::new(format!("Caisse sci-fi {}", i + 1)),
            Transform::from_translation(Vec3::new(
                side * (half + 1.6),
                GROUND_Y + 0.65,
                half + 0.6,
            ))
            .with_uniform_scale(1.3),
            MeshRenderer::new(assets.white_cube)
                .with_textures(assets.panel)
                .with_material(material),
            Spin {
                axis: Vec3::Y,
                speed: 0.25 * side,
            },
            RigidBody::kinematic(),
            Collider::cuboid(Vec3::splat(0.5)),
            // Les néons bourdonnent : plus fort quand on s'approche.
            SoundEmitter::new(HUM_SOUND, 0.35),
            // ... et éclairent le sol autour d'eux, en cyan.
            PointLight {
                color: Color::rgb(0.3, 0.85, 1.0),
                intensity: 1.5,
                range: 4.0,
            },
        ));
    }
}

/// Les 5 images d'un matériau complet : une plaque de métal découpée en
/// `cells × cells` panneaux par des rainures où brille un néon.
pub struct PanelImages {
    pub base_color: Image,
    pub normal: Image,
    pub metallic_roughness: Image,
    pub occlusion: Image,
    pub emissive: Image,
}

/// Fabrique les images d'une plaque « sci-fi » de `size × size` pixels :
///
/// - un panneau sur deux est en **métal** poli, les autres en **peinture**
///   mate (texture métal/rugosité) ;
/// - les **rainures** entre les panneaux sont creusées (normal map) et
///   assombries (occlusion : la lumière ambiante y entre mal) ;
/// - un **néon** cyan court au fond des rainures (émission).
pub fn panel_images(size: u32, cells: u32) -> PanelImages {
    let cell = size as f32 / cells as f32;
    let groove = cell * 0.08;
    // Distance (en pixels) au bord de panneau le plus proche, et numéro du panneau.
    let to_edge = |p: u32| {
        let p = (p as f32 + 0.5) % cell;
        p.min(cell - p)
    };
    let distance = move |x: u32, y: u32| to_edge(x).min(to_edge(y));
    let is_metal = move |x: u32, y: u32| {
        ((x as f32 / cell) as u32 + (y as f32 / cell) as u32).is_multiple_of(2)
    };

    let base_color = Image::from_fn(size, size, |x, y| {
        let gray = if distance(x, y) < groove {
            Color::rgb(0.03, 0.03, 0.035)
        } else if is_metal(x, y) {
            Color::rgb(0.75, 0.76, 0.78) // couleur des reflets du métal
        } else {
            Color::rgb(0.16, 0.18, 0.22) // peinture gris-bleu
        };
        gray.to_srgba8()
    });
    // Données (pas de sRGB) : vert = rugosité, bleu = métal, de 0 à 255.
    let metallic_roughness = Image::from_fn(size, size, |x, y| {
        let (roughness, metallic) = if distance(x, y) < groove {
            (0.9, 0.0)
        } else if is_metal(x, y) {
            (0.22, 1.0)
        } else {
            (0.65, 0.0)
        };
        [0, (roughness * 255.0) as u8, (metallic * 255.0) as u8, 255]
    });
    let occlusion = Image::from_fn(size, size, |x, y| {
        // De 0,3 au fond de la rainure à 1 un peu plus loin.
        let t = (distance(x, y) / (groove * 2.5)).min(1.0);
        let ao = 0.3 + 0.7 * t * t * (3.0 - 2.0 * t); // « smoothstep »
        let v = (ao * 255.0) as u8;
        [v, v, v, 255]
    });
    let emissive = Image::from_fn(size, size, |x, y| {
        if distance(x, y) < groove * 0.35 {
            Color::rgb(0.05, 0.75, 1.0).to_srgba8()
        } else {
            [0, 0, 0, 255]
        }
    });
    // La rainure est un creux de profondeur `groove / 2`, aux bords inclinés.
    let normal = Image::normal_map_from_heights(size, size, 1.0, |x, y| {
        (distance(x, y) - groove * 0.5).clamp(0.0, groove * 0.5)
    });
    PanelImages {
        base_color,
        normal,
        metallic_roughness,
        occlusion,
        emissive,
    }
}

/// Les noms des textures de la plaque dans les scènes sauvegardées.
pub fn panel_texture_names(panel: &MaterialTextures) -> Vec<(&'static str, TextureHandle)> {
    [
        ("plaque_couleur", panel.base_color),
        ("plaque_relief", panel.normal),
        ("plaque_metal_rugosite", panel.metallic_roughness),
        ("plaque_occlusion", panel.occlusion),
        ("plaque_emission", panel.emissive),
    ]
    .into_iter()
    // `filter_map` : garde les textures présentes, sans le `Some`.
    .filter_map(|(name, texture)| Some((name, texture?)))
    .collect()
}

/// La normal map du sol : `cells × cells` dalles dont les bords sont
/// biseautés et séparés par des joints creux. La texture du sol (un damier
/// de 2 × 2 cases) se répète comme elle : chaque case est une dalle.
pub fn tiles_normal_map(size: u32, cells: u32) -> Image {
    let cell = size as f32 / cells as f32;
    // Largeur du biseau, en pixels.
    let bevel = cell * 0.08;
    Image::normal_map_from_heights(size, size, 1.0, |x, y| {
        // Distance (en pixels) au bord de dalle le plus proche.
        let to_edge = |p: u32| {
            let p = (p as f32 + 0.5) % cell;
            p.min(cell - p)
        };
        let d = to_edge(x).min(to_edge(y));
        // 0 dans le joint, monte sur le biseau, plat (= `bevel`) sur la dalle.
        (d - bevel * 0.3).clamp(0.0, bevel)
    })
}

/// La normal map des sphères en plastique : des alvéoles rondes, disposées
/// en `columns × rows` sur l'image. Une sphère UV étire son image deux fois
/// plus en largeur (tour complet) qu'en hauteur (pôle à pôle) : l'image fait
/// donc deux fois plus de colonnes que de lignes pour des alvéoles rondes
/// à l'équateur.
pub fn dimples_normal_map(width: u32, height: u32, columns: u32, rows: u32) -> Image {
    let (cw, ch) = (width as f32 / columns as f32, height as f32 / rows as f32);
    let radius = cw.min(ch) * 0.42;
    Image::normal_map_from_heights(width, height, 1.5, |x, y| {
        let dx = (x as f32 + 0.5) % cw - cw / 2.0;
        let dy = (y as f32 + 0.5) % ch - ch / 2.0;
        let d2 = (dx * dx + dy * dy) / (radius * radius);
        // Un creux en forme de bol, de profondeur `radius / 3`.
        if d2 < 1.0 {
            -(1.0 - d2) * radius / 3.0
        } else {
            0.0
        }
    })
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
            lattice_texture: TextureHandle::from_index(10),
            ground_texture: TextureHandle::from_index(2),
            ground_normal_map: TextureHandle::from_index(3),
            dimples_normal_map: TextureHandle::from_index(4),
            panel: MaterialTextures {
                base_color: Some(TextureHandle::from_index(5)),
                normal: Some(TextureHandle::from_index(6)),
                metallic_roughness: Some(TextureHandle::from_index(7)),
                occlusion: Some(TextureHandle::from_index(8)),
                emissive: Some(TextureHandle::from_index(9)),
            },
            sun: MeshHandle::from_index(3),
            planet: MeshHandle::from_index(4),
            moon: MeshHandle::from_index(5),
            sphere: MeshHandle::from_index(6),
            tree: None,
        }
    }

    #[test]
    fn le_champ_de_stress_ne_deplace_pas_la_camera() {
        let mut scene = Scene::default();
        spawn(&mut scene, &assets());
        scene.update_transforms();
        let before = scene.camera().unwrap();
        spawn_stress_field(&mut scene.world, &assets());
        for _ in 0..10 {
            spin_system(&mut scene.world, 1.0 / 60.0);
            scene.update_transforms();
        }
        assert_eq!(scene.camera().unwrap(), before);
    }

    #[test]
    fn verre_et_grillage() {
        let mut scene = Scene::default();
        spawn(&mut scene, &assets());
        let materials: Vec<Material> = scene
            .world
            .query::<&MeshRenderer>()
            .iter()
            .map(|r| r.material)
            .collect();
        let glass = materials.iter().filter(|m| m.is_transparent()).count();
        assert_eq!(glass, 4, "la vitre et les 3 cubes");
        assert!(
            materials
                .iter()
                .any(|m| matches!(m.alpha_mode, AlphaMode::Mask { .. }) && m.double_sided)
        );
    }

    #[test]
    fn la_texture_de_grillage_a_des_trous() {
        let image = lattice_image(64, 4);
        let alphas: Vec<u8> = (0..64).map(|x| image.pixel(x, 2)[3]).collect();
        assert!(alphas.contains(&0) && alphas.contains(&255));
        // Au milieu d'une cellule (un barreau vertical), toujours plein.
        assert_eq!(image.pixel(8, 0)[3], 255);
    }

    #[test]
    fn champ_de_stress() {
        let mut scene = Scene::default();
        spawn(&mut scene, &assets());
        let before = scene.entity_count();
        spawn_stress_field(&mut scene.world, &assets());
        let cubes = (STRESS_SIDE * STRESS_SIDE) as usize;
        assert_eq!(scene.entity_count() as usize, before as usize + cubes);
        let mut draws = Vec::new();
        scene.extract_draws(&mut draws);
        assert_eq!(draws.len(), 60 + cubes);
        assert_eq!(remove_stress_field(&mut scene.world), cubes);
        assert_eq!(scene.entity_count(), before);
    }

    #[test]
    fn la_scene_contient_camera_sol_et_cubes() {
        let mut scene = Scene::default();
        let camera = spawn(&mut scene, &assets());
        // 1 caméra + 1 lumière + 1 sol + 25 cubes + soleil, planète, lune
        // + 10 sphères + 2 caisses sci-fi + 10 caisses empilées.
        // + la vitre, 3 cubes de verre, la lanterne et sa flamme.
        // + le manège, ses 3 lampes et le projecteur.
        assert_eq!(scene.entity_count(), 64);
        assert!(scene.world.get::<&Camera3d>(camera).is_ok());

        let mut draws = Vec::new();
        scene.extract_draws(&mut draws);
        assert_eq!(
            draws.len(),
            60,
            "ni la caméra ni la lumière ne sont dessinées"
        );
        assert!(scene.lighting().intensity > 0.0, "la scène a un soleil");
        // 13 caisses texturées (cases paires du damier 5×5) + 10 empilées.
        let caisses = draws
            .iter()
            .filter(|d| d.textures.base_color == Some(assets().crate_texture))
            .count();
        assert_eq!(caisses, 23);
        // La vitrine : 5 sphères en or, de plus en plus rugueuses.
        let mut or: Vec<f32> = draws
            .iter()
            .filter(|d| d.material.metallic == 1.0 && d.mesh == assets().sphere)
            .map(|d| d.material.roughness)
            .collect();
        or.sort_by(f32::total_cmp);
        assert_eq!(or.len(), 5);
        assert!(or.windows(2).all(|w| w[0] < w[1]));
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
    fn la_scene_de_demo_survit_a_une_sauvegarde() {
        let assets = assets();
        let mut names = zentrix_scene::AssetNames::default();
        for (name, mesh) in [
            ("cube", assets.cube),
            ("cube_blanc", assets.white_cube),
            ("sol", assets.ground),
            ("soleil", assets.sun),
            ("planete", assets.planet),
            ("lune", assets.moon),
            ("sphere", assets.sphere),
        ] {
            names.add_mesh(name, mesh);
        }
        names.add_texture("caisse", assets.crate_texture);
        names.add_texture("damier", assets.ground_texture);
        names.add_texture("grillage", assets.lattice_texture);
        names.add_texture("dalles_relief", assets.ground_normal_map);
        names.add_texture("alveoles_relief", assets.dimples_normal_map);
        for (name, texture) in panel_texture_names(&assets.panel) {
            names.add_texture(name, texture);
        }

        let mut scene = Scene::default();
        spawn(&mut scene, &assets);
        spin_system(&mut scene.world, 1.3); // un état « en cours de partie »

        let serializer = crate::scene_serializer();
        let texte = serializer.save(&scene, &names).expect("sauvegarde");
        let mut copie = Scene::default();
        serializer
            .load(&mut copie, &texte, &names)
            .expect("chargement");

        assert_eq!(copie.entity_count(), scene.entity_count());
        // Les composants du jeu (Spin, FlyCamera) ont bien été restitués...
        // 25 cubes + soleil + planète + lumière + 2 caisses sci-fi + 3 cubes
        // de verre (la lune, elle, est emportée par sa planète).
        assert_eq!(copie.world.query::<&Spin>().iter().count(), 34);
        // Les lampes et le projecteur aussi.
        assert_eq!(copie.world.query::<&PointLight>().iter().count(), 6);
        assert_eq!(copie.world.query::<&SpotLight>().iter().count(), 1);
        assert_eq!(copie.world.query::<&FlyCamera>().iter().count(), 1);
        // ... et une nouvelle sauvegarde redonne exactement le même fichier.
        assert_eq!(serializer.save(&copie, &names).unwrap(), texte);
    }

    #[test]
    fn la_vitrine_et_le_sol_ont_du_relief() {
        let mut scene = Scene::default();
        spawn(&mut scene, &assets());
        let mut draws = Vec::new();
        scene.extract_draws(&mut draws);
        let avec_relief = |map| {
            draws
                .iter()
                .filter(|d| d.textures.normal == Some(map))
                .count()
        };
        assert_eq!(
            avec_relief(assets().dimples_normal_map),
            5,
            "sphères en plastique"
        );
        assert_eq!(avec_relief(assets().ground_normal_map), 1, "le sol");
        let completes = draws
            .iter()
            .filter(|d| d.textures == assets().panel)
            .count();
        assert_eq!(completes, 2, "les caisses sci-fi utilisent les 5 textures");
    }

    #[test]
    fn normal_maps_generees() {
        // Au centre d'une dalle : plat. Dans un joint : plat aussi (fond du
        // joint). Sur le biseau gauche d'une dalle : la normale penche vers
        // la gauche (rouge < 128).
        let dalles = tiles_normal_map(128, 2);
        assert_eq!(dalles.pixel(32, 32), [128, 128, 255, 255]);
        let [r, g, _, _] = dalles.pixel(2, 32);
        assert!(r < 128 && g == 128, "{r} {g}");

        let alveoles = dimples_normal_map(64, 32, 8, 4);
        // Alvéoles de 8 × 8 pixels, centrées entre les pixels 3 et 4. Côté
        // droit, la paroi monte vers la droite : la normale penche vers la
        // gauche (rouge < 128) ; côté gauche, l'inverse.
        let [droite, _, _, _] = alveoles.pixel(6, 3);
        let [gauche, _, _, _] = alveoles.pixel(1, 3);
        assert!(droite < 128 && gauche > 128, "{droite} {gauche}");
        // Entre deux alvéoles (coin de la case) : plat.
        assert_eq!(alveoles.pixel(7, 7)[..3], [128, 128, 255]);
    }

    #[test]
    fn la_physique_de_la_demo() {
        use zentrix_physics::{BodyKind, Physics};

        let mut scene = Scene::default();
        spawn(&mut scene, &assets());
        let compter = |kind| {
            scene
                .world
                .query::<&RigidBody>()
                .iter()
                .filter(|b| b.kind == kind)
                .count()
        };
        assert_eq!(compter(BodyKind::Dynamic), 10, "la pile de caisses");
        assert_eq!(
            compter(BodyKind::Kinematic),
            30,
            "25 cubes + 2 caisses sci-fi + 3 cubes de verre"
        );
        assert_eq!(
            compter(BodyKind::Fixed),
            13,
            "le sol + 10 sphères + la vitre + la lanterne"
        );

        // Deux secondes de simulation : la pile tient debout, posée sur le sol.
        let mut physics = Physics::new();
        for _ in 0..120 {
            physics.step(&mut scene, 1.0 / 60.0);
        }
        assert_eq!(physics.body_count(), 53);
        for (name, t) in scene.world.query::<(&Name, &Transform)>().iter() {
            if name.0 == "Caisse empilée 1" {
                assert!((t.translation.y - (GROUND_Y + 0.35)).abs() < 0.02, "{t:?}");
            }
            if name.0 == "Caisse empilée 10" {
                assert!(t.translation.y > GROUND_Y + 2.3, "le sommet tient : {t:?}");
            }
        }
    }

    #[test]
    fn les_balles_en_trop_disparaissent() {
        let mut scene = Scene::default();
        let camera = Transform::from_translation(Vec3::new(0.0, 2.0, 5.0));
        for serial in 0..(MAX_BALLS as u32 + 5) {
            launch_ball(&mut scene.world, &assets(), &camera, serial);
        }
        // Une balle tombée hors du monde.
        let chute = scene.world.spawn((
            Transform::from_translation(Vec3::new(0.0, -50.0, 0.0)),
            RigidBody::dynamic(),
        ));
        cleanup_system(&mut scene.world);
        assert_eq!(scene.world.query::<&Ball>().iter().count(), MAX_BALLS);
        assert!(!scene.world.contains(chute));
        // Les 5 plus anciennes (0 à 4) ont disparu.
        let plus_ancienne = scene.world.query::<&Ball>().iter().map(|b| b.serial).min();
        assert_eq!(plus_ancienne, Some(5));
    }

    #[test]
    fn l_audio_de_la_demo() {
        let mut scene = Scene::default();
        let camera = spawn(&mut scene, &assets());
        assert!(scene.world.get::<&AudioListener>(camera).is_ok());
        let emetteurs = scene.world.query::<&SoundEmitter>().iter().count();
        assert_eq!(emetteurs, 2, "les deux caisses sci-fi");
    }

    #[test]
    fn plaque_sci_fi() {
        let p = panel_images(64, 2);
        // Centre du panneau (16, 16) : métal poli, sans occlusion ni néon.
        assert_eq!(p.metallic_roughness.pixel(16, 16), [0, 56, 255, 255]);
        assert_eq!(p.occlusion.pixel(16, 16)[0], 255);
        assert_eq!(p.emissive.pixel(16, 16), [0, 0, 0, 255]);
        // Panneau voisin (48, 16) : peinture, non métallique.
        assert_eq!(p.metallic_roughness.pixel(48, 16)[2], 0);
        // Au fond de la rainure (x = 0) : néon allumé, occlusion forte.
        assert!(p.emissive.pixel(0, 16)[2] > 200);
        assert!(p.occlusion.pixel(0, 16)[0] < 100);
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
