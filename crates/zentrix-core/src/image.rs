//! [`Image`] : une image en mémoire, pixel par pixel.

use crate::Color;

/// Une image **RGBA 8 bits** : 4 octets par pixel (rouge, vert, bleu, opacité),
/// chacun entre 0 et 255, rangés ligne par ligne en partant du haut.
///
/// C'est un simple paquet de données : `zentrix-assets` en fabrique à partir de
/// fichiers PNG/JPEG, `zentrix-render` les envoie au GPU. Ni l'un ni l'autre ne
/// dépend de l'autre : ils partagent seulement ce type du noyau.
///
/// ```text
///  pixels : [ R G B A | R G B A | R G B A | ... ]
///             (0, 0)    (1, 0)    (2, 0)          ← première ligne (le haut)
/// ```
///
/// Les couleurs sont en **sRGB** (l'espace des fichiers image et des écrans).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    /// `width * height * 4` octets.
    pub pixels: Vec<u8>,
}

impl Image {
    /// Crée une image à partir de pixels déjà décodés.
    ///
    /// # Panics
    ///
    /// Si `pixels` n'a pas exactement `width * height * 4` octets : c'est une
    /// erreur de programmation, pas une situation à gérer.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        assert_eq!(
            pixels.len(),
            width as usize * height as usize * 4,
            "une image {width}x{height} RGBA doit faire {} octets",
            width as usize * height as usize * 4
        );
        Self {
            width,
            height,
            pixels,
        }
    }

    /// Une image d'une seule couleur.
    pub fn solid(width: u32, height: u32, color: Color) -> Self {
        let rgba = color.to_srgba8();
        Self::from_fn(width, height, |_, _| rgba)
    }

    /// Un damier de `cells × cells` cases, alternant `a` et `b`.
    /// Idéal pour un sol : on perçoit bien la perspective et le mouvement.
    pub fn checkerboard(size: u32, cells: u32, a: Color, b: Color) -> Self {
        let (a, b) = (a.to_srgba8(), b.to_srgba8());
        let cell = (size / cells.max(1)).max(1);
        Self::from_fn(size, size, |x, y| {
            // `^` (ou exclusif) : vrai quand exactement une des deux parités
            // est impaire. C'est ce qui fait alterner les cases.
            if ((x / cell) % 2 == 1) ^ ((y / cell) % 2 == 1) {
                b
            } else {
                a
            }
        })
    }

    /// Construit une image en appelant `f(x, y)` pour chaque pixel.
    ///
    /// `impl Fn(u32, u32) -> [u8; 4]` : « n'importe quelle fonction ou closure
    /// qui prend deux `u32` et renvoie 4 octets ».
    pub fn from_fn(width: u32, height: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Self {
        let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
        for y in 0..height {
            for x in 0..width {
                pixels.extend_from_slice(&f(x, y));
            }
        }
        Self::new(width, height, pixels)
    }

    /// Le pixel en `(x, y)`.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = (y as usize * self.width as usize + x as usize) * 4;
        // `try_into` convertit la tranche de 4 octets en tableau `[u8; 4]`.
        self.pixels[i..i + 4]
            .try_into()
            .expect("tranche de 4 octets")
    }

    /// La version deux fois plus petite de l'image (moyenne de chaque bloc 2×2).
    ///
    /// Une dimension impaire est arrondie vers le bas, sans descendre sous 1.
    /// Simplification assumée : la moyenne est faite directement sur les
    /// valeurs sRGB (et non en espace linéaire), ce qui assombrit très
    /// légèrement les contrastes forts. Invisible pour nos usages actuels.
    pub fn downsample(&self) -> Image {
        let width = (self.width / 2).max(1);
        let height = (self.height / 2).max(1);
        Image::from_fn(width, height, |x, y| {
            // Les (jusqu'à) 4 pixels sources ; `.min(...)` évite de sortir de
            // l'image quand une dimension vaut déjà 1.
            let x0 = (x * 2).min(self.width - 1);
            let y0 = (y * 2).min(self.height - 1);
            let x1 = (x0 + 1).min(self.width - 1);
            let y1 = (y0 + 1).min(self.height - 1);
            let samples = [
                self.pixel(x0, y0),
                self.pixel(x1, y0),
                self.pixel(x0, y1),
                self.pixel(x1, y1),
            ];
            // `std::array::from_fn` construit le tableau canal par canal.
            std::array::from_fn(|c| {
                let sum: u32 = samples.iter().map(|p| p[c] as u32).sum();
                ((sum + 2) / 4) as u8 // +2 : arrondi au plus proche
            })
        })
    }

    /// La **chaîne de mipmaps** : l'image, puis ses réductions successives
    /// jusqu'à 1×1 pixel.
    ///
    /// # Pourquoi des mipmaps ?
    ///
    /// Un sol en damier vu de loin couvre peu de pixels à l'écran, mais beaucoup
    /// de cases de la texture : le GPU en « saute » la plupart, et l'image
    /// fourmille (effet de moiré). Avec les mipmaps, il lit une version
    /// pré-réduite adaptée à la distance : l'image reste stable.
    ///
    /// Coût : seulement +33 % de mémoire (1/4 + 1/16 + ... ≈ 1/3).
    pub fn mip_chain(&self) -> Vec<Image> {
        let mut chain = vec![self.clone()];
        loop {
            let last = chain.last().expect("la chaîne n'est jamais vide");
            if last.width == 1 && last.height == 1 {
                break;
            }
            let next = last.downsample();
            chain.push(next);
        }
        chain
    }

    /// Fabrique une **normal map** à partir d'une carte de hauteurs.
    ///
    /// `height(x, y)` donne l'altitude du relief en chaque pixel (en
    /// « pixels de hauteur » : 1 = autant qu'un pixel de large). L'image est
    /// supposée **répétée** : les bords se raccordent (`x = -1` est lu en
    /// `x = width - 1`). `strength` exagère (> 1) ou adoucit (< 1) le relief.
    ///
    /// Chaque pixel reçoit la normale de la surface `z = hauteur(x, y)`,
    /// encodée en couleur : `composante × 0,5 + 0,5`. Une zone plate donne
    /// (0, 0, 1), soit le bleu-violet (128, 128, 255) typique des normal maps.
    /// Convention « OpenGL » (celle de glTF et de Blender) : le vert pointe
    /// vers le **haut** de l'image.
    ///
    /// Attention : une normal map contient des **directions**, pas des
    /// couleurs. Elle doit être envoyée au GPU sans conversion sRGB
    /// (`Renderer::add_normal_map`).
    pub fn normal_map_from_heights(
        width: u32,
        height: u32,
        strength: f32,
        height_at: impl Fn(u32, u32) -> f32,
    ) -> Self {
        let (w, h) = (width.max(1), height.max(1));
        // Les hauteurs, calculées une seule fois.
        let heights: Vec<f32> = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .map(|(x, y)| height_at(x, y))
            .collect();
        let at = |x: u32, y: u32| heights[(y * w + x) as usize];
        Self::from_fn(w, h, |x, y| {
            // Pente par « différences centrées » : (droite - gauche) / 2.
            let (left, right) = ((x + w - 1) % w, (x + 1) % w);
            let (up, down) = ((y + h - 1) % h, (y + 1) % h);
            let dx = (at(right, y) - at(left, y)) * 0.5 * strength;
            // Les lignes descendent dans l'image : « vers le haut » = -y.
            let dy_up = (at(x, up) - at(x, down)) * 0.5 * strength;
            // La normale d'une pente penche à l'opposé de la montée.
            let n = crate::math::Vec3::new(-dx, -dy_up, 1.0).normalize();
            let encode = |c: f32| ((c * 0.5 + 0.5) * 255.0).round() as u8;
            [encode(n.x), encode(n.y), encode(n.z), 255]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_map_d_une_surface_plate_puis_d_une_pente() {
        let plate = Image::normal_map_from_heights(4, 4, 1.0, |_, _| 3.0);
        assert!(plate.pixels.chunks(4).all(|p| p == [128, 128, 255, 255]));

        // Une rampe qui monte vers la droite : la normale penche vers la
        // gauche (rouge < 128). Pas de pente verticale : vert = 128.
        let rampe = Image::normal_map_from_heights(8, 1, 1.0, |x, _| x as f32 * 0.1);
        let [r, g, b, _] = rampe.pixel(3, 0);
        assert!(r < 128 && g == 128 && b > 200, "{r} {g} {b}");

        // Une rampe qui monte vers le BAS de l'image : la normale penche vers
        // le haut de l'image (vert > 128).
        let rampe = Image::normal_map_from_heights(1, 8, 1.0, |_, y| y as f32 * 0.1);
        let [_, g, _, _] = rampe.pixel(0, 3);
        assert!(g > 128, "{g}");
    }

    #[test]
    fn damier_alterne() {
        let img = Image::checkerboard(4, 2, Color::BLACK, Color::WHITE);
        assert_eq!(img.pixel(0, 0), [0, 0, 0, 255]);
        assert_eq!(img.pixel(2, 0), [255, 255, 255, 255]);
        assert_eq!(img.pixel(2, 2), [0, 0, 0, 255]);
    }

    #[test]
    fn reduction_fait_la_moyenne() {
        let img = Image::checkerboard(2, 2, Color::BLACK, Color::WHITE);
        let small = img.downsample();
        assert_eq!((small.width, small.height), (1, 1));
        // 2 noirs + 2 blancs → gris moyen (arrondi : (510 + 2) / 4 = 128).
        assert_eq!(small.pixel(0, 0), [128, 128, 128, 255]);
    }

    #[test]
    fn chaine_de_mipmaps_jusqu_a_1x1() {
        let img = Image::solid(8, 2, Color::WHITE);
        let sizes: Vec<_> = img
            .mip_chain()
            .iter()
            .map(|m| (m.width, m.height))
            .collect();
        assert_eq!(sizes, [(8, 2), (4, 1), (2, 1), (1, 1)]);
    }

    #[test]
    #[should_panic]
    fn taille_incoherente_refusee() {
        Image::new(2, 2, vec![0; 3]);
    }
}
