# Documentation de ZENTRIX

Bienvenue ! Cette page est le **point de départ** de toute la documentation.

## Je débute en Rust : par où commencer ?

Suivez le guide **dans l'ordre**. Chaque chapitre s'appuie sur le précédent et renvoie au vrai code du moteur.

| # | Chapitre | Ce que vous apprendrez | Durée |
|---|----------|------------------------|-------|
| 1 | [Installer et lancer](guide-debutant/01-installer-et-lancer.md) | Installer Rust, VS Code, lancer ZENTRIX, lire les logs, dépanner | 30 min |
| 2 | [Les bases de Rust](guide-debutant/02-bases-de-rust.md) | Variables, types, structs, enums, `Option`/`Result`, propriété, emprunts, traits... | 2–3 h |
| 3 | [Cargo et le workspace](guide-debutant/03-cargo-et-workspace.md) | Crates, dépendances, workspace, ajouter une crate | 45 min |
| 4 | [Comprendre le GPU](guide-debutant/04-comprendre-le-gpu.md) | CPU vs GPU, pipeline de rendu, shaders, buffers, objets wgpu | 1–2 h |
| 5 | [Visite guidée du code](guide-debutant/05-visite-du-code.md) | Le trajet complet de `main()` jusqu'au pixel affiché | 1 h |
| 6 | [Exercices](guide-debutant/06-exercices.md) | Modifier le moteur soi-même, du plus simple au plus ambitieux | à votre rythme |
| 7 | [Feuille de route](guide-debutant/07-feuille-de-route.md) | Les prochaines étapes concrètes du moteur | 15 min |
| 8 | [Glossaire](guide-debutant/08-glossaire.md) | Tous les termes techniques expliqués simplement | référence |

> 💡 **Conseil** : gardez le code ouvert à côté du guide. Chaque fichier `.rs` est lui-même
> abondamment commenté : le guide donne la vue d'ensemble, les commentaires donnent le détail.

## La documentation générée depuis le code

Les commentaires `///` et `//!` du code produisent un site de documentation :

```bash
cargo doc --open --no-deps
```

## Documents de conception (vision à long terme)

Ces documents décrivent **où le moteur veut aller**. Ils sont plus avancés : lisez-les une fois le guide terminé.

| Document | Contenu |
|----------|---------|
| [GAME_ENGINE_DEVELOPMENT_PHASES.md](GAME_ENGINE_DEVELOPMENT_PHASES.md) | Les phases 0 à 12 du développement d'un moteur de jeu complet |
| [moteurGraphique.md](moteurGraphique.md) | Architecture cible du moteur de rendu (render graph, ressources, caméra) et son intégration dans un moteur de jeu |
| [crates.md](crates.md) | Découpage en crates `zentrix-*` — **c'est ce découpage qui a été mis en place** |
| [StructureProjet.md](StructureProjet.md) | Une arborescence alternative plus détaillée (`crates/core`, `crates/render/...`). Le projet suit plutôt [crates.md](crates.md) ; ce document reste une source d'idées pour l'organisation interne future de `zentrix-render` |
| [nanites_et_lumen.md](nanites_et_lumen.md) | Pistes pour des équivalents de Nanite (géométrie virtualisée) et Lumen (éclairage global) — objectif très lointain |
| [git.md](git.md) | Guide pour créer un VCS façon Git en Rust (projet annexe, sans lien direct avec le moteur) |

## Ce qui existe aujourd'hui vs ce qui est prévu

| Fonctionnalité | État |
|----------------|------|
| Workspace Cargo multi-crates | ✅ fait |
| Fenêtre, clavier, boucle de jeu, delta time, FPS | ✅ fait |
| Initialisation GPU (wgpu), surface, redimensionnement | ✅ fait |
| Pipeline + shader WGSL + vertex buffer (triangle coloré) | ✅ fait |
| Index buffer, plusieurs maillages | ⏳ prochaine étape ([exercice 4](guide-debutant/06-exercices.md)) |
| Uniforms + caméra 3D (glam) | ⏳ prévu |
| Textures, profondeur (depth buffer) | ⏳ prévu |
| `zentrix-assets`, `zentrix-scene` (ECS) | ⏳ prévu ([feuille de route](guide-debutant/07-feuille-de-route.md)) |
