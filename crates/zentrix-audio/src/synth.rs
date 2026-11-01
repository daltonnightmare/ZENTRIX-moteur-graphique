//! **Synthèse** : des sons fabriqués par calcul, sans fichier.
//!
//! Un son numérique est une suite d'**échantillons** : la position de la
//! membrane du haut-parleur, entre -1 et 1, mesurée `sample_rate` fois par
//! seconde (44 100 pour un CD). Une onde sinusoïdale à 440 Hz donne un « la » ;
//! des valeurs au hasard donnent un souffle (« bruit blanc »).
//!
//! ```text
//!   sinus 440 Hz          bruit                 « toc » (sinus qui s'éteint)
//!   ╭╮  ╭╮  ╭╮            ╷ ╷╷ ╷  ╷╷            ╭╮
//!  ─╯╰──╯╰──╯╰─          ─┼┼┼┼┼┼┼┼┼─           ─╯╰╮╭─╮╭─~~─────
//!       ╰╯  ╰╯            ╵╵ ╵╵╵ ╵                 ╰╯ ╰╯
//! ```
//!
//! Ces sons servent à la démo ; un jeu chargera plutôt des fichiers
//! (`Audio::load_sound`, formats WAV et OGG).

use std::f32::consts::TAU;

/// La fréquence d'échantillonnage des sons synthétisés.
pub const SAMPLE_RATE: u32 = 44_100;

/// Un générateur de nombres pseudo-aléatoires minuscule (« xorshift ») :
/// déterministe, il donne toujours le même bruit, donc le même son.
struct Noise(u32);

impl Noise {
    /// Une valeur entre -1 et 1.
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}

/// Le nombre d'échantillons pour `seconds` secondes.
fn len(seconds: f32) -> usize {
    (seconds * SAMPLE_RATE as f32) as usize
}

/// Un **choc** (« toc ») : un claquement de bruit très bref, puis une note
/// grave (`pitch` Hz) qui s'éteint vite, comme un objet creux qu'on frappe.
pub fn impact(pitch: f32) -> Vec<f32> {
    let mut noise = Noise(0x1234_5678);
    (0..len(0.3))
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            // `exp(-t × k)` : une décroissance exponentielle, comme une
            // vraie résonance. Plus k est grand, plus le son s'éteint vite.
            let body = (TAU * pitch * t).sin() * (-t * 18.0).exp();
            let click = noise.next() * (-t * 90.0).exp();
            0.6 * body + 0.4 * click
        })
        .collect()
}

/// Un **lancer** (« fffiou ») : un souffle qui enfle puis retombe. Le bruit
/// est « adouci » par un filtre passe-bas (moyenne glissante) dont la
/// coupure monte : le souffle devient plus aigu, comme quelque chose qui
/// passe vite.
pub fn whoosh() -> Vec<f32> {
    let mut noise = Noise(0x0bad_cafe);
    let n = len(0.35);
    let mut filtered = 0.0;
    let samples = (0..n)
        .map(|i| {
            let progress = i as f32 / n as f32;
            // Filtre passe-bas d'ordre 1 : on avance d'une fraction `k` vers
            // la nouvelle valeur. Petit k = son sourd ; grand k = son clair.
            let k = 0.02 + 0.25 * progress;
            filtered += k * (noise.next() - filtered);
            // Enveloppe en cloche : 0 au début et à la fin.
            let envelope = (progress * std::f32::consts::PI).sin();
            filtered * envelope
        })
        .collect();
    // Le filtre rend le bruit plus faible, d'un facteur difficile à
    // prévoir : on ramène le pic à 0,8 (« normalisation »).
    normalize(samples, 0.8)
}

/// Multiplie tous les échantillons pour que le plus fort vaille `peak`.
fn normalize(mut samples: Vec<f32>, peak: f32) -> Vec<f32> {
    let max = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if max > 0.0 {
        for s in &mut samples {
            *s *= peak / max;
        }
    }
    samples
}

/// Un **bourdonnement** électrique (néon, machine), fait pour être joué en
/// boucle : une fondamentale à 110 Hz et quelques harmoniques.
///
/// Pour boucler sans « clic », chaque fréquence fait un nombre ENTIER de
/// périodes sur la durée du son : la fin rejoint exactement le début.
pub fn hum() -> Vec<f32> {
    let seconds = 2.0;
    (0..len(seconds))
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            // Un battement lent (0,5 Hz) fait « respirer » le son.
            let wobble = 0.85 + 0.15 * (TAU * 0.5 * t).sin();
            let tone = 0.5 * (TAU * 110.0 * t).sin()
                + 0.25 * (TAU * 220.0 * t).sin()
                + 0.12 * (TAU * 330.0 * t).sin()
                + 0.06 * (TAU * 550.0 * t).sin();
            0.6 * tone * wobble
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn peak(samples: &[f32]) -> f32 {
        samples.iter().fold(0.0, |m, s| m.max(s.abs()))
    }

    #[test]
    fn les_sons_restent_entre_moins_un_et_un() {
        for (nom, son) in [
            ("impact", impact(180.0)),
            ("whoosh", whoosh()),
            ("hum", hum()),
        ] {
            assert!(!son.is_empty(), "{nom}");
            assert!(peak(&son) <= 1.0, "{nom} sature : {}", peak(&son));
            assert!(peak(&son) > 0.2, "{nom} est presque muet");
        }
    }

    #[test]
    fn le_choc_s_eteint() {
        let son = impact(180.0);
        let fin = &son[son.len() - 1000..];
        assert!(peak(fin) < 0.01, "{}", peak(fin));
    }

    #[test]
    fn le_bourdonnement_boucle_sans_saut() {
        let son = hum();
        // Le dernier échantillon est proche du premier : pas de « clic »
        // quand la lecture reprend au début.
        let saut = (son[son.len() - 1] - son[0]).abs();
        assert!(saut < 0.02, "{saut}");
    }
}
