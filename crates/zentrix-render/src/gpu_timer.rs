//! Les **mesures GPU** : combien de temps la carte graphique passe sur
//! chaque étape de l'image.
//!
//! Le profileur d'image (`FrameProfiler`, dans zentrix-core) mesure le
//! **CPU** : le temps passé à *préparer* et à *envoyer* les commandes. Mais
//! le GPU les exécute plus tard, à son rythme : une passe d'ombre très
//! coûteuse pour le GPU ne coûte presque rien au CPU. Pour savoir où part
//! le temps de la carte graphique, il faut le lui demander.
//!
//! ## Le principe : des horodatages (*timestamp queries*)
//!
//! On demande au GPU d'écrire l'heure de son horloge à des endroits
//! précis de l'image, entre les groupes de passes :
//!
//! ```text
//!   repère 0 │ ombres │ 1 │ passe principale │ 2 │ post-traitement │ 3 │ interface │ 4
//! ```
//!
//! La durée de chaque étape est l'écart entre deux repères. Un repère est
//! une passe de calcul **vide**, qui n'écrit qu'un horodatage à son début :
//! les passes existantes n'ont pas à changer.
//!
//! ## Sans jamais attendre le GPU
//!
//! Les horodatages sont écrits dans un `QuerySet`, recopiés dans un buffer
//! lisible par le CPU, puis lus... quand le GPU a fini, une ou deux images
//! plus tard. Attendre le GPU à chaque image figerait le CPU et fausserait
//! justement ce qu'on mesure. On utilise donc un **anneau** de buffers : à
//! chaque image, on relit ceux qui sont prêts et on en prend un libre.
//!
//! L'option `TIMESTAMP_QUERY` n'existe pas sur tous les GPU (ni dans tous
//! les navigateurs) : sans elle, [`GpuTimer::new`] renvoie `None` et le
//! moteur fonctionne sans mesures GPU.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use zentrix_rhi::wgpu;

/// Les étapes mesurées, dans l'ordre de l'image.
pub const GPU_SECTIONS: [&str; 4] = ["ombres", "passe principale", "post-traitement", "interface"];
/// Un repère avant la première étape, puis un après chacune.
const MARKS: u32 = GPU_SECTIONS.len() as u32 + 1;
/// La taille des horodatages d'une image (8 octets chacun).
const MARKS_SIZE: u64 = MARKS as u64 * wgpu::QUERY_SIZE as u64;
/// Combien d'images peuvent attendre leur lecture en même temps. Au-delà,
/// une image n'est simplement pas mesurée.
const READBACK_COUNT: usize = 3;
/// Poids de la dernière mesure dans la moyenne glissante.
const SMOOTHING: f32 = 0.1;

/// L'état d'un buffer de lecture, partagé avec la fonction de rappel de
/// `map_async` (appelée par wgpu quand le buffer est lisible).
const FREE: u8 = 0;
const PENDING: u8 = 1;
const READY: u8 = 2;
const FAILED: u8 = 3;

/// Le temps GPU d'une étape, en millisecondes.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GpuSection {
    pub name: &'static str,
    /// La dernière image mesurée.
    pub last_ms: f32,
    /// La moyenne glissante, plus lisible.
    pub average_ms: f32,
}

/// Les dernières mesures GPU : chaque étape, et leur total.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GpuTimings {
    pub sections: [GpuSection; GPU_SECTIONS.len()],
    /// Du premier au dernier repère.
    pub total: GpuSection,
    /// Au moins une image a-t-elle été mesurée ?
    pub measured: bool,
}

struct Readback {
    buffer: wgpu::Buffer,
    state: Arc<AtomicU8>,
}

/// Mesure le temps GPU des étapes de l'image (voir le module).
pub struct GpuTimer {
    queries: wgpu::QuerySet,
    /// Où le GPU « résout » les horodatages (usage `QUERY_RESOLVE`) ; recopié
    /// ensuite dans un buffer de lecture (usage `MAP_READ`).
    resolve: wgpu::Buffer,
    readbacks: Vec<Readback>,
    /// Le buffer de lecture de l'image en cours, s'il y en avait un libre.
    current: Option<usize>,
    /// Nanosecondes par graduation de l'horloge du GPU.
    period_ns: f32,
    timings: GpuTimings,
}

impl GpuTimer {
    /// `None` si le GPU ne propose pas les horodatages (option
    /// `TIMESTAMP_QUERY`, demandée par zentrix-rhi quand elle existe).
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Self> {
        if !device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            return None;
        }
        let queries = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("GPU Timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: MARKS,
        });
        let resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GPU Timestamps Resolve"),
            size: MARKS_SIZE,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readbacks = (0..READBACK_COUNT)
            .map(|_| Readback {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("GPU Timestamps Readback"),
                    size: MARKS_SIZE,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                state: Arc::new(AtomicU8::new(FREE)),
            })
            .collect();
        let mut timings = GpuTimings::default();
        for (section, name) in timings.sections.iter_mut().zip(GPU_SECTIONS) {
            section.name = name;
        }
        timings.total.name = "total";
        Some(Self {
            queries,
            resolve,
            readbacks,
            current: None,
            period_ns: queue.get_timestamp_period(),
            timings,
        })
    }

    /// Les dernières mesures.
    pub fn timings(&self) -> &GpuTimings {
        &self.timings
    }

    /// Début d'image : relit les mesures arrivées, et choisit un buffer de
    /// lecture libre pour cette image. Renvoie `false` si aucun n'est libre
    /// (le GPU a du retard) : cette image ne sera pas mesurée.
    pub fn begin_frame(&mut self, device: &wgpu::Device) -> bool {
        // Sans bloquer : laisse wgpu appeler les rappels des lectures finies.
        let _ = device.poll(wgpu::PollType::Poll);
        for i in 0..self.readbacks.len() {
            match self.readbacks[i].state.load(Ordering::Acquire) {
                READY => {
                    // `None` si la lecture échoue : cette image est perdue,
                    // sans plus de conséquence.
                    let ticks: Option<Vec<u64>> = self.readbacks[i]
                        .buffer
                        .slice(..)
                        .get_mapped_range()
                        .ok()
                        .map(|data| bytemuck::cast_slice(&data).to_vec());
                    self.readbacks[i].buffer.unmap();
                    self.readbacks[i].state.store(FREE, Ordering::Release);
                    if let Some(ticks) = ticks {
                        self.record(&durations_ms(&ticks, self.period_ns));
                    }
                }
                // Une lecture refusée : le buffer redevient simplement libre.
                FAILED => self.readbacks[i].state.store(FREE, Ordering::Release),
                _ => {}
            }
        }
        self.current = self
            .readbacks
            .iter()
            .position(|r| r.state.load(Ordering::Acquire) == FREE);
        self.current.is_some()
    }

    /// Pose le repère `index` (0 à 4) : une passe de calcul vide, dont le
    /// GPU note l'heure du début. Sans effet si l'image n'est pas mesurée.
    pub fn mark(&self, encoder: &mut wgpu::CommandEncoder, index: u32) {
        if self.current.is_none() {
            return;
        }
        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("GPU Timestamp"),
            timestamp_writes: Some(wgpu::ComputePassTimestampWrites {
                query_set: &self.queries,
                beginning_of_pass_write_index: Some(index),
                end_of_pass_write_index: None,
            }),
        });
    }

    /// Fin de l'enregistrement : recopie les horodatages vers le buffer de
    /// lecture de l'image (à appeler juste avant `encoder.finish()`).
    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        let Some(i) = self.current else { return };
        encoder.resolve_query_set(&self.queries, 0..MARKS, &self.resolve, 0);
        encoder.copy_buffer_to_buffer(&self.resolve, 0, &self.readbacks[i].buffer, 0, MARKS_SIZE);
    }

    /// Après l'envoi au GPU : demande la lecture du buffer de l'image. wgpu
    /// la fera quand le GPU aura fini, et préviendra par la fonction de rappel.
    pub fn after_submit(&mut self) {
        let Some(i) = self.current.take() else { return };
        let readback = &self.readbacks[i];
        readback.state.store(PENDING, Ordering::Release);
        let state = Arc::clone(&readback.state);
        readback
            .buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                state.store(
                    if result.is_ok() { READY } else { FAILED },
                    Ordering::Release,
                );
            });
    }

    /// Ajoute une image mesurée aux moyennes.
    fn record(&mut self, durations: &[f32; GPU_SECTIONS.len()]) {
        let first = !self.timings.measured;
        let update = |section: &mut GpuSection, ms: f32| {
            section.last_ms = ms;
            section.average_ms = if first {
                ms
            } else {
                section.average_ms + (ms - section.average_ms) * SMOOTHING
            };
        };
        for (section, &ms) in self.timings.sections.iter_mut().zip(durations) {
            update(section, ms);
        }
        update(&mut self.timings.total, durations.iter().sum());
        self.timings.measured = true;
    }
}

/// Les durées des étapes, en millisecondes, à partir des horodatages des
/// repères (en graduations de l'horloge du GPU).
///
/// Un écart négatif (horloge qui repart à zéro, repère écrit dans le
/// désordre par certains pilotes) compte pour zéro plutôt que pour une
/// durée absurde.
fn durations_ms(ticks: &[u64], period_ns: f32) -> [f32; GPU_SECTIONS.len()] {
    std::array::from_fn(|i| match (ticks.get(i), ticks.get(i + 1)) {
        (Some(&start), Some(&end)) if end >= start => {
            (end - start) as f64 as f32 * period_ns / 1_000_000.0
        }
        _ => 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duree_de_chaque_etape() {
        // Une graduation = 10 ns (horloge à 100 MHz).
        let ticks = [1_000, 201_000, 501_000, 601_000, 621_000];
        let d = durations_ms(&ticks, 10.0);
        // 200 000 graduations × 10 ns = 2 ms, etc.
        assert_eq!(d, [2.0, 3.0, 1.0, 0.2]);
    }

    #[test]
    fn un_ecart_negatif_compte_pour_zero() {
        let ticks = [500, 400, 600, 600, 700];
        let d = durations_ms(&ticks, 1_000_000.0);
        assert_eq!(d[0], 0.0);
        assert_eq!(d[1], 200.0);
        assert_eq!(d[2], 0.0, "deux repères au même instant : une étape vide");
        // Des horodatages manquants : zéro.
        assert_eq!(durations_ms(&[10], 1.0), [0.0; 4]);
    }

    #[test]
    fn la_taille_respecte_les_regles_de_wgpu() {
        // `copy_buffer_to_buffer` exige une taille multiple de 4.
        assert_eq!(MARKS_SIZE % wgpu::COPY_BUFFER_ALIGNMENT, 0);
        const { assert!(MARKS <= wgpu::QUERY_SET_MAX_QUERIES) };
    }
}
