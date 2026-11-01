//! L'**éditeur** : sélectionner un objet en cliquant dessus, le déplacer, le
//! tourner ou l'agrandir à la souris (les *gizmos*), annuler et refaire.
//!
//! Le mode édition est actif quand l'inspecteur est ouvert (`F4`) :
//!
//! | Action | Effet |
//! |--------|-------|
//! | clic gauche sur un objet | le sélectionner (dans le vide : désélectionner) |
//! | glisser une poignée du gizmo | déplacer / tourner / agrandir (`Ctrl` : par crans) |
//! | `1` `2` `3` | gizmo de déplacement, de rotation, d'agrandissement |
//! | clic droit maintenu | piloter la caméra (souris + `Z` `Q` `S` `D`) |
//! | `Suppr` | supprimer l'objet sélectionné |
//! | `Ctrl+Z` / `Ctrl+Y` | annuler / refaire |
//!
//! ## Trouver l'objet sous le curseur
//!
//! Le curseur désigne un **rayon** : tous les points de la scène qui
//! s'affichent sur ce pixel ([`screen_ray`]). On teste ce rayon contre la
//! boîte englobante de chaque maillage, dans le repère de l'objet (une boîte
//! tournée reste ainsi une boîte exacte), et on garde le plus proche ([`pick`]).
//!
//! ## Les gizmos
//!
//! Ils sont dessinés par egui, à plat sur l'écran, sous les fenêtres : on
//! projette les axes de l'objet à l'écran, et on compare la position du
//! curseur aux traits obtenus. Glisser le long d'un axe à l'écran se
//! traduit en mètres le long de l'axe dans le monde ([`axis_drag`]).
//!
//! ## Annuler
//!
//! L'[`History`] garde chaque modification ([`Edit`]) avec de quoi la
//! défaire : le `Transform` avant et après un geste, les composants des
//! entités supprimées. Une première version gardait une copie de toute la
//! scène à chaque modification : simple, mais 350 ms par geste avec le champ
//! de stress (4 000 entités) et jusqu'à 110 Mo de mémoire. Ici, le coût ne
//! dépend que de ce qui a changé.

use zentrix_core::math::{Mat4, Quat, Vec2, Vec3};
use zentrix_core::{Aabb, Color, Material, Transform};
use zentrix_physics::{Collider, RigidBody};
use zentrix_render::MeshHandle;
use zentrix_scene::{
    Entity, GlobalTransform, MeshRenderer, Name, Parent, PointLight, SpotLight, hecs,
};
use zentrix_ui::egui;

use crate::demo::DemoAssets;

/// Combien de modifications on peut annuler.
const HISTORY_LIMIT: usize = 50;
/// Taille des gizmos : une fraction de la distance à la caméra, pour qu'ils
/// gardent la même taille à l'écran.
const GIZMO_SCALE: f32 = 0.18;
/// Distance (en points d'écran) en dessous de laquelle le curseur « touche »
/// une poignée.
const GRAB_DISTANCE: f32 = 9.0;
/// Les lampes n'ont pas de maillage : on les sélectionne dans une sphère de ce rayon.
const LIGHT_PICK_RADIUS: f32 = 0.3;
/// Les crans avec `Ctrl` : 25 cm, 15°, 10 %.
const SNAP_DISTANCE: f32 = 0.25;
const SNAP_ANGLE: f32 = 15.0;
const SNAP_SCALE: f32 = 0.1;

/// Ce que fait le gizmo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

impl GizmoMode {
    pub const ALL: [Self; 3] = [Self::Translate, Self::Rotate, Self::Scale];

    pub fn label(self) -> &'static str {
        match self {
            Self::Translate => "Déplacer (1)",
            Self::Rotate => "Tourner (2)",
            Self::Scale => "Agrandir (3)",
        }
    }
}

/// Une poignée du gizmo : un axe (0 = X, 1 = Y, 2 = Z), ou le carré central
/// de l'agrandissement (les trois axes à la fois).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Handle {
    Axis(usize),
    Uniform,
}

/// Une poignée en train d'être tirée.
#[derive(Debug, Clone, Copy)]
struct Drag {
    handle: Handle,
    /// Le `Transform` au début du geste : chaque image repart de lui, et non
    /// de l'image précédente (pas d'erreurs qui s'accumulent).
    start: Transform,
    start_cursor: egui::Pos2,
    /// L'axe à l'écran (du centre au bout) et sa longueur dans le monde, au
    /// début du geste.
    screen_axis: egui::Vec2,
    world_length: f32,
}

/// Ce que le gizmo a fait pendant cette image.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GizmoResponse {
    /// Le curseur est sur une poignée, ou en train d'en tirer une : le clic
    /// appartient au gizmo (il ne doit pas sélectionner un autre objet).
    pub active: bool,
    /// Un geste vient de commencer (le moment de prendre la photo pour « annuler »).
    pub started: bool,
}

/// L'état du gizmo, d'une image à l'autre.
#[derive(Debug, Default)]
pub struct Gizmo {
    pub mode: GizmoMode,
    drag: Option<Drag>,
}

/// Où se trouve le gizmo à l'écran.
struct Screen {
    rect: egui::Rect,
    view_projection: Mat4,
}

impl Screen {
    /// Un point du monde, à l'écran (en points egui). `None` derrière la caméra.
    fn project(&self, p: Vec3) -> Option<egui::Pos2> {
        let clip = self.view_projection * p.extend(1.0);
        if clip.w <= 1e-4 {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        Some(egui::pos2(
            self.rect.left() + (ndc.x + 1.0) * 0.5 * self.rect.width(),
            self.rect.top() + (1.0 - ndc.y) * 0.5 * self.rect.height(),
        ))
    }
}

impl Gizmo {
    /// Un geste est-il en cours ?
    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Dessine le gizmo de l'objet sélectionné et applique le geste en cours
    /// à son `transform`.
    ///
    /// - `global` : sa position dans le monde (`GlobalTransform`) ;
    /// - `parent` : la matrice de son parent (identité s'il n'en a pas) : le
    ///   gizmo travaille dans le monde, le `Transform` dans le repère du parent ;
    /// - `snap` : avancer par crans (`Ctrl`).
    #[allow(clippy::too_many_arguments)]
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        view_projection: Mat4,
        camera_position: Vec3,
        global: Mat4,
        parent: Mat4,
        transform: &mut Transform,
        snap: bool,
    ) -> GizmoResponse {
        let screen = Screen {
            rect: ctx.content_rect(),
            view_projection,
        };
        let origin = global.w_axis.truncate();
        let length = origin.distance(camera_position).max(0.5) * GIZMO_SCALE;
        let Some(center) = screen.project(origin) else {
            self.drag = None;
            return GizmoResponse::default();
        };
        let axes = self.axes(global);

        let (cursor, pressed, down) = ctx.input(|i| {
            (
                i.pointer.hover_pos(),
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
            )
        });
        let mut response = GizmoResponse::default();

        // Le geste en cours : appliqué, ou terminé si le bouton est relâché.
        if let Some(drag) = self.drag {
            match cursor.filter(|_| down) {
                Some(cursor) => {
                    self.apply(
                        &drag,
                        cursor,
                        center,
                        &axes,
                        parent,
                        transform,
                        snap,
                        camera_position,
                        origin,
                    );
                    response.active = true;
                }
                None => self.drag = None,
            }
        }

        // Sinon : la poignée sous le curseur (hors des fenêtres de l'interface).
        let hovered = match self.drag {
            Some(drag) => Some(drag.handle),
            None if ctx.is_pointer_over_egui() => None,
            None => {
                cursor.and_then(|c| self.handle_under(c, &screen, center, origin, &axes, length))
            }
        };
        if self.drag.is_none()
            && let (Some(handle), Some(cursor), true) = (hovered, cursor, pressed)
        {
            let screen_axis = match handle {
                Handle::Axis(i) => screen
                    .project(origin + axes[i] * length)
                    .map_or(egui::Vec2::ZERO, |end| end - center),
                Handle::Uniform => egui::vec2(100.0, 0.0),
            };
            self.drag = Some(Drag {
                handle,
                start: *transform,
                start_cursor: cursor,
                screen_axis,
                world_length: length,
            });
            response.started = true;
        }
        response.active |= hovered.is_some();

        self.paint(ctx, &screen, center, origin, &axes, length, hovered);
        response
    }

    /// Les trois axes du gizmo dans le monde : ceux du monde pour déplacer et
    /// tourner, ceux de l'objet pour l'agrandir (on agrandit dans SON repère).
    fn axes(&self, global: Mat4) -> [Vec3; 3] {
        match self.mode {
            GizmoMode::Scale => [
                global.x_axis.truncate().normalize_or(Vec3::X),
                global.y_axis.truncate().normalize_or(Vec3::Y),
                global.z_axis.truncate().normalize_or(Vec3::Z),
            ],
            _ => [Vec3::X, Vec3::Y, Vec3::Z],
        }
    }

    /// Les traits d'une poignée, à l'écran.
    fn handle_lines(
        &self,
        handle: Handle,
        screen: &Screen,
        center: egui::Pos2,
        origin: Vec3,
        axes: &[Vec3; 3],
        length: f32,
    ) -> Vec<egui::Pos2> {
        match (self.mode, handle) {
            (GizmoMode::Rotate, Handle::Axis(i)) => {
                // Un cercle dans le plan perpendiculaire à l'axe.
                let (u, v) = (axes[(i + 1) % 3], axes[(i + 2) % 3]);
                (0..=48)
                    .filter_map(|k| {
                        let a = k as f32 / 48.0 * std::f32::consts::TAU;
                        screen.project(origin + (u * a.cos() + v * a.sin()) * length)
                    })
                    .collect()
            }
            (_, Handle::Axis(i)) => screen
                .project(origin + axes[i] * length)
                .map_or_else(Vec::new, |end| vec![center, end]),
            (_, Handle::Uniform) => vec![center],
        }
    }

    fn handles(&self) -> Vec<Handle> {
        let mut handles = vec![Handle::Axis(0), Handle::Axis(1), Handle::Axis(2)];
        if self.mode == GizmoMode::Scale {
            handles.push(Handle::Uniform);
        }
        handles
    }

    /// La poignée la plus proche du curseur, s'il est assez près.
    fn handle_under(
        &self,
        cursor: egui::Pos2,
        screen: &Screen,
        center: egui::Pos2,
        origin: Vec3,
        axes: &[Vec3; 3],
        length: f32,
    ) -> Option<Handle> {
        self.handles()
            .into_iter()
            .map(|handle| {
                let lines = self.handle_lines(handle, screen, center, origin, axes, length);
                let distance = match lines.as_slice() {
                    [point] => point.distance(cursor),
                    _ => lines
                        .windows(2)
                        .map(|w| distance_to_segment(cursor, w[0], w[1]))
                        .fold(f32::INFINITY, f32::min),
                };
                (handle, distance)
            })
            .filter(|&(_, distance)| distance < GRAB_DISTANCE)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(handle, _)| handle)
    }

    /// Applique le geste : le `Transform` de départ, modifié selon le
    /// chemin parcouru par le curseur depuis le début du geste.
    #[allow(clippy::too_many_arguments)]
    fn apply(
        &self,
        drag: &Drag,
        cursor: egui::Pos2,
        center: egui::Pos2,
        axes: &[Vec3; 3],
        parent: Mat4,
        transform: &mut Transform,
        snap: bool,
        camera_position: Vec3,
        origin: Vec3,
    ) {
        let start = drag.start;
        let moved = cursor - drag.start_cursor;
        let along = axis_drag(
            Vec2::new(moved.x, moved.y),
            Vec2::new(drag.screen_axis.x, drag.screen_axis.y),
            drag.world_length,
        );
        let parent_inverse = parent.inverse();
        match (self.mode, drag.handle) {
            (GizmoMode::Translate, Handle::Axis(i)) => {
                let distance = if snap {
                    snapped(along, SNAP_DISTANCE)
                } else {
                    along
                };
                // Le déplacement dans le monde, ramené dans le repère du parent.
                let local = parent_inverse.transform_vector3(axes[i] * distance);
                transform.translation = start.translation + local;
            }
            (GizmoMode::Rotate, Handle::Axis(i)) => {
                // L'angle parcouru par le curseur autour du centre du gizmo.
                let angle_of = |p: egui::Pos2| (p.y - center.y).atan2(p.x - center.x);
                let mut angle = wrap_angle(angle_of(cursor) - angle_of(drag.start_cursor));
                // L'écran a son y vers le bas : un tour dans le sens des
                // aiguilles d'une montre y donne un angle positif. Vu depuis la
                // pointe de l'axe, une rotation positive tourne dans l'autre
                // sens : on inverse quand l'axe pointe vers la caméra.
                if axes[i].dot(camera_position - origin) > 0.0 {
                    angle = -angle;
                }
                if snap {
                    angle = snapped(angle.to_degrees(), SNAP_ANGLE).to_radians();
                }
                let (_, parent_rotation, _) = parent.to_scale_rotation_translation();
                let world = Quat::from_axis_angle(axes[i], angle);
                transform.rotation =
                    (parent_rotation.inverse() * world * parent_rotation * start.rotation)
                        .normalize();
            }
            (GizmoMode::Scale, handle) => {
                let mut factor = 1.0 + along / drag.world_length;
                if snap {
                    factor = snapped(factor, SNAP_SCALE);
                }
                let factor = factor.max(0.05);
                transform.scale = match handle {
                    Handle::Axis(i) => {
                        let mut scale = start.scale;
                        scale[i] = (scale[i] * factor).max(0.01);
                        scale
                    }
                    Handle::Uniform => (start.scale * factor).max(Vec3::splat(0.01)),
                };
            }
            _ => {}
        }
    }

    /// Le dessin, sous les fenêtres de l'interface. L'axe survolé ou tiré
    /// est en jaune.
    #[allow(clippy::too_many_arguments)]
    fn paint(
        &self,
        ctx: &egui::Context,
        screen: &Screen,
        center: egui::Pos2,
        origin: Vec3,
        axes: &[Vec3; 3],
        length: f32,
        hovered: Option<Handle>,
    ) {
        let painter = ctx.layer_painter(egui::LayerId::background());
        let colors = [
            egui::Color32::from_rgb(235, 70, 70),
            egui::Color32::from_rgb(90, 210, 90),
            egui::Color32::from_rgb(80, 130, 255),
        ];
        let highlight = egui::Color32::from_rgb(255, 220, 60);
        for handle in self.handles() {
            let color = if hovered == Some(handle) {
                highlight
            } else {
                match handle {
                    Handle::Axis(i) => colors[i],
                    Handle::Uniform => egui::Color32::WHITE,
                }
            };
            let stroke = egui::Stroke::new(3.0, color);
            let lines = self.handle_lines(handle, screen, center, origin, axes, length);
            match (self.mode, lines.as_slice()) {
                (_, [point]) => {
                    painter.rect_filled(
                        egui::Rect::from_center_size(*point, egui::vec2(12.0, 12.0)),
                        2.0,
                        color,
                    );
                }
                (GizmoMode::Rotate, _) => {
                    painter.add(egui::Shape::line(lines.clone(), stroke));
                }
                (mode, [start, end]) => {
                    painter.line_segment([*start, *end], stroke);
                    if mode == GizmoMode::Scale {
                        painter.rect_filled(
                            egui::Rect::from_center_size(*end, egui::vec2(10.0, 10.0)),
                            1.0,
                            color,
                        );
                    } else {
                        // Une pointe de flèche.
                        let direction = (*end - *start).normalized();
                        let side = egui::vec2(-direction.y, direction.x) * 6.0;
                        let back = *end - direction * 14.0;
                        painter.add(egui::Shape::convex_polygon(
                            vec![*end, back + side, back - side],
                            color,
                            egui::Stroke::NONE,
                        ));
                    }
                }
                _ => {}
            }
        }
    }
}

/// Le gizmo de l'entité `entity` : va chercher son `Transform` et la matrice
/// de son parent, puis appelle [`Gizmo::show`].
///
/// La matrice du parent vient de SON `GlobalTransform` (via le composant
/// `Parent`) et non de celui de l'entité : celui-ci a une image de retard
/// pendant un geste, ce qui ferait dériver l'objet.
pub fn show_gizmo(
    gizmo: &mut Gizmo,
    ctx: &egui::Context,
    world: &hecs::World,
    entity: Entity,
    view_projection: Mat4,
    camera_position: Vec3,
    snap: bool,
) -> GizmoResponse {
    let parent = world
        .get::<&Parent>(entity)
        .ok()
        .and_then(|p| world.get::<&GlobalTransform>(p.0).ok().map(|g| g.0))
        .unwrap_or(Mat4::IDENTITY);
    let Ok(mut transform) = world.get::<&mut Transform>(entity) else {
        return GizmoResponse::default();
    };
    let global = parent * transform.matrix();
    gizmo.show(
        ctx,
        view_projection,
        camera_position,
        global,
        parent,
        &mut transform,
        snap,
    )
}

/// Distance d'un point à un segment, à l'écran.
fn distance_to_segment(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2) -> f32 {
    let ab = b - a;
    let t = if ab.length_sq() > 0.0 {
        ((p - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    p.distance(a + ab * t)
}

/// Combien de mètres le long de l'axe correspondent à un déplacement du
/// curseur de `moved` points, quand l'axe mesure `screen_axis` à l'écran et
/// `world_length` mètres dans le monde. Seule la part du geste **dans la
/// direction de l'axe à l'écran** compte.
pub fn axis_drag(moved: Vec2, screen_axis: Vec2, world_length: f32) -> f32 {
    let pixels = screen_axis.length();
    if pixels < 1.0 {
        return 0.0;
    }
    moved.dot(screen_axis / pixels) / pixels * world_length
}

/// `value` arrondi au multiple de `step` le plus proche.
fn snapped(value: f32, step: f32) -> f32 {
    (value / step).round() * step
}

/// Un angle ramené entre -π et π.
fn wrap_angle(angle: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    (angle + PI).rem_euclid(TAU) - PI
}

/// Le rayon (origine, direction de longueur 1) qui part de la caméra et
/// passe par le point `cursor` de l'écran (de 0 à 1 en x et en y, y vers le bas).
///
/// On « dé-projette » deux points de ce pixel : l'un sur le plan proche de
/// la caméra (profondeur 0), l'autre sur le plan lointain (profondeur 1).
pub fn screen_ray(view_projection: Mat4, cursor: Vec2) -> (Vec3, Vec3) {
    let x = cursor.x * 2.0 - 1.0;
    let y = 1.0 - cursor.y * 2.0;
    let inverse = view_projection.inverse();
    let near = inverse.project_point3(Vec3::new(x, y, 0.0));
    let far = inverse.project_point3(Vec3::new(x, y, 1.0));
    (near, (far - near).normalize_or_zero())
}

/// L'entité la plus proche touchée par le rayon : un maillage (sa boîte
/// englobante, `bounds`) ou une lampe (une petite sphère autour d'elle).
pub fn pick(
    world: &hecs::World,
    bounds: impl Fn(MeshHandle) -> Option<Aabb>,
    origin: Vec3,
    direction: Vec3,
) -> Option<Entity> {
    let meshes = world
        .query::<(Entity, &GlobalTransform, &MeshRenderer)>()
        .iter()
        .filter_map(|(entity, global, renderer)| {
            // Dans le repère de l'objet, la boîte est alignée sur les axes. Le
            // rayon y est déformé (rotation, échelle), mais `t` reste le même :
            // le point touché est `origin + direction × t` dans les deux repères.
            let inverse = global.0.inverse();
            let t = ray_box(
                inverse.transform_point3(origin),
                inverse.transform_vector3(direction),
                &bounds(renderer.mesh)?,
            )?;
            Some((entity, t))
        })
        .collect::<Vec<_>>();
    let lights = world
        .query::<(
            Entity,
            &GlobalTransform,
            Option<&PointLight>,
            Option<&SpotLight>,
        )>()
        .without::<&MeshRenderer>()
        .iter()
        .filter(|(_, _, point, spot)| point.is_some() || spot.is_some())
        .filter_map(|(entity, global, _, _)| {
            let center = global.0.w_axis.truncate();
            Some((
                entity,
                ray_sphere(origin, direction, center, LIGHT_PICK_RADIUS)?,
            ))
        })
        .collect::<Vec<_>>();
    meshes
        .into_iter()
        .chain(lights)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(entity, _)| entity)
}

/// Le premier `t ≥ 0` où le rayon entre dans la boîte (méthode des
/// « tranches » : on coupe le rayon par les deux plans de chaque axe).
fn ray_box(origin: Vec3, direction: Vec3, aabb: &Aabb) -> Option<f32> {
    let mut t_min = 0.0_f32;
    let mut t_max = f32::INFINITY;
    for axis in 0..3 {
        let (o, d) = (origin[axis], direction[axis]);
        let (lo, hi) = (aabb.min[axis], aabb.max[axis]);
        if d.abs() < 1e-8 {
            // Parallèle à ces plans : il faut déjà être entre eux.
            if o < lo || o > hi {
                return None;
            }
            continue;
        }
        let (a, b) = ((lo - o) / d, (hi - o) / d);
        t_min = t_min.max(a.min(b));
        t_max = t_max.min(a.max(b));
        if t_min > t_max {
            return None;
        }
    }
    Some(t_min)
}

/// Le premier `t ≥ 0` où le rayon (direction de longueur 1) touche la sphère.
fn ray_sphere(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let to_center = center - origin;
    let along = to_center.dot(direction);
    let distance_sq = to_center.length_squared() - along * along;
    if distance_sq > radius * radius {
        return None;
    }
    let t = along - (radius * radius - distance_sq).sqrt();
    (t >= 0.0).then_some(t).or((along >= 0.0).then_some(0.0))
}

/// Les objets que l'éditeur sait créer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnKind {
    Cube,
    Sphere,
    Lamp,
}

impl SpawnKind {
    pub const ALL: [Self; 3] = [Self::Cube, Self::Sphere, Self::Lamp];

    pub fn label(self) -> &'static str {
        match self {
            Self::Cube => "Cube",
            Self::Sphere => "Sphère",
            Self::Lamp => "Lampe",
        }
    }
}

/// Crée un objet à `position`. Le cube et la sphère sont **fixes** et
/// solides (on peut marcher dessus, les balles rebondissent) ; la lampe
/// éclaire, sans forme de collision. `number` le distingue dans la liste.
pub fn spawn_object(
    world: &mut hecs::World,
    assets: &DemoAssets,
    kind: SpawnKind,
    position: Vec3,
    number: u32,
) -> Entity {
    let name = Name::new(format!("{} {number}", kind.label()));
    let transform = Transform::from_translation(position);
    let gray = Material::dielectric(Color::rgb(0.75, 0.75, 0.78), 0.5);
    match kind {
        SpawnKind::Cube => world.spawn((
            name,
            transform,
            MeshRenderer::new(assets.white_cube).with_material(gray),
            RigidBody::fixed(),
            Collider::cuboid(Vec3::splat(0.5)),
        )),
        SpawnKind::Sphere => world.spawn((
            name,
            transform,
            MeshRenderer::new(assets.sphere).with_material(gray),
            RigidBody::fixed(),
            Collider::ball(0.5),
        )),
        SpawnKind::Lamp => {
            let color = Color::rgb(1.0, 0.85, 0.6);
            world.spawn((
                name,
                transform.with_uniform_scale(0.2),
                MeshRenderer::new(assets.sphere).with_material(Material {
                    emissive: Color::rgb(4.0, 3.4, 2.4),
                    ..Material::dielectric(color, 0.5)
                }),
                PointLight {
                    color,
                    intensity: 2.0,
                    range: 6.0,
                },
            ))
        }
    }
}

/// Une modification de l'éditeur, avec de quoi la défaire et la refaire.
pub enum Edit {
    /// Un geste du gizmo : le `Transform` de l'objet avant et après.
    Transform {
        entity: Entity,
        before: Transform,
        after: Transform,
    },
    /// Des entités supprimées (`deleted`) ou créées par l'éditeur. Tant
    /// qu'elles sont absentes de la scène, leurs composants attendent dans
    /// `stash` (un `EntityBuilder` par entité).
    Presence {
        deleted: bool,
        entities: Vec<Entity>,
        stash: Vec<(Entity, hecs::EntityBuilder)>,
    },
}

/// Les entités qui ont changé d'identifiant pendant une annulation :
/// (ancien, nouveau).
pub type Remap = Vec<(Entity, Entity)>;

impl Edit {
    /// Applique la modification (`forward`) ou la défait.
    fn apply(&mut self, world: &mut hecs::World, forward: bool) -> Remap {
        match self {
            Self::Transform {
                entity,
                before,
                after,
            } => {
                if let Ok(mut transform) = world.get::<&mut Transform>(*entity) {
                    *transform = if forward { *after } else { *before };
                }
                Vec::new()
            }
            Self::Presence {
                deleted,
                entities,
                stash,
            } => {
                // Une création refaite, ou une suppression annulée : présentes.
                if forward != *deleted {
                    restore(world, entities, stash)
                } else {
                    put_away(world, entities, stash);
                    Vec::new()
                }
            }
        }
    }

    /// Les entités désignées par cette modification suivent leur nouvel identifiant.
    fn remap(&mut self, remap: &[(Entity, Entity)]) {
        let follow = |entity: &mut Entity| {
            if let Some(&(_, new)) = remap.iter().find(|(old, _)| old == entity) {
                *entity = new;
            }
        };
        match self {
            Self::Transform { entity, .. } => follow(entity),
            Self::Presence {
                entities, stash, ..
            } => {
                entities.iter_mut().for_each(follow);
                stash.iter_mut().for_each(|(entity, _)| follow(entity));
            }
        }
    }
}

/// Retire les entités de la scène en gardant leurs composants (tous, même
/// ceux que l'éditeur ne connaît pas) dans `stash`.
fn put_away(
    world: &mut hecs::World,
    entities: &[Entity],
    stash: &mut Vec<(Entity, hecs::EntityBuilder)>,
) {
    for &entity in entities {
        if let Ok(components) = world.take(entity) {
            let mut builder = hecs::EntityBuilder::new();
            builder.add_bundle(components);
            stash.push((entity, builder));
        }
    }
}

/// Remet dans la scène les entités de `stash`.
///
/// Elles reçoivent de **nouveaux** identifiants : réutiliser les anciens
/// (`World::spawn_at`) détruirait sans prévenir une entité créée entre-temps
/// au même numéro (une balle, par exemple). Les liens `Parent` entre elles
/// sont recâblés, et l'appelant fait suivre le reste (historique, sélection).
fn restore(
    world: &mut hecs::World,
    entities: &mut [Entity],
    stash: &mut Vec<(Entity, hecs::EntityBuilder)>,
) -> Remap {
    let remap: Remap = stash
        .drain(..)
        .map(|(old, mut builder)| (old, world.spawn(builder.build())))
        .collect();
    for &(_, new) in &remap {
        if let Ok(mut parent) = world.get::<&mut Parent>(new)
            && let Some(&(_, moved)) = remap.iter().find(|(old, _)| *old == parent.0)
        {
            parent.0 = moved;
        }
    }
    for entity in entities.iter_mut() {
        if let Some(&(_, new)) = remap.iter().find(|(old, _)| old == entity) {
            *entity = new;
        }
    }
    remap
}

/// L'historique des modifications de l'éditeur (voir le module).
#[derive(Default)]
pub struct History {
    undo: Vec<Edit>,
    redo: Vec<Edit>,
}

impl History {
    /// Une modification qui vient d'être faite. Une nouvelle modification
    /// efface ce qu'on pouvait refaire.
    pub fn record(&mut self, edit: Edit) {
        self.undo.push(edit);
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Supprime `entities` de la scène, d'une façon qu'on peut annuler.
    pub fn delete(&mut self, world: &mut hecs::World, entities: Vec<Entity>) {
        let mut edit = Edit::Presence {
            deleted: true,
            entities,
            stash: Vec::new(),
        };
        edit.apply(world, true);
        self.record(edit);
    }

    /// Oublie tout (la scène a été remplacée : `F9`).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Défait la dernière modification. Renvoie les entités qui ont changé
    /// d'identifiant (pour que la sélection les suive), ou `None` s'il n'y
    /// avait rien à annuler.
    pub fn undo(&mut self, world: &mut hecs::World) -> Option<Remap> {
        let mut edit = self.undo.pop()?;
        let remap = edit.apply(world, false);
        self.redo.push(edit);
        self.follow(&remap);
        Some(remap)
    }

    /// Refait la dernière modification annulée.
    pub fn redo(&mut self, world: &mut hecs::World) -> Option<Remap> {
        let mut edit = self.redo.pop()?;
        let remap = edit.apply(world, true);
        self.undo.push(edit);
        self.follow(&remap);
        Some(remap)
    }

    /// Toutes les modifications suivent les entités qui ont changé d'identifiant.
    fn follow(&mut self, remap: &[(Entity, Entity)]) {
        if remap.is_empty() {
            return;
        }
        for edit in self.undo.iter_mut().chain(&mut self.redo) {
            edit.remap(remap);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zentrix_render::Camera;
    use zentrix_scene::Scene;

    fn cube_at(scene: &mut Scene, transform: Transform) -> Entity {
        scene
            .world
            .spawn((transform, MeshRenderer::new(MeshHandle::from_index(0))))
    }

    /// Toutes les boîtes : un cube de côté 1 centré sur l'origine.
    fn unit_box(_: MeshHandle) -> Option<Aabb> {
        Some(Aabb {
            min: Vec3::splat(-0.5),
            max: Vec3::splat(0.5),
        })
    }

    #[test]
    fn le_centre_de_l_ecran_regarde_droit_devant() {
        let camera = Camera {
            position: Vec3::new(0.0, 2.0, 10.0),
            target: Vec3::new(0.0, 2.0, 0.0),
            ..Camera::default()
        };
        let (origin, direction) = screen_ray(camera.view_projection(16.0 / 9.0), Vec2::splat(0.5));
        assert!((origin - camera.position).length() < 0.5, "{origin}");
        assert!(direction.abs_diff_eq(Vec3::NEG_Z, 1e-4), "{direction}");
        // Le bord droit de l'écran : le rayon part vers la droite (+X).
        let (_, right) = screen_ray(camera.view_projection(16.0 / 9.0), Vec2::new(1.0, 0.5));
        assert!(right.x > 0.5);
    }

    #[test]
    fn le_rayon_choisit_l_objet_le_plus_proche() {
        let mut scene = Scene::default();
        let loin = cube_at(
            &mut scene,
            Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)),
        );
        let pres = cube_at(
            &mut scene,
            Transform::from_translation(Vec3::new(0.0, 0.0, -4.0)),
        );
        cube_at(
            &mut scene,
            Transform::from_translation(Vec3::new(5.0, 0.0, -4.0)),
        );
        scene.update_transforms();
        let hit = pick(&scene.world, unit_box, Vec3::ZERO, Vec3::NEG_Z);
        assert_eq!(hit, Some(pres));
        scene.world.despawn(pres).unwrap();
        assert_eq!(
            pick(&scene.world, unit_box, Vec3::ZERO, Vec3::NEG_Z),
            Some(loin)
        );
        // Vers le haut : rien.
        assert_eq!(pick(&scene.world, unit_box, Vec3::ZERO, Vec3::Y), None);
    }

    #[test]
    fn une_boite_tournee_et_agrandie_est_exacte() {
        let mut scene = Scene::default();
        // Un cube tourné de 45° autour de Y et agrandi 2 fois : son coin
        // avance jusqu'à √2 ≈ 1,41 m de son centre.
        let cube = cube_at(
            &mut scene,
            Transform::from_translation(Vec3::new(0.0, 0.0, -5.0))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_4))
                .with_uniform_scale(2.0),
        );
        scene.update_transforms();
        // À 1,3 m sur le côté : touché (le coin dépasse) ; à 1,5 m : manqué.
        let hit = |x: f32| pick(&scene.world, unit_box, Vec3::new(x, 0.0, 0.0), Vec3::NEG_Z);
        assert_eq!(hit(1.3), Some(cube));
        assert_eq!(hit(1.5), None);
    }

    #[test]
    fn une_lampe_se_selectionne_sans_maillage() {
        let mut scene = Scene::default();
        let lampe = scene.world.spawn((
            Transform::from_translation(Vec3::new(0.0, 0.0, -3.0)),
            PointLight::default(),
        ));
        scene.update_transforms();
        assert_eq!(
            pick(&scene.world, unit_box, Vec3::ZERO, Vec3::NEG_Z),
            Some(lampe)
        );
        assert_eq!(pick(&scene.world, unit_box, Vec3::X, Vec3::NEG_Z), None);
    }

    #[test]
    fn glisser_le_long_d_un_axe() {
        // L'axe mesure 100 points à l'écran pour 2 m dans le monde.
        let axis = Vec2::new(100.0, 0.0);
        assert_eq!(axis_drag(Vec2::new(50.0, 0.0), axis, 2.0), 1.0);
        // Un geste perpendiculaire à l'axe ne compte pas.
        assert_eq!(axis_drag(Vec2::new(0.0, 80.0), axis, 2.0), 0.0);
        // En diagonale : seule la part dans le sens de l'axe compte.
        assert_eq!(axis_drag(Vec2::new(-25.0, 40.0), axis, 2.0), -0.5);
        // Un axe vu de face (presque un point à l'écran) : pas de mouvement.
        assert_eq!(
            axis_drag(Vec2::new(50.0, 0.0), Vec2::new(0.2, 0.0), 2.0),
            0.0
        );
    }

    #[test]
    fn crans_et_angles() {
        assert_eq!(snapped(0.37, 0.25), 0.25);
        assert_eq!(snapped(0.38, 0.25), 0.5);
        assert!(
            (wrap_angle(3.0 * std::f32::consts::PI / 2.0) + std::f32::consts::FRAC_PI_2).abs()
                < 1e-5
        );
    }

    #[test]
    fn creer_des_objets() {
        let mut scene = Scene::default();
        let assets = crate::demo::tests::assets();
        for (i, kind) in SpawnKind::ALL.into_iter().enumerate() {
            spawn_object(
                &mut scene.world,
                &assets,
                kind,
                Vec3::X * i as f32,
                i as u32,
            );
        }
        // Le cube et la sphère sont solides, la lampe éclaire.
        assert_eq!(scene.world.query::<&Collider>().iter().count(), 2);
        assert_eq!(scene.world.query::<&PointLight>().iter().count(), 1);
        scene.update_transforms();
        // Et tous se sélectionnent d'un clic.
        let hit = pick(
            &scene.world,
            unit_box,
            Vec3::new(2.0, 0.0, 5.0),
            Vec3::NEG_Z,
        );
        let name = scene.world.get::<&Name>(hit.unwrap()).unwrap().0.clone();
        assert_eq!(name, "Lampe 2");
    }

    fn deplacement(entity: Entity, x: f32) -> Edit {
        Edit::Transform {
            entity,
            before: Transform::IDENTITY,
            after: Transform::from_translation(Vec3::X * x),
        }
    }

    fn x(world: &hecs::World, entity: Entity) -> f32 {
        world.get::<&Transform>(entity).unwrap().translation.x
    }

    #[test]
    fn annuler_puis_refaire_un_geste() {
        let mut world = hecs::World::new();
        let cube = world.spawn((Transform::from_translation(Vec3::X * 2.0),));
        let mut history = History::default();
        assert!(!history.can_undo());
        history.record(deplacement(cube, 2.0));

        history.undo(&mut world).unwrap();
        assert_eq!(x(&world, cube), 0.0);
        assert!(!history.can_undo() && history.can_redo());
        history.redo(&mut world).unwrap();
        assert_eq!(x(&world, cube), 2.0);

        // Une nouvelle modification efface ce qu'on pouvait refaire.
        history.undo(&mut world);
        history.record(deplacement(cube, 5.0));
        assert!(!history.can_redo());
        // La limite : les plus anciennes modifications sont oubliées.
        for _ in 0..HISTORY_LIMIT + 10 {
            history.record(deplacement(cube, 1.0));
        }
        assert_eq!(history.undo.len(), HISTORY_LIMIT);
    }

    #[test]
    fn annuler_une_suppression_rend_tout_et_suit_les_identifiants() {
        let mut scene = Scene::default();
        let parent = scene
            .world
            .spawn((Name::new("Parent"), Transform::IDENTITY));
        let enfant = scene.world.spawn((
            Name::new("Enfant"),
            Transform::from_translation(Vec3::Y),
            Parent(parent),
        ));
        let mut history = History::default();
        // Un geste sur le parent, puis la suppression du parent et de son enfant.
        history.record(deplacement(parent, 3.0));
        scene
            .world
            .get::<&mut Transform>(parent)
            .unwrap()
            .translation
            .x = 3.0;
        let mut branche = scene.descendants(parent);
        branche.insert(0, parent);
        history.delete(&mut scene.world, branche);
        assert_eq!(scene.entity_count(), 0);

        // Annuler : les deux reviennent, avec de nouveaux identifiants, et
        // l'enfant est toujours rattaché à son parent.
        let remap = history.undo(&mut scene.world).unwrap();
        assert_eq!(remap.len(), 2);
        let nouveau = |old: Entity| remap.iter().find(|(o, _)| *o == old).unwrap().1;
        let (parent2, enfant2) = (nouveau(parent), nouveau(enfant));
        assert_eq!(scene.world.get::<&Parent>(enfant2).unwrap().0, parent2);
        assert_eq!(scene.world.get::<&Name>(enfant2).unwrap().0, "Enfant");

        // Le geste plus ancien vise maintenant le parent restauré.
        history.undo(&mut scene.world).unwrap();
        assert_eq!(x(&scene.world, parent2), 0.0);

        // Tout refaire : le geste, puis la suppression.
        history.redo(&mut scene.world).unwrap();
        assert_eq!(x(&scene.world, parent2), 3.0);
        history.redo(&mut scene.world).unwrap();
        assert_eq!(scene.entity_count(), 0);
    }

    #[test]
    fn annuler_une_creation() {
        let mut world = hecs::World::new();
        let cube = world.spawn((Name::new("Cube 1"), Transform::IDENTITY));
        let mut history = History::default();
        history.record(Edit::Presence {
            deleted: false,
            entities: vec![cube],
            stash: Vec::new(),
        });
        history.undo(&mut world).unwrap();
        assert!(!world.contains(cube));
        let remap = history.redo(&mut world).unwrap();
        assert_eq!(world.get::<&Name>(remap[0].1).unwrap().0, "Cube 1");
    }

    /// Le coût ne dépend plus de la taille de la scène : supprimer puis
    /// rendre un objet parmi des milliers est immédiat.
    #[test]
    fn l_historique_ne_coute_rien_dans_une_grosse_scene() {
        let mut world = hecs::World::new();
        for i in 0..5_000 {
            world.spawn((Transform::from_translation(Vec3::X * i as f32),));
        }
        let cube = world.spawn((Name::new("Cube"), Transform::IDENTITY));
        let mut history = History::default();
        let start = std::time::Instant::now();
        history.delete(&mut world, vec![cube]);
        history.undo(&mut world);
        history.redo(&mut world);
        assert!(start.elapsed().as_millis() < 20, "{:?}", start.elapsed());
    }
}
