//! # zentrix-physics
//!
//! La **physique** : gravité, collisions, rebonds. Le calcul est confié à
//! [rapier](https://rapier.rs), un moteur physique écrit en Rust ; cette
//! crate le relie à la scène.
//!
//! ## Utilisation
//!
//! ```text
//!  au démarrage :  world.spawn((transform, mesh, RigidBody::dynamic(), Collider::cuboid(..)));
//!  chaque image :  physics.step(&mut scene, dt);   // avant scene.update_transforms()
//! ```
//!
//! Le jeu ne manipule jamais rapier : il ajoute des composants
//! ([`RigidBody`], [`Collider`], [`CharacterController`] pour un personnage)
//! et lit les `Transform`, comme d'habitude.
//!
//! ## Le trajet d'une image
//!
//! ```text
//!  Physics::step(scene, dt)
//!    ├─ 1. créer les corps des nouvelles entités, supprimer ceux des entités disparues
//!    ├─ 1 bis. déplacer les PERSONNAGES (`CharacterController`) : marche,
//!    │         saut, gravité, marches, glissement le long des murs
//!    ├─ 2. scène → rapier : position des corps cinématiques (et des corps
//!    │                      déplacés à la main, par l'inspecteur par exemple)
//!    ├─ 3. simuler par PAS FIXES de 1/60 s (0, 1 ou plusieurs selon `dt`),
//!    │     en relevant les CHOCS (voir `Physics::impacts`)
//!    └─ 4. rapier → scène : position des corps dynamiques
//! ```
//!
//! ## Pourquoi un pas fixe ?
//!
//! Une simulation dépend de la durée de ses pas : avec des pas variables
//! (la durée de chaque image), une pile de caisses peut être stable sur un
//! ordinateur et s'effondrer sur un autre. On accumule donc le temps écoulé,
//! et on avance toujours par tranches de 1/60 s.
//!
//! ## Deux versions de glam
//!
//! rapier utilise sa propre version de glam (0.33), différente de celle de
//! ZENTRIX (0.34). Leurs `Vec3` sont des types distincts pour Rust : on
//! convertit à la frontière en passant par des tableaux (`to_array` /
//! `from_array`), ce qui ne coûte rien.
//!
//! Limites actuelles : seules les entités **sans parent** ont un corps
//! physique ; la forme est calculée avec l'échelle du `Transform` au moment
//! de la création (la changer ensuite n'a pas d'effet) ; pas
//! d'articulations ; seul le DÉBUT d'un contact est signalé (pas la fin).

mod components;

use std::collections::HashMap;
use std::sync::Mutex;

use rapier3d::control::{
    CharacterAutostep, CharacterCollision, CharacterLength, KinematicCharacterController,
};
use rapier3d::math::{Pose, Rotation, Vector};
use rapier3d::prelude as rapier;
use zentrix_core::math::{Mat4, Quat, Vec3};
use zentrix_core::{Color, Transform};
use zentrix_render::DebugLines;
use zentrix_scene::{Entity, Parent, Scene};

pub use components::{BodyKind, CharacterController, Collider, RigidBody, Shape};

/// Durée d'un pas de simulation (60 pas par seconde).
pub const TIME_STEP: f32 = 1.0 / 60.0;
/// Vitesse de chute maximale d'un personnage (la résistance de l'air,
/// grossièrement), en unités par seconde.
const MAX_FALL_SPEED: f32 = 50.0;
/// Au plus 5 pas par image. Si une image a duré très longtemps (chargement,
/// fenêtre déplacée), on « oublie » le retard au lieu de le rattraper :
/// sinon chaque image demanderait toujours plus de pas, et le jeu se figerait
/// (la « spirale de la mort »).
const MAX_STEPS_PER_FRAME: u32 = 5;

/// Composant interne : le corps rapier d'une entité, et le dernier
/// `Transform` échangé avec lui (pour détecter un déplacement à la main).
#[derive(Debug, Clone, Copy)]
struct PhysicsBody {
    handle: rapier::RigidBodyHandle,
    kind: BodyKind,
    last: Transform,
}

/// Un choc entre deux objets (voir [`Physics::impacts`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Impact {
    pub entities: (Entity, Entity),
    /// La vitesse à laquelle les deux objets se sont rencontrés (vitesse
    /// relative juste avant le choc, en unités par seconde). Elle mesure la
    /// violence du choc : un son d'impact peut en dépendre.
    pub speed: f32,
    /// Où a eu lieu le choc : la position de l'objet le plus rapide des
    /// deux (une approximation, suffisante pour placer un son).
    pub position: Vec3,
}

/// Reçoit les événements de rapier pendant un pas de simulation.
///
/// rapier peut appeler ses méthodes depuis plusieurs threads (trait
/// `EventHandler` : `Send + Sync`, et `&self` seulement) : la liste est
/// protégée par un `Mutex`.
#[derive(Default)]
struct ContactCollector {
    started: Mutex<Vec<(rapier::ColliderHandle, rapier::ColliderHandle)>>,
}

impl rapier::EventHandler for ContactCollector {
    fn handle_collision_event(
        &self,
        _bodies: &rapier::RigidBodySet,
        _colliders: &rapier::ColliderSet,
        event: rapier::CollisionEvent,
        _contact_pair: Option<&rapier::ContactPair>,
    ) {
        if let rapier::CollisionEvent::Started(a, b, _) = event {
            self.started
                .lock()
                .expect("aucun thread n'a paniqué en tenant le verrou")
                .push((a, b));
        }
    }

    fn handle_contact_force_event(
        &self,
        _dt: f32,
        _bodies: &rapier::RigidBodySet,
        _colliders: &rapier::ColliderSet,
        _contact_pair: &rapier::ContactPair,
        _total_force_magnitude: f32,
    ) {
    }

    fn handle_soft_body_tear_event(
        &self,
        _soft_bodies: &rapier::SoftBodySet,
        _event: &rapier::SoftBodyTearEvent,
    ) {
    }
}

/// Ce que touche un rayon (voir [`Physics::cast_ray`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    pub entity: Entity,
    /// Distance depuis l'origine du rayon.
    pub distance: f32,
    pub point: Vec3,
    /// La normale de la surface touchée.
    pub normal: Vec3,
}

/// Le monde physique, parallèle à la scène.
pub struct Physics {
    world: rapier::PhysicsWorld,
    /// Temps écoulé pas encore simulé (moins d'un pas, en général).
    accumulator: f32,
    /// Les corps créés, pour retrouver ceux dont l'entité a disparu.
    bodies: HashMap<Entity, rapier::RigidBodyHandle>,
    /// Les chocs du dernier appel à `step`.
    impacts: Vec<Impact>,
    /// Des corps ont été créés depuis le dernier pas simulé : la broad phase
    /// de rapier ne les connaît pas encore.
    unseen_bodies: bool,
}

impl Default for Physics {
    fn default() -> Self {
        Self::new()
    }
}

// --- Conversions glam 0.34 (ZENTRIX) ↔ glam 0.33 (rapier) --------------------

fn to_rapier(v: Vec3) -> Vector {
    Vector::from_array(v.to_array())
}

fn from_rapier(v: Vector) -> Vec3 {
    Vec3::from_array(v.to_array())
}

fn pose_of(transform: &Transform) -> Pose {
    Pose::from_parts(
        to_rapier(transform.translation),
        Rotation::from_array(transform.rotation.to_array()),
    )
}

fn entity_of(user_data: u128) -> Option<Entity> {
    Entity::from_bits(user_data as u64)
}

impl Physics {
    pub fn new() -> Self {
        let mut world = rapier::PhysicsWorld::new();
        world.gravity = Vector::new(0.0, -9.81, 0.0);
        world.integration_parameters.dt = TIME_STEP;
        Self {
            world,
            accumulator: 0.0,
            bodies: HashMap::new(),
            impacts: Vec::new(),
            unseen_bodies: false,
        }
    }

    /// La gravité, en unités par seconde² (par défaut 9,81 vers le bas).
    pub fn gravity(&self) -> Vec3 {
        from_rapier(self.world.gravity)
    }

    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.world.gravity = to_rapier(gravity);
    }

    /// Nombre de corps physiques.
    pub fn body_count(&self) -> usize {
        self.world.bodies.len()
    }

    /// Oublie tous les corps (avant de charger une nouvelle scène). Ils
    /// seront recréés au prochain `step` pour les entités qui en demandent.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Fait avancer la simulation de `dt` secondes. Renvoie le nombre de pas
    /// simulés (souvent 1, parfois 0 ou 2 selon la durée de l'image).
    pub fn step(&mut self, scene: &mut Scene, dt: f32) -> u32 {
        profiling::scope!("Physics::step");
        self.remove_dead_bodies(scene);
        self.forget_stale_bodies(scene);
        self.create_new_bodies(scene);
        // Les personnages AVANT `push_transforms` : leur nouvelle position
        // est transmise à leur corps cinématique dans la foulée.
        self.move_characters(scene, dt);
        self.push_transforms(scene);

        self.impacts.clear();
        self.accumulator += dt;
        let mut steps = 0;
        while self.accumulator >= TIME_STEP && steps < MAX_STEPS_PER_FRAME {
            self.simulate_one_step();
            self.accumulator -= TIME_STEP;
            steps += 1;
        }
        if steps == MAX_STEPS_PER_FRAME {
            self.accumulator = 0.0;
        }
        if steps > 0 {
            self.pull_transforms(scene);
        }
        steps
    }

    /// Les chocs survenus pendant le dernier [`Physics::step`] : deux objets
    /// qui viennent de se toucher. Un objet posé ne produit pas de nouveau
    /// choc tant qu'il reste en contact.
    pub fn impacts(&self) -> &[Impact] {
        &self.impacts
    }

    /// Un pas de simulation, en relevant les contacts qui commencent.
    fn simulate_one_step(&mut self) {
        // Les vitesses AVANT le pas : après, le choc les a déjà changées
        // (une balle qui rebondit repart dans l'autre sens).
        let velocities: HashMap<rapier::RigidBodyHandle, Vec3> = self
            .world
            .bodies
            .iter()
            .filter(|(_, rb)| !rb.is_fixed())
            .map(|(handle, rb)| (handle, from_rapier(rb.linvel())))
            .collect();

        let collector = ContactCollector::default();
        self.world.step_with_events(&(), &collector);
        self.unseen_bodies = false;

        let started = collector
            .started
            .into_inner()
            .expect("aucun thread n'a paniqué en tenant le verrou");
        for (a, b) in started {
            if let Some(impact) = self.impact_between(a, b, &velocities) {
                self.impacts.push(impact);
            }
        }
    }

    /// Décrit le choc entre deux formes, à partir des vitesses d'avant le pas.
    fn impact_between(
        &self,
        a: rapier::ColliderHandle,
        b: rapier::ColliderHandle,
        velocities: &HashMap<rapier::RigidBodyHandle, Vec3>,
    ) -> Option<Impact> {
        let body = |c: rapier::ColliderHandle| self.world.colliders.get(c)?.parent();
        let (body_a, body_b) = (body(a)?, body(b)?);
        // Un corps fixe a une vitesse nulle.
        let velocity = |h| velocities.get(&h).copied().unwrap_or(Vec3::ZERO);
        let (va, vb) = (velocity(body_a), velocity(body_b));
        let fastest = if va.length_squared() >= vb.length_squared() {
            body_a
        } else {
            body_b
        };
        let entity = |h| entity_of(self.world.bodies.get(h)?.user_data);
        Some(Impact {
            entities: (entity(body_a)?, entity(body_b)?),
            speed: (va - vb).length(),
            position: from_rapier(self.world.bodies.get(fastest)?.translation()),
        })
    }

    /// Crée un corps rapier pour chaque entité qui a `Transform`,
    /// `RigidBody` et `Collider`, mais pas encore de corps.
    fn create_new_bodies(&mut self, scene: &mut Scene) {
        // On collecte d'abord : on ne peut pas ajouter de composant à une
        // entité pendant qu'une requête parcourt le monde.
        let new: Vec<(Entity, Transform, RigidBody, Collider)> = scene
            .world
            .query::<(Entity, &Transform, &RigidBody, &Collider)>()
            .without::<&PhysicsBody>()
            .without::<&Parent>()
            .iter()
            .map(|(e, t, b, c)| (e, *t, *b, *c))
            .collect();
        self.unseen_bodies |= !new.is_empty();

        for (entity, transform, body, collider) in new {
            let builder = match body.kind {
                BodyKind::Dynamic => rapier::RigidBodyBuilder::dynamic()
                    // CCD (« Continuous Collision Detection ») : un objet
                    // rapide ne traverse pas un mur fin entre deux pas.
                    .ccd_enabled(true),
                BodyKind::Fixed => rapier::RigidBodyBuilder::fixed(),
                BodyKind::Kinematic => rapier::RigidBodyBuilder::kinematic_position_based(),
            };
            let rigid_body = builder
                .pose(pose_of(&transform))
                .linvel(to_rapier(body.initial_velocity))
                // On range l'entité dans le corps, pour la retrouver (rayons).
                .user_data(entity.to_bits().get() as u128);

            let scale = transform.scale;
            let shape = match collider.shape {
                Shape::Cuboid { half_extents } => {
                    let h = half_extents * scale;
                    rapier::ColliderBuilder::cuboid(h.x, h.y, h.z)
                }
                // Une sphère reste une sphère : on prend la plus grande échelle.
                Shape::Ball { radius } => {
                    rapier::ColliderBuilder::ball(radius * scale.max_element())
                }
                // La capsule reste verticale : sa hauteur suit l'échelle en
                // Y, son rayon la plus grande échelle horizontale.
                Shape::Capsule {
                    half_height,
                    radius,
                } => rapier::ColliderBuilder::capsule_y(
                    half_height * scale.y,
                    radius * scale.x.max(scale.z),
                ),
            };
            let shape = shape
                // Signaler le début des contacts (pour `impacts`).
                .active_events(rapier::ActiveEvents::COLLISION_EVENTS)
                .translation(to_rapier(collider.offset * scale))
                .friction(collider.friction)
                .restitution(collider.restitution)
                .density(collider.density);

            let (handle, _) = self.world.insert(rigid_body, shape);
            self.bodies.insert(entity, handle);
            scene
                .world
                .insert_one(
                    entity,
                    PhysicsBody {
                        handle,
                        kind: body.kind,
                        last: transform,
                    },
                )
                .expect("l'entité existe : elle vient d'être trouvée");
        }
    }

    /// Déplace les personnages ([`CharacterController`]) selon leur vitesse
    /// voulue, la gravité et les sauts, en tenant compte du décor.
    ///
    /// Le contrôleur de rapier « sonde » le décor avec la forme du personnage
    /// dans la direction du mouvement (*shape cast*) : il s'arrête contre un
    /// mur, ou glisse le long de lui s'il l'aborde en biais, monte les
    /// marches et les pentes douces, et reste collé au sol en descendant.
    ///
    /// Contrairement à la simulation, ce calcul se fait une fois par IMAGE :
    /// le personnage doit répondre aux touches sans attendre le prochain pas.
    fn move_characters(&mut self, scene: &mut Scene, dt: f32) {
        if dt <= 0.0
            || scene
                .world
                .query_mut::<&CharacterController>()
                .into_iter()
                .next()
                .is_none()
        {
            return;
        }
        // Les recherches du contrôleur passent par la « broad phase » de
        // rapier, qui n'apprend l'existence des nouveaux corps qu'en simulant.
        // Tant qu'un pas n'a pas eu lieu depuis leur création, les personnages
        // attendent (une image ou deux) : sinon, un personnage créé en même
        // temps que le sol passerait à travers.
        //
        // (Forcer cette mise à jour avec `PhysicsWorld::detect_collisions`
        // dérègle les « îlots » de rapier, ses groupes d'objets en contact :
        // rapier panique ensuite si le personnage touche un objet endormi.)
        if self.unseen_bodies {
            return;
        }
        let gravity = from_rapier(self.world.gravity);
        // Les objets heurtés, poussés APRÈS la boucle : il faut alors un accès
        // en écriture aux corps, que la recherche de collisions emprunte.
        let mut pushes = Vec::new();

        for (transform, character, body) in
            scene
                .world
                .query_mut::<(&mut Transform, &mut CharacterController, &PhysicsBody)>()
        {
            let Some(collider_handle) = self
                .world
                .bodies
                .get(body.handle)
                .and_then(|rb| rb.colliders().first().copied())
            else {
                continue;
            };
            let collider = &self.world.colliders[collider_handle];

            // La vitesse verticale : au sol, nulle (ou un saut) ; en l'air, la
            // gravité. Sans gravité au sol, un personnage immobile ne demande
            // AUCUN déplacement : rapier en profite pour le sortir d'un objet
            // qui se serait glissé en lui. Et en descendant une pente, le
            // « collage au sol » de rapier le garde sur la pente.
            let mut vertical = if character.grounded {
                if character.jump {
                    character.jump_speed
                } else {
                    0.0
                }
            } else {
                (character.vertical_speed + gravity.y * dt).max(-MAX_FALL_SPEED)
            };
            character.jump = false;

            let horizontal = character.desired_velocity * Vec3::new(1.0, 0.0, 1.0);
            let desired = (horizontal + Vec3::Y * vertical) * dt;

            let controller = KinematicCharacterController {
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(character.step_height),
                    min_width: CharacterLength::Absolute(0.2),
                    // On ne « monte » pas sur une caisse qui bouge : on la pousse.
                    include_dynamic_bodies: false,
                }),
                max_slope_climb_angle: character.max_slope,
                min_slope_slide_angle: character.max_slope,
                ..KinematicCharacterController::default()
            };

            // La position de la forme d'après le `Transform` (et non le corps
            // rapier, qui a peut-être une image de retard si le jeu vient de
            // téleporter le personnage). Le personnage s'ignore lui-même.
            let offset = collider.position_wrt_parent().copied().unwrap_or_default();
            let shape_pose = pose_of(transform) * offset;
            let filter = rapier::QueryFilter::default().exclude_rigid_body(body.handle);
            let queries = self.world.query_pipeline_with_filter(filter);
            let mut collisions = Vec::new();
            let movement = controller.move_shape(
                dt,
                &queries,
                collider.shape(),
                &shape_pose,
                to_rapier(desired),
                |collision| collisions.push(collision),
            );

            let moved = from_rapier(movement.translation);
            transform.translation += moved;
            character.grounded = movement.grounded;
            // Au sol, on ne tombe plus ; contre un plafond, on ne monte plus.
            if (movement.grounded && vertical < 0.0)
                || (vertical > 0.0 && moved.y < desired.y * 0.5)
            {
                vertical = 0.0;
            }
            character.vertical_speed = vertical;

            if !collisions.is_empty() {
                pushes.push((
                    controller,
                    body.handle,
                    collider_handle,
                    character.mass,
                    collisions,
                ));
            }
        }

        for (controller, handle, collider_handle, mass, collisions) in pushes {
            self.push_bodies(&controller, handle, collider_handle, mass, &collisions, dt);
        }
    }

    /// Les objets dynamiques heurtés par un personnage reçoivent une
    /// impulsion : il pousse les caisses au lieu de buter dessus.
    fn push_bodies(
        &mut self,
        controller: &KinematicCharacterController,
        character: rapier::RigidBodyHandle,
        collider: rapier::ColliderHandle,
        mass: f32,
        collisions: &[CharacterCollision],
        dt: f32,
    ) {
        let shape = self.world.colliders[collider].shared_shape().clone();
        let world = &mut self.world;
        let mut queries = world.broad_phase.as_query_pipeline_mut(
            world.narrow_phase.query_dispatcher(),
            &mut world.bodies,
            &mut world.colliders,
            rapier::QueryFilter::default().exclude_rigid_body(character),
        );
        controller.solve_character_collision_impulses(dt, &mut queries, &*shape, mass, collisions);
    }

    /// Une entité dont le `PhysicsBody` ne désigne plus SON corps : ses
    /// composants ont été déplacés vers une nouvelle entité (l'éditeur
    /// restaure ainsi un objet supprimé, quand on annule). On retire ce
    /// composant périmé : le corps sera recréé, comme pour une nouvelle entité.
    fn forget_stale_bodies(&mut self, scene: &mut Scene) {
        let stale: Vec<Entity> = scene
            .world
            .query::<(Entity, &PhysicsBody)>()
            .iter()
            .filter(|(entity, body)| self.bodies.get(entity) != Some(&body.handle))
            .map(|(entity, _)| entity)
            .collect();
        for entity in stale {
            let _ = scene.world.remove_one::<PhysicsBody>(entity);
        }
    }

    /// Supprime les corps dont l'entité a disparu (ou a été remplacée).
    fn remove_dead_bodies(&mut self, scene: &Scene) {
        let world = &mut self.world;
        self.bodies.retain(|&entity, &mut handle| {
            let alive = scene
                .world
                .get::<&PhysicsBody>(entity)
                .is_ok_and(|b| b.handle == handle);
            if !alive {
                world.remove_body_with_colliders(handle, true);
            }
            alive
        });
    }

    /// Scène → rapier : les corps cinématiques suivent leur `Transform` ;
    /// un corps déplacé à la main (inspecteur, chargement) est téléporté.
    fn push_transforms(&mut self, scene: &mut Scene) {
        for (transform, body) in scene.world.query_mut::<(&Transform, &mut PhysicsBody)>() {
            let Some(rb) = self.world.bodies.get_mut(body.handle) else {
                continue;
            };
            match body.kind {
                // La physique calcule la vitesse nécessaire pour atteindre
                // cette position : les corps poussés reçoivent un vrai choc.
                BodyKind::Kinematic => rb.set_next_kinematic_position(pose_of(transform)),
                _ if *transform != body.last => {
                    rb.set_position(pose_of(transform), true);
                    // Une téléportation n'est pas un lancer : vitesse remise à zéro.
                    rb.set_linvel(Vector::ZERO, true);
                    rb.set_angvel(Vector::ZERO, true);
                }
                _ => {}
            }
            body.last = *transform;
        }
    }

    /// Rapier → scène : les corps dynamiques écrivent leur position.
    fn pull_transforms(&mut self, scene: &mut Scene) {
        for (transform, body) in scene
            .world
            .query_mut::<(&mut Transform, &mut PhysicsBody)>()
        {
            if body.kind != BodyKind::Dynamic {
                continue;
            }
            let Some(rb) = self.world.bodies.get(body.handle) else {
                continue;
            };
            transform.translation = from_rapier(rb.translation());
            transform.rotation = Quat::from_array(rb.rotation().to_array());
            body.last = *transform;
        }
    }

    /// Lance un rayon depuis `origin` dans la direction `direction` (de
    /// longueur 1), jusqu'à `max_distance`. Renvoie le premier objet touché.
    ///
    /// Sert à viser : « quel objet est sous le curseur ? », « le personnage
    /// touche-t-il le sol ? »...
    pub fn cast_ray(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RayHit> {
        self.cast_ray_excluding(origin, direction, max_distance, None)
    }

    /// Comme [`Physics::cast_ray`], en ignorant le corps de `exclude`. Sert
    /// à viser depuis les yeux d'un personnage : sans cela, le rayon
    /// toucherait d'abord sa propre capsule.
    pub fn cast_ray_excluding(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        exclude: Option<Entity>,
    ) -> Option<RayHit> {
        let ray = rapier::Ray::new(to_rapier(origin), to_rapier(direction));
        let mut filter = rapier::QueryFilter::default();
        if let Some(&handle) = exclude.and_then(|e| self.bodies.get(&e)) {
            filter = filter.exclude_rigid_body(handle);
        }
        let (collider, hit) =
            self.world
                .cast_ray_and_get_normal(&ray, max_distance, true, filter)?;
        let body = self.world.colliders.get(collider)?.parent()?;
        let entity = entity_of(self.world.bodies.get(body)?.user_data)?;
        Some(RayHit {
            entity,
            distance: hit.time_of_impact,
            point: origin + direction * hit.time_of_impact,
            normal: from_rapier(hit.normal),
        })
    }

    /// Donne une impulsion (un « coup ») au corps de `entity`, appliquée au
    /// point `point` : un coup excentré fait aussi tourner l'objet. Sans
    /// effet si l'entité n'a pas de corps dynamique.
    pub fn apply_impulse(&mut self, entity: Entity, impulse: Vec3, point: Vec3) {
        if let Some(&handle) = self.bodies.get(&entity)
            && let Some(rb) = self.world.bodies.get_mut(handle)
            && rb.is_dynamic()
        {
            rb.apply_impulse_at_point(to_rapier(impulse), to_rapier(point), true);
        }
    }

    /// Dessine les formes de collision : gris = fixe, orange = cinématique,
    /// vert = dynamique en mouvement, vert foncé = dynamique endormi (rapier
    /// cesse de simuler les objets immobiles, pour gagner du temps).
    ///
    /// Les formes sont grossies de 3 % : exactement à la surface des
    /// objets, leurs lignes seraient à moitié cachées par le test de profondeur.
    pub fn debug_draw(&self, debug: &mut DebugLines) {
        const INFLATE: f32 = 1.03;
        for (_, rb) in self.world.bodies.iter() {
            let color = match rb.body_type() {
                rapier::RigidBodyType::Fixed => Color::rgb(0.5, 0.5, 0.5),
                rapier::RigidBodyType::Dynamic if rb.is_sleeping() => Color::rgb(0.1, 0.4, 0.1),
                rapier::RigidBodyType::Dynamic => Color::rgb(0.2, 1.0, 0.3),
                _ => Color::rgb(1.0, 0.6, 0.1),
            };
            for &handle in rb.colliders() {
                let Some(collider) = self.world.colliders.get(handle) else {
                    continue;
                };
                let pose = collider.position();
                let matrix = Mat4::from_rotation_translation(
                    Quat::from_array(pose.rotation.to_array()),
                    from_rapier(pose.translation),
                );
                if let Some(cuboid) = collider.shape().as_cuboid() {
                    let size = from_rapier(cuboid.half_extents) * 2.0 * INFLATE;
                    debug.wire_box(matrix * Mat4::from_scale(size), color);
                } else if let Some(ball) = collider.shape().as_ball() {
                    wire_sphere(debug, matrix, ball.radius * INFLATE, color);
                } else if let Some(capsule) = collider.shape().as_capsule() {
                    // Deux sphères (les bouts) reliées par quatre traits.
                    let radius = capsule.radius * INFLATE;
                    let (a, b) = (
                        from_rapier(capsule.segment.a),
                        from_rapier(capsule.segment.b),
                    );
                    for end in [a, b] {
                        wire_sphere(debug, matrix * Mat4::from_translation(end), radius, color);
                    }
                    for side in [Vec3::X, -Vec3::X, Vec3::Z, -Vec3::Z] {
                        debug.line(
                            matrix.transform_point3(a + side * radius),
                            matrix.transform_point3(b + side * radius),
                            color,
                        );
                    }
                }
            }
        }
    }
}

/// Trois cercles (un par plan) : le contour d'une sphère.
fn wire_sphere(debug: &mut DebugLines, matrix: Mat4, radius: f32, color: Color) {
    const SEGMENTS: usize = 16;
    let point = |axis: usize, i: usize| {
        let a = std::f32::consts::TAU * i as f32 / SEGMENTS as f32;
        let (s, c) = (a.sin() * radius, a.cos() * radius);
        let local = match axis {
            0 => Vec3::new(0.0, c, s),
            1 => Vec3::new(c, 0.0, s),
            _ => Vec3::new(c, s, 0.0),
        };
        matrix.transform_point3(local)
    };
    for axis in 0..3 {
        for i in 0..SEGMENTS {
            debug.line(point(axis, i), point(axis, i + 1), color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un sol fixe (dessus en y = 0) et une balle de rayon 0,5 lâchée à 3 m.
    fn sol_et_balle(scene: &mut Scene) -> Entity {
        scene.world.spawn((
            Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)),
            RigidBody::fixed(),
            Collider::cuboid(Vec3::new(10.0, 0.5, 10.0)),
        ));
        scene.world.spawn((
            Transform::from_translation(Vec3::new(0.0, 3.0, 0.0)),
            RigidBody::dynamic(),
            Collider::ball(0.5),
        ))
    }

    fn simuler(physics: &mut Physics, scene: &mut Scene, secondes: f32) {
        for _ in 0..(secondes / TIME_STEP) as u32 {
            physics.step(scene, TIME_STEP);
        }
    }

    fn position(scene: &Scene, e: Entity) -> Vec3 {
        scene.world.get::<&Transform>(e).unwrap().translation
    }

    #[test]
    fn une_balle_tombe_et_se_pose_sur_le_sol() {
        let mut scene = Scene::default();
        let balle = sol_et_balle(&mut scene);
        let mut physics = Physics::new();
        simuler(&mut physics, &mut scene, 0.25);
        assert!(position(&scene, balle).y < 3.0, "elle tombe");
        simuler(&mut physics, &mut scene, 3.0);
        // Posée : son centre est à un rayon au-dessus du sol.
        let y = position(&scene, balle).y;
        assert!((y - 0.5).abs() < 0.05, "y = {y}");
        assert_eq!(physics.body_count(), 2);
    }

    #[test]
    fn pas_fixe_quelle_que_soit_la_duree_des_images() {
        let mut scene = Scene::default();
        sol_et_balle(&mut scene);
        let mut physics = Physics::new();
        // 1/120 s : un pas toutes les deux images.
        let pas: Vec<u32> = (0..4)
            .map(|_| physics.step(&mut scene, TIME_STEP / 2.0 + 1e-6))
            .collect();
        assert_eq!(pas.iter().sum::<u32>(), 2);
        // Une image d'une seconde : plafonnée à 5 pas.
        assert_eq!(physics.step(&mut scene, 1.0), MAX_STEPS_PER_FRAME);
    }

    #[test]
    fn le_corps_d_une_entite_supprimee_disparait() {
        let mut scene = Scene::default();
        let balle = sol_et_balle(&mut scene);
        let mut physics = Physics::new();
        physics.step(&mut scene, TIME_STEP);
        scene.world.despawn(balle).unwrap();
        physics.step(&mut scene, TIME_STEP);
        assert_eq!(physics.body_count(), 1);
    }

    #[test]
    fn deplacer_un_transform_teleporte_le_corps() {
        let mut scene = Scene::default();
        let balle = sol_et_balle(&mut scene);
        let mut physics = Physics::new();
        simuler(&mut physics, &mut scene, 0.5);
        // Comme le ferait l'inspecteur : la balle est replacée en hauteur.
        scene
            .world
            .get::<&mut Transform>(balle)
            .unwrap()
            .translation = Vec3::new(4.0, 6.0, 0.0);
        physics.step(&mut scene, TIME_STEP);
        let p = position(&scene, balle);
        assert!((p.x - 4.0).abs() < 1e-3 && p.y > 5.9, "{p}");
    }

    #[test]
    fn un_corps_cinematique_suit_son_transform() {
        let mut scene = Scene::default();
        let plateforme = scene.world.spawn((
            Transform::IDENTITY,
            RigidBody::kinematic(),
            Collider::cuboid(Vec3::splat(0.5)),
        ));
        let mut physics = Physics::new();
        physics.step(&mut scene, TIME_STEP);
        scene
            .world
            .get::<&mut Transform>(plateforme)
            .unwrap()
            .translation = Vec3::X * 2.0;
        physics.step(&mut scene, TIME_STEP);
        // Le rayon vertical en x = 2 touche maintenant la plateforme.
        let hit = physics.cast_ray(Vec3::new(2.0, 5.0, 0.0), Vec3::NEG_Y, 10.0);
        assert_eq!(hit.map(|h| h.entity), Some(plateforme));
    }

    #[test]
    fn un_rayon_trouve_l_entite_touchee() {
        let mut scene = Scene::default();
        let balle = sol_et_balle(&mut scene);
        let mut physics = Physics::new();
        physics.step(&mut scene, TIME_STEP);
        let hit = physics
            .cast_ray(Vec3::new(0.0, 10.0, 0.0), Vec3::NEG_Y, 100.0)
            .expect("touché");
        assert_eq!(hit.entity, balle);
        // La balle est vers y = 3 : son sommet vers y = 3,5.
        assert!((hit.point.y - 3.5).abs() < 0.05, "{:?}", hit.point);
        assert!(hit.normal.abs_diff_eq(Vec3::Y, 1e-3));
        // Rien vers le haut.
        assert!(
            physics
                .cast_ray(Vec3::ZERO + Vec3::Y * 10.0, Vec3::Y, 100.0)
                .is_none()
        );
    }

    #[test]
    fn la_forme_suit_l_echelle_du_transform() {
        let mut scene = Scene::default();
        let caisse = scene.world.spawn((
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)).with_uniform_scale(4.0),
            RigidBody::fixed(),
            Collider::cuboid(Vec3::splat(0.5)),
        ));
        let mut physics = Physics::new();
        physics.step(&mut scene, TIME_STEP);
        // Un cube de côté 4 : sa face du dessus est en y = 2.
        let hit = physics
            .cast_ray(Vec3::Y * 10.0, Vec3::NEG_Y, 100.0)
            .unwrap();
        assert_eq!(hit.entity, caisse);
        assert!((hit.point.y - 2.0).abs() < 1e-4);
    }

    #[test]
    fn un_choc_est_signale_avec_sa_vitesse() {
        let mut scene = Scene::default();
        let balle = sol_et_balle(&mut scene);
        let mut physics = Physics::new();
        let mut chocs = Vec::new();
        for _ in 0..120 {
            physics.step(&mut scene, TIME_STEP);
            chocs.extend_from_slice(physics.impacts());
        }
        // La balle tombe de 2,5 m (3 m moins son rayon) : elle touche le
        // sol à environ √(2 × 9,81 × 2,5) ≈ 7 m/s.
        let premier = chocs.first().expect("un choc");
        assert!(premier.entities.0 == balle || premier.entities.1 == balle);
        assert!((premier.speed - 7.0).abs() < 0.5, "{}", premier.speed);
        assert!(
            premier.position.y < 1.0,
            "près du sol : {:?}",
            premier.position
        );
        // Posée, elle ne produit plus de chocs.
        physics.step(&mut scene, TIME_STEP);
        assert!(physics.impacts().is_empty());
    }

    #[test]
    fn dessin_des_formes() {
        let mut scene = Scene::default();
        sol_et_balle(&mut scene);
        let mut physics = Physics::new();
        physics.step(&mut scene, TIME_STEP);
        let mut debug = DebugLines::default();
        physics.debug_draw(&mut debug);
        // Le sol : une boîte de 12 arêtes ; la balle : 3 cercles de 16 segments.
        assert_eq!(debug.line_count(), 12 + 3 * 16);
    }

    #[test]
    fn une_impulsion_met_en_mouvement() {
        let mut scene = Scene::default();
        let balle = sol_et_balle(&mut scene);
        let mut physics = Physics::new();
        simuler(&mut physics, &mut scene, 2.0);
        let avant = position(&scene, balle);
        physics.apply_impulse(balle, Vec3::X * 5.0, avant);
        simuler(&mut physics, &mut scene, 0.5);
        assert!(position(&scene, balle).x > avant.x + 0.5);
    }

    /// Un sol (dessus en y = 0) et un personnage de 1,8 m, pieds en `pieds`.
    fn sol_et_personnage(scene: &mut Scene, pieds: Vec3) -> Entity {
        scene.world.spawn((
            Transform::from_translation(Vec3::new(0.0, -0.5, 0.0)),
            RigidBody::fixed(),
            Collider::cuboid(Vec3::new(20.0, 0.5, 20.0)),
        ));
        scene.world.spawn((
            Transform::from_translation(pieds),
            RigidBody::kinematic(),
            // Hauteur 2 × (0,6 + 0,3) = 1,8 m, le bas de la capsule aux pieds.
            Collider::capsule(0.6, 0.3).with_offset(Vec3::Y * 0.9),
            CharacterController::default(),
        ))
    }

    fn personnage(scene: &Scene, e: Entity) -> CharacterController {
        *scene.world.get::<&CharacterController>(e).unwrap()
    }

    fn vouloir(scene: &mut Scene, e: Entity, vitesse: Vec3, saut: bool) {
        let mut c = scene.world.get::<&mut CharacterController>(e).unwrap();
        c.desired_velocity = vitesse;
        c.jump = saut;
    }

    #[test]
    fn un_personnage_tombe_puis_tient_debout_sur_le_sol() {
        let mut scene = Scene::default();
        let perso = sol_et_personnage(&mut scene, Vec3::new(0.0, 2.0, 0.0));
        let mut physics = Physics::new();
        simuler(&mut physics, &mut scene, 2.0);
        let pieds = position(&scene, perso);
        assert!(pieds.y.abs() < 0.05, "pieds en {pieds}");
        assert!(personnage(&scene, perso).grounded);
        assert_eq!(personnage(&scene, perso).vertical_speed, 0.0);
    }

    #[test]
    fn un_personnage_marche_et_s_arrete_contre_un_mur() {
        let mut scene = Scene::default();
        let perso = sol_et_personnage(&mut scene, Vec3::ZERO);
        // Un mur dont la face avant est en x = 3.
        scene.world.spawn((
            Transform::from_translation(Vec3::new(3.5, 1.5, 0.0)),
            RigidBody::fixed(),
            Collider::cuboid(Vec3::new(0.5, 1.5, 5.0)),
        ));
        let mut physics = Physics::new();
        simuler(&mut physics, &mut scene, 0.2);
        vouloir(&mut scene, perso, Vec3::X * 4.0, false);
        simuler(&mut physics, &mut scene, 0.5);
        // 0,5 s à 4 m/s : 2 m parcourus.
        let x = position(&scene, perso).x;
        assert!((x - 2.0).abs() < 0.1, "x = {x}");
        simuler(&mut physics, &mut scene, 2.0);
        // Bloqué par le mur : la capsule (rayon 0,3) s'arrête vers x = 2,7.
        let p = position(&scene, perso);
        assert!(p.x > 2.6 && p.x < 2.71, "{p}");
        assert!(p.y.abs() < 0.05, "reste au sol : {p}");
    }

    #[test]
    fn un_personnage_saute_puis_retombe() {
        let mut scene = Scene::default();
        let perso = sol_et_personnage(&mut scene, Vec3::ZERO);
        let mut physics = Physics::new();
        simuler(&mut physics, &mut scene, 0.3);
        assert!(personnage(&scene, perso).grounded);

        vouloir(&mut scene, perso, Vec3::ZERO, true);
        let mut plus_haut: f32 = 0.0;
        for _ in 0..30 {
            physics.step(&mut scene, TIME_STEP);
            plus_haut = plus_haut.max(position(&scene, perso).y);
        }
        // 5 m/s vers le haut : v² / 2g ≈ 1,27 m.
        assert!((plus_haut - 1.27).abs() < 0.1, "sommet à {plus_haut}");
        assert!(!personnage(&scene, perso).jump, "la demande est consommée");
        simuler(&mut physics, &mut scene, 1.0);
        assert!(position(&scene, perso).y.abs() < 0.05);
        assert!(personnage(&scene, perso).grounded);

        // En l'air, un saut ne relance pas.
        vouloir(&mut scene, perso, Vec3::ZERO, true);
        physics.step(&mut scene, TIME_STEP);
        vouloir(&mut scene, perso, Vec3::ZERO, true);
        for _ in 0..10 {
            physics.step(&mut scene, TIME_STEP);
        }
        assert!(position(&scene, perso).y < 1.4);
    }

    #[test]
    fn un_personnage_monte_une_marche() {
        let mut scene = Scene::default();
        let perso = sol_et_personnage(&mut scene, Vec3::ZERO);
        // Une marche de 20 cm, de x = 1 à x = 5.
        scene.world.spawn((
            Transform::from_translation(Vec3::new(3.0, 0.1, 0.0)),
            RigidBody::fixed(),
            Collider::cuboid(Vec3::new(2.0, 0.1, 2.0)),
        ));
        let mut physics = Physics::new();
        simuler(&mut physics, &mut scene, 0.2);
        vouloir(&mut scene, perso, Vec3::X * 3.0, false);
        simuler(&mut physics, &mut scene, 1.0);
        let p = position(&scene, perso);
        assert!(p.x > 2.5, "il avance sur la marche : {p}");
        assert!((p.y - 0.2).abs() < 0.05, "les pieds sur la marche : {p}");
    }

    #[test]
    fn un_personnage_pousse_une_caisse() {
        let mut scene = Scene::default();
        let perso = sol_et_personnage(&mut scene, Vec3::ZERO);
        let caisse = scene.world.spawn((
            Transform::from_translation(Vec3::new(1.5, 0.25, 0.0)),
            RigidBody::dynamic(),
            Collider::cuboid(Vec3::splat(0.25)).with_density(0.5),
        ));
        let mut physics = Physics::new();
        simuler(&mut physics, &mut scene, 0.3);
        let avant = position(&scene, caisse);
        vouloir(&mut scene, perso, Vec3::X * 3.0, false);
        simuler(&mut physics, &mut scene, 1.0);
        assert!(
            position(&scene, caisse).x > avant.x + 0.3,
            "la caisse a avancé : {}",
            position(&scene, caisse)
        );
    }

    /// Régression : créer un personnage pendant que des objets dynamiques
    /// reposent les uns sur les autres (la pile de caisses de la démo) ne
    /// doit pas dérégler rapier (« touching pair not linked in the
    /// persistent islands »).
    #[test]
    fn creer_un_personnage_au_milieu_d_objets_poses() {
        let mut scene = Scene::default();
        sol_et_balle(&mut scene);
        for i in 0..3 {
            scene.world.spawn((
                Transform::from_translation(Vec3::new(3.0, 0.25 + 0.5 * i as f32, 0.0)),
                RigidBody::dynamic(),
                Collider::cuboid(Vec3::splat(0.25)),
            ));
        }
        let mut physics = Physics::new();
        // Assez longtemps pour que la pile s'ENDORME (rapier ne simule plus
        // les objets immobiles) : c'est dans cet état que le bug apparaissait.
        simuler(&mut physics, &mut scene, 5.0);
        // Les pieds POSÉS sur la caisse du haut (dessus en y = 1,5), comme
        // quand on passe en marche au-dessus de la pile.
        let perso = scene.world.spawn((
            Transform::from_translation(Vec3::new(3.0, 1.5, 0.0)),
            RigidBody::kinematic(),
            Collider::capsule(0.6, 0.3).with_offset(Vec3::Y * 0.9),
            CharacterController::default(),
        ));
        simuler(&mut physics, &mut scene, 0.5);
        // Une balle lancée vers la pile pendant que le personnage existe.
        scene.world.spawn((
            Transform::from_translation(Vec3::new(1.0, 0.5, 0.0)),
            RigidBody::dynamic().with_velocity(Vec3::X * 10.0),
            Collider::ball(0.2),
        ));
        vouloir(&mut scene, perso, Vec3::NEG_X * 3.0, false);
        simuler(&mut physics, &mut scene, 2.0);
        assert!(personnage(&scene, perso).grounded);
    }

    /// Comme l'éditeur quand on annule une suppression : les composants
    /// d'une entité (dont son `PhysicsBody` interne) passent à une nouvelle.
    #[test]
    fn une_entite_restauree_retrouve_un_corps() {
        let mut scene = Scene::default();
        let balle = sol_et_balle(&mut scene);
        let mut physics = Physics::new();
        simuler(&mut physics, &mut scene, 0.2);
        let mut builder = zentrix_scene::hecs::EntityBuilder::new();
        builder.add_bundle(scene.world.take(balle).unwrap());
        physics.step(&mut scene, TIME_STEP);
        assert_eq!(
            physics.body_count(),
            1,
            "le corps de l'entité disparue est retiré"
        );

        let restauree = scene.world.spawn(builder.build());
        physics.step(&mut scene, TIME_STEP);
        assert_eq!(physics.body_count(), 2, "un nouveau corps");
        let avant = position(&scene, restauree);
        simuler(&mut physics, &mut scene, 0.3);
        assert!(
            position(&scene, restauree).y < avant.y,
            "elle tombe à nouveau"
        );
    }

    #[test]
    fn un_rayon_peut_ignorer_un_corps() {
        let mut scene = Scene::default();
        let perso = sol_et_personnage(&mut scene, Vec3::ZERO);
        let mut physics = Physics::new();
        physics.step(&mut scene, TIME_STEP);
        // Depuis les yeux (1,6 m), vers le bas : d'abord la capsule...
        let yeux = Vec3::Y * 1.6;
        let hit = physics.cast_ray(yeux, Vec3::NEG_Y, 10.0).unwrap();
        assert_eq!(hit.entity, perso);
        // ...sauf si on l'ignore : alors le sol.
        let hit = physics
            .cast_ray_excluding(yeux, Vec3::NEG_Y, 10.0, Some(perso))
            .unwrap();
        assert_ne!(hit.entity, perso);
        assert!(hit.point.y.abs() < 1e-3);
    }

    #[test]
    fn dessin_d_une_capsule() {
        let mut scene = Scene::default();
        scene.world.spawn((
            Transform::IDENTITY,
            RigidBody::kinematic(),
            Collider::capsule(0.6, 0.3),
        ));
        let mut physics = Physics::new();
        physics.step(&mut scene, TIME_STEP);
        let mut debug = DebugLines::default();
        physics.debug_draw(&mut debug);
        // Deux sphères de 3 cercles, et 4 traits verticaux.
        assert_eq!(debug.line_count(), 2 * 3 * 16 + 4);
    }
}
