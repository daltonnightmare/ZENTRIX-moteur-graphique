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
mod hierarchy;
mod serialization;

pub use components::{Camera3d, DirectionalLight, MeshRenderer, Name, PointLight, SpotLight};
pub use hierarchy::{GlobalTransform, Parent};
pub use serialization::{AssetNames, SceneError, SceneSerializer};

// Ré-export : l'application utilise `zentrix_scene::hecs` et a la garantie
// d'avoir la même version que la scène.
pub use hecs;
pub use hecs::Entity;

use zentrix_core::math::{Mat4, Vec3};
use zentrix_core::{Color, Transform};
use zentrix_render::{Camera, DrawCommand, Lighting, LocalLight, SpotCone};

/// Le monde du jeu : toutes les entités et leurs composants.
///
/// Le champ `world` est public : pour les opérations courantes, on utilise
/// directement l'API de `hecs` (`spawn`, `despawn`, `query`, `query_mut`...).
/// `Scene` ajoute par-dessus ce qui concerne le moteur (hiérarchie,
/// extraction vers le rendu).
///
/// # Ordre d'appel à chaque image
///
/// ```text
///  1. les systèmes du jeu modifient les `Transform` (locaux)
///  2. scene.update_transforms()   → calcule les `GlobalTransform`
///  3. scene.extract_draws(...)    → utilise les `GlobalTransform`
///     scene.camera()
/// ```
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
pub struct Scene {
    pub world: hecs::World,
    /// La lumière ambiante, présente partout. Ce n'est pas une entité : il
    /// n'y en a qu'une par scène, et elle n'a ni position ni direction.
    pub ambient_light: Color,
    propagation: hierarchy::PropagationCache,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            world: hecs::World::default(),
            ambient_light: Lighting::default().ambient,
            propagation: hierarchy::PropagationCache::default(),
        }
    }
}

impl Scene {
    /// Nombre d'entités vivantes.
    pub fn entity_count(&self) -> u32 {
        self.world.len()
    }

    /// Calcule le [`GlobalTransform`] (position dans le monde) de chaque
    /// entité, en tenant compte de ses parents (voir [`Parent`]).
    ///
    /// À appeler à chaque image, **après** les systèmes qui déplacent des
    /// entités et **avant** [`Scene::extract_draws`] / [`Scene::camera`].
    /// Sinon, le rendu montrerait les positions de l'image précédente.
    pub fn update_transforms(&mut self) {
        profiling::scope!("Scene::update_transforms");
        hierarchy::propagate(&mut self.world, &mut self.propagation);
    }

    /// Supprime une entité **et tous ses descendants**.
    ///
    /// Avec un simple `world.despawn(entité)`, les enfants survivraient et
    /// deviendraient des racines (leur `Transform` local serait alors
    /// interprété dans le repère du monde : ils « sauteraient »).
    /// Renvoie le nombre d'entités supprimées.
    pub fn despawn_recursive(&mut self, entity: Entity) -> usize {
        let mut doomed = hierarchy::descendants(&self.world, entity);
        doomed.push(entity);
        doomed
            .into_iter()
            .filter(|&e| self.world.despawn(e).is_ok())
            .count()
    }

    /// Ajoute à `out` une commande de dessin pour chaque entité visible
    /// (qui possède un `Transform` **et** un `MeshRenderer`).
    ///
    /// La position utilisée est le [`GlobalTransform`] s'il a été calculé,
    /// sinon le `Transform` local (entité sans parent, ou
    /// `update_transforms` jamais appelé).
    ///
    /// `out` n'est pas vidé : l'appelant le vide (et garde sa mémoire d'une
    /// image à l'autre) ou y ajoute d'autres commandes.
    pub fn extract_draws(&self, out: &mut Vec<DrawCommand>) {
        profiling::scope!("Scene::extract_draws");
        // Une requête ne visite QUE les entités qui ont tous les composants
        // demandés (sauf ceux marqués `Option`). Une caméra, sans
        // `MeshRenderer`, est ignorée ici.
        let mut query = self
            .world
            .query::<(&Transform, Option<&GlobalTransform>, &MeshRenderer)>();
        out.extend(
            query
                .iter()
                .map(|(transform, global, renderer)| DrawCommand {
                    mesh: renderer.mesh,
                    textures: renderer.textures,
                    transform: world_matrix(transform, global),
                    material: renderer.material,
                }),
        );
    }

    /// La caméra de rendu, construite à partir de la **première** entité qui
    /// possède un `Transform` et un `Camera3d`. `None` s'il n'y en a aucune.
    ///
    /// Grâce à la hiérarchie, une caméra peut être l'enfant d'un objet
    /// (caméra embarquée dans un véhicule, caméra « à l'épaule »...).
    pub fn camera(&self) -> Option<Camera> {
        let mut query = self
            .world
            .query::<(&Transform, Option<&GlobalTransform>, &Camera3d)>();
        let (transform, global, lens) = query.iter().next()?;
        Some(camera_from(world_matrix(transform, global), lens))
    }
}

impl Scene {
    /// L'éclairage pour le rendu : la **première** entité qui possède un
    /// `Transform` et une [`DirectionalLight`], plus `ambient_light`.
    ///
    /// Sans lumière directionnelle, seule l'ambiante éclaire la scène
    /// (intensité 0) : les objets restent visibles, mais plats et sombres.
    pub fn lighting(&self) -> Lighting {
        let mut query = self
            .world
            .query::<(&Transform, Option<&GlobalTransform>, &DirectionalLight)>();
        let sun = query.iter().next().map(|(transform, global, light)| {
            let direction = world_matrix(transform, global).transform_vector3(Vec3::NEG_Z);
            (direction, *light)
        });
        let (direction, light) = sun.unwrap_or((
            Vec3::NEG_Y,
            DirectionalLight {
                color: Color::BLACK,
                intensity: 0.0,
            },
        ));
        Lighting {
            direction,
            color: light.color,
            intensity: light.intensity,
            ambient: self.ambient_light,
        }
    }
}

impl Scene {
    /// Ajoute à `out` les lampes ([`PointLight`]) et projecteurs
    /// ([`SpotLight`]) de la scène, placés et orientés par leur position
    /// dans le monde. `out` n'est pas vidé (comme pour `extract_draws`).
    pub fn extract_lights(&self, out: &mut Vec<LocalLight>) {
        for (transform, global, light) in self
            .world
            .query::<(&Transform, Option<&GlobalTransform>, &PointLight)>()
            .iter()
        {
            out.push(LocalLight {
                position: world_matrix(transform, global).transform_point3(Vec3::ZERO),
                color: light.color,
                intensity: light.intensity,
                range: light.range,
                spot: None,
            });
        }
        for (transform, global, light) in self
            .world
            .query::<(&Transform, Option<&GlobalTransform>, &SpotLight)>()
            .iter()
        {
            let matrix = world_matrix(transform, global);
            out.push(LocalLight {
                position: matrix.transform_point3(Vec3::ZERO),
                color: light.color,
                intensity: light.intensity,
                range: light.range,
                spot: Some(SpotCone {
                    direction: matrix.transform_vector3(Vec3::NEG_Z),
                    inner_angle: light.inner_angle,
                    outer_angle: light.outer_angle,
                }),
            });
        }
    }
}

/// La matrice monde d'une entité : sa globale si elle existe, sinon sa locale.
fn world_matrix(transform: &Transform, global: Option<&GlobalTransform>) -> Mat4 {
    // `map_or_else(défaut, f)` : `f(valeur)` si `Some`, sinon `défaut()`.
    global.map_or_else(|| transform.matrix(), |g| g.0)
}

/// Construit la `Camera` de rendu à partir de la matrice monde de l'entité
/// et des réglages de son objectif.
///
/// La caméra regarde vers le -Z de son repère : on transforme ce vecteur
/// par la matrice pour obtenir la direction du regard dans le monde.
/// `transform_vector3` ignore la translation (c'est une direction, pas un
/// point) ; `normalize` annule l'effet d'une éventuelle mise à l'échelle.
fn camera_from(matrix: Mat4, lens: &Camera3d) -> Camera {
    let position = matrix.transform_point3(Vec3::ZERO);
    let forward = matrix.transform_vector3(Vec3::NEG_Z).normalize();
    Camera {
        position,
        target: position + forward,
        up: matrix.transform_vector3(Vec3::Y).normalize(),
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
        assert_eq!(draws[0].textures.base_color, Some(bois));
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

    /// Position monde d'une entité après `update_transforms`.
    fn position(scene: &Scene, entity: Entity) -> Vec3 {
        scene
            .world
            .get::<&GlobalTransform>(entity)
            .expect("GlobalTransform calculé")
            .translation()
    }

    #[test]
    fn un_enfant_suit_son_parent() {
        let mut scene = Scene::default();
        let parent = scene.world.spawn((Transform::from_translation(Vec3::X),));
        let enfant = scene
            .world
            .spawn((Transform::from_translation(Vec3::Y * 2.0), Parent(parent)));
        scene.update_transforms();
        assert_eq!(position(&scene, enfant), Vec3::new(1.0, 2.0, 0.0));

        // On déplace le parent : l'enfant suit à l'image suivante.
        scene
            .world
            .get::<&mut Transform>(parent)
            .unwrap()
            .translation = Vec3::Z * 5.0;
        scene.update_transforms();
        assert_eq!(position(&scene, enfant), Vec3::new(0.0, 2.0, 5.0));
    }

    #[test]
    fn la_rotation_du_parent_fait_orbiter_l_enfant() {
        let mut scene = Scene::default();
        // Le « soleil » tourne d'un quart de tour autour de +Y...
        let soleil =
            scene.world.spawn((Transform::IDENTITY
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),));
        // ... la « planète », placée en +X dans son repère, passe en -Z.
        let planete = scene
            .world
            .spawn((Transform::from_translation(Vec3::X * 3.0), Parent(soleil)));
        // ... et la « lune », en +X de la planète, suit les deux rotations.
        let lune = scene
            .world
            .spawn((Transform::from_translation(Vec3::X), Parent(planete)));
        scene.update_transforms();
        assert!(position(&scene, planete).abs_diff_eq(Vec3::new(0.0, 0.0, -3.0), 1e-5));
        assert!(position(&scene, lune).abs_diff_eq(Vec3::new(0.0, 0.0, -4.0), 1e-5));
    }

    #[test]
    fn un_orphelin_redevient_une_racine() {
        let mut scene = Scene::default();
        let parent = scene.world.spawn((Transform::from_translation(Vec3::X),));
        let enfant = scene
            .world
            .spawn((Transform::from_translation(Vec3::Y), Parent(parent)));
        scene.world.despawn(parent).unwrap();
        scene.update_transforms();
        assert_eq!(position(&scene, enfant), Vec3::Y);
    }

    #[test]
    fn un_parent_sans_transform_ne_compte_pas() {
        let mut scene = Scene::default();
        let parent = scene.world.spawn((Transform::from_translation(Vec3::X),));
        let enfant = scene
            .world
            .spawn((Transform::from_translation(Vec3::Y), Parent(parent)));
        scene.update_transforms();
        // Le parent garde son ancien `GlobalTransform`, mais n'a plus de
        // `Transform` : l'enfant ne doit pas suivre cette valeur périmée.
        scene.world.remove_one::<Transform>(parent).unwrap();
        scene.update_transforms();
        assert_eq!(position(&scene, enfant), Vec3::Y);
    }

    #[test]
    fn un_cycle_ne_bloque_pas() {
        let mut scene = Scene::default();
        let a = scene.world.spawn((Transform::IDENTITY,));
        let b = scene.world.spawn((Transform::IDENTITY, Parent(a)));
        scene.world.insert_one(a, Parent(b)).unwrap();
        scene.update_transforms(); // doit se terminer, avec un avertissement
        assert!(scene.world.get::<&GlobalTransform>(a).is_ok());
    }

    #[test]
    fn despawn_recursive_supprime_toute_la_branche() {
        let mut scene = Scene::default();
        let racine = scene.world.spawn((Transform::IDENTITY,));
        let enfant = scene.world.spawn((Transform::IDENTITY, Parent(racine)));
        scene.world.spawn((Transform::IDENTITY, Parent(enfant)));
        let autre = scene.world.spawn((Transform::IDENTITY,));

        assert_eq!(scene.despawn_recursive(racine), 3);
        assert_eq!(scene.entity_count(), 1);
        assert!(scene.world.contains(autre));
    }

    #[test]
    fn extraction_et_camera_utilisent_la_position_monde() {
        let mut scene = Scene::default();
        let vehicule = scene
            .world
            .spawn((Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),));
        scene.world.spawn((
            Transform::from_translation(Vec3::Y),
            MeshRenderer::new(MeshHandle::from_index(0)),
            Parent(vehicule),
        ));
        // Une caméra embarquée, 2 unités au-dessus du véhicule.
        scene.world.spawn((
            Transform::from_translation(Vec3::Y * 2.0),
            Camera3d::default(),
            Parent(vehicule),
        ));
        scene.update_transforms();

        let mut draws = Vec::new();
        scene.extract_draws(&mut draws);
        assert_eq!(
            draws[0].transform.transform_point3(Vec3::ZERO),
            Vec3::new(10.0, 1.0, 0.0)
        );
        assert_eq!(scene.camera().unwrap().position, Vec3::new(10.0, 2.0, 0.0));
    }

    #[test]
    fn la_lumiere_suit_l_orientation_de_son_entite() {
        let mut scene = Scene::default();
        // Sans soleil : seule l'ambiante.
        assert_eq!(scene.lighting().intensity, 0.0);

        // Un soleil penché de 90° vers le bas (rotation autour de X) : son -Z
        // pointe alors vers -Y, les rayons tombent verticalement.
        scene.world.spawn((
            Transform::IDENTITY.with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            DirectionalLight {
                color: Color::RED,
                intensity: 2.0,
            },
        ));
        let lighting = scene.lighting();
        assert!(
            lighting.direction.abs_diff_eq(Vec3::NEG_Y, 1e-6),
            "{}",
            lighting.direction
        );
        assert_eq!(lighting.color, Color::RED);
        assert_eq!(lighting.intensity, 2.0);
        assert_eq!(lighting.ambient, scene.ambient_light);
    }

    #[test]
    fn lampes_et_projecteurs() {
        let mut scene = Scene::default();
        scene.world.spawn((
            Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            PointLight::default(),
        ));
        // Un projecteur tourné vers le bas (le -Z de son repère pointe vers -Y).
        scene.world.spawn((
            Transform::IDENTITY.with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            SpotLight::default(),
        ));
        scene.update_transforms();
        let mut lights = Vec::new();
        scene.extract_lights(&mut lights);
        assert_eq!(lights.len(), 2);
        assert_eq!(lights[0].position, Vec3::new(1.0, 2.0, 3.0));
        assert!(lights[0].spot.is_none());
        let cone = lights[1].spot.expect("un projecteur");
        assert!(
            cone.direction.abs_diff_eq(Vec3::NEG_Y, 1e-5),
            "{}",
            cone.direction
        );
    }
}
