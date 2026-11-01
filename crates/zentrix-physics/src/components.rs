//! Les composants de la physique : de simples données, comme les autres.

use zentrix_core::math::Vec3;

/// Comment un corps réagit aux forces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BodyKind {
    /// Soumis à la gravité et aux chocs : c'est la physique qui le déplace
    /// (une caisse qui tombe, une balle). Son `Transform` est **écrit** par
    /// la physique.
    Dynamic,
    /// Ne bouge jamais (le sol, un mur). Les autres corps rebondissent dessus.
    Fixed,
    /// Déplacé par le **jeu** (son `Transform` est **lu** par la physique) :
    /// une plateforme mobile, une porte, nos cubes qui tournent. Il pousse
    /// les corps dynamiques mais rien ne le pousse.
    Kinematic,
}

/// « Cette entité est un corps physique. » Il lui faut aussi un
/// [`Collider`] (sa forme) et un `Transform`.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct RigidBody {
    pub kind: BodyKind,
    /// Vitesse au moment de la création du corps, en unités par seconde
    /// (une balle lancée). Ensuite, c'est la simulation qui la fait évoluer.
    pub initial_velocity: Vec3,
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::dynamic()
    }
}

impl RigidBody {
    pub fn dynamic() -> Self {
        Self {
            kind: BodyKind::Dynamic,
            initial_velocity: Vec3::ZERO,
        }
    }

    pub fn fixed() -> Self {
        Self {
            kind: BodyKind::Fixed,
            ..Self::dynamic()
        }
    }

    pub fn kinematic() -> Self {
        Self {
            kind: BodyKind::Kinematic,
            ..Self::dynamic()
        }
    }

    /// Variante « builder » : la vitesse de départ.
    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.initial_velocity = velocity;
        self
    }
}

/// La forme utilisée pour les collisions. Elle est souvent plus simple que
/// le maillage affiché (une boîte pour une caisse, une sphère pour une
/// balle) : les calculs de collision sont bien plus rapides.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Shape {
    /// Une boîte, décrite par ses **demi**-dimensions : `(0.5, 0.5, 0.5)`
    /// est un cube de côté 1, comme `MeshData::cube`.
    Cuboid { half_extents: Vec3 },
    /// Une sphère.
    Ball { radius: f32 },
    /// Une capsule verticale : un cylindre de hauteur `2 × half_height`
    /// terminé par deux demi-sphères de rayon `radius`. La forme classique
    /// d'un personnage : arrondie en bas, elle glisse sur les bords des marches.
    Capsule { half_height: f32, radius: f32 },
}

/// « Cette entité a cette forme de collision. »
///
/// La forme est exprimée dans le repère de l'entité : elle est agrandie par
/// le `scale` du `Transform`, au moment de la création du corps.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Collider {
    pub shape: Shape,
    /// Décalage de la forme par rapport à l'origine de l'entité.
    pub offset: Vec3,
    /// Frottement : 0 = glace, 1 = caoutchouc.
    pub friction: f32,
    /// « Rebond » : 0 = aucun (pâte à modeler), 1 = rebond parfait.
    pub restitution: f32,
    /// Masse par unité de volume : un cube de côté 1 et de densité 1 pèse 1.
    pub density: f32,
}

impl Default for Collider {
    fn default() -> Self {
        Self::cuboid(Vec3::splat(0.5))
    }
}

impl Collider {
    /// Une boîte de demi-dimensions `half_extents`.
    pub fn cuboid(half_extents: Vec3) -> Self {
        Self {
            shape: Shape::Cuboid { half_extents },
            offset: Vec3::ZERO,
            friction: 0.6,
            restitution: 0.1,
            density: 1.0,
        }
    }

    /// Une sphère de rayon `radius`.
    pub fn ball(radius: f32) -> Self {
        Self {
            shape: Shape::Ball { radius },
            ..Self::cuboid(Vec3::ZERO)
        }
    }

    /// Une capsule verticale (voir [`Shape::Capsule`]), centrée sur
    /// l'origine de l'entité : sa hauteur totale est `2 × (half_height + radius)`.
    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self {
            shape: Shape::Capsule {
                half_height,
                radius,
            },
            ..Self::cuboid(Vec3::ZERO)
        }
    }

    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }
}

/// « Cette entité est un **personnage** » : elle marche, saute, monte les
/// marches et glisse le long des murs, au lieu de rouler comme une caisse.
///
/// Il lui faut aussi un `Transform` (la position de ses **pieds**), un
/// [`RigidBody::kinematic`] et un [`Collider`] (une capsule, en général,
/// décalée vers le haut pour que son bas touche les pieds).
///
/// Le jeu écrit [`desired_velocity`](Self::desired_velocity) et
/// [`jump`](Self::jump) ; la physique déplace le personnage pendant
/// `Physics::step` (contrôleur cinématique de rapier) et met à jour
/// [`grounded`](Self::grounded) et [`vertical_speed`](Self::vertical_speed).
///
/// Pourquoi pas un corps dynamique ? Une capsule dynamique se comporte comme
/// un objet : elle glisse sur les pentes, rebondit, se fait pousser par une
/// caisse, accélère lentement. Un personnage doit au contraire obéir
/// **immédiatement** aux touches : on calcule son déplacement nous-mêmes, en
/// « sondant » le décor avec sa forme (*shape cast*).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct CharacterController {
    /// La vitesse voulue, en unités par seconde. Seule la partie
    /// **horizontale** compte : la verticale vient de la gravité et des sauts.
    pub desired_velocity: Vec3,
    /// Le jeu demande un saut. Pris en compte seulement au sol, puis remis
    /// à `false` par la physique.
    pub jump: bool,
    /// Vitesse verticale au départ d'un saut (5 m/s : environ 1,3 m de haut).
    pub jump_speed: f32,
    /// Hauteur maximale des marches franchies sans sauter.
    pub step_height: f32,
    /// La pente la plus raide que le personnage peut gravir, en radians.
    pub max_slope: f32,
    /// Sa masse, en kilogrammes : elle règle la force avec laquelle il pousse
    /// les objets dynamiques qu'il rencontre.
    pub mass: f32,
    /// Écrit par la physique : la vitesse verticale actuelle (positive en
    /// montant, négative en tombant).
    pub vertical_speed: f32,
    /// Écrit par la physique : le personnage touche-t-il le sol ?
    pub grounded: bool,
}

impl Default for CharacterController {
    fn default() -> Self {
        Self {
            desired_velocity: Vec3::ZERO,
            jump: false,
            jump_speed: 5.0,
            step_height: 0.35,
            max_slope: 45f32.to_radians(),
            mass: 70.0,
            vertical_speed: 0.0,
            grounded: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_composant_incomplet_se_relit_avec_les_valeurs_par_defaut() {
        let body: RigidBody = ron::from_str("(kind: Fixed)").unwrap();
        assert_eq!(body, RigidBody::fixed());
        let collider: Collider = ron::from_str("(shape: Ball(radius: 2.0))").unwrap();
        assert_eq!(collider.shape, Shape::Ball { radius: 2.0 });
        assert_eq!(collider.friction, Collider::default().friction);
    }
}
