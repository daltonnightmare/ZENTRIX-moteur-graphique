//! Le « contrat » entre la plateforme et l'application : le trait [`Application`].

use std::sync::Arc;

use winit::keyboard::KeyCode;
use winit::window::Window;

use crate::input::Input;

/// Paramètres de création de la fenêtre.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Titre affiché dans la barre de la fenêtre.
    ///
    /// `String` (et non `&str`) : la config **possède** son texte. Elle n'a donc
    /// pas besoin de durée de vie (`'a`) et peut être déplacée librement.
    pub title: String,
    /// Largeur initiale en pixels « logiques » (indépendants du zoom de l'écran).
    pub width: u32,
    /// Hauteur initiale en pixels logiques.
    pub height: u32,
    /// La fenêtre peut-elle être redimensionnée par l'utilisateur ?
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            // `.to_string()` convertit un `&str` (texte figé dans le programme)
            // en `String` (texte possédé, modifiable, alloué sur le tas).
            title: "ZENTRIX".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
        }
    }
}

/// Le trait que toute application ZENTRIX doit implémenter.
///
/// # Qu'est-ce qu'un trait ?
///
/// Un **trait** est une liste de méthodes qu'un type promet de fournir — un peu
/// comme une *interface* en Java/C#. La plateforme ne connaît pas votre type
/// concret ; elle sait seulement qu'il « sait faire » `on_start`, `on_render`...
///
/// Certaines méthodes ont un **corps par défaut** (`{}`) : vous n'êtes pas obligé
/// de les écrire si vous n'en avez pas besoin. Seules `on_start` et `on_render`
/// sont obligatoires.
///
/// # Ordre des appels
///
/// ```text
/// on_start ──▶ [ on_update ──▶ on_render ] ──▶ [ on_update ──▶ on_render ] ──▶ ... ──▶ on_exit
///                  ▲ entre deux images : on_resize, on_key (quand ils arrivent)
/// ```
///
/// Pour le clavier, deux styles sont possibles :
///
/// - réagir à l'événement dans [`Application::on_key`] ;
/// - ou interroger l'état dans [`Application::on_update`] via [`Input`]
///   (`input.is_key_down(...)`), ce qui convient mieux aux déplacements.
pub trait Application {
    /// Appelée **une seule fois**, juste après la création de la fenêtre.
    /// C'est ici qu'on initialise le GPU.
    ///
    /// # Pourquoi `Arc<Window>` ?
    ///
    /// La fenêtre doit être partagée entre la plateforme (qui demande les
    /// redessins) et le GPU (dont la *surface* dessine dedans). En Rust, une
    /// valeur n'a qu'**un seul propriétaire**. `Arc` (« Atomically Reference
    /// Counted ») permet d'avoir **plusieurs propriétaires** : chaque
    /// `Arc::clone` incrémente un compteur, et la fenêtre n'est détruite que
    /// lorsque le dernier `Arc` disparaît.
    ///
    /// # Le type de retour `anyhow::Result<()>`
    ///
    /// - `Ok(())` : tout s'est bien passé (`()` = « rien », le type vide) ;
    /// - `Err(erreur)` : l'initialisation a échoué, la plateforme ferme proprement.
    fn on_start(&mut self, window: Arc<Window>) -> anyhow::Result<()>;

    /// Appelée quand la taille de la fenêtre change (en pixels physiques).
    ///
    /// Les paramètres commencent par `_` : cela indique au compilateur qu'ils
    /// sont volontairement inutilisés dans l'implémentation par défaut
    /// (sinon il afficherait un avertissement).
    fn on_resize(&mut self, _width: u32, _height: u32) {}

    /// Appelée quand une touche est enfoncée (`pressed == true`) ou relâchée.
    fn on_key(&mut self, _key: KeyCode, _pressed: bool) {}

    /// Mise à jour de la logique, une fois par image, **avant** le rendu.
    ///
    /// - `dt` = delta time en secondes (voir `zentrix_core::FrameTimer`) ;
    /// - `input` = l'état du clavier et de la souris pour cette image. Il est
    ///   seulement **prêté** (`&`) : l'application le lit, la plateforme le met à jour.
    fn on_update(&mut self, _dt: f32, _input: &Input) {}

    /// Dessine une image. Une erreur renvoyée ici arrête l'application.
    fn on_render(&mut self) -> anyhow::Result<()>;

    /// Interrogée après chaque `on_update` : renvoyer `true` ferme proprement
    /// l'application (par exemple après un appui sur Échap).
    ///
    /// `&self` (et non `&mut self`) : la méthode ne fait que **lire** l'état.
    fn should_exit(&self) -> bool {
        false
    }

    /// Appelée juste avant la fermeture : dernier moment pour sauvegarder, etc.
    fn on_exit(&mut self) {}
}
