//! # zentrix-app
//!
//! L'exécutable de démonstration de ZENTRIX. Il assemble les crates :
//!
//! ```text
//!                        zentrix-app  (ce fichier + demo.rs + fly_camera.rs)
//!          ┌──────────────┬────┴─────────┬───────────────┐
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
//! | `P`                 | mettre en pause l'animation des cubes         |
//! | `F1`                | afficher / masquer la grille et les axes      |
//! | `F2`                | afficher / masquer les boîtes des cubes       |
//! | `Échap`             | libérer la souris, puis quitter               |

mod demo;
mod fly_camera;

use std::sync::Arc;

// `anyhow::Context` ajoute la méthode `.context("...")` aux `Result` :
// elle enrichit le message d'erreur pour savoir OÙ l'erreur s'est produite.
use anyhow::Context;

use zentrix_core::math::Vec3;
use zentrix_core::{Color, Image};
use zentrix_platform::{
    Application, Input, KeyCode, MouseButton, Window, WindowConfig, set_cursor_captured,
};
use zentrix_render::{DebugLines, DrawCommand, MeshData, Renderer};
use zentrix_rhi::GpuContext;
use zentrix_scene::Scene;

use crate::demo::{DemoAssets, GROUND_Y};

/// Intervalle de mise à jour du compteur d'images par seconde (en secondes).
const FPS_REFRESH: f32 = 0.5;

/// Les objets qui n'existent qu'une fois la fenêtre créée.
///
/// On les regroupe dans une struct pour n'avoir qu'**une seule** `Option` à
/// gérer : soit tout est initialisé, soit rien ne l'est.
struct GraphicsState {
    window: Arc<Window>,
    gpu: GpuContext,
    renderer: Renderer,
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
    /// La souris est-elle capturée par la fenêtre (mode « regard ») ?
    mouse_captured: bool,
    /// Les commandes de dessin de l'image en cours. Gardé d'une image à
    /// l'autre pour réutiliser sa mémoire.
    draws: Vec<DrawCommand>,
    /// Les lignes de debug de l'image en cours.
    debug: DebugLines,
    /// F1 : grille et axes ; F2 : boîtes englobantes des cubes.
    show_grid: bool,
    show_bounds: bool,
    fps: FpsCounter,
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
            mouse_captured: false,
            draws: Vec::new(),
            debug: DebugLines::default(),
            show_grid: true,
            show_bounds: false,
            fps: FpsCounter::default(),
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

        // Échap libère d'abord la souris ; un second appui quitte.
        if input.was_key_pressed(KeyCode::Escape) {
            if self.mouse_captured {
                self.set_mouse_captured(false);
            } else {
                self.quit_requested = true;
            }
        }

        if input.was_button_pressed(MouseButton::Left) && !self.mouse_captured {
            self.set_mouse_captured(true);
        }

        // Alt+Tab pendant la capture : on rend la souris au système.
        if !input.is_focused() && self.mouse_captured {
            self.set_mouse_captured(false);
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
        demo::spawn(&mut self.scene, &assets);
        log::info!("Scène créée : {} entités", self.scene.entity_count());

        let info = gpu.adapter_info();
        let title = format!("ZENTRIX — {} ({:?})", info.name, info.backend);
        window.set_title(&title);
        log::info!(
            "Commandes : clic gauche = capturer la souris, ZQSD = se déplacer, Espace/Ctrl = monter/descendre"
        );
        log::info!("            Maj = vite, molette = vitesse, P = pause, Échap = libérer/quitter");
        log::info!("            F1 = grille et axes, F2 = boîtes englobantes");

        self.graphics = Some(GraphicsState {
            window,
            gpu,
            renderer,
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

    fn on_update(&mut self, dt: f32, input: &Input) {
        self.handle_commands(input);

        // --- Les systèmes : la logique du jeu, entité par entité ---------------
        if !self.paused {
            // Grâce au delta time, l'animation a la même vitesse sur toutes les machines.
            demo::spin_system(&mut self.scene.world, dt);
        }
        // La souris ne fait tourner la vue que si elle est capturée.
        demo::fly_camera_system(&mut self.scene.world, input, dt, self.mouse_captured);

        // --- L'extraction : ce que le rendu doit dessiner ----------------------
        // Le Renderer ne connaît pas l'ECS : la scène lui prépare des listes simples.
        self.draws.clear();
        self.scene.extract_draws(&mut self.draws);
        self.debug.clear();

        let Some(g) = &self.graphics else { return };

        if let Some(fps) = self.fps.tick(dt) {
            g.window.set_title(&format!(
                "{} — {fps:.0} FPS — clic : piloter, Échap : libérer",
                g.title
            ));
        }

        if self.show_grid {
            // La grille flotte 1 cm au-dessus du sol : exactement à la même
            // hauteur, lignes et sol auraient la même profondeur et
            // « clignoteraient » en se disputant les pixels (z-fighting).
            let ground = Vec3::new(0.0, GROUND_Y + 0.01, 0.0);
            self.debug
                .grid(ground, 16.0, 16, Color::rgb(0.30, 0.32, 0.42));
            self.debug.axes(ground, 3.0);
        }

        if self.show_bounds {
            demo::bounds_system(&self.scene.world, &mut self.debug);
        }
    }

    fn should_exit(&self) -> bool {
        self.quit_requested
    }

    fn on_render(&mut self) -> anyhow::Result<()> {
        if let Some(g) = &mut self.graphics {
            // Sans entité caméra, on utilise une caméra par défaut plutôt que
            // de ne rien afficher.
            let camera = self.scene.camera().unwrap_or_default();
            g.renderer
                .render(&mut g.gpu, &camera, &self.draws, &self.debug)?;
            // `pre_present_notify` n'est pas obligatoire, mais aide winit à
            // mieux planifier les images sur certaines plateformes (Wayland).
            g.window.pre_present_notify();
        }
        Ok(())
    }

    fn on_exit(&mut self) {
        log::info!("Arrêt de ZENTRIX");
    }
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
}
