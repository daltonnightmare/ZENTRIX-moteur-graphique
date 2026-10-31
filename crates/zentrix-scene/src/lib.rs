//! # zentrix-scene
//!
//! La **scène** : tout ce qui existe dans le monde du jeu, organisé selon le
//! modèle **ECS** (*Entity Component System*).
//!
//! ## L'ECS en trois mots
//!
//! | Notion        | Ce que c'est                               | Exemple                                    |
//! |---------------|--------------------------------------------|--------------------------------------------|
//! | **Entité**    | un simple identifiant, sans données        | « l'entité n°42 »                          |
//! | **Composant** | une donnée attachée à une entité           | `Transform`, `MeshRenderer`, `Vitesse`     |
//! | **Système**   | une fonction qui traite toutes les entités ayant certains composants | « déplacer tout ce qui a un `Transform` et une `Vitesse` » |
//!
//! Plutôt qu'une hiérarchie de classes (`Caisse` hérite de `ObjetVisible`
//! qui hérite de `Objet`...), on **compose** : une caisse est une entité avec
//! un `Transform` et un `MeshRenderer` ; pour la faire tourner, on lui ajoute
//! un composant `Rotation`. Ajouter un comportement = ajouter un composant.
//!
//! Bonus performance : la bibliothèque [`hecs`] range les composants de même
//! type côte à côte en mémoire. Parcourir 10 000 `Transform` lit donc la
//! mémoire de façon continue, ce que le processeur adore (voir le module 17
//! du cours : conception orientée données).
//!
//! ## Le lien avec le rendu
//!
//! ```text
//!   Scene (hecs::World)                 extraction            zentrix-render
//!   ┌───────────────────────────────┐                     ┌──────────────────┐
//!   │ Transform + MeshRenderer      │ ── extract_draws ─▶ │ Vec<DrawCommand> │
//!   │ Transform + Camera3d          │ ── camera ────────▶ │ Camera           │
//!   │ Transform + composants du jeu │                     └────────┬─────────┘
//!   └───────────────────────────────┘                     Renderer::render
//! ```
//!
//! Le `Renderer` ne connaît **jamais** l'ECS : il reçoit des listes simples.
//! On pourrait remplacer `hecs` par `bevy_ecs` sans toucher au rendu.

mod components;

pub use components::{Camera3d, MeshRenderer, Name};

// Ré-export : l'application utilise `zentrix_scene::hecs` et a la garantie
// d'avoir la même version que la scène.
pub use hecs;
pub use hecs::Entity;

use zentrix_core::Transform;
use zentrix_core::math::Vec3;
use zentrix_render::{Camera, DrawCommand};

/// Le monde du jeu : toutes les entités et leurs composants.
///
/// Le champ `world` est public : pour les opérations courantes, on utilise
/// directement l'API de `hecs` (`spawn`, `despawn`, `query`, `query_mut`...).
/// `Scene` ajoute par-dessus ce qui concerne le moteur (extraction vers le rendu).
///
/// ```
/// use zentrix_core::Transform;
/// use zentrix_scene::Scene;
///
/// // Un composant propre au jeu : n'importe quelle struct convient.
/// struct Vie(u32);
///
/// let mut scene = Scene::default();
/// scene.world.spawn((Transform::IDENTITY, Vie(100)));
///
/// // Un « système » : une boucle sur une requête.
/// for vie in scene.world.query_mut::<&mut Vie>() {
///     vie.0 -= 10;
/// }
/// assert_eq!(scene.world.query::<&Vie>().iter().next().unwrap().0, 90);
/// ```
#[derive(Default)]
pub struct Scene {
    pub world: hecs::World,
}

impl Scene {
    /// Nombre d'entités vivantes.
    pub fn entity_count(&self) -> u32 {
        self.world.len()
    }

    /// Ajoute à `out` une commande de dessin pour chaque entité visible
    /// (qui possède un `Transform` **et** un `MeshRenderer`).
    ///
    /// `out` n'est pas vidé : l'appelant le vide (et garde sa mémoire d'une
    /// image à l'autre) ou y ajoute d'autres commandes.
    pub fn extract_draws(&self, out: &mut Vec<DrawCommand>) {
        // Une requête ne visite QUE les entités qui ont tous les composants
        // demandés. Une caméra (sans `MeshRenderer`) est ignorée ici.
        let mut query = self.world.query::<(&Transform, &MeshRenderer)>();
        out.extend(query.iter().map(|(transform, renderer)| DrawCommand {
            mesh: renderer.mesh,
            texture: renderer.texture,
            transform: transform.matrix(),
        }));
    }

    /// La caméra de rendu, construite à partir de la **première** entité qui
    /// possède un `Transform` et un `Camera3d`. `None` s'il n'y en a aucune.
    pub fn camera(&self) -> Option<Camera> {
        let mut query = self.world.query::<(&Transform, &Camera3d)>();
        let (transform, lens) = query.iter().next()?;
        Some(camera_from(transform, lens))
    }
}

/// Convertit le couple (`Transform`, `Camera3d`) en `Camera` de rendu.
///
/// La caméra regarde vers le -Z de son repère : on fait tourner ce vecteur
/// par la rotation de l'entité pour obtenir la direction du regard.
fn camera_from(transform: &Transform, lens: &Camera3d) -> Camera {
    let forward = transform.rotation * Vec3::NEG_Z;
    Camera {
        position: transform.translation,
        target: transform.translation + forward,
        up: transform.rotation * Vec3::Y,
        fov_y: lens.fov_y,
        near: lens.near,
        far: lens.far,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zentrix_core::math::Quat;
    use zentrix_render::{MeshHandle, TextureHandle};

    #[test]
    fn seules_les_entites_visibles_sont_dessinees() {
        let mut scene = Scene::default();
        let cube = MeshHandle::from_index(3);
        let bois = TextureHandle::from_index(1);
        scene.world.spawn((
            Transform::from_translation(Vec3::X),
            MeshRenderer::new(cube).with_texture(bois),
        ));
        scene
            .world
            .spawn((Transform::IDENTITY, Camera3d::default())); // pas de maillage
        scene.world.spawn((MeshRenderer::new(cube),)); // pas de Transform

        let mut draws = Vec::new();
        scene.extract_draws(&mut draws);
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0].mesh, cube);
        assert_eq!(draws[0].texture, Some(bois));
        assert_eq!(draws[0].transform.transform_point3(Vec3::ZERO), Vec3::X);
    }

    #[test]
    fn pas_de_camera_pas_de_rendu() {
        assert!(Scene::default().camera().is_none());
    }

    #[test]
    fn la_camera_suit_son_transform() {
        let mut scene = Scene::default();
        // Placée en (0, 2, 5), tournée de 90° vers la gauche (autour de +Y) :
        // le -Z de la caméra devient le -X du monde.
        let transform = Transform::from_translation(Vec3::new(0.0, 2.0, 5.0))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
        scene.world.spawn((transform, Camera3d::default()));

        let camera = scene.camera().expect("une caméra");
        assert_eq!(camera.position, Vec3::new(0.0, 2.0, 5.0));
        let direction = camera.target - camera.position;
        assert!(direction.abs_diff_eq(Vec3::NEG_X, 1e-6), "{direction}");
        assert!(camera.up.abs_diff_eq(Vec3::Y, 1e-6));
    }
}
