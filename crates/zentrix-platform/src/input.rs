//! [`Input`] : l'état du clavier et de la souris, interrogeable à chaque image.

use std::collections::HashSet;

use winit::event::MouseButton;
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Window};

use zentrix_core::math::Vec2;

/// L'état des entrées pour l'image en cours.
///
/// # Événements contre état
///
/// Le système envoie des **événements** (« la touche Z vient d'être enfoncée »).
/// Mais un jeu pose plutôt des questions d'**état** à chaque image (« la touche
/// Z est-elle enfoncée en ce moment ? »). `Input` fait la traduction : le
/// `PlatformRunner` lui transmet chaque événement, puis l'application
/// l'interroge dans `on_update`.
///
/// Trois questions différentes, à ne pas confondre :
///
/// | Méthode              | Vraie...                                   | Usage typique        |
/// |----------------------|--------------------------------------------|----------------------|
/// | [`Input::is_key_down`]      | tant que la touche est maintenue     | se déplacer          |
/// | [`Input::was_key_pressed`]  | une seule image, au moment de l'appui | sauter, mettre en pause |
/// | [`Input::was_key_released`] | une seule image, au relâchement       | relâcher un tir chargé |
///
/// # Touches physiques
///
/// Les [`KeyCode`] désignent l'**emplacement** d'une touche sur un clavier
/// QWERTY, pas la lettre imprimée dessus. `KeyCode::KeyW` est donc la touche
/// en haut à gauche du pavé de déplacement : `W` en QWERTY, **`Z` en AZERTY**.
/// Résultat : des contrôles « WASD » deviennent automatiquement « ZQSD » sur
/// un clavier français, sans aucune configuration.
#[derive(Debug, Default)]
pub struct Input {
    keys_down: HashSet<KeyCode>,
    keys_pressed: HashSet<KeyCode>,
    keys_released: HashSet<KeyCode>,
    buttons_down: HashSet<MouseButton>,
    buttons_pressed: HashSet<MouseButton>,
    buttons_released: HashSet<MouseButton>,
    /// Déplacement **brut** de la souris depuis l'image précédente.
    mouse_delta: Vec2,
    /// Crans de molette depuis l'image précédente (positif = vers l'avant).
    scroll_delta: f32,
    /// Position du curseur dans la fenêtre, en pixels (`None` s'il est dehors).
    cursor_position: Option<Vec2>,
    /// La fenêtre a-t-elle le focus clavier ?
    focused: bool,
}

impl Input {
    // -------------------------------------------------------------------------
    // Questions posées par l'application
    // -------------------------------------------------------------------------

    /// La touche est-elle enfoncée en ce moment ?
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    /// La touche a-t-elle été enfoncée pendant cette image ?
    pub fn was_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// La touche a-t-elle été relâchée pendant cette image ?
    pub fn was_key_released(&self, key: KeyCode) -> bool {
        self.keys_released.contains(&key)
    }

    /// Le bouton de la souris est-il enfoncé en ce moment ?
    pub fn is_button_down(&self, button: MouseButton) -> bool {
        self.buttons_down.contains(&button)
    }

    /// Le bouton a-t-il été enfoncé pendant cette image ?
    pub fn was_button_pressed(&self, button: MouseButton) -> bool {
        self.buttons_pressed.contains(&button)
    }

    /// Le bouton a-t-il été relâché pendant cette image ?
    pub fn was_button_released(&self, button: MouseButton) -> bool {
        self.buttons_released.contains(&button)
    }

    /// Déplacement de la souris depuis l'image précédente (x vers la droite,
    /// y vers le **bas**, comme sur l'écran).
    ///
    /// Ce sont des valeurs **brutes** du périphérique : elles continuent
    /// d'arriver même quand le curseur est bloqué au bord de l'écran, ce qui
    /// est indispensable pour une caméra à la première personne.
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Crans de molette depuis l'image précédente.
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    /// Position du curseur dans la fenêtre, en pixels depuis le coin haut-gauche.
    pub fn cursor_position(&self) -> Option<Vec2> {
        self.cursor_position
    }

    /// La fenêtre a-t-elle le focus ? Quand elle le perd (Alt+Tab...), toutes
    /// les touches sont considérées comme relâchées.
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    // -------------------------------------------------------------------------
    // Alimentation par le PlatformRunner (`pub(crate)` : invisible de l'extérieur)
    // -------------------------------------------------------------------------

    pub(crate) fn on_key(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            // `insert` renvoie `true` si la touche n'y était pas déjà : on ne
            // compte un « appui » qu'à la transition relâchée → enfoncée.
            if self.keys_down.insert(key) {
                self.keys_pressed.insert(key);
            }
        } else if self.keys_down.remove(&key) {
            self.keys_released.insert(key);
        }
    }

    pub(crate) fn on_button(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            if self.buttons_down.insert(button) {
                self.buttons_pressed.insert(button);
            }
        } else if self.buttons_down.remove(&button) {
            self.buttons_released.insert(button);
        }
    }

    pub(crate) fn on_mouse_motion(&mut self, dx: f32, dy: f32) {
        // On ACCUMULE : plusieurs événements peuvent arriver entre deux images.
        self.mouse_delta += Vec2::new(dx, dy);
    }

    pub(crate) fn on_scroll(&mut self, lines: f32) {
        self.scroll_delta += lines;
    }

    pub(crate) fn on_cursor_moved(&mut self, position: Option<Vec2>) {
        self.cursor_position = position;
    }

    pub(crate) fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            // Sans cela, une touche relâchée pendant que la fenêtre n'a pas le
            // focus resterait « enfoncée » pour toujours (la caméra avancerait seule).
            let keys: Vec<KeyCode> = self.keys_down.iter().copied().collect();
            for key in keys {
                self.on_key(key, false);
            }
            let buttons: Vec<MouseButton> = self.buttons_down.iter().copied().collect();
            for button in buttons {
                self.on_button(button, false);
            }
        }
    }

    /// Fin d'image : on oublie ce qui ne vaut que pour une image (appuis,
    /// relâchements, déplacements). L'état « enfoncé » est conservé.
    pub(crate) fn end_frame(&mut self) {
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.buttons_pressed.clear();
        self.buttons_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
    }
}

/// Capture (ou libère) le curseur de la souris : caché et retenu dans la
/// fenêtre, comme dans un jeu à la première personne.
///
/// Renvoie `true` si le curseur est effectivement capturé.
///
/// Il existe deux façons de retenir le curseur, et chaque système n'en
/// supporte qu'une : `Locked` (figé sur place : macOS, Wayland) ou `Confined`
/// (libre mais enfermé dans la fenêtre : Windows, X11). On essaie l'une puis l'autre.
pub fn set_cursor_captured(window: &Window, captured: bool) -> bool {
    if !captured {
        // Libérer ne peut pas échouer de façon gênante : on ignore le résultat.
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        window.set_cursor_visible(true);
        return false;
    }

    let grabbed = window
        .set_cursor_grab(CursorGrabMode::Locked)
        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
        .is_ok();
    if grabbed {
        window.set_cursor_visible(false);
    } else {
        log::warn!("Impossible de capturer le curseur sur cette plateforme");
    }
    grabbed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appui_dure_une_image_maintien_dure_toujours() {
        let mut input = Input::default();
        input.on_key(KeyCode::KeyW, true);
        assert!(input.is_key_down(KeyCode::KeyW));
        assert!(input.was_key_pressed(KeyCode::KeyW));

        input.end_frame();
        assert!(input.is_key_down(KeyCode::KeyW), "toujours maintenue");
        assert!(
            !input.was_key_pressed(KeyCode::KeyW),
            "l'appui ne dure qu'une image"
        );

        input.on_key(KeyCode::KeyW, false);
        assert!(!input.is_key_down(KeyCode::KeyW));
        assert!(input.was_key_released(KeyCode::KeyW));
    }

    #[test]
    fn la_souris_s_accumule_puis_se_remet_a_zero() {
        let mut input = Input::default();
        input.on_mouse_motion(3.0, -1.0);
        input.on_mouse_motion(2.0, 4.0);
        input.on_scroll(1.0);
        assert_eq!(input.mouse_delta(), Vec2::new(5.0, 3.0));
        assert_eq!(input.scroll_delta(), 1.0);

        input.end_frame();
        assert_eq!(input.mouse_delta(), Vec2::ZERO);
        assert_eq!(input.scroll_delta(), 0.0);
    }

    #[test]
    fn perdre_le_focus_relache_tout() {
        let mut input = Input::default();
        input.on_focus(true);
        input.on_key(KeyCode::ShiftLeft, true);
        input.on_button(MouseButton::Left, true);
        input.end_frame();

        input.on_focus(false);
        assert!(!input.is_key_down(KeyCode::ShiftLeft));
        assert!(!input.is_button_down(MouseButton::Left));
        assert!(input.was_key_released(KeyCode::ShiftLeft));
    }
}
