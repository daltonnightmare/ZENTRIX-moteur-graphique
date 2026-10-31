//! Les composants fournis par le moteur.
//!
//! Un **composant** est une simple donnée attachée à une entité. Le moteur
//! en fournit quelques-uns ; le jeu peut définir les siens (n'importe quelle
//! struct convient : `struct Vitesse(f32);` est un composant valide).
//!
//! `Transform` (position, rotation, taille) vient de `zentrix-core` et sert
//! directement de composant.

use zentrix_render::{MeshHandle, TextureHandle};

/// « Cette entité est visible : dessine ce maillage avec cette texture. »
///
/// Associé à un `Transform`, il produit un `DrawCommand` à chaque image
/// (voir `Scene::extract_draws`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshRenderer {
    pub mesh: MeshHandle,
    /// `None` : seule la couleur des sommets compte.
    pub texture: Option<TextureHandle>,
}

impl MeshRenderer {
    /// Un maillage sans texture.
    pub fn new(mesh: MeshHandle) -> Self {
        Self {
            mesh,
            texture: None,
        }
    }

    /// Variante « builder » : ajoute une texture.
    pub fn with_texture(mut self, texture: TextureHandle) -> Self {
        self.texture = Some(texture);
        self
    }
}

/// « Cette entité est une caméra. »
///
/// Seuls les **réglages de l'objectif** sont ici. La position et
/// l'orientation viennent du `Transform` de la même entité : la caméra
/// regarde vers le -Z de son repère, avec son +Y comme « haut ». On peut donc
/// la déplacer comme n'importe quel objet de la scène.
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Un nom lisible pour une entité, utile dans les logs et les futurs outils
/// (inspecteur de scène, éditeur).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(pub String);

impl Name {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}
