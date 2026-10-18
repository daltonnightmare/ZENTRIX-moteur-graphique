//! Mesure du temps entre deux images (« frames »).

// `use` importe des éléments de la bibliothèque standard (`std`).
use std::time::{Duration, Instant};

/// Chronomètre de la boucle principale.
///
/// À chaque image, le moteur appelle [`FrameTimer::tick`] qui calcule le
/// **delta time** : le temps écoulé depuis l'image précédente.
///
/// # Pourquoi le delta time est crucial
///
/// Si on déplace un objet de `1.0` par image, il ira deux fois plus vite sur un
/// écran 120 Hz que sur un écran 60 Hz. En multipliant par le delta time
/// (`vitesse * dt`), le mouvement devient identique quelle que soit la machine.
#[derive(Debug)]
pub struct FrameTimer {
    // Champs PRIVÉS (pas de `pub`) : l'extérieur ne peut pas les modifier
    // n'importe comment, il doit passer par nos méthodes. C'est l'encapsulation.
    start: Instant,
    last_tick: Instant,
    delta: Duration,
    frame_count: u64,

    // Pour calculer les FPS (images par seconde) sur une fenêtre d'une seconde.
    fps_window_start: Instant,
    fps_window_frames: u32,
    fps: f32,
}

impl FrameTimer {
    /// Crée un chronomètre qui démarre maintenant.
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_tick: now,
            delta: Duration::ZERO,
            frame_count: 0,
            fps_window_start: now,
            fps_window_frames: 0,
            fps: 0.0,
        }
    }

    /// À appeler **une fois par image**. Met à jour le delta time et les FPS.
    ///
    /// Renvoie `true` quand la valeur des FPS vient d'être recalculée
    /// (environ une fois par seconde), pratique pour l'afficher sans spammer.
    ///
    /// `&mut self` : la méthode emprunte le chronomètre **en écriture** car elle
    /// modifie ses champs. (Avec `&self` elle pourrait seulement les lire.)
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        self.delta = now - self.last_tick;
        self.last_tick = now;
        self.frame_count += 1;

        self.fps_window_frames += 1;
        let window = now - self.fps_window_start;
        if window >= Duration::from_secs(1) {
            // `as f32` convertit explicitement un entier en flottant :
            // Rust ne fait JAMAIS de conversion numérique implicite.
            self.fps = self.fps_window_frames as f32 / window.as_secs_f32();
            self.fps_window_frames = 0;
            self.fps_window_start = now;
            return true;
        }
        false
    }

    /// Temps écoulé depuis l'image précédente, en secondes.
    pub fn delta_seconds(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    /// Temps total écoulé depuis la création du chronomètre, en secondes.
    pub fn elapsed_seconds(&self) -> f32 {
        self.start.elapsed().as_secs_f32()
    }

    /// Nombre d'images comptées depuis le début.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Images par seconde, mesurées sur la dernière seconde écoulée.
    pub fn fps(&self) -> f32 {
        self.fps
    }
}

// Clippy (l'outil de conseils de Rust) recommande d'implémenter `Default`
// quand un type possède un `new()` sans argument.
impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_incremente_le_compteur() {
        let mut timer = FrameTimer::new();
        timer.tick();
        timer.tick();
        assert_eq!(timer.frame_count(), 2);
    }

    #[test]
    fn delta_est_positif() {
        let mut timer = FrameTimer::new();
        std::thread::sleep(Duration::from_millis(5));
        timer.tick();
        assert!(timer.delta_seconds() > 0.0);
    }
}
