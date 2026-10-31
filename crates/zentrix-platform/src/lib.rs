//! # zentrix-platform
//!
//! Tout ce qui touche au **système d'exploitation** : la fenêtre, le clavier,
//! la souris (via [`Input`]) et la **boucle d'événements**.
//!
//! On s'appuie sur la crate [`winit`], mais le reste du moteur ne parle qu'à
//! notre trait [`Application`]. Avantage : si un jour on change de bibliothèque
//! de fenêtrage, seule cette crate est à réécrire.
//!
//! ## Comment ça marche ?
//!
//! ```text
//!    zentrix-app                 zentrix-platform              winit / OS
//!  ┌─────────────┐ run(cfg, app) ┌────────────────┐ événements ┌─────────┐
//!  │    impl     │ ────────────▶ │ PlatformRunner │ ◀───────── │ Windows │
//!  │ Application │ ◀──────────── │                │            │ Linux   │
//!  └─────────────┘  on_start,    └────────────────┘            │ macOS   │
//!                   on_update,                                 └─────────┘
//!                   on_render...
//! ```
//!
//! 1. L'application appelle [`run`] en lui donnant une [`WindowConfig`] et un
//!    objet qui implémente [`Application`].
//! 2. [`run`] crée la boucle d'événements winit et **ne rend la main qu'à la
//!    fermeture** de la fenêtre.
//! 3. À chaque événement (redimensionnement, touche, besoin de redessiner...),
//!    le `PlatformRunner` appelle la méthode correspondante de l'application.

mod application;
mod input;
mod runner;

pub use application::{Application, WindowConfig};
pub use input::{Input, set_cursor_captured};
pub use runner::run;

// On ré-exporte quelques types de winit dont l'application a besoin.
// Ainsi `zentrix-app` n'a pas besoin de dépendre directement de winit
// pour ces cas simples.
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;
pub use winit::window::Window;
