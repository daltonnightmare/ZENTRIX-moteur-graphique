//! La **hiérarchie** : des entités attachées à d'autres entités.
//!
//! Une roue suit la voiture, une arme suit la main, une lune suit sa planète.
//! Plutôt que de recalculer leur position à la main, on déclare un lien de
//! parenté et le `Transform` de l'enfant devient **relatif à son parent** :
//!
//! ```text
//!   Soleil   Transform = au centre, tourne sur lui-même
//!     └── Planète   Transform = 3 unités à droite DU SOLEIL
//!           └── Lune   Transform = 1 unité à droite DE LA PLANÈTE
//! ```
//!
//! Quand le soleil tourne, son repère tourne : la planète, fixe dans ce
//! repère, décrit donc une orbite — sans une ligne de code d'orbite.
//!
//! On distingue alors deux transformations :
//!
//! | Composant          | Repère              | Qui l'écrit ?                       |
//! |--------------------|---------------------|-------------------------------------|
//! | `Transform`        | **local** (parent)  | le jeu                              |
//! | [`GlobalTransform`] | **monde**          | le moteur ([`propagate`])           |

use std::collections::HashMap;

use hecs::{Entity, World};
use zentrix_core::Transform;
use zentrix_core::math::{Mat4, Vec3};

/// Profondeur maximale de la hiérarchie. Au-delà, on suppose une erreur
/// (par exemple un cycle : A parent de B, B parent de A) et on s'arrête
/// au lieu de boucler indéfiniment.
const MAX_DEPTH: usize = 64;

/// Composant posé sur l'**enfant** : « mon `Transform` est relatif à cette entité ».
///
/// Si le parent n'existe plus (ou n'a pas de `Transform`), l'enfant redevient
/// une racine : son `Transform` est alors directement dans le monde.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parent(pub Entity);

/// La transformation **dans le monde**, calculée par le moteur à partir des
/// `Transform` locaux de l'entité et de tous ses ancêtres.
///
/// Ne pas la modifier à la main : elle est réécrite à chaque appel de
/// `Scene::update_transforms`. La lire est en revanche très utile (« où est
/// vraiment cette lune ? »).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalTransform(pub Mat4);

impl GlobalTransform {
    /// La position dans le monde.
    pub fn translation(&self) -> Vec3 {
        self.0.transform_point3(Vec3::ZERO)
    }
}

/// Tableaux de travail réutilisés d'une image à l'autre (évite d'allouer
/// deux `HashMap` à chaque image).
#[derive(Default)]
pub(crate) struct PropagationCache {
    /// Pour chaque entité : sa matrice locale et son parent éventuel.
    locals: HashMap<Entity, (Mat4, Option<Entity>)>,
    /// Les matrices globales déjà calculées.
    globals: HashMap<Entity, Mat4>,
    /// Les entités qui n'ont pas encore de composant `GlobalTransform`.
    missing: Vec<(Entity, Mat4)>,
}

/// Calcule le [`GlobalTransform`] de toutes les entités qui ont un `Transform`.
///
/// Méthode : pour chaque entité, global = global(parent) × local. Les
/// résultats sont mémorisés, si bien que chaque entité n'est calculée
/// qu'une fois, même si elle a beaucoup de descendants.
pub(crate) fn propagate(world: &mut World, cache: &mut PropagationCache) {
    let PropagationCache {
        locals,
        globals,
        missing,
    } = cache;
    locals.clear();
    globals.clear();
    missing.clear();

    // 1. Relever les matrices locales et les liens de parenté.
    //    `Option<&Parent>` : le composant est facultatif dans cette requête.
    for (entity, transform, parent) in world
        .query::<(Entity, &Transform, Option<&Parent>)>()
        .iter()
    {
        locals.insert(entity, (transform.matrix(), parent.map(|p| p.0)));
    }

    // 2. Calculer les matrices globales.
    let entities: Vec<Entity> = locals.keys().copied().collect();
    for entity in entities {
        global_of(entity, locals, globals, 0);
    }

    // 3. Les écrire dans le monde. On ne peut pas AJOUTER un composant pendant
    //    qu'on parcourt une requête : les entités sans `GlobalTransform` sont
    //    mises de côté et complétées ensuite.
    for (&entity, &matrix) in globals.iter() {
        match world.get::<&mut GlobalTransform>(entity) {
            Ok(mut global) => global.0 = matrix,
            Err(_) => missing.push((entity, matrix)),
        }
    }
    for &(entity, matrix) in missing.iter() {
        // L'entité existe forcément : on vient de la lire.
        let _ = world.insert_one(entity, GlobalTransform(matrix));
    }
}

/// La matrice globale d'une entité (récursif, avec mémorisation).
fn global_of(
    entity: Entity,
    locals: &HashMap<Entity, (Mat4, Option<Entity>)>,
    globals: &mut HashMap<Entity, Mat4>,
    depth: usize,
) -> Mat4 {
    if let Some(&global) = globals.get(&entity) {
        return global;
    }
    let (local, parent) = locals[&entity];
    let global = match parent {
        // Un parent valide (il a un `Transform`) : on compose.
        Some(parent) if locals.contains_key(&parent) => {
            if depth >= MAX_DEPTH {
                log::warn!(
                    "Hiérarchie trop profonde ou cyclique autour de {entity:?} (> {MAX_DEPTH} niveaux)"
                );
                local
            } else {
                // L'ordre compte : d'abord le local (repère du parent), puis
                // le global du parent (repère du monde).
                global_of(parent, locals, globals, depth + 1) * local
            }
        }
        // Pas de parent, ou un parent disparu : l'entité est une racine.
        _ => local,
    };
    globals.insert(entity, global);
    global
}

/// Toutes les entités descendantes de `root` (enfants, petits-enfants...),
/// sans `root` elle-même.
pub(crate) fn descendants(world: &World, root: Entity) -> Vec<Entity> {
    let links: Vec<(Entity, Entity)> = world
        .query::<(Entity, &Parent)>()
        .iter()
        .map(|(child, parent)| (child, parent.0))
        .collect();

    let mut found = Vec::new();
    let mut to_visit = vec![root];
    while let Some(current) = to_visit.pop() {
        for &(child, parent) in &links {
            // `!found.contains` protège contre les cycles.
            if parent == current && child != root && !found.contains(&child) {
                found.push(child);
                to_visit.push(child);
            }
        }
    }
    found
}
