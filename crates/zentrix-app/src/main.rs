//! # zentrix-app
//!
//! L'exécutable de démonstration de ZENTRIX. Il assemble les crates :
//!
//! ```text
//!                 zentrix-app  (ce fichier)
//!                /      |      \
//!   zentrix-platform  zentrix-render
//!          |              |
//!          |         zentrix-rhi ── wgpu ── GPU
//!          |              |
//!        winit       zentrix-core
//! ```
//!
//! Lancer avec : `cargo run` (depuis la racine du projet).
//! Logs détaillés : `RUST_LOG=debug cargo run` (bash) ou
//! `$env:RUST_LOG="debug"; cargo run` (PowerShell).
//!
//! La démo : une grille de cubes (caisses en bois texturées et cubes colorés)
//! qui tournent sur un sol en damier, explorée avec une caméra libre.
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
//! | `Échap`             | libérer la souris, puis quitter               |

mod fly_camera;

use std::sync::Arc;

// `anyhow::Context` ajoute la méthode `.context("...")` aux `Result` :
// elle enrichit le message d'erreur pour savoir OÙ l'erreur s'est produite.
use anyhow::Context;

use zentrix_core::math::{Quat, Vec3};
use zentrix_core::{Color, Image, Transform};
use zentrix_platform::{
    Application, Input, KeyCode, MouseButton, Window, WindowConfig, set_cursor_captured,
};
use zentrix_render::{Camera, DrawCommand, MeshData, MeshHandle, Renderer, TextureHandle};
use zentrix_rhi::GpuContext;

use crate::fly_camera::FlyCamera;

/// Nombre de cubes par côté de la grille (la scène en contient le carré).
const GRID_SIZE: i32 = 5;
/// Distance entre deux cubes voisins.
const GRID_SPACING: f32 = 1.8;

/// Les objets qui n'existent qu'une fois la fenêtre créée.
///
/// On les regroupe dans une struct pour n'avoir qu'**une seule** `Option` à
/// gérer : soit tout est initialisé, soit rien ne l'est.
struct GraphicsState {
    window: Arc<Window>,
    gpu: GpuContext,
    renderer: Renderer,
    cube: MeshHandle,
    /// Le même cube, mais tout blanc : la texture y garde ses vraies couleurs.
    white_cube: MeshHandle,
    ground: MeshHandle,
    crate_texture: TextureHandle,
    ground_texture: TextureHandle,
}

/// L'état de notre application de démonstration.
struct ZentrixApp {
    /// `None` avant `on_start`, `Some(...)` après.
    graphics: Option<GraphicsState>,
    /// Temps d'animation accumulé (en secondes).
    time: f32,
    /// La caméra de rendu, et le contrôleur qui la pilote.
    camera: Camera,
    fly_camera: FlyCamera,
    /// La souris est-elle capturée par la fenêtre (mode « regard ») ?
    mouse_captured: bool,
    /// Les commandes de dessin de l'image en cours. Gardé d'une image à
    /// l'autre pour réutiliser sa mémoire.
    draws: Vec<DrawCommand>,
    /// L'animation est-elle en pause ?
    paused: bool,
    /// Demande de fermeture (touche Échap).
    quit_requested: bool,
}

impl ZentrixApp {
    fn new() -> Self {
        Self {
            graphics: None,
            time: 0.0,
            camera: Camera::default(),
            fly_camera: FlyCamera::looking_at(Vec3::new(0.0, 4.0, 11.0), Vec3::new(0.0, 0.5, 0.0)),
            mouse_captured: false,
            draws: Vec::new(),
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
        // toutes ses instances grâce à son handle.
        let cube = renderer.add_mesh(&gpu, &MeshData::cube());
        let white_cube = renderer.add_mesh(&gpu, &MeshData::cube().with_color([1.0; 3]));
        // Le sol répète sa texture 8 × 8 fois (le sampler est en mode `Repeat`).
        let ground = renderer.add_mesh(&gpu, &MeshData::plane([1.0; 3]).with_uv_scale(8.0));

        // Les textures : une image chargée depuis le disque, une autre générée.
        let crate_texture = renderer.add_texture(&gpu, &load_crate_image());
        let ground_texture = renderer.add_texture(
            &gpu,
            &Image::checkerboard(
                64,
                2,
                Color::rgb(0.10, 0.11, 0.16),
                Color::rgb(0.16, 0.17, 0.24),
            ),
        );

        let info = gpu.adapter_info();
        window.set_title(&format!(
            "ZENTRIX — {} ({:?}) — clic : piloter, Échap : libérer",
            info.name, info.backend
        ));
        log::info!(
            "Commandes : clic gauche = capturer la souris, ZQSD = se déplacer, Espace/Ctrl = monter/descendre"
        );
        log::info!("            Maj = vite, molette = vitesse, P = pause, Échap = libérer/quitter");

        self.graphics = Some(GraphicsState {
            window,
            gpu,
            renderer,
            cube,
            white_cube,
            ground,
            crate_texture,
            ground_texture,
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

        if !self.paused {
            // Grâce au delta time, l'animation a la même vitesse sur toutes les machines.
            self.time += dt;
        }

        // La souris ne fait tourner la vue que si elle est capturée.
        self.fly_camera.update(input, dt, self.mouse_captured);
        self.fly_camera.apply_to(&mut self.camera);

        // On reconstruit la liste des objets à dessiner. Plus tard, c'est
        // l'ECS (zentrix-scene) qui produira cette liste à partir des entités.
        let Some(g) = &self.graphics else { return };
        self.draws.clear();

        // Le sol : un plan de 1×1 agrandi 16 fois, juste sous les cubes.
        self.draws.push(DrawCommand {
            mesh: g.ground,
            texture: Some(g.ground_texture),
            transform: Transform::from_translation(Vec3::new(0.0, -0.5, 0.0))
                .with_uniform_scale(16.0)
                .matrix(),
        });

        // Une grille de cubes centrée sur l'origine. `half` décale la grille
        // pour que son centre soit en (0, 0) plutôt que son coin.
        let half = (GRID_SIZE - 1) as f32 * GRID_SPACING / 2.0;
        for x in 0..GRID_SIZE {
            for z in 0..GRID_SIZE {
                let position = Vec3::new(
                    x as f32 * GRID_SPACING - half,
                    0.3,
                    z as f32 * GRID_SPACING - half,
                );
                // Chaque cube tourne à sa propre vitesse, autour d'un axe incliné.
                let speed = 0.5 + 0.15 * (x + z) as f32;
                let axis = Vec3::new(1.0, 1.0, 0.3).normalize();
                // Une case sur deux (comme un damier) : une caisse en bois.
                let is_crate = (x + z) % 2 == 0;
                self.draws.push(DrawCommand {
                    mesh: if is_crate { g.white_cube } else { g.cube },
                    texture: is_crate.then_some(g.crate_texture),
                    transform: Transform::from_translation(position)
                        .with_rotation(Quat::from_axis_angle(axis, self.time * speed))
                        .matrix(),
                });
            }
        }
    }

    fn should_exit(&self) -> bool {
        self.quit_requested
    }

    fn on_render(&mut self) -> anyhow::Result<()> {
        if let Some(g) = &mut self.graphics {
            g.renderer.render(&mut g.gpu, &self.camera, &self.draws)?;
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
