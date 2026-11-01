//! Le **tri des objets** avant de dessiner : qui est vu par la caméra, qui
//! projette une ombre, et dans quel ordre les envoyer au GPU.
//!
//! ```text
//!  DrawCommand ─┬─ boîte dans le monde ─┬─ vue par la caméra ? ─┬─ opaque ou découpe ──▶ liste opaque
//!   (toutes)    │  (Aabb::transformed)  │                       └─ mélange (verre)   ──▶ liste transparente
//!               │                       └─ dans la zone d'ombre ? (sauf verre)       ──▶ liste des ombres
//!               └─ ni l'un ni l'autre : écartée, le GPU ne la verra pas
//! ```
//!
//! Les deux tests sont indépendants : un arbre derrière la caméra est
//! invisible, mais son ombre peut s'allonger jusque devant elle. Il est donc
//! dessiné dans la shadow map et pas à l'écran.
//!
//! # Deux ordres différents
//!
//! - Les objets **opaques** sont triés par **lot** (pipeline, textures,
//!   maillage) : changer d'état coûte cher, et le depth buffer rend l'ordre
//!   sans importance pour l'image.
//! - Les objets **transparents** sont triés **du plus loin au plus proche**
//!   de la caméra (« algorithme du peintre ») : chacun se mélange avec ce
//!   qui est déjà dessiné derrière lui. Dans l'autre ordre, une vitre
//!   proche serait dessinée d'abord, et celle de derrière, dessinée ensuite
//!   *par-dessus*, apparaîtrait devant.
//!
//! # Plusieurs threads
//!
//! Chaque commande se traite sans regarder les autres : c'est un travail
//! idéal à **partager entre les cœurs** du processeur. Avec la bibliothèque
//! `rayon`, on découpe la liste en paquets (`par_chunks`), chaque cœur
//! traite les siens, et les résultats sont recollés **dans l'ordre** : le
//! résultat est exactement celui d'un seul thread. En dessous de
//! [`PARALLEL_THRESHOLD`] commandes, répartir le travail coûterait plus cher
//! que le travail lui-même : on reste sur un seul thread.

use rayon::prelude::*;
use zentrix_core::math::Vec3;
use zentrix_core::{Aabb, AlphaMode, Frustum};

use crate::pipeline::MeshVariant;
use crate::renderer::{BatchKey, DrawCommand, NO_TEXTURES, resolve};
use crate::vertex::InstanceRaw;

/// Une instance prête à envoyer, avec ce qui décide de son lot.
pub(crate) type Item = (BatchKey, InstanceRaw);

/// Le numéro du pipeline d'ombre des objets à découpe (0 : les autres).
pub(crate) const SHADOW_MASKED: u8 = 1;

/// À partir de combien de commandes le travail est réparti entre les cœurs.
pub(crate) const PARALLEL_THRESHOLD: usize = 1024;

/// La taille des paquets confiés à chaque cœur. Assez grands pour que le
/// coût du partage reste négligeable, assez petits pour que tous les cœurs
/// aient du travail.
const CHUNK_SIZE: usize = 256;

/// Les pyramides de vision utilisées pour écarter les objets.
pub(crate) struct Culling {
    /// Celle de la caméra : pour la passe principale.
    pub camera: Frustum,
    /// La boîte de la lumière (projection orthographique) : pour les ombres.
    pub light: Frustum,
}

/// Ce que doit faire le tri.
pub(crate) struct Options<'a> {
    /// La boîte englobante de chaque maillage, dans son repère (indice =
    /// numéro du `MeshHandle`).
    pub bounds: &'a [Aabb],
    /// `None` : pas de culling, tout est dessiné.
    pub culling: Option<Culling>,
    /// Faut-il remplir la liste de la passe d'ombre ?
    pub shadows: bool,
    /// Le travail peut-il être réparti entre plusieurs threads ?
    pub parallel: bool,
    /// La position de l'œil, pour trier les objets transparents.
    pub camera_position: Vec3,
}

/// Les trois listes produites par le tri.
#[derive(Default)]
pub(crate) struct DrawLists {
    /// Opaques et découpes, triés par lot.
    pub opaque: Vec<Item>,
    /// Transparents (mélange), du plus loin au plus proche.
    pub transparent: Vec<Item>,
    /// Ceux qui projettent une ombre, triés par lot.
    pub shadow: Vec<Item>,
}

impl DrawLists {
    /// Vide les listes en gardant leur mémoire.
    fn clear(&mut self) {
        self.opaque.clear();
        self.transparent.clear();
        self.shadow.clear();
    }

    fn append(&mut self, mut other: DrawLists) {
        self.opaque.append(&mut other.opaque);
        self.transparent.append(&mut other.transparent);
        self.shadow.append(&mut other.shadow);
    }
}

/// Remplit les trois listes (vidées d'abord, mais qui gardent leur mémoire).
pub(crate) fn sort_draws(draws: &[DrawCommand], options: &Options, lists: &mut DrawLists) {
    lists.clear();
    // La distance (au carré : inutile de calculer la racine pour comparer)
    // entre l'œil et l'origine de l'objet.
    let eye = options.camera_position;
    let distance = |item: &Item| {
        let m = &item.1.model[3];
        eye.distance_squared(Vec3::new(m[0], m[1], m[2]))
    };
    // `b` avant `a` : du plus LOIN au plus proche.
    let far_first = |a: &Item, b: &Item| distance(b).total_cmp(&distance(a));

    let parallel = options.parallel && draws.len() >= PARALLEL_THRESHOLD;
    if parallel {
        // Chaque paquet produit ses petites listes ; `collect` les range
        // dans l'ordre des paquets, quel que soit le cœur qui a fini le premier.
        let parts: Vec<DrawLists> = draws
            .par_chunks(CHUNK_SIZE)
            .map(|chunk| {
                let mut part = DrawLists::default();
                for draw in chunk {
                    classify(draw, options, &mut part);
                }
                part
            })
            .collect();
        for part in parts {
            lists.append(part);
        }
        // Le tri aussi se partage. Il est **stable** comme `sort_by_key` :
        // l'ordre des commandes de même lot est conservé.
        lists.opaque.par_sort_by_key(|(key, _)| *key);
        lists.shadow.par_sort_by_key(|(key, _)| *key);
        lists.transparent.par_sort_by(far_first);
    } else {
        for draw in draws {
            classify(draw, options, lists);
        }
        // Les tuples se comparent élément par élément : pipeline, textures,
        // puis maillage. Les instances d'un même lot se suivent donc.
        lists.opaque.sort_by_key(|(key, _)| *key);
        lists.shadow.sort_by_key(|(key, _)| *key);
        lists.transparent.sort_by(far_first);
    }
}

/// Range une commande dans les listes qui la concernent.
fn classify(draw: &DrawCommand, options: &Options, lists: &mut DrawLists) {
    let material = &draw.material;
    let transparent = material.is_transparent();
    // Le verre ne projette pas d'ombre : la shadow map ne sait dire que
    // « bloqué » ou « pas bloqué », pas « bloqué à 30 % ». Une vitre
    // ferait une ombre aussi noire qu'un mur.
    let shadows = options.shadows && !transparent;
    let (visible, casts_shadow) = match &options.culling {
        None => (true, shadows),
        Some(culling) => {
            let world = options.bounds[draw.mesh.0].transformed(&draw.transform);
            (
                culling.camera.intersects(&world),
                shadows && culling.light.intersects(&world),
            )
        }
    };
    if !visible && !casts_shadow {
        return;
    }
    let raw = InstanceRaw::new(draw.transform, material);
    let textures = resolve(&draw.textures);
    if visible {
        let variant = MeshVariant::new(transparent, material.double_sided).index();
        let item = ((variant, textures, draw.mesh), raw);
        if transparent {
            lists.transparent.push(item);
        } else {
            lists.opaque.push(item);
        }
    }
    if casts_shadow {
        // L'ombre ne dépend que de la forme : les textures ne coupent pas
        // les lots... sauf pour les objets à découpe, dont la passe d'ombre
        // lit la texture pour y trouver les trous.
        let key = match material.alpha_mode {
            AlphaMode::Mask { .. } => (SHADOW_MASKED, textures, draw.mesh),
            _ => (0, NO_TEXTURES, draw.mesh),
        };
        lists.shadow.push((key, raw));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MaterialTextures, MeshHandle, TextureHandle};
    use zentrix_core::math::Mat4;
    use zentrix_core::math::camera::rh::{proj, view};
    use zentrix_core::{Color, Material};

    const CUBE: MeshHandle = MeshHandle(0);

    fn draw_at(position: Vec3) -> DrawCommand {
        DrawCommand {
            mesh: CUBE,
            textures: MaterialTextures::default(),
            transform: Mat4::from_translation(position),
            material: Material::DEFAULT,
        }
    }

    fn glass_at(position: Vec3) -> DrawCommand {
        DrawCommand {
            material: Material::glass(Color::WHITE, 0.3),
            ..draw_at(position)
        }
    }

    /// La caméra en (0, 0, 5) regarde vers -Z ; l'ombre couvre x ∈ [-10, 10].
    fn culling() -> Culling {
        let camera = proj::directx::perspective(1.0, 1.0, 0.1, 100.0)
            * view::look_at_mat4(EYE, Vec3::ZERO, Vec3::Y);
        let light = proj::directx::orthographic(-10.0, 10.0, -10.0, 10.0, 0.0, 40.0)
            * view::look_at_mat4(Vec3::new(0.0, 20.0, 0.0), Vec3::ZERO, Vec3::Z);
        Culling {
            camera: Frustum::from_view_projection(&camera),
            light: Frustum::from_view_projection(&light),
        }
    }

    const EYE: Vec3 = Vec3::new(0.0, 0.0, 5.0);
    const BOUNDS: [Aabb; 1] = [Aabb::new(Vec3::splat(-0.5), Vec3::splat(0.5))];

    fn options(culling: Option<Culling>, parallel: bool) -> Options<'static> {
        Options {
            bounds: &BOUNDS,
            culling,
            shadows: true,
            parallel,
            camera_position: EYE,
        }
    }

    fn run(draws: &[DrawCommand], options: &Options) -> DrawLists {
        let mut lists = DrawLists::default();
        sort_draws(draws, options, &mut lists);
        lists
    }

    fn z(items: &[Item]) -> Vec<f32> {
        items.iter().map(|(_, raw)| raw.model[3][2]).collect()
    }

    #[test]
    fn sans_culling_tout_est_dessine() {
        let draws = [draw_at(Vec3::ZERO), draw_at(Vec3::new(0.0, 0.0, 50.0))];
        let lists = run(&draws, &options(None, false));
        assert_eq!((lists.opaque.len(), lists.shadow.len()), (2, 2));
    }

    #[test]
    fn derriere_la_camera_mais_dans_l_ombre() {
        let draws = [
            draw_at(Vec3::ZERO),                 // vu, et projette une ombre
            draw_at(Vec3::new(0.0, 0.0, 8.0)),   // derrière la caméra, dans la zone d'ombre
            draw_at(Vec3::new(0.0, 0.0, -60.0)), // vu, mais loin de la zone d'ombre
            draw_at(Vec3::new(40.0, 0.0, 40.0)), // ni l'un ni l'autre
        ];
        let lists = run(&draws, &options(Some(culling()), false));
        assert_eq!(z(&lists.opaque), [0.0, -60.0]);
        assert_eq!(z(&lists.shadow), [0.0, 8.0]);
    }

    #[test]
    fn les_ombres_ignorent_les_textures() {
        let mut textured = draw_at(Vec3::ZERO);
        textured.textures = MaterialTextures::color(TextureHandle(7));
        let lists = run(&[textured, draw_at(Vec3::ZERO)], &options(None, false));
        // Deux lots à l'écran (textures différentes), un seul pour l'ombre.
        assert_ne!(lists.opaque[0].0, lists.opaque[1].0);
        assert_eq!(lists.shadow[0].0, lists.shadow[1].0);
    }

    #[test]
    fn le_verre_est_trie_du_plus_loin_au_plus_proche() {
        let draws = [
            glass_at(Vec3::new(0.0, 0.0, -1.0)),
            glass_at(Vec3::new(0.0, 0.0, -9.0)),
            draw_at(Vec3::ZERO),
            glass_at(Vec3::new(0.0, 0.0, -4.0)),
        ];
        let lists = run(&draws, &options(Some(culling()), false));
        assert_eq!(z(&lists.opaque), [0.0]);
        assert_eq!(z(&lists.transparent), [-9.0, -4.0, -1.0]);
        // Le verre ne projette pas d'ombre.
        assert_eq!(z(&lists.shadow), [0.0]);
    }

    #[test]
    fn la_decoupe_reste_opaque_mais_garde_ses_textures_pour_l_ombre() {
        let mut grillage = draw_at(Vec3::ZERO);
        grillage.material.alpha_mode = AlphaMode::Mask { cutoff: 0.5 };
        grillage.material.double_sided = true;
        grillage.textures = MaterialTextures::color(TextureHandle(7));
        let lists = run(&[grillage], &options(None, false));
        assert!(lists.transparent.is_empty());
        let (variant, textures, _) = lists.opaque[0].0;
        assert_eq!(variant, MeshVariant::new(false, true).index());
        assert_eq!(textures[0], TextureHandle(7));
        let (shadow_pipeline, shadow_textures, _) = lists.shadow[0].0;
        assert_eq!(shadow_pipeline, SHADOW_MASKED);
        assert_eq!(shadow_textures[0], TextureHandle(7));
    }

    #[test]
    fn plusieurs_threads_donnent_le_meme_resultat() {
        // Assez de commandes pour dépasser le seuil, des deux côtés de la
        // caméra, avec trois matériaux pour que le tri ait du travail.
        let draws: Vec<_> = (0..PARALLEL_THRESHOLD * 3)
            .map(|i| {
                let position = Vec3::new((i % 50) as f32 - 25.0, 0.0, -((i / 50) as f32));
                let mut draw = if i % 7 == 0 {
                    glass_at(position)
                } else {
                    draw_at(position)
                };
                if i % 3 == 0 {
                    draw.textures = MaterialTextures::color(TextureHandle(5));
                }
                draw
            })
            .collect();
        let one = run(&draws, &options(Some(culling()), false));
        let many = run(&draws, &options(Some(culling()), true));
        assert!(
            !one.opaque.is_empty() && one.opaque.len() < draws.len(),
            "{}",
            one.opaque.len()
        );
        assert!(!one.transparent.is_empty());
        let same = |a: &[Item], b: &[Item]| {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(x, y)| x.0 == y.0 && x.1.model == y.1.model)
        };
        assert!(same(&one.opaque, &many.opaque));
        assert!(same(&one.transparent, &many.transparent));
        assert!(same(&one.shadow, &many.shadow));
    }
}
