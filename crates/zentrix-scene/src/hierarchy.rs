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
    /// Pour chaque entité qui a un parent : sa matrice locale et ce parent.
    locals: HashMap<Entity, (Mat4, Entity)>,
    /// Les matrices globales des enfants déjà calculées.
    globals: HashMap<Entity, Mat4>,
    /// Les entités qui n'ont pas encore de composant `GlobalTransform`.
    missing: Vec<(Entity, Mat4)>,
}

/// Calcule le [`GlobalTransform`] de toutes les entités qui ont un `Transform`.
///
/// Méthode : pour chaque entité, global = global(parent) × local.
///
/// # Le cas courant d'abord
///
/// Presque toutes les entités sont des **racines** (sans parent) : leur
/// global est simplement leur local. Elles sont traitées par une requête
/// directe, sans aucune table. Seuls les enfants passent par les `HashMap`
/// et la récursion, avec mémorisation : chaque enfant n'est calculé qu'une
/// fois, même s'il a beaucoup de descendants.
///
/// (Avant, toutes les entités passaient par les tables : avec 4 000 cubes,
/// le profileur montrait cette fonction comme la plus lente de l'image.)
pub(crate) fn propagate(world: &mut World, cache: &mut PropagationCache) {
    let PropagationCache {
        locals,
        globals,
        missing,
    } = cache;
    locals.clear();
    globals.clear();
    missing.clear();

    // 1. Donner un `GlobalTransform` aux nouvelles entités. On ne peut pas
    //    AJOUTER un composant pendant qu'on parcourt une requête : on les
    //    relève d'abord, puis on les complète. En régime normal, la liste
    //    est vide.
    missing.extend(
        world
            .query::<hecs::Without<(Entity, &Transform), &GlobalTransform>>()
            .iter()
            .map(|(entity, transform)| (entity, transform.matrix())),
    );
    for &(entity, matrix) in missing.iter() {
        // L'entité existe forcément : on vient de la lire.
        let _ = world.insert_one(entity, GlobalTransform(matrix));
    }

    // 2. Les racines : global = local. `query_mut` donne un accès en
    //    écriture sans vérification d'emprunt à chaque entité (plus rapide).
    for (transform, global) in
        world.query_mut::<hecs::Without<(&Transform, &mut GlobalTransform), &Parent>>()
    {
        global.0 = transform.matrix();
    }

    // 3. Les enfants : relever leurs matrices locales et leurs parents...
    for (entity, transform, parent) in world.query::<(Entity, &Transform, &Parent)>().iter() {
        locals.insert(entity, (transform.matrix(), parent.0));
    }
    if locals.is_empty() {
        return;
    }
    // ... calculer leurs matrices globales...
    let children: Vec<Entity> = locals.keys().copied().collect();
    for entity in children {
        global_of(entity, world, locals, globals, 0);
    }
    // ... et les écrire.
    for (&entity, &matrix) in globals.iter() {
        if let Ok(mut global) = world.get::<&mut GlobalTransform>(entity) {
            global.0 = matrix;
        }
    }
}

/// La matrice globale d'un enfant (récursif, avec mémorisation).
fn global_of(
    entity: Entity,
    world: &World,
    locals: &HashMap<Entity, (Mat4, Entity)>,
    globals: &mut HashMap<Entity, Mat4>,
    depth: usize,
) -> Mat4 {
    if let Some(&global) = globals.get(&entity) {
        return global;
    }
    let (local, parent) = locals[&entity];
    let parent_global = if locals.contains_key(&parent) {
        // Le parent est lui-même un enfant : on remonte.
        if depth >= MAX_DEPTH {
            log::warn!(
                "Hiérarchie trop profonde ou cyclique autour de {entity:?} (> {MAX_DEPTH} niveaux)"
            );
            None
        } else {
            Some(global_of(parent, world, locals, globals, depth + 1))
        }
    } else {
        // Le parent est une racine, déjà calculée à l'étape 2... s'il a
        // bien un `Transform`.
        world
            .entity(parent)
            .ok()
            .filter(|parent| parent.has::<Transform>())
            .and_then(|parent| parent.get::<&GlobalTransform>().map(|g| g.0))
    };
    // L'ordre compte : d'abord le local (repère du parent), puis le global
    // du parent (repère du monde). Sans parent valide (disparu, sans
    // `Transform`), l'entité est une racine.
    let global = parent_global.map_or(local, |parent| parent * local);
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
