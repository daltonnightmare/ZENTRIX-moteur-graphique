//! # zentrix-audio
//!
//! Le **son** : des bruitages placés dans la scène 3D. Le mixage et la
//! sortie vers la carte son sont confiés à [kira](https://docs.rs/kira),
//! une bibliothèque audio pour les jeux ; cette crate la relie à la scène.
//!
//! ## Le son « spatialisé »
//!
//! ```text
//!        émetteur (néon)                    Un son placé dans la scène est
//!            ♪                              plus FORT quand l'auditeur est
//!             ╲  distance                   proche, et sort du côté où il se
//!              ╲                            trouve (gauche / droite).
//!   (caméra) 👂 ──▶ regarde vers -Z
//!   AudioListener
//! ```
//!
//! - l'**auditeur** ([`AudioListener`]) : en général la caméra ;
//! - les **émetteurs** ([`SoundEmitter`]) : des entités qui jouent un son en
//!   boucle, qui les suit ;
//! - les sons **ponctuels** ([`Audio::play_at`]) : un choc, une explosion...
//!
//! ## Utilisation
//!
//! ```text
//!  au démarrage :  let toc = audio.add_sound("toc", &synth::impact(180.0));
//!  chaque image :  audio.update(&scene);                // auditeur et émetteurs
//!  à un choc :     audio.play_at(toc, position, 0.8, 1.0);
//! ```
//!
//! ## Sans carte son
//!
//! Si aucun périphérique audio n'est disponible (serveur, pilote absent),
//! [`Audio::new`] ne fait pas échouer le programme : l'audio est simplement
//! **muet**, et toutes les fonctions deviennent sans effet.
//!
//! Le son est joué par un **thread** de kira, en parallèle du jeu : les
//! fonctions ci-dessous ne font qu'envoyer des commandes, elles ne
//! bloquent jamais.

mod components;
pub mod synth;

use std::collections::HashMap;
use std::sync::Arc;

use kira::backend::Backend;
use kira::listener::ListenerHandle;
use kira::sound::static_sound::{StaticSoundData, StaticSoundSettings};
use kira::track::{SpatialTrackBuilder, SpatialTrackHandle};
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Frame, Tween};
use zentrix_core::Transform;
use zentrix_core::math::{Quat, Vec3};
use zentrix_scene::{Entity, GlobalTransform, Scene};

pub use components::{AudioListener, SoundEmitter};

/// Un son chargé (indice dans la liste des sons de [`Audio`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundHandle(usize);

/// Les erreurs de l'audio.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("impossible de lire le son « {path} » : {source}")]
    Load {
        path: std::path::PathBuf,
        source: kira::sound::FromFileError,
    },
}

/// En dessous de cette distance, un son est entendu à plein volume ; au-delà
/// de la distance maximale, il est inaudible.
const HEARING_DISTANCES: (f32, f32) = (1.5, 40.0);

/// Volume (0 = muet, 1 = normal) → décibels, l'unité de kira.
///
/// L'oreille perçoit les volumes de façon **logarithmique** : diviser
/// l'amplitude par 2 retire environ 6 dB. En dessous de 0,1 % (-60 dB), on
/// considère que c'est le silence.
pub fn to_decibels(volume: f32) -> Decibels {
    if volume <= 0.001 {
        Decibels::SILENCE
    } else {
        Decibels(20.0 * volume.log10())
    }
}

/// Un émetteur en cours de lecture : sa piste spatiale (qui porte la position).
struct Emitter {
    track: SpatialTrackHandle,
    sound: String,
    volume: f32,
}

/// Le système audio. `B` : la sortie son (la carte son par défaut ; un
/// backend « factice » dans les tests).
pub struct Audio<B: Backend = DefaultBackend> {
    /// `None` : audio muet (pas de carte son).
    manager: Option<AudioManager<B>>,
    listener: Option<ListenerHandle>,
    sounds: Vec<StaticSoundData>,
    names: HashMap<String, SoundHandle>,
    emitters: HashMap<Entity, Emitter>,
    master_volume: f32,
}

impl Audio<DefaultBackend> {
    /// Ouvre la sortie son par défaut. En cas d'échec, l'audio est muet
    /// (avec un avertissement dans les logs) : le jeu continue sans son.
    pub fn new() -> Self {
        match Self::with_backend(Default::default()) {
            Ok(audio) => {
                log::info!("Audio initialisé (kira)");
                audio
            }
            Err(error) => {
                log::warn!("Pas de sortie audio ({error}) : le jeu sera muet");
                Self::muted()
            }
        }
    }
}

impl Default for Audio<DefaultBackend> {
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> Audio<B> {
    /// Un audio qui ne joue rien (aucune sortie son).
    pub fn muted() -> Self {
        Self {
            manager: None,
            listener: None,
            sounds: Vec::new(),
            names: HashMap::new(),
            emitters: HashMap::new(),
            master_volume: 1.0,
        }
    }

    /// Ouvre une sortie son précise (`B::Settings` : ses réglages).
    pub fn with_backend(settings: B::Settings) -> Result<Self, B::Error> {
        let mut manager = AudioManager::<B>::new(AudioManagerSettings {
            backend_settings: settings,
            capacities: Default::default(),
            main_track_builder: Default::default(),
            // La valeur par défaut de kira : les réglages internes sont
            // mis à jour tous les 128 échantillons (~3 ms).
            internal_buffer_size: 128,
        })?;
        // L'auditeur existe dès le départ (à l'origine) : `update` le place
        // ensuite sur l'entité `AudioListener`.
        let listener = manager.add_listener(Vec3::ZERO, Quat::IDENTITY).ok();
        Ok(Self {
            manager: Some(manager),
            listener,
            ..Self::muted()
        })
    }

    /// `true` si aucune sortie son n'est ouverte.
    pub fn is_muted(&self) -> bool {
        self.manager.is_none()
    }

    /// Ajoute un son à partir de ses échantillons (mono, entre -1 et 1, à
    /// `synth::SAMPLE_RATE`), sous le nom `name`.
    pub fn add_sound(&mut self, name: &str, samples: &[f32]) -> SoundHandle {
        let frames: Arc<[Frame]> = samples.iter().map(|&s| Frame::from_mono(s)).collect();
        self.insert(
            name,
            StaticSoundData {
                sample_rate: synth::SAMPLE_RATE,
                frames,
                settings: StaticSoundSettings::default(),
                slice: None,
            },
        )
    }

    /// Charge un son depuis un fichier (WAV ou OGG), sous le nom `name`.
    pub fn load_sound(
        &mut self,
        name: &str,
        path: impl AsRef<std::path::Path>,
    ) -> Result<SoundHandle, AudioError> {
        let path = path.as_ref();
        let data = StaticSoundData::from_file(path).map_err(|source| AudioError::Load {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(self.insert(name, data))
    }

    fn insert(&mut self, name: &str, data: StaticSoundData) -> SoundHandle {
        let handle = SoundHandle(self.sounds.len());
        self.sounds.push(data);
        self.names.insert(name.to_owned(), handle);
        handle
    }

    /// Le son nommé `name`, s'il existe.
    pub fn sound(&self, name: &str) -> Option<SoundHandle> {
        self.names.get(name).copied()
    }

    /// Le volume général (0 = muet, 1 = normal).
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        if volume == self.master_volume {
            return;
        }
        self.master_volume = volume;
        if let Some(manager) = &mut self.manager {
            manager
                .main_track()
                .set_volume(to_decibels(volume), Tween::default());
        }
    }

    /// Les données du son `sound`, réglées pour cette lecture.
    fn prepared(&self, sound: SoundHandle, volume: f32, rate: f32) -> Option<StaticSoundData> {
        let data = self.sounds.get(sound.0)?;
        Some(data.volume(to_decibels(volume)).playback_rate(rate as f64))
    }

    /// Joue un son « partout » (interface, musique), sans position.
    /// `rate` : vitesse de lecture (2 = deux fois plus vite et une octave
    /// plus aigu).
    pub fn play(&mut self, sound: SoundHandle, volume: f32, rate: f32) {
        let Some(data) = self.prepared(sound, volume, rate) else {
            return;
        };
        if let Some(manager) = &mut self.manager
            && let Err(error) = manager.play(data)
        {
            log::debug!("Son non joué : {error}");
        }
    }

    /// Joue un son une fois, à la position `position` de la scène.
    ///
    /// Chaque son ponctuel reçoit sa propre **piste spatiale** (kira calcule
    /// le volume et la balance gauche/droite selon l'auditeur), qui
    /// disparaît d'elle-même à la fin du son.
    pub fn play_at(&mut self, sound: SoundHandle, position: Vec3, volume: f32, rate: f32) {
        let Some(data) = self.prepared(sound, volume, rate) else {
            return;
        };
        let (Some(manager), Some(listener)) = (&mut self.manager, &self.listener) else {
            return;
        };
        let builder = SpatialTrackBuilder::new()
            .distances(HEARING_DISTANCES)
            .persist_until_sounds_finish(true);
        // Trop de sons à la fois (limite de kira) : on ignore celui-ci.
        match manager.add_spatial_sub_track(listener, position, builder) {
            Ok(mut track) => {
                if let Err(error) = track.play(data) {
                    log::debug!("Son non joué : {error}");
                }
            }
            Err(error) => log::debug!("Son non joué : {error}"),
        }
    }

    /// Nombre d'émetteurs en cours de lecture.
    pub fn emitter_count(&self) -> usize {
        self.emitters.len()
    }

    /// À appeler à chaque image, après `Scene::update_transforms` : place
    /// l'auditeur, démarre, déplace et arrête les émetteurs.
    pub fn update(&mut self, scene: &Scene) {
        profiling::scope!("Audio::update");
        self.update_listener(scene);
        self.update_emitters(scene);
    }

    fn update_listener(&mut self, scene: &Scene) {
        let Some(listener) = &mut self.listener else {
            return;
        };
        let mut query = scene
            .world
            .query::<(&Transform, Option<&GlobalTransform>, &AudioListener)>();
        if let Some((transform, global, _)) = query.iter().next() {
            let (position, rotation) = world_pose(transform, global);
            listener.set_position(position, Tween::default());
            listener.set_orientation(rotation, Tween::default());
        }
    }

    fn update_emitters(&mut self, scene: &Scene) {
        let mut seen = Vec::new();
        for (entity, transform, global, emitter) in scene
            .world
            .query::<(Entity, &Transform, Option<&GlobalTransform>, &SoundEmitter)>()
            .iter()
        {
            seen.push(entity);
            let (position, _) = world_pose(transform, global);
            // Un émetteur dont le son a changé est recréé.
            if let Some(current) = self.emitters.get_mut(&entity)
                && current.sound == emitter.sound
            {
                current.track.set_position(position, Tween::default());
                if current.volume != emitter.volume {
                    current
                        .track
                        .set_volume(to_decibels(emitter.volume), Tween::default());
                    current.volume = emitter.volume;
                }
                continue;
            }
            self.emitters.remove(&entity);
            if let Some(track) = self.start_emitter(emitter, position) {
                self.emitters.insert(
                    entity,
                    Emitter {
                        track,
                        sound: emitter.sound.clone(),
                        volume: emitter.volume,
                    },
                );
            }
        }
        // Une entité disparue (ou sans `SoundEmitter`) : la piste est
        // supprimée avec sa poignée, et le son s'arrête.
        self.emitters.retain(|entity, _| seen.contains(entity));
    }

    /// Démarre la lecture en boucle d'un émetteur. `None` si l'audio est
    /// muet ou si le son est inconnu.
    fn start_emitter(
        &mut self,
        emitter: &SoundEmitter,
        position: Vec3,
    ) -> Option<SpatialTrackHandle> {
        let Some(sound) = self.sound(&emitter.sound) else {
            log::warn!("Son inconnu pour un émetteur : « {} »", emitter.sound);
            return None;
        };
        // `..` : boucler sur tout le son.
        let data = self.prepared(sound, 1.0, 1.0)?.loop_region(..);
        let (manager, listener) = (self.manager.as_mut()?, self.listener.as_ref()?);
        let builder = SpatialTrackBuilder::new()
            .distances(HEARING_DISTANCES)
            .volume(to_decibels(emitter.volume));
        let mut track = manager
            .add_spatial_sub_track(listener, position, builder)
            .inspect_err(|error| log::debug!("Émetteur non créé : {error}"))
            .ok()?;
        track
            .play(data)
            .inspect_err(|error| log::debug!("Émetteur non joué : {error}"))
            .ok()?;
        Some(track)
    }
}

/// Position et orientation dans le monde : le `GlobalTransform` s'il a été
/// calculé (entité enfant), sinon le `Transform` local.
fn world_pose(transform: &Transform, global: Option<&GlobalTransform>) -> (Vec3, Quat) {
    match global {
        Some(global) => {
            let (_, rotation, translation) = global.0.to_scale_rotation_translation();
            (translation, rotation)
        }
        None => (transform.translation, transform.rotation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kira::backend::mock::{MockBackend, MockBackendSettings};

    /// Un audio branché sur une sortie « factice » : tout fonctionne, mais
    /// aucun son ne sort (parfait pour les tests).
    fn audio() -> Audio<MockBackend> {
        Audio::with_backend(MockBackendSettings::default()).expect("backend factice")
    }

    #[test]
    fn volumes_en_decibels() {
        assert_eq!(to_decibels(1.0), Decibels(0.0));
        // Moitié de l'amplitude : environ -6 dB.
        assert!((to_decibels(0.5).0 + 6.02).abs() < 0.01);
        assert_eq!(to_decibels(0.0), Decibels::SILENCE);
    }

    #[test]
    fn sons_nommes() {
        let mut audio = audio();
        assert!(!audio.is_muted());
        let toc = audio.add_sound("toc", &synth::impact(200.0));
        let souffle = audio.add_sound("souffle", &synth::whoosh());
        assert_ne!(toc, souffle);
        assert_eq!(audio.sound("toc"), Some(toc));
        assert_eq!(audio.sound("inconnu"), None);
        // Jouer ne plante pas (et ne fait rien d'audible ici).
        audio.play_at(toc, Vec3::new(1.0, 0.0, -2.0), 0.8, 1.2);
        audio.play(souffle, 0.5, 1.0);
    }

    #[test]
    fn les_emetteurs_suivent_les_entites() {
        let mut audio = audio();
        audio.add_sound("bourdon", &synth::hum());
        let mut scene = Scene::default();
        scene
            .world
            .spawn((Transform::from_translation(Vec3::Z * 5.0), AudioListener));
        let neon = scene.world.spawn((
            Transform::from_translation(Vec3::X * 3.0),
            SoundEmitter::new("bourdon", 0.5),
        ));
        // Un son inconnu : ignoré, avec un avertissement.
        scene
            .world
            .spawn((Transform::IDENTITY, SoundEmitter::new("absent", 1.0)));

        audio.update(&scene);
        assert_eq!(audio.emitter_count(), 1);
        audio.update(&scene);
        assert_eq!(audio.emitter_count(), 1, "pas de doublon");

        scene.world.despawn(neon).unwrap();
        audio.update(&scene);
        assert_eq!(audio.emitter_count(), 0);
    }

    #[test]
    fn un_audio_muet_ne_fait_rien() {
        let mut audio = Audio::<MockBackend>::muted();
        assert!(audio.is_muted());
        let toc = audio.add_sound("toc", &synth::impact(200.0));
        audio.play_at(toc, Vec3::ZERO, 1.0, 1.0);
        audio.set_master_volume(0.3);
        assert_eq!(audio.master_volume(), 0.3);
        let mut scene = Scene::default();
        scene
            .world
            .spawn((Transform::IDENTITY, SoundEmitter::new("toc", 1.0)));
        audio.update(&scene);
        assert_eq!(audio.emitter_count(), 0);
    }
}
