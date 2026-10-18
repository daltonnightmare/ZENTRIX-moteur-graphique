//! Représentation d'une couleur RGBA.

/// Une couleur au format **RGBA** (Rouge, Vert, Bleu, Alpha/opacité).
///
/// Chaque composante est un `f32` (nombre à virgule sur 32 bits) entre `0.0` et `1.0`.
/// C'est le format qu'attendent les GPU, contrairement au `0..=255` des logiciels de dessin.
///
/// # Les `derive`
///
/// `#[derive(...)]` demande au compilateur d'écrire automatiquement du code pour nous :
/// - `Debug` : permet `println!("{:?}", couleur)` ;
/// - `Clone`, `Copy` : la couleur est **copiée** (comme un nombre) au lieu d'être
///   « déplacée ». C'est possible car elle est petite (16 octets) et ne possède
///   aucune ressource (pas de mémoire allouée, pas de fichier...) ;
/// - `PartialEq` : permet de comparer deux couleurs avec `==`.
///
/// # Exemple
///
/// ```
/// use zentrix_core::Color;
///
/// let orange = Color::rgb(1.0, 0.5, 0.0);
/// assert_eq!(orange.a, 1.0); // `rgb` met l'opacité à 1 (opaque)
/// ```
///
/// (Ce bloc de code est un « doc-test » : `cargo test` l'exécute vraiment !)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    // `pub` devant chaque champ : on peut lire/écrire `couleur.r` depuis l'extérieur.
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

// Un bloc `impl` contient les fonctions (« méthodes ») associées à un type.
impl Color {
    // `const` : ces valeurs sont calculées à la compilation, elles ne coûtent rien.
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    /// Le bleu nuit utilisé comme fond par défaut du moteur.
    pub const ZENTRIX_NIGHT: Color = Color::rgb(0.05, 0.07, 0.12);

    /// Crée une couleur opaque (alpha = 1.0).
    ///
    /// `const fn` : la fonction peut être exécutée pendant la compilation,
    /// ce qui nous permet de l'utiliser pour définir les constantes ci-dessus.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        // `Self` est un alias pour le type en cours, ici `Color`.
        // Quand une variable porte le même nom qu'un champ, on peut écrire
        // `r` au lieu de `r: r` (raccourci d'initialisation).
        //
        // Pas de `return` ni de `;` à la fin : en Rust, la dernière EXPRESSION
        // d'un bloc est automatiquement sa valeur de retour.
        Self { r, g, b, a: 1.0 }
    }

    /// Crée une couleur avec une opacité choisie.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Interpolation linéaire (« lerp ») entre `self` et `other`.
    ///
    /// `t = 0.0` renvoie `self`, `t = 1.0` renvoie `other`, `t = 0.5` le mélange à moitié.
    ///
    /// Remarque : `self` (sans `&`) prend la couleur **par valeur**. Comme `Color`
    /// est `Copy`, l'appelant garde quand même sa couleur intacte.
    pub fn lerp(self, other: Color, t: f32) -> Color {
        // `clamp` borne t dans [0, 1] pour éviter des couleurs absurdes.
        // On « masque » (shadowing) l'ancien `t` par un nouveau : c'est idiomatique en Rust.
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    /// Convertit en tableau `[r, g, b, a]`, pratique pour envoyer au GPU.
    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

// Le trait `Default` définit la valeur « par défaut » d'un type :
// `Color::default()` renverra notre bleu nuit.
impl Default for Color {
    fn default() -> Self {
        Self::ZENTRIX_NIGHT
    }
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------
// `#[cfg(test)]` : ce module n'est compilé QUE lors de `cargo test`.
// Il n'alourdit donc pas le programme final.
#[cfg(test)]
mod tests {
    // `super` = le module parent (ce fichier). `*` importe tout ce qu'il contient.
    use super::*;

    #[test]
    fn lerp_aux_extremites() {
        assert_eq!(Color::BLACK.lerp(Color::WHITE, 0.0), Color::BLACK);
        assert_eq!(Color::BLACK.lerp(Color::WHITE, 1.0), Color::WHITE);
    }

    #[test]
    fn lerp_au_milieu() {
        let gris = Color::BLACK.lerp(Color::WHITE, 0.5);
        assert_eq!(gris, Color::rgb(0.5, 0.5, 0.5));
    }

    #[test]
    fn lerp_borne_t() {
        // t hors de [0, 1] est ramené dans l'intervalle.
        assert_eq!(Color::BLACK.lerp(Color::WHITE, 42.0), Color::WHITE);
    }
}
