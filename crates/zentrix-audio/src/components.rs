//! Les composants de l'audio.

/// « Cette entité écoute » : les sons sont entendus depuis sa position et
/// selon son orientation (un son à sa droite sort à droite). En général,
/// la caméra. S'il y en a plusieurs, seule la première compte.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AudioListener;

/// « Cette entité émet un son en continu » (un moteur, un néon, une
/// cascade). Le son, joué en boucle, suit l'entité : on l'entend plus fort
/// en s'approchant, et du côté où elle se trouve.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SoundEmitter {
    /// Le nom du son (voir `Audio::add_sound`). Un nom plutôt qu'un handle :
    /// le composant se sauvegarde avec la scène.
    pub sound: String,
    /// Volume, de 0 (muet) à 1.
    pub volume: f32,
}

impl SoundEmitter {
    pub fn new(sound: impl Into<String>, volume: f32) -> Self {
        Self {
            sound: sound.into(),
            volume,
        }
    }
}
