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
//! Commandes : `Échap` pour quitter, `Espace` pour mettre en pause l'animation du fond.

use std::sync::Arc;

// `anyhow::Context` ajoute la méthode `.context("...")` aux `Result` :
// elle enrichit le message d'erreur pour savoir OÙ l'erreur s'est produite.
use anyhow::Context;

use zentrix_core::Color;
use zentrix_platform::{Application, KeyCode, Window, WindowConfig};
use zentrix_render::Renderer;
use zentrix_rhi::GpuContext;

/// Les objets qui n'existent qu'une fois la fenêtre créée.
///
/// On les regroupe dans une struct pour n'avoir qu'**une seule** `Option` à
/// gérer : soit tout est initialisé, soit rien ne l'est.
struct GraphicsState {
    window: Arc<Window>,
    gpu: GpuContext,
    renderer: Renderer,
}

/// L'état de notre application de démonstration.
struct ZentrixApp {
    /// `None` avant `on_start`, `Some(...)` après.
    graphics: Option<GraphicsState>,
    /// Temps d'animation accumulé (en secondes).
    time: f32,
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
            paused: false,
            quit_requested: false,
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

        let renderer = Renderer::new(&gpu);

        let info = gpu.adapter_info();
        window.set_title(&format!("ZENTRIX — {} ({:?})", info.name, info.backend));

        self.graphics = Some(GraphicsState {
            window,
            gpu,
            renderer,
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

    fn on_key(&mut self, key: KeyCode, pressed: bool) {
        // On ne réagit qu'à l'appui, pas au relâchement.
        if !pressed {
            return;
        }
        match key {
            KeyCode::Escape => self.quit_requested = true,
            KeyCode::Space => {
                self.paused = !self.paused;
                log::info!(
                    "Animation {}",
                    if self.paused { "en pause" } else { "reprise" }
                );
            }
            _ => {}
        }
    }

    fn on_update(&mut self, dt: f32) {
        if !self.paused {
            // Grâce au delta time, l'animation a la même vitesse sur toutes les machines.
            self.time += dt;
        }

        if let Some(g) = &mut self.graphics {
            // Le fond oscille doucement entre deux couleurs.
            // `sin` varie entre -1 et 1 ; `* 0.5 + 0.5` le ramène entre 0 et 1.
            let t = (self.time * 0.8).sin() * 0.5 + 0.5;
            g.renderer.clear_color = Color::ZENTRIX_NIGHT.lerp(Color::rgb(0.15, 0.05, 0.25), t);
        }
    }

    fn should_exit(&self) -> bool {
        self.quit_requested
    }

    fn on_render(&mut self) -> anyhow::Result<()> {
        if let Some(g) = &mut self.graphics {
            g.renderer.render(&mut g.gpu)?;
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
