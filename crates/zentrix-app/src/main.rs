//! # zentrix-app
//!
//! L'exécutable de démonstration de ZENTRIX. Il assemble les crates :
//!
//! ```text
//!                        zentrix-app  (ce fichier + demo.rs + fly_camera.rs)
//!          ┌──────────────┬────┴─────────┬───────────────┐
//!          │  zentrix-physics  zentrix-audio            │
//!          │    (rapier3d)       (kira)  │               │
//!          │              ▼              │               │
//!  zentrix-platform  zentrix-scene  zentrix-assets        │
//!     (winit)         (hecs ECS)     (image)             │
//!          │              └──────▶ zentrix-render ◀──────┘
//!          │                            │
//!          │                       zentrix-rhi ── wgpu ── GPU
//!          └──────────▶ zentrix-core ◀──┘
//! ```
//!
//! Lancer avec : `cargo run` (depuis la racine du projet).
//! Logs détaillés : `RUST_LOG=debug cargo run` (bash) ou
//! `$env:RUST_LOG="debug"; cargo run` (PowerShell).
//!
//! La démo : une grille de cubes (caisses en bois texturées et cubes colorés)
//! qui tournent sur un sol en damier, explorée avec une caméra libre. Tout ce
//! qui existe dans le monde est une **entité** de la scène (voir demo.rs).
//!
//! Commandes (touches données pour un clavier AZERTY) :
//!
//! | Touche              | Action                                        |
//! |---------------------|-----------------------------------------------|
//! | clic gauche         | capturer la souris (regarder autour de soi)   |
//! | `Z` `Q` `S` `D`     | avancer, aller à gauche, reculer, aller à droite |
//! | `Espace` / `Ctrl`   | monter / descendre                            |
//! | `Maj` (maintenue)   | aller 3 fois plus vite                        |
//! | molette             | régler la vitesse                             |
//! | `P`                 | mettre en pause l'animation et la physique    |
//! | `F1`                | afficher / masquer la grille, les axes et la direction de la lumière |
//! | `F2`                | afficher / masquer les boîtes et les liens parent → enfant |
//! | `F3`                | activer / désactiver les ombres portées       |
//! | `F4`                | afficher / masquer l'inspecteur (interface egui) |
//! | `F5` / `F9`         | sauvegarder / recharger la scène (assets/scenes/demo.ron) |
//! | `F6`                | activer / désactiver le tone mapping          |
//! | `F7`                | activer / désactiver l'environnement (ciel, reflets) |
//! | `B`                 | lancer une balle droit devant                 |
//! | `F8`                | couper / remettre le son                      |
//! | `N`                 | ajouter / retirer le champ de stress (4 096 cubes) |
//! | `L`                 | passer à la nuit (seules les lampes éclairent) / au jour |
//! | `G`                 | lancer une partie de « chasse aux cristaux » (voir game.rs) |
//! | `C`                 | passer du vol à la marche (personnage à la première personne, voir player.rs) et inversement ; en marche : `Espace` saute, `Maj` court |
//! | clic droit (souris capturée) | pousser l'objet visé                  |
//! | `Échap`             | libérer la souris, puis quitter               |

mod demo;
mod editor;
mod fly_camera;
mod game;
mod inspector;
mod model;
mod player;

use std::sync::Arc;

// `anyhow::Context` ajoute la méthode `.context("...")` aux `Result` :
// elle enrichit le message d'erreur pour savoir OÙ l'erreur s'est produite.
use anyhow::Context;

use zentrix_audio::{Audio, AudioListener, SoundEmitter};
use zentrix_core::math::{Mat4, Quat, Vec2, Vec3};
use zentrix_core::{Color, FrameProfiler, Image, Transform};
use zentrix_physics::{CharacterController, Collider, Physics, RigidBody};
use zentrix_platform::{
    Application, Input, KeyCode, MouseButton, Window, WindowConfig, WindowEvent,
    set_cursor_captured,
};
use zentrix_render::{DebugLines, DrawCommand, MaterialTextures, MeshData, Renderer};
use zentrix_rhi::GpuContext;
use zentrix_scene::{AssetNames, DirectionalLight, Entity, Scene, SceneSerializer, hecs};
use zentrix_ui::UiLayer;

use crate::demo::{Ball, DemoAssets, GROUND_Y, Spin, StressCube};
use crate::editor::{Edit, Gizmo, GizmoMode, GizmoResponse, History, SpawnKind};
use crate::fly_camera::FlyCamera;
use crate::game::{Game, GameEvent, GameItem};
use crate::inspector::{Inspector, Perf, Toggles};
use crate::player::Player;

/// Intervalle de mise à jour du compteur d'images par seconde (en secondes).
const FPS_REFRESH: f32 = 0.5;

/// L'écran du GPU (le « NDC ») : le pavé [-1, 1] × [-1, 1] × [0, 1]. Multiplié
/// par l'inverse d'une matrice vue-projection, il redevient la zone du
/// monde que cette matrice « voit ». `wire_box` dessine un cube de -0,5 à
/// 0,5 : on l'étire d'abord à la taille de ce pavé.
const NDC_BOX: Mat4 = Mat4::from_cols_array(&[
    2.0, 0.0, 0.0, 0.0, //
    0.0, 2.0, 0.0, 0.0, //
    0.0, 0.0, 1.0, 0.0, //
    0.0, 0.0, 0.5, 1.0, //
]);

/// Les objets qui n'existent qu'une fois la fenêtre créée.
///
/// On les regroupe dans une struct pour n'avoir qu'**une seule** `Option` à
/// gérer : soit tout est initialisé, soit rien ne l'est.
struct GraphicsState {
    window: Arc<Window>,
    gpu: GpuContext,
    renderer: Renderer,
    /// L'interface graphique (egui), dessinée par-dessus la scène.
    ui: UiLayer,
    /// Début du titre de la fenêtre (nom du GPU), complété par les FPS.
    title: String,
}

/// Compte les images pour afficher une moyenne lisible des FPS.
///
/// Afficher `1 / dt` à chaque image donnerait un nombre illisible qui saute
/// sans arrêt. On compte plutôt les images sur une demi-seconde.
#[derive(Default)]
struct FpsCounter {
    frames: u32,
    elapsed: f32,
}

impl FpsCounter {
    /// Enregistre une image. Renvoie `Some(fps)` quand une nouvelle moyenne
    /// est disponible.
    fn tick(&mut self, dt: f32) -> Option<f32> {
        self.frames += 1;
        self.elapsed += dt;
        if self.elapsed < FPS_REFRESH {
            return None;
        }
        let fps = self.frames as f32 / self.elapsed;
        *self = Self::default();
        Some(fps)
    }
}

/// L'état de notre application de démonstration.
struct ZentrixApp {
    /// `None` avant `on_start`, `Some(...)` après.
    graphics: Option<GraphicsState>,
    /// Le monde du jeu : toutes les entités (caméra, sol, cubes...).
    scene: Scene,
    /// Le monde physique, synchronisé avec la scène à chaque image.
    physics: Physics,
    /// Les ressources de la démo (pour créer des entités en cours de jeu :
    /// les balles). `None` avant `on_start`.
    assets: Option<DemoAssets>,
    /// Numéro de la prochaine balle lancée.
    next_ball: u32,
    /// Le son. Muet tant que `on_start` ne l'a pas ouvert.
    audio: Audio,
    /// Le volume voulu (0 à 1), réglé dans l'inspecteur ; `F8` coupe le son.
    volume: f32,
    sound_off: bool,
    /// L'annuaire « nom ↔ handle » des ressources, et ce qu'il faut pour
    /// sauvegarder / recharger la scène.
    names: AssetNames,
    serializer: SceneSerializer,
    /// La souris est-elle capturée par la fenêtre (mode « regard ») ?
    mouse_captured: bool,
    /// Les commandes de dessin de l'image en cours. Gardé d'une image à
    /// l'autre pour réutiliser sa mémoire.
    draws: Vec<DrawCommand>,
    /// Les lignes de debug de l'image en cours.
    debug: DebugLines,
    /// F1 : grille et axes ; F2 : boîtes englobantes et liens parent → enfant.
    show_grid: bool,
    show_bounds: bool,
    fps: FpsCounter,
    /// La dernière moyenne d'images par seconde (affichée par l'inspecteur).
    last_fps: f32,
    /// La fenêtre d'inspection de la scène (F4).
    inspector: Inspector,
    /// Le temps de chaque étape de l'image (affiché par l'inspecteur).
    profiler: FrameProfiler,
    /// Le champ de stress (touche `N`) est-il présent ?
    stress: bool,
    /// La nuit (touche `L`) : les réglages du jour, gardés pour les
    /// rétablir (intensité du soleil, lumière ambiante, intensité du ciel).
    day: Option<(f32, Color, f32)>,
    /// La partie de « chasse aux cristaux » (touche `G`).
    game: Game,
    /// L'éditeur (inspecteur ouvert) : le gizmo, l'historique pour annuler,
    /// le nombre d'objets créés (pour leur nom).
    gizmo: Gizmo,
    history: History,
    spawned: u32,
    /// Le geste du gizmo en cours : l'objet et son `Transform` au début.
    gesture: Option<(Entity, Transform)>,
    /// La souris est capturée par le clic droit de l'éditeur : elle sera
    /// libérée quand le bouton sera relâché.
    editor_look: bool,
    /// L'animation est-elle en pause ?
    paused: bool,
    /// Demande de fermeture (touche Échap).
    quit_requested: bool,
}

impl ZentrixApp {
    fn new() -> Self {
        Self {
            graphics: None,
            scene: Scene::default(),
            physics: Physics::new(),
            assets: None,
            next_ball: 0,
            audio: Audio::muted(),
            volume: 0.8,
            sound_off: false,
            names: AssetNames::default(),
            serializer: scene_serializer(),
            mouse_captured: false,
            draws: Vec::new(),
            debug: DebugLines::default(),
            show_grid: true,
            show_bounds: false,
            fps: FpsCounter::default(),
            last_fps: 0.0,
            inspector: Inspector::default(),
            profiler: FrameProfiler::default(),
            stress: false,
            day: None,
            game: Game::default(),
            gizmo: Gizmo::default(),
            history: History::default(),
            spawned: 0,
            gesture: None,
            editor_look: false,
            paused: false,
            quit_requested: false,
        }
    }

    /// Capture ou libère la souris, et retient le résultat.
    fn set_mouse_captured(&mut self, captured: bool) {
        if let Some(g) = &self.graphics {
            self.mouse_captured = set_cursor_captured(&g.window, captured);
        }
    }

    /// Un son pour chaque choc de la physique, d'autant plus fort que le
    /// choc est violent. Seuls les 6 plus forts de l'image sont joués :
    /// au-delà, on n'entendrait qu'un brouhaha.
    fn play_impact_sounds(&mut self) {
        let Some(sound) = self.audio.sound(demo::IMPACT_SOUND) else {
            return;
        };
        let mut impacts: Vec<_> = self
            .physics
            .impacts()
            .iter()
            .filter(|i| i.speed > 1.0)
            .copied()
            .collect();
        impacts.sort_by(|a, b| b.speed.total_cmp(&a.speed));
        for (i, impact) in impacts.iter().take(6).enumerate() {
            let volume = ((impact.speed - 1.0) / 9.0).clamp(0.05, 1.0);
            // Un peu de variété : chaque choc n'a pas exactement la même
            // hauteur (sinon l'oreille repère tout de suite la répétition).
            let rate = 0.85 + 0.1 * ((self.next_ball as usize + i) % 4) as f32;
            self.audio.play_at(sound, impact.position, volume, rate);
        }
    }

    /// Les sons et messages qui accompagnent les événements de la partie.
    fn play_game_events(&mut self, events: &[GameEvent]) {
        let impact = self.audio.sound(demo::IMPACT_SOUND);
        let whoosh = self.audio.sound(demo::LAUNCH_SOUND);
        for event in events {
            match *event {
                GameEvent::Started => {
                    log::info!(
                        "Chasse aux cristaux : {} cristaux à ramasser, {} cibles à toucher (B), {} s",
                        game::CRYSTAL_COUNT,
                        game::TARGET_COUNT,
                        game::ROUND_TIME
                    );
                    if let Some(sound) = whoosh {
                        self.audio.play(sound, 0.7, 0.6);
                    }
                }
                // Le choc synthétisé, joué deux fois plus aigu : un « tintement ».
                GameEvent::Collected { position } => {
                    if let Some(sound) = impact {
                        self.audio.play_at(sound, position, 0.9, 2.4);
                    }
                }
                GameEvent::TargetHit { position } => {
                    if let Some(sound) = impact {
                        self.audio.play_at(sound, position, 1.0, 0.6);
                    }
                }
                GameEvent::Finished {
                    score,
                    perfect,
                    record,
                } => {
                    log::info!(
                        "Partie terminée : {score} points{}{}",
                        if perfect { ", sans faute" } else { "" },
                        if record { " — nouveau record !" } else { "" }
                    );
                    if let Some(sound) = whoosh {
                        self.audio.play(sound, 0.8, 1.4);
                    }
                }
            }
        }
    }

    /// `L` : passe à la nuit (soleil, ciel et lumière ambiante presque
    /// éteints : seules les lampes éclairent), ou revient au jour.
    fn toggle_night(&mut self) {
        let Some(g) = &mut self.graphics else { return };
        let sun = self
            .scene
            .world
            .query_mut::<&mut DirectionalLight>()
            .into_iter()
            .next();
        let Some(sun) = sun else { return };
        match self.day.take() {
            // Retour au jour : les valeurs gardées.
            Some((intensity, ambient, sky)) => {
                sun.intensity = intensity;
                self.scene.ambient_light = ambient;
                g.renderer.environment_intensity = sky;
                log::info!("Jour");
            }
            None => {
                self.day = Some((
                    sun.intensity,
                    self.scene.ambient_light,
                    g.renderer.environment_intensity,
                ));
                sun.intensity = 0.02;
                self.scene.ambient_light = Color::rgb(0.01, 0.012, 0.02);
                g.renderer.environment_intensity = 0.04;
                log::info!("Nuit : seules les lampes éclairent");
            }
        }
    }

    /// `N` : ajoute ou retire le champ de stress.
    fn toggle_stress_field(&mut self) {
        if self.stress {
            let removed = demo::remove_stress_field(&mut self.scene.world);
            log::info!("Champ de stress retiré ({removed} cubes)");
            self.stress = false;
        } else if let Some(assets) = &self.assets {
            demo::spawn_stress_field(&mut self.scene.world, assets);
            log::info!(
                "Champ de stress ajouté : {} entités",
                self.scene.entity_count()
            );
            self.stress = true;
        }
    }

    /// La position et l'orientation de la caméra pilotée.
    fn camera_transform(&self) -> Option<Transform> {
        self.scene
            .world
            .query::<hecs::With<&Transform, &FlyCamera>>()
            .iter()
            .next()
            .copied()
    }

    /// F5 : écrit la scène dans `assets/scenes/demo.ron`.
    fn save_scene(&self) {
        let path = scene_path();
        // `and_then` enchaîne les étapes : on ne continue que si la précédente a réussi.
        let result = self
            .serializer
            .save(&self.scene, &self.names)
            .map_err(anyhow::Error::from)
            .and_then(|text| {
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir)?;
                }
                std::fs::write(&path, text)?;
                Ok(())
            });
        match result {
            Ok(()) => log::info!(
                "Scène sauvegardée : {} ({} entités)",
                path.display(),
                self.scene.entity_count()
            ),
            Err(error) => log::error!("Sauvegarde impossible : {error:#}"),
        }
    }

    /// F9 : remplace la scène par celle de `assets/scenes/demo.ron`.
    ///
    /// On charge dans une scène NEUVE : si le fichier est invalide, la scène
    /// actuelle reste intacte.
    fn load_scene(&mut self) {
        let path = scene_path();
        let result = std::fs::read_to_string(&path)
            .map_err(anyhow::Error::from)
            .and_then(|text| self.replace_scene(&text));
        match result {
            Ok(()) => log::info!(
                "Scène rechargée : {} ({} entités)",
                path.display(),
                self.scene.entity_count()
            ),
            Err(error) => log::error!("Chargement impossible ({}) : {error:#}", path.display()),
        }
    }

    /// Remplace la scène par celle décrite par `text` (le format de `F5`).
    ///
    /// On charge dans une scène NEUVE : si le texte est invalide, la scène
    /// actuelle reste intacte.
    fn replace_scene(&mut self, text: &str) -> anyhow::Result<()> {
        let mut scene = Scene::default();
        self.serializer.load(&mut scene, text, &self.names)?;
        self.scene = scene;
        // L'historique désigne des entités de l'ancienne scène.
        self.history.clear();
        self.gesture = None;
        // Une partie en cours ne survit pas au changement de scène : on
        // retire aussi les cristaux et cibles que le texte contenait.
        self.game.reset(&mut self.scene.world);
        // Le texte contient peut-être le champ de stress.
        self.stress = self
            .scene
            .world
            .query::<&StressCube>()
            .iter()
            .next()
            .is_some();
        // Les corps de l'ancienne scène n'ont plus d'entité : on repart de
        // zéro (ils seront recréés depuis les composants).
        self.physics.clear();
        // Après les balles du texte, pour garder des numéros uniques.
        self.next_ball = self
            .scene
            .world
            .query::<&Ball>()
            .iter()
            .map(|b| b.serial + 1)
            .max()
            .unwrap_or(0);
        Ok(())
    }

    /// `Ctrl+Z` (annuler) ou `Ctrl+Y` (refaire, `redo = true`).
    fn undo_redo(&mut self, redo: bool) {
        if self.gizmo.is_dragging() {
            return;
        }
        let remap = if redo {
            self.history.redo(&mut self.scene.world)
        } else {
            self.history.undo(&mut self.scene.world)
        };
        let Some(remap) = remap else { return };
        // Un objet restauré a un nouvel identifiant : la sélection le suit.
        if let Some(selected) = self.inspector.selected()
            && let Some(&(_, moved)) = remap.iter().find(|(old, _)| *old == selected)
        {
            self.inspector.select(Some(moved));
        }
        log::info!("{}", if redo { "Refait" } else { "Annulé" });
    }

    /// `Suppr` : supprime l'objet sélectionné et ses enfants (d'une façon
    /// qu'on peut annuler).
    fn delete_selected(&mut self) {
        let Some(entity) = self.inspector.selected() else {
            return;
        };
        let mut branch = self.scene.descendants(entity);
        branch.insert(0, entity);
        let count = branch.len();
        self.history.delete(&mut self.scene.world, branch);
        self.inspector.select(None);
        log::info!("Supprimé : {count} entité(s) (Ctrl+Z pour annuler)");
    }

    /// Le geste du gizmo vient de se terminer : il entre dans l'historique.
    fn finish_gesture(&mut self) {
        if self.gizmo.is_dragging() {
            return;
        }
        let Some((entity, before)) = self.gesture.take() else {
            return;
        };
        if let Ok(after) = self.scene.world.get::<&Transform>(entity).map(|t| *t)
            && after != before
        {
            self.history.record(Edit::Transform {
                entity,
                before,
                after,
            });
        }
    }

    /// Ajoute un objet devant la caméra (à 4 m, ou devant le premier
    /// obstacle s'il est plus près), et le sélectionne.
    fn spawn_object(&mut self, kind: SpawnKind) {
        let Some(eye) = self.camera_transform() else {
            return;
        };
        let forward = eye.rotation * Vec3::NEG_Z;
        let me = player::find(&self.scene.world);
        let distance = self
            .physics
            .cast_ray_excluding(eye.translation, forward, 4.0, me)
            .map_or(4.0, |hit| (hit.distance - 0.6).max(0.5));
        let Some(assets) = &self.assets else { return };
        self.spawned += 1;
        let entity = editor::spawn_object(
            &mut self.scene.world,
            assets,
            kind,
            eye.translation + forward * distance,
            self.spawned,
        );
        self.history.record(Edit::Presence {
            deleted: false,
            entities: vec![entity],
            stash: Vec::new(),
        });
        self.inspector.select(Some(entity));
    }

    /// La souris dans l'éditeur : clic droit maintenu pour piloter, clic
    /// gauche pour sélectionner l'objet sous le curseur.
    fn editor_mouse(&mut self, input: &Input, over_ui: bool, gizmo: GizmoResponse) {
        if input.was_button_pressed(MouseButton::Right) && !over_ui && !self.mouse_captured {
            self.set_mouse_captured(true);
            self.editor_look = self.mouse_captured;
        }
        if self.editor_look && !input.is_button_down(MouseButton::Right) {
            self.set_mouse_captured(false);
            self.editor_look = false;
        }

        if input.was_button_pressed(MouseButton::Left)
            && !over_ui
            && !gizmo.active
            && !self.mouse_captured
            && let (Some(g), Some(cursor)) = (&self.graphics, input.cursor_position())
        {
            let (width, height) = g.gpu.size();
            let size = Vec2::new(width as f32, height.max(1) as f32);
            let camera = self.scene.camera().unwrap_or_default();
            let (origin, direction) =
                editor::screen_ray(camera.view_projection(size.x / size.y), cursor / size);
            let hit = editor::pick(
                &self.scene.world,
                |mesh| g.renderer.mesh_bounds(mesh),
                origin,
                direction,
            );
            self.inspector.select(hit);
        }
    }

    /// Les commandes « ponctuelles » : elles réagissent à l'image où la
    /// touche est enfoncée (`was_key_pressed`), pas tant qu'elle est maintenue.
    fn handle_commands(&mut self, input: &Input) {
        if input.was_key_pressed(KeyCode::KeyP) {
            self.paused = !self.paused;
            log::info!(
                "Animation {}",
                if self.paused { "en pause" } else { "reprise" }
            );
        }

        if input.was_key_pressed(KeyCode::F1) {
            self.show_grid = !self.show_grid;
        }
        if input.was_key_pressed(KeyCode::F2) {
            self.show_bounds = !self.show_bounds;
        }
        if input.was_key_pressed(KeyCode::F5) {
            self.save_scene();
        }
        if input.was_key_pressed(KeyCode::F9) {
            self.load_scene();
        }
        if input.was_key_pressed(KeyCode::F3)
            && let Some(g) = &mut self.graphics
        {
            g.renderer.shadows_enabled = !g.renderer.shadows_enabled;
            log::info!(
                "Ombres {}",
                if g.renderer.shadows_enabled {
                    "activées"
                } else {
                    "désactivées"
                }
            );
        }
        if input.was_key_pressed(KeyCode::F7)
            && let Some(g) = &mut self.graphics
        {
            g.renderer.environment_enabled = !g.renderer.environment_enabled;
            log::info!(
                "Environnement {}",
                if g.renderer.environment_enabled {
                    "activé"
                } else {
                    "désactivé"
                }
            );
        }
        if input.was_key_pressed(KeyCode::F6)
            && let Some(g) = &mut self.graphics
        {
            g.renderer.tone_mapping = !g.renderer.tone_mapping;
            log::info!(
                "Tone mapping {}",
                if g.renderer.tone_mapping {
                    "activé"
                } else {
                    "désactivé"
                }
            );
        }

        // Échap libère d'abord la souris ; un second appui quitte.
        if input.was_key_pressed(KeyCode::Escape) {
            if self.mouse_captured {
                self.set_mouse_captured(false);
            } else {
                self.quit_requested = true;
            }
        }

        if input.was_key_pressed(KeyCode::F4) {
            self.inspector.visible = !self.inspector.visible;
        }
        if input.was_key_pressed(KeyCode::KeyN) {
            self.toggle_stress_field();
        }
        if input.was_key_pressed(KeyCode::KeyL) {
            self.toggle_night();
        }
        if input.was_key_pressed(KeyCode::KeyC) {
            let walking = player::toggle(&mut self.scene.world, &self.physics);
            log::info!(
                "{}",
                if walking {
                    "Mode marche : ZQSD, Espace = sauter, Maj = courir (C : revenir au vol)"
                } else {
                    "Mode vol"
                }
            );
        }
        if input.was_key_pressed(KeyCode::KeyG)
            && let Some(assets) = &self.assets
        {
            let event = self.game.start(&mut self.scene.world, assets);
            self.play_game_events(&[event]);
        }
        if input.was_key_pressed(KeyCode::F8) {
            self.sound_off = !self.sound_off;
            log::info!("Son {}", if self.sound_off { "coupé" } else { "remis" });
        }

        // `B` (même place en AZERTY et QWERTY) : lancer une balle.
        if input.was_key_pressed(KeyCode::KeyB)
            && let (Some(assets), Some(eye)) = (&self.assets, self.camera_transform())
        {
            demo::launch_ball(&mut self.scene.world, assets, &eye, self.next_ball);
            self.next_ball += 1;
            if let Some(sound) = self.audio.sound(demo::LAUNCH_SOUND) {
                let forward = eye.rotation * Vec3::NEG_Z;
                self.audio
                    .play_at(sound, eye.translation + forward, 0.6, 1.0);
            }
        }

        // Clic droit en mode regard : un rayon part du centre de l'écran ; le
        // premier objet touché reçoit un coup.
        if self.mouse_captured
            && input.was_button_pressed(MouseButton::Right)
            && let Some(eye) = self.camera_transform()
        {
            let forward = eye.rotation * Vec3::NEG_Z;
            // En marche, les yeux sont DANS la capsule du joueur : le rayon
            // doit l'ignorer.
            let me = player::find(&self.scene.world);
            if let Some(hit) = self
                .physics
                .cast_ray_excluding(eye.translation, forward, 100.0, me)
            {
                self.physics
                    .apply_impulse(hit.entity, forward * 4.0, hit.point);
            }
        }

        // Alt+Tab pendant la capture : on rend la souris au système.
        if !input.is_focused() && self.mouse_captured {
            self.set_mouse_captured(false);
        }

        // L'éditeur (inspecteur ouvert). Ctrl+Z et Ctrl+Y d'après la lettre
        // IMPRIMÉE sur la touche : en AZERTY, la touche Z n'est pas à la place
        // du Z d'un clavier QWERTY.
        if self.inspector.visible {
            let ctrl =
                input.is_key_down(KeyCode::ControlLeft) || input.is_key_down(KeyCode::ControlRight);
            let shift =
                input.is_key_down(KeyCode::ShiftLeft) || input.is_key_down(KeyCode::ShiftRight);
            if ctrl && input.was_character_pressed('z') {
                // Ctrl+Maj+Z refait aussi, comme dans beaucoup de logiciels.
                self.undo_redo(shift);
            } else if ctrl && input.was_character_pressed('y') {
                self.undo_redo(true);
            }
            let modes = [
                (KeyCode::Digit1, GizmoMode::Translate),
                (KeyCode::Digit2, GizmoMode::Rotate),
                (KeyCode::Digit3, GizmoMode::Scale),
            ];
            for (key, mode) in modes {
                if input.was_key_pressed(key) && !self.gizmo.is_dragging() {
                    self.gizmo.mode = mode;
                }
            }
            if input.was_key_pressed(KeyCode::Delete) {
                self.delete_selected();
            }
        }
    }
}

// On « signe le contrat » : ZentrixApp implémente le trait Application de la
// plateforme. Le compilateur vérifie qu'on fournit bien les méthodes obligatoires.
impl Application for ZentrixApp {
    fn on_start(&mut self, window: Arc<Window>) -> anyhow::Result<()> {
        // `GpuContext::new` est `async` : `pollster::block_on` bloque le thread
        // jusqu'à ce que le résultat soit prêt.
        let gpu = pollster::block_on(GpuContext::new(Arc::clone(&window)))
            .context("échec de l'initialisation du GPU")?;

        let mut renderer = Renderer::new(&gpu);
        renderer.clear_color = Color::ZENTRIX_NIGHT;
        // Chaque maillage n'est envoyé qu'UNE fois au GPU, puis réutilisé par
        // toutes les entités qui y font référence grâce à son handle.
        let assets = DemoAssets {
            cube: renderer.add_mesh(&gpu, &MeshData::cube()),
            white_cube: renderer.add_mesh(&gpu, &MeshData::cube().with_color([1.0; 3])),
            // Le sol répète sa texture 8 × 8 fois (le sampler est en mode `Repeat`).
            ground: renderer.add_mesh(&gpu, &MeshData::plane([1.0; 3]).with_uv_scale(8.0)),
            // Les textures : une image chargée depuis le disque, une autre générée.
            crate_texture: renderer.add_texture(&gpu, &load_crate_image()),
            lattice_texture: renderer.add_texture(&gpu, &demo::lattice_image(128, 8)),
            sun: renderer.add_mesh(&gpu, &MeshData::cube().with_color([1.0, 0.75, 0.2])),
            planet: renderer.add_mesh(&gpu, &MeshData::cube().with_color([0.25, 0.5, 1.0])),
            moon: renderer.add_mesh(&gpu, &MeshData::cube().with_color([0.8, 0.8, 0.85])),
            sphere: renderer.add_mesh(&gpu, &MeshData::sphere(48, 24)),
            // Les normal maps sont générées (voir demo.rs) et envoyées sans
            // conversion sRGB.
            ground_normal_map: renderer.add_normal_map(&gpu, &demo::tiles_normal_map(256, 2)),
            dimples_normal_map: renderer
                .add_normal_map(&gpu, &demo::dimples_normal_map(512, 256, 24, 12)),
            // Couleur et émission en sRGB ; les autres sont des données.
            panel: {
                let images = demo::panel_images(256, 2);
                MaterialTextures {
                    base_color: Some(renderer.add_texture(&gpu, &images.base_color)),
                    normal: Some(renderer.add_normal_map(&gpu, &images.normal)),
                    metallic_roughness: Some(
                        renderer.add_data_texture(&gpu, &images.metallic_roughness),
                    ),
                    occlusion: Some(renderer.add_data_texture(&gpu, &images.occlusion)),
                    emissive: Some(renderer.add_texture(&gpu, &images.emissive)),
                }
            },
            // Un modèle 3D glTF (généré par tools/generer_arbre.py).
            tree: load_tree_model()
                .map(|tree| model::upload(&mut renderer, &gpu, &tree, "arbre", &mut self.names)),
            ground_texture: renderer.add_texture(
                &gpu,
                &Image::checkerboard(
                    64,
                    2,
                    Color::rgb(0.10, 0.11, 0.16),
                    Color::rgb(0.16, 0.17, 0.24),
                ),
            ),
        };
        // Les noms sous lesquels les ressources apparaissent dans les scènes sauvegardées.
        let names = &mut self.names;
        names.add_mesh("cube", assets.cube);
        names.add_mesh("cube_blanc", assets.white_cube);
        names.add_mesh("sol", assets.ground);
        names.add_mesh("soleil", assets.sun);
        names.add_mesh("planete", assets.planet);
        names.add_mesh("lune", assets.moon);
        names.add_mesh("sphere", assets.sphere);
        names.add_texture("dalles_relief", assets.ground_normal_map);
        names.add_texture("alveoles_relief", assets.dimples_normal_map);
        for (name, texture) in demo::panel_texture_names(&assets.panel) {
            names.add_texture(name, texture);
        }
        names.add_texture("caisse", assets.crate_texture);
        names.add_texture("grillage", assets.lattice_texture);
        names.add_texture("damier", assets.ground_texture);

        demo::spawn(&mut self.scene, &assets);
        self.assets = Some(assets);

        // Le son : ouvert ici (et non dans `new`) pour ne pas occuper la
        // carte son avant que la fenêtre existe. Les sons sont synthétisés.
        let mut audio = Audio::new();
        audio.add_sound(demo::IMPACT_SOUND, &zentrix_audio::synth::impact(170.0));
        audio.add_sound(demo::LAUNCH_SOUND, &zentrix_audio::synth::whoosh());
        audio.add_sound(demo::HUM_SOUND, &zentrix_audio::synth::hum());
        self.audio = audio;
        log::info!("Scène créée : {} entités", self.scene.entity_count());

        let info = gpu.adapter_info();
        let title = format!("ZENTRIX — {} ({:?})", info.name, info.backend);
        window.set_title(&title);
        log::info!(
            "Commandes : clic gauche = capturer la souris, ZQSD = se déplacer, Espace/Ctrl = monter/descendre"
        );
        log::info!("            Maj = vite, molette = vitesse, P = pause, Échap = libérer/quitter");
        log::info!("            F1 = grille et axes, F2 = boîtes englobantes, F3 = ombres");
        log::info!("            F4 = inspecteur, F5 = sauvegarder la scène, F9 = la recharger");
        log::info!("            F6 = tone mapping, F7 = environnement (ciel et reflets)");
        let ui = UiLayer::new(&window, &gpu);

        self.graphics = Some(GraphicsState {
            window,
            gpu,
            renderer,
            ui,
            title,
        });
        Ok(())
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        // `if let Some(g) = &mut self.graphics` : « si les graphismes existent,
        // emprunte-les en écriture sous le nom `g` ».
        if let Some(g) = &mut self.graphics {
            g.gpu.resize(width, height);
        }
    }

    fn on_window_event(&mut self, event: &WindowEvent) -> bool {
        // Quand la souris pilote la caméra, l'interface ne reçoit rien :
        // le curseur caché ne doit pas cliquer sur des boutons invisibles.
        // Exception : les RELÂCHEMENTS de bouton, pour qu'egui ne croie
        // jamais qu'un bouton est resté enfoncé.
        let is_release = matches!(
            event,
            WindowEvent::MouseInput {
                state: zentrix_platform::winit::event::ElementState::Released,
                ..
            }
        );
        if self.mouse_captured && !is_release {
            return false;
        }
        match &mut self.graphics {
            Some(g) => g.ui.on_window_event(&g.window, event),
            None => false,
        }
    }

    fn on_update(&mut self, dt: f32, input: &Input) {
        // Une nouvelle image commence : chaque `lap` mesure l'étape qui vient
        // de se terminer.
        self.profiler.begin_frame();
        self.handle_commands(input);

        // --- L'interface : elle peut modifier la scène, avant les systèmes ----
        let mut requests = inspector::Requests::default();
        let mut gizmo = GizmoResponse::default();
        if let Some(g) = &mut self.graphics {
            let mut vsync = g.gpu.vsync();
            let (width, height) = g.gpu.size();
            let camera = self.scene.camera().unwrap_or_default();
            let view_projection = camera.view_projection(width as f32 / height.max(1) as f32);
            let snap =
                input.is_key_down(KeyCode::ControlLeft) || input.is_key_down(KeyCode::ControlRight);
            let (can_undo, can_redo) = (self.history.can_undo(), self.history.can_redo());
            let perf = Perf {
                fps: self.last_fps,
                stats: g.renderer.stats(),
                profiler: &self.profiler,
                gpu: g.renderer.gpu_timings(),
                stress: self.stress,
            };
            // La fermeture capture des champs DIFFÉRENTS de `self` et de `g` :
            // Rust accepte de les emprunter en même temps.
            g.ui.run(&g.window, |ctx| {
                let toggles = Toggles {
                    show_grid: &mut self.show_grid,
                    show_bounds: &mut self.show_bounds,
                    shadows: &mut g.renderer.shadows_enabled,
                    tone_mapping: &mut g.renderer.tone_mapping,
                    bloom: &mut g.renderer.bloom_enabled,
                    bloom_intensity: &mut g.renderer.bloom_intensity,
                    bloom_threshold: &mut g.renderer.bloom_threshold,
                    fxaa: &mut g.renderer.fxaa_enabled,
                    show_cascades: &mut g.renderer.show_cascades,
                    shadow_distance: &mut g.renderer.shadow_settings.distance,
                    environment: &mut g.renderer.environment_enabled,
                    environment_intensity: &mut g.renderer.environment_intensity,
                    volume: &mut self.volume,
                    sound_off: &mut self.sound_off,
                    paused: &mut self.paused,
                    culling: &mut g.renderer.culling_enabled,
                    freeze_culling: &mut g.renderer.freeze_culling,
                    multithreading: &mut g.renderer.multithreading,
                    vsync: &mut vsync,
                    gpu_timing: &mut g.renderer.gpu_timing_enabled,
                    gizmo_mode: &mut self.gizmo.mode,
                    can_undo,
                    can_redo,
                };
                requests = self.inspector.show(ctx, &mut self.scene, toggles, perf);
                // Le gizmo de l'objet sélectionné (pas pendant qu'on pilote).
                if self.inspector.visible
                    && !self.mouse_captured
                    && let Some(entity) = self.inspector.selected()
                {
                    gizmo = editor::show_gizmo(
                        &mut self.gizmo,
                        ctx,
                        &self.scene.world,
                        entity,
                        view_projection,
                        camera.position,
                        snap,
                    );
                }
                self.game.hud(ctx, self.mouse_captured);
            });
            // Changer de mode d'affichage reconfigure la surface : on ne le
            // fait que si la case a vraiment changé.
            if vsync != g.gpu.vsync() {
                g.gpu.set_vsync(vsync);
            }
        }
        // Un clic capture la souris... sauf s'il vise l'interface. On décide
        // APRÈS que l'interface a traité l'image : avant, egui ne saurait pas
        // encore que le curseur vient d'arriver sur un de ses panneaux.
        let over_ui = self.graphics.as_ref().is_some_and(|g| g.ui.wants_pointer());
        // Un clic dans la scène rend le clavier au moteur.
        let clicked = input.was_button_pressed(MouseButton::Left)
            || input.was_button_pressed(MouseButton::Right);
        if clicked
            && !over_ui
            && let Some(g) = &self.graphics
        {
            g.ui.clear_focus();
        }
        if self.inspector.visible {
            // L'éditeur : le `Transform` est noté au DÉBUT du geste, avant que
            // le gizmo ne modifie l'objet (à l'image suivante) ; le geste
            // entre dans l'historique quand il se termine.
            if gizmo.started
                && let Some(entity) = self.inspector.selected()
                && let Ok(before) = self.scene.world.get::<&Transform>(entity).map(|t| *t)
            {
                self.gesture = Some((entity, before));
            }
            self.finish_gesture();
            self.editor_mouse(input, over_ui, gizmo);
        } else {
            self.editor_look = false;
            if input.was_button_pressed(MouseButton::Left) && !self.mouse_captured && !over_ui {
                self.set_mouse_captured(true);
            }
        }

        if requests.undo || requests.redo {
            self.undo_redo(requests.redo);
        }
        if let Some(kind) = requests.spawn {
            self.spawn_object(kind);
        }
        if requests.delete {
            self.delete_selected();
        }
        if requests.save {
            self.save_scene();
        }
        if requests.load {
            self.load_scene();
        }
        if requests.toggle_stress {
            self.toggle_stress_field();
        }
        self.profiler.lap("interface");

        // --- Les systèmes : la logique du jeu, entité par entité ---------------
        if !self.paused {
            // Grâce au delta time, l'animation a la même vitesse sur toutes les machines.
            demo::spin_system(&mut self.scene.world, dt);
        }
        // La souris ne fait tourner la vue que si elle est capturée. En
        // marche, les touches dirigent le personnage ; en vol, la caméra.
        // Dans l'éditeur, seulement pendant le clic droit : le reste du temps,
        // les touches servent aux raccourcis (Ctrl+Z ne doit pas faire avancer).
        let piloting = !self.inspector.visible || self.mouse_captured;
        if player::find(&self.scene.world).is_some() {
            if piloting {
                player::input_system(&mut self.scene.world, input, self.mouse_captured);
            } else {
                player::stop(&mut self.scene.world);
            }
        } else if piloting {
            demo::fly_camera_system(&mut self.scene.world, input, dt, self.mouse_captured);
        }
        demo::cleanup_system(&mut self.scene.world);
        self.profiler.lap("systèmes");

        // La physique, APRÈS les systèmes qui déplacent les objets cinématiques
        // (les cubes qui tournent) et AVANT le calcul des positions dans le monde.
        if !self.paused {
            self.physics.step(&mut self.scene, dt);
            self.play_impact_sounds();
        }
        // La caméra rejoint les yeux du personnage, qui vient de bouger.
        player::camera_follow_system(&mut self.scene.world);
        self.profiler.lap("physique");

        // La partie, APRÈS la physique : elle lit les chocs du pas qui vient
        // d'être simulé (une balle a-t-elle touché une cible ?).
        if !self.paused
            && let (Some(assets), Some(eye)) = (&self.assets, self.camera_transform())
        {
            let events = self.game.update(
                &mut self.scene.world,
                assets,
                self.physics.impacts(),
                eye.translation,
                dt,
            );
            self.play_game_events(&events);
        }
        self.profiler.lap("jeu");

        // Les positions dans le monde, une fois TOUS les déplacements faits.
        self.scene.update_transforms();
        self.profiler.lap("transforms");

        // Le son suit la scène : l'oreille (la caméra) et les émetteurs.
        self.audio
            .set_master_volume(if self.sound_off { 0.0 } else { self.volume });
        self.audio.update(&self.scene);
        self.profiler.lap("audio");

        // --- L'extraction : ce que le rendu doit dessiner ----------------------
        // Le Renderer ne connaît pas l'ECS : la scène lui prépare des listes simples.
        self.draws.clear();
        self.scene.extract_draws(&mut self.draws);
        self.debug.clear();
        self.profiler.lap("extraction");

        let Some(g) = &self.graphics else { return };

        if let Some(fps) = self.fps.tick(dt) {
            self.last_fps = fps;
            let hint = if self.inspector.visible {
                "éditeur : clic = sélectionner, clic droit maintenu = piloter, F4 = jouer"
            } else {
                "clic : piloter, Échap : libérer, F4 = éditeur"
            };
            g.window
                .set_title(&format!("{} — {fps:.0} FPS — {hint}", g.title));
        }

        if self.show_grid {
            // La grille flotte 1 cm au-dessus du sol : exactement à la même
            // hauteur, lignes et sol auraient la même profondeur et
            // « clignoteraient » en se disputant les pixels (z-fighting).
            let ground = Vec3::new(0.0, GROUND_Y + 0.01, 0.0);
            self.debug
                .grid(ground, 16.0, 16, Color::rgb(0.30, 0.32, 0.42));
            self.debug.axes(ground, 3.0);

            // La direction de la lumière : une flèche jaune qui « tombe »
            // depuis le ciel, dans le sens des rayons.
            let direction = self.scene.lighting().direction.normalize_or_zero();
            let start = Vec3::new(0.0, 7.0, 0.0);
            let end = start + direction * 3.0;
            let yellow = Color::rgb(1.0, 0.9, 0.3);
            self.debug.line(start, end, yellow);
            self.debug.wire_box(
                Mat4::from_scale_rotation_translation(Vec3::splat(0.15), Quat::IDENTITY, end),
                yellow,
            );
        }

        // Les boîtes couvertes par les tranches d'ombre, de la couleur dont
        // le shader teinte chaque tranche. Dans l'espace « vu depuis la
        // lumière » (NDC), chacune est le pavé [-1, 1] × [-1, 1] × [0, 1] ;
        // l'inverse de sa matrice le replace dans le monde (voir `NDC_BOX`).
        if g.renderer.show_cascades && g.renderer.shadows_enabled {
            let colors = [
                Color::rgb(1.0, 0.3, 0.3),
                Color::rgb(0.3, 1.0, 0.3),
                Color::rgb(0.3, 0.4, 1.0),
            ];
            for (cascade, color) in g.renderer.cascades().iter().zip(colors) {
                self.debug
                    .wire_box(cascade.view_projection.inverse() * NDC_BOX, color);
            }
        }

        // La pyramide de vision figée (inspecteur, « Figer ») : le même
        // pavé, replacé dans le monde par l'inverse de la matrice de la
        // caméra. La perspective en fait une pyramide tronquée.
        if let Some(frozen) = g.renderer.frozen_view_projection() {
            self.debug
                .wire_box(frozen.inverse() * NDC_BOX, Color::rgb(1.0, 0.6, 0.1));
        }

        if self.show_bounds {
            demo::bounds_system(&self.scene.world, &mut self.debug);
            self.physics.debug_draw(&mut self.debug);
            // Les lampes : un petit cube de leur couleur ; les projecteurs,
            // en plus, un trait dans la direction éclairée.
            let mut lights = Vec::new();
            self.scene.extract_lights(&mut lights);
            for light in &lights {
                let marker = Mat4::from_scale_rotation_translation(
                    Vec3::splat(0.25),
                    Quat::IDENTITY,
                    light.position,
                );
                self.debug.wire_box(marker, light.color);
                if let Some(cone) = light.spot {
                    let end = light.position + cone.direction.normalize_or_zero() * 2.0;
                    self.debug.line(light.position, end, light.color);
                }
            }
        }

        // L'entité sélectionnée dans l'inspecteur, entourée en cyan.
        if let Some(outline) = self
            .inspector
            .selected()
            .and_then(|e| inspector::selection_outline(&self.scene, e))
        {
            self.debug.wire_box(outline, Color::rgb(0.3, 1.0, 1.0));
        }
        self.profiler.lap("lignes de debug");
    }

    fn should_exit(&self) -> bool {
        self.quit_requested
    }

    fn on_render(&mut self) -> anyhow::Result<()> {
        if let Some(g) = &mut self.graphics {
            // Sans entité caméra, on utilise une caméra par défaut plutôt que
            // de ne rien afficher.
            let camera = self.scene.camera().unwrap_or_default();
            g.renderer.lighting = self.scene.lighting();
            // Les lampes et projecteurs : la liste est refaite à chaque image
            // (`clear` garde sa mémoire).
            g.renderer.lights.clear();
            self.scene.extract_lights(&mut g.renderer.lights);
            // L'interface est dessinée en dernier, par-dessus la scène.
            g.renderer.render_with_overlay(
                &mut g.gpu,
                &camera,
                &self.draws,
                &self.debug,
                Some(&mut g.ui),
            )?;
            // `pre_present_notify` n'est pas obligatoire, mais aide winit à
            // mieux planifier les images sur certaines plateformes (Wayland).
            g.window.pre_present_notify();
            // Le rendu a mesuré lui-même le temps passé à attendre une image
            // libre (la V-Sync) : ce n'est pas du travail, on le range à part.
            self.profiler.lap("rendu");
            self.profiler
                .transfer("rendu", "attente de l'écran", g.renderer.stats().wait_time);
        }
        Ok(())
    }

    fn on_exit(&mut self) {
        log::info!("Arrêt de ZENTRIX");
    }
}

/// Le fichier de sauvegarde de la démo.
fn scene_path() -> std::path::PathBuf {
    zentrix_assets::asset_dir().join("scenes/demo.ron")
}

/// Le sérialiseur de la démo : les composants du moteur, plus ceux du jeu.
fn scene_serializer() -> SceneSerializer {
    let mut serializer = SceneSerializer::default();
    serializer
        .register::<Spin>("Spin")
        .register::<FlyCamera>("FlyCamera")
        .register::<Ball>("Ball")
        .register::<StressCube>("StressCube")
        .register::<GameItem>("GameItem")
        .register::<Player>("Player")
        .register::<CharacterController>("CharacterController")
        .register::<RigidBody>("RigidBody")
        .register::<Collider>("Collider")
        .register::<AudioListener>("AudioListener")
        .register::<SoundEmitter>("SoundEmitter");
    serializer
}

/// Charge le modèle d'arbre `assets/models/arbre.glb`. En cas d'échec, la
/// démo continue sans arbres (`None`).
fn load_tree_model() -> Option<zentrix_assets::Model> {
    let path = zentrix_assets::asset_dir().join("models/arbre.glb");
    // `.inspect_err` : agir sur l'erreur (l'afficher) sans la consommer ;
    // `.ok()` : `Result` → `Option` (l'erreur est abandonnée).
    zentrix_assets::load_gltf(&path)
        .inspect_err(|error| log::warn!("{error} — pas d'arbres dans la démo"))
        .ok()
}

/// Charge la texture des caisses depuis `assets/textures/caisse.png`.
///
/// Si le fichier manque ou est invalide, la démo ne s'arrête pas : on
/// affiche l'erreur et on utilise une image de remplacement bien visible
/// (un damier magenta, la convention des moteurs pour « texture manquante »).
fn load_crate_image() -> Image {
    let path = zentrix_assets::asset_dir().join("textures/caisse.png");
    zentrix_assets::load_image(&path).unwrap_or_else(|error| {
        log::warn!("{error} — texture de remplacement utilisée");
        Image::checkerboard(64, 8, Color::rgb(1.0, 0.0, 1.0), Color::BLACK)
    })
}

/// Le point d'entrée du programme.
///
/// `main` peut renvoyer un `Result` : si c'est une `Err`, Rust affiche
/// l'erreur et le programme se termine avec un code d'échec.
fn main() -> anyhow::Result<()> {
    // Active l'affichage des logs. Par défaut, on montre les messages de niveau
    // `info` et plus graves pour nos crates, et seulement les avertissements
    // pour wgpu (très bavard). Surchargeable avec la variable RUST_LOG.
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn,naga=warn"),
    )
    .init();

    log::info!("Démarrage de ZENTRIX v{}", env!("CARGO_PKG_VERSION"));

    let config = WindowConfig {
        title: "ZENTRIX".to_string(),
        ..WindowConfig::default()
    };

    zentrix_platform::run(config, ZentrixApp::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zentrix_scene::Name;

    #[test]
    fn fps_moyenne_sur_une_demi_seconde() {
        let mut fps = FpsCounter::default();
        // 29 images de 1/60 s : pas encore une demi-seconde.
        for _ in 0..29 {
            assert_eq!(fps.tick(1.0 / 60.0), None);
        }
        // La 30e atteint 0,5 s : 30 images / 0,5 s = 60 FPS.
        let moyenne = fps.tick(1.0 / 60.0).expect("une moyenne");
        assert!((moyenne - 60.0).abs() < 0.01, "{moyenne}");
    }

    /// L'application sans fenêtre : une caméra et un cube nommé « Cube ».
    fn app_avec_un_cube() -> (ZentrixApp, Entity) {
        let mut app = ZentrixApp::new();
        let mesh = zentrix_render::MeshHandle::from_index(0);
        app.names.add_mesh("cube", mesh);
        let (fly, transform) = FlyCamera::looking_at(Vec3::new(0.0, 2.0, 8.0), Vec3::ZERO);
        app.scene
            .world
            .spawn((transform, fly, zentrix_scene::Camera3d::default()));
        let cube = app.scene.world.spawn((
            Name::new("Cube"),
            Transform::IDENTITY,
            zentrix_scene::MeshRenderer::new(mesh),
        ));
        (app, cube)
    }

    fn position_du_cube(app: &ZentrixApp) -> Vec3 {
        app.scene
            .world
            .query::<(&Name, &Transform)>()
            .iter()
            .find(|(n, _)| n.0 == "Cube")
            .map(|(_, t)| t.translation)
            .unwrap()
    }

    #[test]
    fn annuler_et_refaire_un_deplacement() {
        let (mut app, cube) = app_avec_un_cube();
        app.inspector.select(Some(cube));

        // Un geste du gizmo : noté au début, déplacé, puis terminé.
        app.gesture = Some((cube, Transform::IDENTITY));
        app.scene
            .world
            .get::<&mut Transform>(cube)
            .unwrap()
            .translation = Vec3::X * 3.0;
        app.finish_gesture();
        assert!(app.history.can_undo());
        // Entre-temps, la caméra a bougé : annuler ne doit pas la ramener.
        let camera = Vec3::new(5.0, 5.0, 5.0);
        for (t, _) in app.scene.world.query_mut::<(&mut Transform, &FlyCamera)>() {
            t.translation = camera;
        }

        app.undo_redo(false);
        assert_eq!(position_du_cube(&app), Vec3::ZERO, "annulé");
        assert_eq!(app.camera_transform().unwrap().translation, camera);
        assert_eq!(
            app.inspector.selected(),
            Some(cube),
            "même entité, toujours sélectionnée"
        );

        app.undo_redo(true);
        assert_eq!(position_du_cube(&app), Vec3::X * 3.0, "refait");
        assert!(!app.history.can_redo());
    }

    #[test]
    fn un_geste_sans_effet_n_entre_pas_dans_l_historique() {
        let (mut app, cube) = app_avec_un_cube();
        app.gesture = Some((cube, Transform::IDENTITY));
        app.finish_gesture();
        assert!(!app.history.can_undo());
    }

    #[test]
    fn supprimer_puis_annuler() {
        let (mut app, cube) = app_avec_un_cube();
        app.inspector.select(Some(cube));
        app.delete_selected();
        assert!(!app.scene.world.contains(cube));
        assert_eq!(app.inspector.selected(), None);
        app.undo_redo(false);
        assert_eq!(position_du_cube(&app), Vec3::ZERO, "le cube est revenu");
        app.undo_redo(true);
        assert!(
            app.scene
                .world
                .query::<&Name>()
                .iter()
                .all(|n| n.0 != "Cube"),
            "et supprimé de nouveau"
        );
    }
}
