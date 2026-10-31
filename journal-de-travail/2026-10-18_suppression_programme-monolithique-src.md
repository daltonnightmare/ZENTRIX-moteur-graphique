# 2026-10-18 — Suppression : ancien programme monolithique `src/`

> Entrée reconstituée à partir du commit `a80ea0b`.

## Résumé

Suppression de `src/main.rs` (312 lignes) et `src/shader.wgsl` (15 lignes).

## Pourquoi

Leur contenu a été réparti dans les crates du workspace
(voir [workspace multi-crates](2026-10-18_changement-majeur_workspace-multi-crates.md)).
Selon le guide (`zentrix-rhi/src/context.rs`), l'ancienne version utilisait `Box::leak` pour la
fenêtre ; c'est remplacé par un `Arc<Window>`.

## Récupération

L'ancien code reste consultable : `git show 7020a67:src/main.rs`.
