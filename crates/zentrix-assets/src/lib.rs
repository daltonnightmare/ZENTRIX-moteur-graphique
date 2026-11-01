//! # zentrix-assets
//!
//! Le chargement des **ressources** (« assets ») : tout ce qui vient de
//! fichiers sur le disque : les images (PNG, JPEG) et les modèles 3D (glTF).
//!
//! Cette crate ne connaît **ni le GPU, ni la fenêtre** : elle transforme des
//! octets de fichier en données du noyau ([`zentrix_core::Image`],
//! [`zentrix_core::MeshData`]). C'est ensuite `zentrix-render` qui les envoie
//! au GPU.
//!
//! ```text
//!   caisse.png ──▶ zentrix-assets ──▶ Image (pixels RGBA) ──▶ zentrix-render ──▶ texture GPU
//!                  load_image()        zentrix-core           add_texture()
//!
//!   arbre.glb  ──▶ zentrix-assets ──▶ Model (MeshData + Image + transformations)
//!                  load_gltf()          └─▶ add_mesh / add_texture, puis des entités
//! ```
//!
//! Prévu plus tard (voir la feuille de route) : sons, chargement en
//! arrière-plan, rechargement à chaud.

mod model;

pub use model::{Model, ModelPart, PartTextures, decode_gltf, load_gltf};

use std::path::{Path, PathBuf};

use zentrix_core::Image;

/// Une erreur de chargement de ressource.
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    /// Le fichier n'a pas pu être lu (absent, droits insuffisants...).
    #[error("impossible de lire « {path} » : {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Le fichier a été lu mais son contenu n'est pas une image valide
    /// (format non supporté, fichier corrompu...).
    #[error("impossible de décoder l'image « {path} » : {source}")]
    Decode {
        path: PathBuf,
        source: image::ImageError,
    },

    /// Le modèle glTF n'a pas pu être chargé (fichier absent, JSON invalide,
    /// buffer ou image manquant...).
    #[error("impossible de charger le modèle « {path} » : {source}")]
    Gltf { path: PathBuf, source: gltf::Error },
}

/// Le dossier qui contient les ressources du jeu.
///
/// Un chemin relatif comme `"assets/caisse.png"` dépend du dossier depuis
/// lequel on lance le programme : il marche avec `cargo run` à la racine du
/// projet, mais pas ailleurs. On cherche donc, dans l'ordre :
///
/// 1. la variable d'environnement `ZENTRIX_ASSETS` (pour forcer un dossier) ;
/// 2. `assets/` dans le dossier courant ;
/// 3. `assets/` à côté de l'exécutable (pour un jeu distribué) ;
/// 4. `assets/` à la racine du workspace (pendant le développement).
pub fn asset_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("ZENTRIX_ASSETS") {
        return PathBuf::from(dir);
    }
    let candidates = [
        Some(PathBuf::from("assets")),
        // `.ok()` puis `?` dans la closure : `None` si une étape échoue.
        std::env::current_exe()
            .ok()
            .and_then(|exe| Some(exe.parent()?.join("assets"))),
        // `env!` est évalué à la COMPILATION : c'est le dossier de cette crate.
        Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")),
    ];
    candidates
        .into_iter()
        .flatten() // ignore les `None`
        .find(|dir| dir.is_dir())
        .unwrap_or_else(|| PathBuf::from("assets"))
}

/// Charge une image depuis un fichier PNG ou JPEG.
///
/// Le format est deviné à partir du **contenu** du fichier (ses premiers
/// octets), pas de son extension. L'image est toujours convertie en RGBA
/// 8 bits, quel que soit son format d'origine (niveaux de gris, RGB sans
/// transparence...).
///
/// `impl AsRef<Path>` : on accepte un `&str`, une `String`, un `Path`, un
/// `PathBuf`... tout ce qui peut être vu comme un chemin.
pub fn load_image(path: impl AsRef<Path>) -> Result<Image, AssetError> {
    let path = path.as_ref();

    // On lit nous-mêmes le fichier pour distinguer clairement les deux erreurs
    // possibles : « fichier introuvable » et « fichier invalide ».
    let bytes = std::fs::read(path).map_err(|source| AssetError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let image = decode_image(&bytes).map_err(|source| AssetError::Decode {
        path: path.to_path_buf(),
        source,
    })?;

    log::info!(
        "Image chargée : {} ({}x{})",
        path.display(),
        image.width,
        image.height
    );
    Ok(image)
}

/// Décode une image déjà en mémoire (par exemple intégrée à l'exécutable
/// avec `include_bytes!`).
pub fn decode_image(bytes: &[u8]) -> Result<Image, image::ImageError> {
    let rgba = image::load_from_memory(bytes)?.into_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(Image::new(width, height, rgba.into_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encode une petite image en PNG, en mémoire.
    fn png_2x1() -> Vec<u8> {
        let img = image::RgbaImage::from_raw(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 128])
            .expect("taille correcte");
        let mut bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("encodage PNG");
        bytes
    }

    #[test]
    fn decode_un_png() {
        let image = decode_image(&png_2x1()).expect("PNG valide");
        assert_eq!((image.width, image.height), (2, 1));
        assert_eq!(image.pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(image.pixel(1, 0), [0, 0, 255, 128]);
    }

    #[test]
    fn le_dossier_assets_du_projet_est_trouve() {
        // Les tests s'exécutent depuis le dossier de la crate : c'est le
        // dernier candidat (racine du workspace) qui doit répondre.
        assert!(asset_dir().join("textures/caisse.png").is_file());
    }

    #[test]
    fn fichier_absent() {
        let err = load_image("n_existe_pas.png").unwrap_err();
        assert!(matches!(err, AssetError::Io { .. }), "{err}");
    }

    #[test]
    fn contenu_invalide() {
        let path = std::env::temp_dir().join("zentrix_test_invalide.png");
        std::fs::write(&path, b"ceci n'est pas une image").unwrap();
        let err = load_image(&path).unwrap_err();
        std::fs::remove_file(&path).ok();
        assert!(matches!(err, AssetError::Decode { .. }), "{err}");
    }
}
