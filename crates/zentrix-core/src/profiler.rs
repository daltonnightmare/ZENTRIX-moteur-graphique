//! Le **profileur d'image** : combien de millisecondes prend chaque étape de
//! l'image (physique, rendu...).
//!
//! « Le jeu rame » ne dit pas *quoi* optimiser. On mesure d'abord, on
//! optimise ensuite ce qui coûte vraiment. À 60 images par seconde, une
//! image dispose de 16,7 ms en tout :
//!
//! ```text
//!   0 ms                                                         16,7 ms
//!   ├─ interface ─┼─ systèmes ─┼── physique ──┼─ rendu ─┼── attente ──┤
//!       lap()          lap()         lap()       lap()     (écran)
//! ```
//!
//! Le principe est celui du **chronomètre à tours** : `begin_frame` démarre
//! le chrono, puis chaque `lap("nom")` note le temps écoulé depuis le tour
//! précédent sous ce nom. Il suffit donc d'appeler `lap` à la fin de chaque
//! étape, dans l'ordre.
//!
//! Pour une analyse plus fine (chaque fonction, chaque thread), le moteur
//! est aussi instrumenté pour **Tracy** : voir la feature `tracy` de
//! zentrix-app.

use std::time::{Duration, Instant};

/// Le poids d'une nouvelle mesure dans la moyenne : 10 %. Une moyenne
/// « glissante » se lit facilement, au lieu de chiffres qui sautent à
/// chaque image.
const SMOOTHING: f32 = 0.1;

/// Une étape mesurée.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub name: &'static str,
    /// La durée à la dernière image, en millisecondes.
    pub last_ms: f32,
    /// La moyenne glissante, en millisecondes.
    pub average_ms: f32,
}

/// Le chronomètre à tours de la boucle de jeu (voir le module).
#[derive(Debug)]
pub struct FrameProfiler {
    /// Le début de l'image en cours, et la fin du dernier tour.
    frame_start: Instant,
    mark: Instant,
    /// Les étapes, dans l'ordre de leur première mesure.
    sections: Vec<Section>,
    /// Ce qui a été mesuré pendant l'image en cours (remis à 0 à chaque
    /// image : une étape peut être mesurée en plusieurs fois).
    current: Vec<Duration>,
    /// La durée totale des dernières images, en millisecondes (la plus
    /// ancienne d'abord).
    history: Vec<f32>,
}

impl Default for FrameProfiler {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            frame_start: now,
            mark: now,
            sections: Vec::new(),
            current: Vec::new(),
            history: Vec::with_capacity(Self::HISTORY_LEN),
        }
    }
}

impl FrameProfiler {
    /// Le nombre d'images gardées dans l'historique (2 s à 60 images/s).
    pub const HISTORY_LEN: usize = 120;

    /// Termine l'image précédente (ses mesures rejoignent les moyennes et
    /// l'historique) et démarre la nouvelle.
    pub fn begin_frame(&mut self) {
        let now = Instant::now();
        // Ce qui s'est passé entre la fin du dernier tour et maintenant
        // (la boucle d'événements de la fenêtre...) est compté aussi : la
        // somme des étapes vaut toujours la durée totale de l'image.
        self.add("autre", now - self.mark);
        let total = now - self.frame_start;
        self.finish_frame(total);
        self.frame_start = now;
        self.mark = now;
    }

    /// Note le temps écoulé depuis le tour précédent sous le nom `name`.
    pub fn lap(&mut self, name: &'static str) {
        let now = Instant::now();
        self.add(name, now - self.mark);
        self.mark = now;
    }

    /// Ajoute une durée mesurée ailleurs (par exemple par le rendu) à une
    /// étape. Ne déplace pas le chrono : à utiliser pour **détailler** un
    /// tour, en retirant soi-même cette durée du tour concerné si besoin
    /// (voir [`FrameProfiler::transfer`]).
    pub fn add(&mut self, name: &'static str, duration: Duration) {
        let index = match self.sections.iter().position(|s| s.name == name) {
            Some(index) => index,
            None => {
                self.sections.push(Section {
                    name,
                    last_ms: 0.0,
                    average_ms: 0.0,
                });
                self.current.push(Duration::ZERO);
                self.sections.len() - 1
            }
        };
        self.current[index] += duration;
    }

    /// Déplace `duration` de l'étape `from` vers l'étape `to` (créée au
    /// besoin). Exemple : le tour « rendu » contient l'attente de l'écran,
    /// mesurée par le rendu lui-même ; on la range à part.
    pub fn transfer(&mut self, from: &'static str, to: &'static str, duration: Duration) {
        if let Some(index) = self.sections.iter().position(|s| s.name == from) {
            let moved = duration.min(self.current[index]);
            self.current[index] -= moved;
            self.add(to, moved);
        }
    }

    /// Les étapes mesurées, dans l'ordre où elles sont apparues.
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// La durée totale des dernières images, en millisecondes, de la plus
    /// ancienne à la plus récente.
    pub fn history(&self) -> &[f32] {
        &self.history
    }

    /// La moyenne glissante de la durée totale d'une image, en millisecondes.
    pub fn average_frame_ms(&self) -> f32 {
        self.sections.iter().map(|s| s.average_ms).sum()
    }

    fn finish_frame(&mut self, total: Duration) {
        for (section, duration) in self.sections.iter_mut().zip(&mut self.current) {
            let ms = duration.as_secs_f32() * 1000.0;
            section.last_ms = ms;
            // La première mesure donne directement la moyenne.
            section.average_ms = if section.average_ms == 0.0 {
                ms
            } else {
                section.average_ms + SMOOTHING * (ms - section.average_ms)
            };
            *duration = Duration::ZERO;
        }
        if self.history.len() == Self::HISTORY_LEN {
            self.history.remove(0);
        }
        self.history.push(total.as_secs_f32() * 1000.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn les_etapes_sont_rangees_dans_l_ordre() {
        let mut profiler = FrameProfiler::default();
        profiler.add("physique", ms(3));
        profiler.add("rendu", ms(5));
        profiler.add("physique", ms(1)); // une deuxième fois dans l'image
        profiler.begin_frame();
        let names: Vec<_> = profiler.sections().iter().map(|s| s.name).collect();
        assert_eq!(names, ["physique", "rendu", "autre"]);
        assert!((profiler.sections()[0].last_ms - 4.0).abs() < 1e-3);
        assert!((profiler.sections()[1].last_ms - 5.0).abs() < 1e-3);
    }

    #[test]
    fn la_moyenne_glisse() {
        let mut profiler = FrameProfiler::default();
        profiler.add("x", ms(10));
        profiler.begin_frame();
        assert!((profiler.sections()[0].average_ms - 10.0).abs() < 1e-3);
        // 10 % du chemin vers la nouvelle valeur : 10 + 0,1 × (20 - 10) = 11.
        profiler.add("x", ms(20));
        profiler.begin_frame();
        assert!((profiler.sections()[0].average_ms - 11.0).abs() < 1e-3);
        assert!((profiler.sections()[0].last_ms - 20.0).abs() < 1e-3);
    }

    #[test]
    fn deplacer_une_duree() {
        let mut profiler = FrameProfiler::default();
        profiler.add("rendu", ms(10));
        profiler.transfer("rendu", "attente", ms(7));
        profiler.begin_frame();
        assert!((profiler.sections()[0].last_ms - 3.0).abs() < 1e-3);
        assert!((profiler.sections()[1].last_ms - 7.0).abs() < 1e-3);
    }

    #[test]
    fn les_tours_couvrent_toute_l_image() {
        let mut profiler = FrameProfiler::default();
        profiler.begin_frame();
        std::thread::sleep(ms(2));
        profiler.lap("a");
        std::thread::sleep(ms(1));
        profiler.begin_frame();
        // La somme des étapes est la durée totale de l'image.
        let sum: f32 = profiler.sections().iter().map(|s| s.last_ms).sum();
        let total = *profiler.history().last().unwrap();
        assert!((sum - total).abs() < 0.01, "{sum} ≠ {total}");
        let a = profiler.sections().iter().find(|s| s.name == "a").unwrap();
        assert!(a.last_ms >= 2.0);
    }

    #[test]
    fn l_historique_est_borne() {
        let mut profiler = FrameProfiler::default();
        for _ in 0..FrameProfiler::HISTORY_LEN + 10 {
            profiler.begin_frame();
        }
        assert_eq!(profiler.history().len(), FrameProfiler::HISTORY_LEN);
    }
}
