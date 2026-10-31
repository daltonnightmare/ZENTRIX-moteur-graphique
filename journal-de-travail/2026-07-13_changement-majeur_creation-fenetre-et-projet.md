# 2026-07-13 — Changement majeur : création de la fenêtre et du projet

> Entrée reconstituée à partir du commit `7020a67` (« creation de fenêtres et structure du projet »).

## Résumé

Premier état du projet : un seul programme Rust qui ouvre une fenêtre et dessine avec wgpu.

## Détail

- `Cargo.toml` : un crate unique.
- `src/main.rs` (312 lignes) : fenêtre, initialisation GPU et rendu dans un même fichier.
- `src/shader.wgsl` : premier shader.
- `documentation/GAME_ENGINE_DEVELOPMENT_PHASES.md`, `documentation/StructureProjet.md` : vision et phases du moteur.

## Impact / suite

Base fonctionnelle mais monolithique : restructurée le 2026-10-18
(voir [workspace multi-crates](2026-10-18_changement-majeur_workspace-multi-crates.md)).
