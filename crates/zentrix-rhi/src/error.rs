//! Les erreurs que peut produire le RHI.
//!
//! ## `thiserror` vs `anyhow` : quelle différence ?
//!
//! - Une **bibliothèque** (comme cette crate) définit ses propres types d'erreur
//!   précis, avec `thiserror`. L'appelant peut ainsi faire un `match` sur
//!   chaque cas et réagir différemment.
//! - Une **application** (comme `zentrix-app`) utilise souvent `anyhow::Error`,
//!   une « boîte » qui accepte n'importe quelle erreur. Pratique quand on veut
//!   juste afficher l'erreur et s'arrêter.
//!
//! Grâce au trait standard `std::error::Error` (implémenté ici par `thiserror`),
//! l'opérateur `?` convertit automatiquement un `RhiError` en `anyhow::Error`.

/// Une erreur survenue lors de l'initialisation ou de l'utilisation du GPU.
///
/// `#[derive(thiserror::Error)]` génère automatiquement l'implémentation du
/// trait `std::error::Error` et de `Display` (le message lisible).
///
/// `#[error("...")]` définit le message affiché pour chaque variante.
/// `{0}` insère le premier champ de la variante.
///
/// `#[from]` génère une conversion automatique : un `wgpu::CreateSurfaceError`
/// devient un `RhiError::Surface` dès qu'on utilise `?`.
#[derive(Debug, thiserror::Error)]
pub enum RhiError {
    #[error("impossible de créer la surface de rendu : {0}")]
    Surface(#[from] wgpu::CreateSurfaceError),

    #[error("aucune carte graphique compatible trouvée : {0}")]
    NoAdapter(#[from] wgpu::RequestAdapterError),

    #[error("impossible d'ouvrir une connexion avec le GPU : {0}")]
    Device(#[from] wgpu::RequestDeviceError),

    #[error("la surface ne propose aucun format de pixel compatible avec ce GPU")]
    NoSurfaceFormat,

    #[error("la surface de rendu a été perdue et n'a pas pu être recréée")]
    SurfaceLost,
}
