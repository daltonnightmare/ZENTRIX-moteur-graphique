# 2026-10-18 — Changement majeur : passage à un workspace multi-crates

> Entrée reconstituée à partir du commit `a80ea0b` (« trainning »).

## Résumé

Le programme unique est découpé en un workspace Cargo de 5 crates, avec un guide débutant complet.

## Pourquoi

Séparer les responsabilités (fenêtre, GPU, rendu, application), compiler plus vite et préparer
les phases suivantes (ECS, assets...).

## Détail

| Crate | Rôle |
|-------|------|
| `zentrix-core` | Types de base : `Color`, `FrameTimer` |
| `zentrix-platform` | Fenêtre et boucle d'événements (winit), trait `Application` |
| `zentrix-rhi` | Connexion au GPU (wgpu) : `GpuContext`, surface, acquisition d'image |
| `zentrix-render` | Pipeline, shader `triangle.wgsl`, vertex buffer, `Renderer` |
| `zentrix-app` | Exécutable : triangle coloré sur fond animé |

Documentation ajoutée : `README.md`, `documentation/README.md`, guide débutant en 8 chapitres,
`git.md`, `crates.md`, `moteurGraphique.md`, `nanites_et_lumen.md`.

## Impact / suite

Phase 1 terminée, Phase 2 (noyau de rendu) commencée. Voir la
[suppression associée](2026-10-18_suppression_programme-monolithique-src.md).
