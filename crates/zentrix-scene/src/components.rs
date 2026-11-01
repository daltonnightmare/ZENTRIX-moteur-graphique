//! Les composants fournis par le moteur.
//!
//! Un **composant** est une simple donnée attachée à une entité. Le moteur
//! en fournit quelques-uns ; le jeu peut définir les siens (n'importe quelle
//! struct convient : `struct Vitesse(f32);` est un composant valide).
//!
//! `Transform` (position, rotation, taille) vient de `zentrix-core` et sert
//! directement de composant.

use zentrix_core::{Color, Material};
use zentrix_render::{MaterialTextures, MeshHandle, TextureHandle};

/// « Cette entité est visible : dessine ce maillage avec ces textures et
/// ce matériau. »
///
/// Associé à un `Transform`, il produit un `DrawCommand` à chaque image
/// (voir `Scene::extract_draws`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshRenderer {
    pub mesh: MeshHandle,
    /// Les textures du matériau (couleur, relief, métal/rugosité, occlusion,
    /// émission), toutes facultatives.
    pub textures: MaterialTextures,
    /// De quoi l'objet est fait (couleur, métal, rugosité).
    pub material: Material,
}

impl MeshRenderer {
    /// Un maillage sans texture, avec le matériau par défaut.
    pub fn new(mesh: MeshHandle) -> Self {
        Self {
            mesh,
            textures: MaterialTextures::default(),
            material: Material::DEFAULT,
        }
    }

    /// Variante « builder » : ajoute une texture de couleur.
    pub fn with_texture(mut self, texture: TextureHandle) -> Self {
        self.textures.base_color = Some(texture);
        self
    }

    /// Variante « builder » : ajoute une normal map.
    pub fn with_normal_map(mut self, normal_map: TextureHandle) -> Self {
        self.textures.normal = Some(normal_map);
        self
    }

    /// Variante « builder » : choisit toutes les textures du matériau.
    pub fn with_textures(mut self, textures: MaterialTextures) -> Self {
        self.textures = textures;
        self
    }

    /// Variante « builder » : choisit le matériau.
    pub fn with_material(mut self, material: Material) -> Self {
        self.material = material;
        self
    }
}

/// « Cette entité est une caméra. »
///
/// Seuls les **réglages de l'objectif** sont ici. La position et
/// l'orientation viennent du `Transform` de la même entité : la caméra
/// regarde vers le -Z de son repère, avec son +Y comme « haut ». On peut donc
/// la déplacer comme n'importe quel objet de la scène.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Camera3d {
    /// Champ de vision vertical, en radians.
    pub fov_y: f32,
    /// Plan de coupe proche (jamais 0).
    pub near: f32,
    /// Plan de coupe lointain.
    pub far: f32,
}

impl Default for Camera3d {
    fn default() -> Self {
        Self {
            fov_y: 60f32.to_radians(),
            near: 0.1,
            far: 200.0,
        }
    }
}

/// « Cette entité est une lumière directionnelle » (un soleil).
///
/// Comme pour `Camera3d`, la direction vient du `Transform` de l'entité :
/// les rayons voyagent vers le -Z de son repère. Faire tourner l'entité fait
/// donc tourner le soleil (cycle jour/nuit, par exemple).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DirectionalLight {
    /// Couleur de la lumière (linéaire).
    pub color: Color,
    /// Multiplie la couleur. 1.0 = normal.
    pub intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
}

/// « Cette entité est une **lampe** » : elle éclaire dans toutes les
/// directions autour de la position de son `Transform`, jusqu'à `range`
/// mètres (une ampoule, une flamme, un néon).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct PointLight {
    pub color: Color,
    /// L'éclairage à 1 m de la lampe (1 = une surface blanche face à elle
    /// est blanche). Il diminue en 1 / distance².
    pub intensity: f32,
    /// La portée, en mètres : au-delà, la lampe n'éclaire plus.
    pub range: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
            range: 8.0,
        }
    }
}

/// « Cette entité est un **projecteur** » : comme une lampe, mais seulement
/// dans un cône, vers le -Z de son `Transform` (comme `DirectionalLight`).
/// Une lampe torche, un phare, un spot de scène.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SpotLight {
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
    /// Le demi-angle du cône en pleine lumière, en radians.
    pub inner_angle: f32,
    /// Le demi-angle au-delà duquel il n'y a plus de lumière, en radians.
    /// Entre les deux, la lumière s'estompe : un bord de cône doux.
    pub outer_angle: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 4.0,
            range: 12.0,
            inner_angle: 20f32.to_radians(),
            outer_angle: 30f32.to_radians(),
        }
    }
}

/// Un nom lisible pour une entité, utile dans les logs et les futurs outils
/// (inspecteur de scène, éditeur).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}
