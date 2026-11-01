//! Le moteur de la boucle d'événements : traduit les événements winit en
//! appels à notre trait [`Application`].

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, PhysicalKey};
use winit::window::{Window, WindowId};

use zentrix_core::FrameTimer;
use zentrix_core::math::Vec2;

use crate::application::{Application, WindowConfig};
use crate::input::Input;

/// Nombre de pixels de défilement (pavé tactile) comptés comme un cran de molette.
const PIXELS_PER_SCROLL_LINE: f32 = 40.0;

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
        input: Input::default(),
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
    /// L'état du clavier et de la souris, mis à jour à chaque événement.
    input: Input,
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
        self.input.on_focus(window.has_focus());
        window.request_redraw();
        self.window = Some(window);
    }

    /// Appelé pour chaque événement concernant une fenêtre.
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // L'application (son interface graphique) voit l'événement en premier.
        let consumed = self.app.on_window_event(&event);

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
                        logical_key,
                        state,
                        repeat: false, // on ignore la répétition auto quand on reste appuyé
                        ..
                    },
                ..
            } => {
                let pressed = state == ElementState::Pressed;
                // Un appui consommé (texte tapé dans l'interface) n'est pas
                // transmis au jeu ; un relâchement l'est toujours.
                if !(consumed && pressed) {
                    self.input.on_key(key, pressed);
                    // La lettre imprimée sur la touche (pour les raccourcis
                    // comme Ctrl+Z). winit la donne sans l'effet de Ctrl.
                    if pressed && let Key::Character(text) = &logical_key {
                        self.input.on_character(text);
                    }
                    self.app.on_key(key, pressed);
                }
            }

            // Un bouton de la souris.
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;
                if !(consumed && pressed) {
                    self.input.on_button(button, pressed);
                }
            }

            // La molette (ou le défilement à deux doigts d'un pavé tactile).
            WindowEvent::MouseWheel { delta, .. } if !consumed => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / PIXELS_PER_SCROLL_LINE,
                };
                self.input.on_scroll(lines);
            }

            // Position du curseur (utile pour cliquer sur des éléments d'interface).
            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .on_cursor_moved(Some(Vec2::new(position.x as f32, position.y as f32)));
            }
            WindowEvent::CursorLeft { .. } => self.input.on_cursor_moved(None),

            // La fenêtre gagne ou perd le focus (Alt+Tab, clic ailleurs...).
            WindowEvent::Focused(focused) => self.input.on_focus(focused),

            // Le système (ou nous-mêmes) demande de redessiner la fenêtre :
            // c'est le cœur de la boucle de jeu.
            WindowEvent::RedrawRequested => {
                if self.timer.tick() {
                    log::debug!("{:.0} FPS", self.timer.fps());
                }

                // `&self.input` et `self.app` sont deux champs différents : Rust
                // accepte de prêter l'un en lecture pendant qu'on modifie l'autre.
                self.app.on_update(self.timer.delta_seconds(), &self.input);

                if self.app.should_exit() {
                    log::info!("L'application a demandé la fermeture");
                    event_loop.exit();
                    return;
                }

                if let Err(e) = self.app.on_render() {
                    self.fail(event_loop, e);
                }

                // Les appuis et déplacements de cette image ont été vus : on
                // repart de zéro pour la suivante.
                self.input.end_frame();

                // Pour Tracy : « une image vient de se terminer ». Sans la
                // feature `tracy`, cette macro ne fait rien.
                profiling::finish_frame!();
            }

            // `_` attrape tous les autres événements : on les ignore pour l'instant.
            _ => {}
        }
    }

    /// Appelé pour les événements « bruts » des périphériques, indépendants
    /// de la fenêtre. On n'utilise que le mouvement de la souris : contrairement
    /// à `CursorMoved`, il continue d'arriver quand le curseur est capturé.
    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        // Ces événements arrivent même si la fenêtre n'a pas le focus : on les
        // ignore dans ce cas, pour ne pas faire tourner la caméra en arrière-plan.
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event
            && self.input.is_focused()
        {
            self.input.on_mouse_motion(dx as f32, dy as f32);
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
