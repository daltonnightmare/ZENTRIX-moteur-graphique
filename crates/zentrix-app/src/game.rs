//! Le petit gameplay de test : « la chasse aux cristaux ».
//!
//! `G` lance une partie de 60 secondes dans la scène de démonstration :
//!
//! - des **cristaux** lumineux flottent autour de la grille : on les ramasse
//!   en passant dessus avec la caméra (+10 points) ;
//! - des **cibles** rouges entourent l'arène : on les touche avec les balles
//!   lancées par `B` (+25 points), et elles tombent ;
//! - tout ramasser et tout toucher avant la fin rapporte un bonus de 2 points
//!   par seconde restante.
//!
//! C'est le premier test de l'API du moteur « du point de vue d'un jeu » :
//! la partie n'utilise que des entités, des composants, les chocs de la
//! physique (`Physics::impacts`) et l'interface egui.
//!
//! La logique ([`Game::start`], [`Game::update`]) ne touche ni au GPU ni au
//! son : elle renvoie des [`GameEvent`], que `main.rs` transforme en sons.
//! Elle est donc testée sans fenêtre (voir les tests en bas du fichier).

use zentrix_core::math::{Quat, Vec3};
use zentrix_core::{Color, Material, Transform};
use zentrix_physics::{Collider, Impact, RigidBody};
use zentrix_scene::{Entity, MeshRenderer, Name, PointLight, hecs};
use zentrix_ui::egui;

use crate::demo::{Ball, DemoAssets, GROUND_Y, Spin};

/// Durée d'une partie, en secondes.
pub const ROUND_TIME: f32 = 60.0;
pub const CRYSTAL_COUNT: usize = 8;
pub const TARGET_COUNT: usize = 5;
pub const CRYSTAL_POINTS: u32 = 10;
pub const TARGET_POINTS: u32 = 25;
/// Bonus par seconde restante, si tout est ramassé et touché.
pub const TIME_BONUS: u32 = 2;
/// Distance entre la caméra et un cristal pour le ramasser.
const PICKUP_RADIUS: f32 = 1.2;
/// Les cristaux sont répartis dans cet anneau autour de l'origine : hors de
/// la grille de cubes, à l'intérieur du sol (16 × 16).
const CRYSTAL_RING: (f32, f32) = (4.8, 7.0);
/// Les cibles sont sur un cercle de ce rayon, tournées vers le centre.
const TARGET_RADIUS: f32 = 7.4;
/// Combien de temps un message (« +10 cristal ») reste affiché.
const MESSAGE_TIME: f32 = 1.6;

/// Composant : « cette entité a été créée par la partie ». Une nouvelle
/// partie, ou le rechargement d'une scène, les supprime toutes.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct GameItem;

/// Composant : un cristal à ramasser.
#[derive(Debug, Clone, Copy)]
pub struct Crystal;

/// Composant : une cible à toucher avec une balle.
#[derive(Debug, Clone, Copy)]
pub struct Target;

/// Ce qui vient de se passer dans la partie (pour jouer des sons, écrire
/// dans le journal...).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameEvent {
    Started,
    Collected {
        position: Vec3,
    },
    TargetHit {
        position: Vec3,
    },
    Finished {
        score: u32,
        perfect: bool,
        record: bool,
    },
}

/// Où en est la partie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Pas encore de partie : le HUD invite à appuyer sur `G`.
    Idle,
    Playing,
    /// La partie est finie : le HUD affiche le score.
    Over {
        perfect: bool,
        record: bool,
    },
}

/// L'état de la partie.
#[derive(Debug, Clone)]
pub struct Game {
    pub phase: Phase,
    pub score: u32,
    pub time_left: f32,
    pub crystals_left: usize,
    pub targets_left: usize,
    /// Le meilleur score depuis le lancement du programme.
    pub best: u32,
    /// Numéro de la partie : il change la place des cristaux.
    round: u32,
    /// Les derniers messages et leur âge, en secondes.
    messages: Vec<(String, f32)>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            phase: Phase::Idle,
            score: 0,
            time_left: 0.0,
            crystals_left: 0,
            targets_left: 0,
            best: 0,
            round: 0,
            messages: Vec::new(),
        }
    }
}

impl Game {
    /// Lance une nouvelle partie : supprime ce que la précédente avait créé,
    /// puis place les cristaux et les cibles.
    pub fn start(&mut self, world: &mut hecs::World, assets: &DemoAssets) -> GameEvent {
        despawn_items(world);
        self.round += 1;
        for position in crystal_positions(self.round) {
            spawn_crystal(world, assets, position);
        }
        for (i, angle) in target_angles().into_iter().enumerate() {
            spawn_target(world, assets, i, angle);
        }
        *self = Self {
            phase: Phase::Playing,
            time_left: ROUND_TIME,
            crystals_left: CRYSTAL_COUNT,
            targets_left: TARGET_COUNT,
            best: self.best,
            round: self.round,
            ..Self::default()
        };
        GameEvent::Started
    }

    /// Abandonne la partie (la scène vient d'être remplacée) : on revient à
    /// l'écran d'accueil, en gardant le record.
    pub fn reset(&mut self, world: &mut hecs::World) {
        despawn_items(world);
        *self = Self {
            best: self.best,
            round: self.round,
            ..Self::default()
        };
    }

    /// Une image de jeu. `player` : la position de la caméra ; `impacts` :
    /// les chocs du dernier pas de physique.
    pub fn update(
        &mut self,
        world: &mut hecs::World,
        assets: &DemoAssets,
        impacts: &[Impact],
        player: Vec3,
        dt: f32,
    ) -> Vec<GameEvent> {
        for (_, age) in &mut self.messages {
            *age += dt;
        }
        self.messages.retain(|(_, age)| *age < MESSAGE_TIME);

        let mut events = Vec::new();
        if self.phase != Phase::Playing {
            return events;
        }

        // Les cristaux assez proches de la caméra sont ramassés.
        let collected: Vec<(Entity, Vec3)> = world
            .query::<hecs::With<(Entity, &Transform), &Crystal>>()
            .iter()
            .filter(|(_, t)| t.translation.distance(player) < PICKUP_RADIUS)
            .map(|(e, t)| (e, t.translation))
            .collect();
        for (entity, position) in collected {
            let _ = world.despawn(entity);
            self.crystals_left -= 1;
            self.add_points(CRYSTAL_POINTS, "cristal");
            events.push(GameEvent::Collected { position });
        }

        // Les cibles touchées par une balle tombent.
        let mut hit = Vec::new();
        for impact in impacts {
            let (a, b) = impact.entities;
            for (target, other) in [(a, b), (b, a)] {
                let is_target = world.satisfies::<&Target>(target);
                let by_ball = world.satisfies::<&Ball>(other);
                if is_target && by_ball && !hit.contains(&target) {
                    hit.push(target);
                }
            }
        }
        for target in hit {
            let Ok(transform) = world.get::<&Transform>(target).map(|t| *t) else {
                continue;
            };
            let _ = world.despawn(target);
            spawn_falling_target(world, assets, transform);
            self.targets_left -= 1;
            self.add_points(TARGET_POINTS, "cible");
            events.push(GameEvent::TargetHit {
                position: transform.translation,
            });
        }

        self.time_left = (self.time_left - dt).max(0.0);
        let perfect = self.crystals_left == 0 && self.targets_left == 0;
        if perfect || self.time_left == 0.0 {
            events.push(self.finish(world, perfect));
        }
        events
    }

    /// Fin de la partie : bonus éventuel, record, et on retire les cristaux
    /// et les cibles qui restent (les cibles tombées restent au sol).
    fn finish(&mut self, world: &mut hecs::World, perfect: bool) -> GameEvent {
        if perfect {
            let bonus = self.time_left.ceil() as u32 * TIME_BONUS;
            self.add_points(bonus, "bonus de temps");
        }
        let leftovers: Vec<Entity> = world
            .query::<(Entity, Option<&Crystal>, Option<&Target>)>()
            .iter()
            .filter(|(_, c, t)| c.is_some() || t.is_some())
            .map(|(e, _, _)| e)
            .collect();
        for entity in leftovers {
            let _ = world.despawn(entity);
        }
        let record = self.score > self.best;
        self.best = self.best.max(self.score);
        self.phase = Phase::Over { perfect, record };
        GameEvent::Finished {
            score: self.score,
            perfect,
            record,
        }
    }

    fn add_points(&mut self, points: u32, label: &str) {
        self.score += points;
        self.messages.push((format!("+{points} {label}"), 0.0));
    }

    /// Dessine le HUD : le score et le temps en haut de l'écran, les
    /// derniers messages, et un viseur au centre quand on joue à la souris.
    ///
    /// C'est une interface de test, dessinée avec egui. La vraie interface
    /// de jeu est prévue dans la Phase 16 de la feuille de route.
    pub fn hud(&self, ctx: &egui::Context, crosshair: bool) {
        let panel = egui::Frame::new()
            .fill(egui::Color32::from_black_alpha(170))
            .corner_radius(8)
            .inner_margin(egui::Margin::symmetric(16, 8));
        let big = |text: String| egui::RichText::new(text).size(20.0).strong();
        let gold = egui::Color32::from_rgb(255, 210, 90);

        egui::Area::new(egui::Id::unique("hud"))
            .anchor(egui::Align2::CENTER_TOP, [0.0, 12.0])
            .interactable(false)
            .show(ctx, |ui| {
                panel.show(ui, |ui| match self.phase {
                    Phase::Idle => {
                        ui.label(big("G : lancer la chasse aux cristaux".into()));
                        ui.label("C : marcher au lieu de voler (Espace : sauter)");
                        if self.best > 0 {
                            ui.label(format!("Record : {}", self.best));
                        }
                    }
                    Phase::Playing => {
                        ui.horizontal(|ui| {
                            ui.label(big(format!("Score {}", self.score)).color(gold));
                            ui.separator();
                            let time = big(format!("{:.0} s", self.time_left.ceil()));
                            // Les 10 dernières secondes, en rouge.
                            ui.label(if self.time_left < 10.0 {
                                time.color(egui::Color32::from_rgb(255, 90, 70))
                            } else {
                                time
                            });
                            ui.separator();
                            ui.label(big(format!(
                                "Cristaux {}/{CRYSTAL_COUNT}",
                                CRYSTAL_COUNT - self.crystals_left
                            )));
                            ui.separator();
                            ui.label(big(format!(
                                "Cibles {}/{TARGET_COUNT}",
                                TARGET_COUNT - self.targets_left
                            )));
                        });
                    }
                    Phase::Over { perfect, record } => {
                        ui.label(
                            big(format!("Partie terminée : {} points", self.score)).color(gold),
                        );
                        if perfect {
                            ui.label("Tout ramassé, tout touché : bonus de temps !");
                        }
                        ui.label(if record {
                            "Nouveau record !".to_string()
                        } else {
                            format!("Record : {}", self.best)
                        });
                        ui.label("G : rejouer");
                    }
                });
                // Les messages s'effacent en remontant vers le score.
                for (text, age) in self.messages.iter().rev() {
                    let alpha = (1.0 - age / MESSAGE_TIME).clamp(0.0, 1.0);
                    let color = egui::Color32::from_rgba_unmultiplied(
                        gold.r(),
                        gold.g(),
                        gold.b(),
                        (alpha * 255.0) as u8,
                    );
                    ui.label(egui::RichText::new(text).size(18.0).strong().color(color));
                }
            });

        if crosshair && self.phase == Phase::Playing {
            let painter = ctx.layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::unique("viseur"),
            ));
            let center = ctx.content_rect().center();
            let stroke = egui::Stroke::new(2.0, egui::Color32::from_white_alpha(200));
            for (dx, dy) in [(1.0, 0.0), (0.0, 1.0)] {
                let d = egui::vec2(dx, dy);
                painter.line_segment([center - d * 10.0, center - d * 4.0], stroke);
                painter.line_segment([center + d * 4.0, center + d * 10.0], stroke);
            }
        }
    }
}

/// Supprime tout ce que les parties ont créé.
pub fn despawn_items(world: &mut hecs::World) {
    let items: Vec<Entity> = world
        .query::<hecs::With<Entity, &GameItem>>()
        .iter()
        .collect();
    for entity in items {
        let _ = world.despawn(entity);
    }
}

/// La place des cristaux de la partie numéro `round` : toujours la même
/// pour un même numéro (un petit générateur pseudo-aléatoire, pas de
/// dépendance), différente d'une partie à l'autre.
fn crystal_positions(round: u32) -> Vec<Vec3> {
    // « xorshift » : quelques décalages et ou exclusifs suffisent à mélanger.
    let mut state = 0x9E37_79B9_u32 ^ round.wrapping_mul(0x85EB_CA6B);
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as f32 / u32::MAX as f32
    };
    (0..CRYSTAL_COUNT)
        .map(|i| {
            // Un cristal par secteur, à une place tirée au hasard dans le
            // secteur : ils sont répartis tout autour de la grille.
            let sector = std::f32::consts::TAU / CRYSTAL_COUNT as f32;
            let angle = (i as f32 + 0.15 + 0.7 * next()) * sector;
            let radius = CRYSTAL_RING.0 + (CRYSTAL_RING.1 - CRYSTAL_RING.0) * next();
            let height = GROUND_Y + 1.2 + 2.0 * next();
            Vec3::new(angle.sin() * radius, height, angle.cos() * radius)
        })
        .collect()
}

/// Les angles des cibles (0° : du côté de la caméra de départ, en +Z). Elles
/// forment un arc sur les côtés et au fond de l'arène : aucune ne cache la
/// vue au départ, et les angles évitent les arbres de la démo.
fn target_angles() -> [f32; TARGET_COUNT] {
    [65.0_f32, 125.0, 190.0, 245.0, 295.0].map(f32::to_radians)
}

/// Un cristal : un cube posé sur une pointe, lumineux, qui tourne, avec une
/// petite lampe pour éclairer ce qui l'entoure.
fn spawn_crystal(world: &mut hecs::World, assets: &DemoAssets, position: Vec3) {
    let cyan = Color::rgb(0.3, 0.9, 1.0);
    // Pencher le cube de 45° sur deux axes le met « sur la pointe ».
    let tilt = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)
        * Quat::from_rotation_x(std::f32::consts::FRAC_PI_4);
    world.spawn((
        Name::new("Cristal"),
        Transform::from_translation(position)
            .with_rotation(tilt)
            .with_uniform_scale(0.35),
        MeshRenderer::new(assets.white_cube).with_material(Material {
            emissive: Color::rgb(0.6, 2.4, 3.0),
            ..Material::dielectric(cyan, 0.2)
        }),
        Spin {
            axis: Vec3::Y,
            speed: 2.0,
        },
        PointLight {
            color: cyan,
            intensity: 1.2,
            range: 3.0,
        },
        Crystal,
        GameItem,
    ));
}

/// Une cible : une plaque rouge lumineuse qui flotte, tournée vers le
/// centre de l'arène. Fixe : les balles rebondissent dessus.
fn spawn_target(world: &mut hecs::World, assets: &DemoAssets, index: usize, angle: f32) {
    let height = GROUND_Y + 1.6 + 0.6 * (index % 3) as f32;
    let position = Vec3::new(angle.sin(), 0.0, angle.cos()) * TARGET_RADIUS + Vec3::Y * height;
    world.spawn((
        Name::new(format!("Cible {}", index + 1)),
        target_transform(position, angle),
        MeshRenderer::new(assets.white_cube).with_material(target_material()),
        RigidBody::fixed(),
        Collider::cuboid(Vec3::splat(0.5)),
        Target,
        GameItem,
    ));
}

/// Une plaque de 1,2 m × 1,2 m × 10 cm. Son axe Z (le plus fin) tourné de
/// `angle` autour de la verticale pointe vers le point du cercle, donc sa
/// face est tournée vers le centre.
fn target_transform(position: Vec3, angle: f32) -> Transform {
    Transform {
        scale: Vec3::new(1.2, 1.2, 0.1),
        ..Transform::from_translation(position).with_rotation(Quat::from_rotation_y(angle))
    }
}

fn target_material() -> Material {
    Material {
        emissive: Color::rgb(2.5, 0.25, 0.1),
        ..Material::dielectric(Color::rgb(0.9, 0.1, 0.05), 0.4)
    }
}

/// Une cible touchée est remplacée par une plaque **dynamique** au même
/// endroit, éteinte et poussée vers l'extérieur : elle tombe.
fn spawn_falling_target(world: &mut hecs::World, assets: &DemoAssets, transform: Transform) {
    let outward = (transform.translation * Vec3::new(1.0, 0.0, 1.0)).normalize_or_zero();
    world.spawn((
        Name::new("Cible touchée"),
        transform,
        MeshRenderer::new(assets.white_cube)
            .with_material(Material::dielectric(Color::rgb(0.35, 0.08, 0.05), 0.6)),
        RigidBody::dynamic().with_velocity(outward * 3.0 + Vec3::Y * 2.0),
        Collider::cuboid(Vec3::splat(0.5)).with_density(0.6),
        GameItem,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo::tests::assets;

    fn count<C: hecs::Component>(world: &hecs::World) -> usize {
        world.query::<&C>().iter().count()
    }

    fn started() -> (Game, hecs::World) {
        let mut game = Game::default();
        let mut world = hecs::World::new();
        assert_eq!(game.start(&mut world, &assets()), GameEvent::Started);
        (game, world)
    }

    /// Un choc entre la première cible et une balle.
    fn ball_hits_target(world: &mut hecs::World) -> Impact {
        let target = world
            .query::<hecs::With<Entity, &Target>>()
            .iter()
            .next()
            .unwrap();
        let ball = world.spawn((Transform::IDENTITY, Ball { serial: 0 }));
        Impact {
            entities: (ball, target),
            speed: 12.0,
            position: Vec3::ZERO,
        }
    }

    fn crystal_position(world: &hecs::World) -> Vec3 {
        world
            .query::<hecs::With<&Transform, &Crystal>>()
            .iter()
            .next()
            .unwrap()
            .translation
    }

    #[test]
    fn une_partie_place_les_cristaux_et_les_cibles() {
        let (mut game, mut world) = started();
        assert_eq!(count::<Crystal>(&world), CRYSTAL_COUNT);
        assert_eq!(count::<Target>(&world), TARGET_COUNT);
        assert_eq!(count::<GameItem>(&world), CRYSTAL_COUNT + TARGET_COUNT);
        assert_eq!(game.phase, Phase::Playing);

        // Relancer remplace la partie au lieu de l'ajouter à la précédente.
        game.start(&mut world, &assets());
        assert_eq!(count::<GameItem>(&world), CRYSTAL_COUNT + TARGET_COUNT);
    }

    #[test]
    fn cristaux_et_cibles_restent_dans_l_arene() {
        for round in 1..20 {
            for p in crystal_positions(round) {
                let distance = p.x.hypot(p.z);
                assert!(distance >= CRYSTAL_RING.0 - 1e-4 && distance <= CRYSTAL_RING.1 + 1e-4);
                // Au-dessus du sol, et sous les 8 m du bord du sol.
                assert!(p.y > GROUND_Y + 1.0 && p.x.abs() < 8.0 && p.z.abs() < 8.0);
            }
        }
        // La même partie donne les mêmes places, une autre partie non.
        assert_eq!(crystal_positions(3), crystal_positions(3));
        assert_ne!(crystal_positions(3), crystal_positions(4));
    }

    #[test]
    fn une_cible_fait_face_au_centre() {
        let angle = 1.0_f32;
        let position = Vec3::new(angle.sin(), 0.0, angle.cos()) * TARGET_RADIUS;
        let t = target_transform(position, angle);
        // La face de la plaque (son axe Z) pointe vers l'extérieur et
        // l'intérieur du cercle : alignée avec le rayon.
        let normal = t.rotation * Vec3::Z;
        assert!(normal.dot(position.normalize()) > 0.999);
    }

    #[test]
    fn passer_sur_un_cristal_le_ramasse() {
        let (mut game, mut world) = started();
        let position = crystal_position(&world);

        // Trop loin : rien.
        let far = position + Vec3::X * 3.0;
        assert!(game.update(&mut world, &assets(), &[], far, 0.1).is_empty());

        let events = game.update(&mut world, &assets(), &[], position, 0.1);
        assert_eq!(events, vec![GameEvent::Collected { position }]);
        assert_eq!(game.score, CRYSTAL_POINTS);
        assert_eq!(game.crystals_left, CRYSTAL_COUNT - 1);
        assert_eq!(count::<Crystal>(&world), CRYSTAL_COUNT - 1);
    }

    #[test]
    fn une_balle_fait_tomber_une_cible() {
        let (mut game, mut world) = started();
        let impact = ball_hits_target(&mut world);
        // Le même choc signalé deux fois ne compte qu'une fois.
        let events = game.update(&mut world, &assets(), &[impact, impact], Vec3::ZERO, 0.1);
        assert!(matches!(events.as_slice(), [GameEvent::TargetHit { .. }]));
        assert_eq!(game.score, TARGET_POINTS);
        assert_eq!(count::<Target>(&world), TARGET_COUNT - 1);
        // La cible est remplacée par une plaque qui tombe.
        let falling = world
            .query::<(&Name, &RigidBody)>()
            .iter()
            .filter(|(n, b)| n.0 == "Cible touchée" && b.kind == zentrix_physics::BodyKind::Dynamic)
            .count();
        assert_eq!(falling, 1);

        // Un choc entre une cible et autre chose qu'une balle ne compte pas.
        let target = world
            .query::<hecs::With<Entity, &Target>>()
            .iter()
            .next()
            .unwrap();
        let crate_box = world.spawn((Transform::IDENTITY,));
        let other = Impact {
            entities: (target, crate_box),
            speed: 5.0,
            position: Vec3::ZERO,
        };
        assert!(
            game.update(&mut world, &assets(), &[other], Vec3::ZERO, 0.1)
                .is_empty()
        );
    }

    #[test]
    fn la_partie_finit_au_bout_du_temps() {
        let (mut game, mut world) = started();
        let position = crystal_position(&world);
        game.update(&mut world, &assets(), &[], position, 0.1);

        let events = game.update(&mut world, &assets(), &[], Vec3::ZERO, ROUND_TIME);
        assert_eq!(
            events,
            vec![GameEvent::Finished {
                score: CRYSTAL_POINTS,
                perfect: false,
                record: true
            }]
        );
        assert_eq!(game.best, CRYSTAL_POINTS);
        // Ce qui restait à ramasser ou à toucher disparaît.
        assert_eq!(count::<Crystal>(&world) + count::<Target>(&world), 0);
        // Ensuite, plus rien ne bouge.
        assert!(
            game.update(&mut world, &assets(), &[], Vec3::ZERO, 1.0)
                .is_empty()
        );

        // Une nouvelle partie à 0 point ne bat pas le record.
        game.start(&mut world, &assets());
        game.update(&mut world, &assets(), &[], Vec3::ZERO, ROUND_TIME);
        assert_eq!(
            game.phase,
            Phase::Over {
                perfect: false,
                record: false
            }
        );
        assert_eq!(game.best, CRYSTAL_POINTS);
    }

    #[test]
    fn tout_faire_donne_le_bonus_de_temps() {
        let (mut game, mut world) = started();
        // Tous les cristaux, en 10 secondes.
        for _ in 0..CRYSTAL_COUNT {
            let position = crystal_position(&world);
            game.update(
                &mut world,
                &assets(),
                &[],
                position,
                10.0 / CRYSTAL_COUNT as f32,
            );
        }
        // Toutes les cibles, sauf la dernière.
        for _ in 0..TARGET_COUNT - 1 {
            let impact = ball_hits_target(&mut world);
            game.update(&mut world, &assets(), &[impact], Vec3::ZERO, 0.0);
        }
        let impact = ball_hits_target(&mut world);
        let events = game.update(&mut world, &assets(), &[impact], Vec3::ZERO, 0.0);
        let base = CRYSTAL_COUNT as u32 * CRYSTAL_POINTS + TARGET_COUNT as u32 * TARGET_POINTS;
        // 50 secondes restantes × 2 points.
        let score = base + 50 * TIME_BONUS;
        assert_eq!(
            events.last(),
            Some(&GameEvent::Finished {
                score,
                perfect: true,
                record: true
            })
        );
    }

    #[test]
    fn recharger_la_scene_abandonne_la_partie() {
        let (mut game, mut world) = started();
        game.score = 30;
        game.best = 80;
        game.reset(&mut world);
        assert_eq!(game.phase, Phase::Idle);
        assert_eq!(game.score, 0);
        assert_eq!(game.best, 80);
        assert_eq!(count::<GameItem>(&world), 0);
    }
}
