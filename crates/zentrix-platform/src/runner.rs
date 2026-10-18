//! Le moteur de la boucle d'événements : traduit les événements winit en
//! appels à notre trait [`Application`].

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

use zentrix_core::FrameTimer;

use crate::application::{Application, WindowConfig};

/// Lance l'application. **Cette fonction ne rend la main qu'à la fermeture.**
///
/// # Les génériques : `<A: Application>`
///
/// `A` est un **paramètre de type** : la fonction accepte n'importe quel type,
/// à condition qu'il implémente le trait `Application` (c'est la « contrainte »
/// après les deux-points). Le compilateur génère une version spécialisée de
/// `run` pour votre type : aucun coût à l'exécution.
///
/// # Exemple
///
/// ```no_run
/// # use std::sync::Arc;
/// # use zentrix_platform::{run, Application, Window, WindowConfig};
/// struct MonJeu;
///
/// impl Application for MonJeu {
///     fn on_start(&mut self, _window: Arc<Window>) -> anyhow::Result<()> { Ok(()) }
///     fn on_render(&mut self) -> anyhow::Result<()> { Ok(()) }
/// }
///
/// fn main() -> anyhow::Result<()> {
///     run(WindowConfig::default(), MonJeu)
/// }
/// ```
pub fn run<A: Application>(config: WindowConfig, app: A) -> anyhow::Result<()> {
    // `EventLoop::new()` renvoie un `Result`. L'opérateur `?` signifie :
    // « si c'est une erreur, renvoie-la immédiatement à l'appelant ;
    //   sinon, donne-moi la valeur contenue dans `Ok` ».
    let event_loop = EventLoop::new()?;

    // `Poll` : la boucle tourne en continu, sans attendre d'événement.
    // Idéal pour un jeu qui redessine en permanence. (L'alternative `Wait`
    // économise le CPU pour les applications qui ne bougent pas, comme un éditeur.)
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = PlatformRunner {
        config,
        app,
        window: None,
        timer: FrameTimer::new(),
        error: None,
    };

    // `&mut runner` : on PRÊTE le runner à winit en écriture. Après cette
    // ligne, `runner` nous appartient toujours : on peut lire `runner.error`.
    event_loop.run_app(&mut runner)?;

    // `take()` sort la valeur de l'`Option` en laissant `None` à sa place.
    // `if let Some(e) = ...` : n'exécute le bloc que s'il y avait une erreur.
    if let Some(e) = runner.error.take() {
        return Err(e);
    }
    Ok(())
}

/// Structure interne (non `pub`) qui fait le lien entre winit et l'application.
struct PlatformRunner<A: Application> {
    config: WindowConfig,
    app: A,
    /// `Option` car la fenêtre n'existe pas encore au démarrage :
    /// winit ne nous autorise à la créer que dans `resumed`.
    window: Option<Arc<Window>>,
    timer: FrameTimer,
    /// Première erreur fatale rencontrée, renvoyée par `run` à la fin.
    error: Option<anyhow::Error>,
}

impl<A: Application> PlatformRunner<A> {
    /// Enregistre une erreur fatale et demande l'arrêt de la boucle.
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        // `{:#}` affiche l'erreur ET toute sa chaîne de causes (format « alternatif » d'anyhow).
        log::error!("Erreur fatale : {error:#}");
        self.error = Some(error);
        event_loop.exit();
    }
}

// On implémente le trait `ApplicationHandler` de winit pour notre runner :
// c'est winit qui appellera ces méthodes.
impl<A: Application> ApplicationHandler for PlatformRunner<A> {
    /// Appelé quand l'application devient active (au démarrage sur PC ;
    /// potentiellement plusieurs fois sur mobile).
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Si la fenêtre existe déjà, il n'y a rien à faire.
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .with_resizable(self.config.resizable);

        // `match` examine TOUS les cas possibles d'une valeur. Le compilateur
        // refuse de compiler si on en oublie un : impossible d'ignorer une erreur par accident.
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                self.fail(event_loop, e.into());
                return;
            }
        };

        log::info!("Fenêtre créée : {:?}", window.inner_size());

        // `Arc::clone(&window)` ne copie pas la fenêtre : il crée un second
        // « pointeur partagé » vers la même fenêtre (et incrémente le compteur).
        if let Err(e) = self.app.on_start(Arc::clone(&window)) {
            self.fail(event_loop, e);
            return;
        }

        // On remet le chronomètre à zéro pour que la première image n'ait pas
        // un delta énorme (le temps d'initialisation du GPU).
        self.timer = FrameTimer::new();
        window.request_redraw();
        self.window = Some(window);
    }

    /// Appelé pour chaque événement concernant une fenêtre.
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            // L'utilisateur a cliqué sur la croix.
            WindowEvent::CloseRequested => {
                log::info!("Fermeture demandée");
                event_loop.exit();
            }

            // La fenêtre a changé de taille.
            WindowEvent::Resized(size) => {
                self.app.on_resize(size.width, size.height);
            }

            // Une touche du clavier. La syntaxe `event: KeyEvent { ... }` est un
            // « motif » (pattern) qui extrait directement les champs voulus.
            // `..` signifie « et j'ignore les autres champs ».
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        repeat: false, // on ignore la répétition auto quand on reste appuyé
                        ..
                    },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                self.app.on_key(key, pressed);
            }

            // Le système (ou nous-mêmes) demande de redessiner la fenêtre :
            // c'est le cœur de la boucle de jeu.
            WindowEvent::RedrawRequested => {
                if self.timer.tick() {
                    log::debug!("{:.0} FPS", self.timer.fps());
                }

                self.app.on_update(self.timer.delta_seconds());

                if self.app.should_exit() {
                    log::info!("L'application a demandé la fermeture");
                    event_loop.exit();
                    return;
                }

                if let Err(e) = self.app.on_render() {
                    self.fail(event_loop, e);
                }
            }

            // `_` attrape tous les autres événements : on les ignore pour l'instant.
            _ => {}
        }
    }

    /// Appelé quand winit a traité tous les événements en attente.
    /// On en profite pour demander l'image suivante : la boucle continue.
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // `as_ref()` transforme `&Option<Arc<Window>>` en `Option<&Arc<Window>>`
        // pour EMPRUNTER la fenêtre sans la sortir de l'Option.
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    /// Appelé une fois, juste avant que la boucle ne se termine.
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.app.on_exit();
    }
}
