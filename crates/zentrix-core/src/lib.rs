//! # zentrix-core
//!
//! Le **noyau** de ZENTRIX : des types simples, utilisés par toutes les autres crates.
//!
//! Règle d'or : cette crate ne connaît **ni le GPU, ni la fenêtre**. Elle ne
//! contient que des données et de la logique pure, ce qui la rend facile à
//! tester (`cargo test -p zentrix-core`).
//!
//! ## Note pour débutant : `//!` et `///`
//!
//! - `//!` documente l'élément **qui contient** le commentaire (ici : toute la crate).
//! - `///` documente l'élément **qui suit** (une fonction, une struct...).
//!
//! Ces commentaires sont transformés en site web par `cargo doc --open`.

// `mod color;` dit à Rust : « il existe un module nommé `color`, son code est
// dans le fichier `color.rs` à côté de moi ». Sans cette ligne, le fichier
// serait tout simplement ignoré par le compilateur !
mod color;
mod time;

// `pub use` ré-exporte des éléments : l'utilisateur peut écrire
// `zentrix_core::Color` au lieu de `zentrix_core::color::Color`.
// Les modules eux-mêmes restent privés (pas de `pub mod`) : on choisit donc
// précisément ce qui fait partie de l'API publique.
pub use color::Color;
pub use time::FrameTimer;
