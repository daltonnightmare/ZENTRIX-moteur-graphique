//! Les **matériaux** : de quoi est fait un objet (plastique, métal, bois...).

use crate::Color;

/// Comment l'**alpha** (la 4e composante de la couleur : 1 = plein, 0 =
/// invisible) est utilisé. Les trois modes de glTF.
///
/// ```text
///   Opaque              Mask (découpe)          Blend (mélange)
///   ┌────────┐          ┌─┐  ┌─┐  ┌─┐           ┌────────┐
///   │████████│          │█│  │█│  │█│           │▒▒▒▒▒▒▒▒│  on voit à travers,
///   │████████│          └─┘  └─┘  └─┘           │▒▒▒▒▒▒▒▒│  teinté
///   └────────┘          trous nets              └────────┘
///   alpha ignoré        alpha < seuil : rien    couleur mélangée au fond
/// ```
///
/// La **découpe** sert au feuillage, aux grillages, aux cheveux : chaque
/// pixel est entièrement là ou entièrement absent, l'objet reste traité
/// comme un opaque (aucun tri, ombres trouées). Le **mélange** sert au
/// verre, à l'eau, à la fumée : plus coûteux, il impose de dessiner ces
/// objets après les autres, du plus loin au plus proche.
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum AlphaMode {
    /// L'alpha est ignoré : l'objet cache tout ce qui est derrière lui.
    #[default]
    Opaque,
    /// Les pixels dont l'alpha est sous `cutoff` ne sont pas dessinés.
    Mask { cutoff: f32 },
    /// La couleur est mélangée à ce qui est derrière, selon l'alpha.
    Blend,
}

/// L'aspect d'une surface, au sens **PBR** (« Physically Based Rendering »,
/// rendu fondé sur la physique).
///
/// Au lieu de régler des valeurs arbitraires (« brillance 32 »), on décrit la
/// matière avec deux nombres qui ont un sens physique. C'est le modèle
/// « métal / rugosité » de glTF, de Blender, d'Unreal ou de Godot : un
/// matériau réglé ailleurs a le même aspect ici.
///
/// | Matière          | `metallic` | `roughness` |
/// |------------------|------------|-------------|
/// | plastique brillant | 0.0      | 0.2         |
/// | bois, craie      | 0.0        | 0.9         |
/// | or poli          | 1.0        | 0.15        |
/// | métal brossé     | 1.0        | 0.5         |
///
/// ```
/// use zentrix_core::{Color, Material};
///
/// let or = Material::metal(Color::rgb(1.0, 0.77, 0.34), 0.15);
/// assert_eq!(or.metallic, 1.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
// `default` : un champ absent du fichier de scène prend sa valeur par défaut.
#[serde(default)]
pub struct Material {
    /// Multiplie la couleur des sommets et de la texture (linéaire). Pour un
    /// métal, c'est la couleur de ses reflets.
    pub base_color: Color,
    /// 0 = isolant (plastique, bois, pierre), 1 = métal. Les valeurs
    /// intermédiaires servent surtout aux transitions (métal rouillé).
    pub metallic: f32,
    /// 0 = lisse comme un miroir (reflet petit et intense), 1 = mat (reflet
    /// étalé sur toute la surface).
    pub roughness: f32,
    /// Intensité du relief de la normal map (s'il y en a une) : 0 = surface
    /// lisse, 1 = relief de l'image, 2 = relief exagéré.
    pub normal_scale: f32,
    /// La lumière **émise** par l'objet lui-même (linéaire) : un écran, un
    /// néon, de la lave. Noir = n'émet rien. Elle s'ajoute à l'éclairage
    /// (elle ne dépend ni du soleil ni des ombres) mais n'éclaire pas les
    /// autres objets. Multipliée par la texture d'émission, s'il y en a une.
    pub emissive: Color,
    /// Intensité de la texture d'occlusion (s'il y en a une) : 0 = ignorée,
    /// 1 = appliquée pleinement.
    pub occlusion_strength: f32,
    /// Ce que fait l'alpha de `base_color` (et de la texture de couleur).
    pub alpha_mode: AlphaMode,
    /// Les deux faces des triangles sont-elles dessinées ? Par défaut, les
    /// faces « arrière » sont ignorées : on ne voit jamais l'intérieur d'un
    /// objet fermé. Mais à travers un grillage ou du verre, on le voit ; et
    /// une simple feuille (un plan) doit être visible des deux côtés.
    pub double_sided: bool,
}

impl Material {
    /// Un isolant blanc, assez mat : l'aspect par défaut.
    pub const DEFAULT: Material = Material {
        base_color: Color::WHITE,
        metallic: 0.0,
        roughness: 0.5,
        normal_scale: 1.0,
        emissive: Color::BLACK,
        occlusion_strength: 1.0,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
    };

    /// Un isolant (plastique, bois, pierre...) de rugosité donnée.
    pub const fn dielectric(base_color: Color, roughness: f32) -> Self {
        Self {
            base_color,
            roughness,
            ..Self::DEFAULT
        }
    }

    /// Un métal de rugosité donnée ; `base_color` est la couleur de ses reflets.
    pub const fn metal(base_color: Color, roughness: f32) -> Self {
        Self {
            base_color,
            metallic: 1.0,
            roughness,
            ..Self::DEFAULT
        }
    }

    /// Du **verre** teinté : lisse, transparent (`alpha` : 0 = invisible,
    /// 1 = opaque), visible des deux côtés. Ses reflets restent vifs même
    /// très transparent (voir mesh.wgsl).
    pub const fn glass(color: Color, alpha: f32) -> Self {
        Self {
            base_color: Color::rgba(color.r, color.g, color.b, alpha),
            roughness: 0.05,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            ..Self::DEFAULT
        }
    }

    /// L'objet est-il dessiné avec mélange (donc après les opaques, trié) ?
    pub fn is_transparent(&self) -> bool {
        self.alpha_mode == AlphaMode::Blend
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_champ_absent_prend_sa_valeur_par_defaut() {
        let m: Material = ron::from_str("(metallic: 1.0)").unwrap();
        assert_eq!(m.metallic, 1.0);
        assert_eq!(m.roughness, Material::DEFAULT.roughness);
        assert_eq!(m.base_color, Color::WHITE);
        assert_eq!(m.alpha_mode, AlphaMode::Opaque);
        assert!(!m.double_sided);
    }

    #[test]
    fn modes_alpha_en_ron() {
        let m: Material =
            ron::from_str("(alpha_mode: Mask(cutoff: 0.4), double_sided: true)").unwrap();
        assert_eq!(m.alpha_mode, AlphaMode::Mask { cutoff: 0.4 });
        assert!(m.double_sided && !m.is_transparent());
        let verre = Material::glass(Color::BLUE, 0.3);
        assert!(verre.is_transparent());
        assert_eq!(verre.base_color.a, 0.3);
        let texte = ron::to_string(&verre).unwrap();
        assert_eq!(ron::from_str::<Material>(&texte).unwrap(), verre);
    }
}
