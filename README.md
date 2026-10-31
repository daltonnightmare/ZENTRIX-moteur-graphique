# ZENTRIX — moteur graphique en Rust

ZENTRIX est un moteur graphique écrit en **Rust**, construit pas à pas et pensé pour **apprendre** :
le code est commenté ligne par ligne, et un guide complet pour débutant accompagne le projet.

Il repose sur [`wgpu`](https://wgpu.rs) (une seule base de code pour Vulkan, DirectX 12, Metal et WebGPU)
et sur [`winit`](https://docs.rs/winit) pour la fenêtre.

> **État actuel : Phase 2 de la feuille de route.** Une fenêtre s'ouvre et affiche une scène 3D :
> une grille de caisses en bois texturées et de cubes colorés qui tournent sur un sol en damier, vue en
> perspective et explorable avec une caméra libre (clavier + souris), avec test de profondeur, rendu
> instancié et textures avec mipmaps.

---

## Démarrage rapide

```bash
# 1. Installer Rust (une seule fois) : https://rustup.rs
# 2. Depuis la racine du projet :
cargo run
```

| Touche (AZERTY)       | Action                                          |
|-----------------------|-------------------------------------------------|
| clic gauche           | Capturer la souris pour regarder autour de soi  |
| `Z` `Q` `S` `D`       | Se déplacer (WASD sur un clavier QWERTY)        |
| `Espace` / `Ctrl`     | Monter / descendre                              |
| `Maj` (maintenue)     | Aller 3 fois plus vite                          |
| molette               | Régler la vitesse de déplacement                |
| `P`                   | Mettre en pause l'animation des cubes           |
| `Échap`               | Libérer la souris, puis quitter                 |

La première compilation prend quelques minutes (wgpu est une grosse dépendance). Les suivantes ne prennent que quelques secondes.

Commandes utiles :

```bash
cargo test --workspace          # lance tous les tests
cargo clippy --workspace        # conseils de qualité de code
cargo fmt --all                 # formate le code
cargo doc --open --no-deps      # génère et ouvre la documentation du code
```

---

## Architecture

Le projet est un **workspace Cargo** : plusieurs petites crates, chacune avec une seule responsabilité.

```text
                 zentrix-app            ← l'exécutable : assemble tout
                /           \
   zentrix-platform      zentrix-render  ← fenêtre/événements | quoi dessiner
          |                    |
          |               zentrix-rhi    ← comment parler au GPU
          |                    |
        winit            wgpu ── GPU
          \                  /
            zentrix-core          ← types de base (couleurs, temps)
```

| Crate | Rôle | Fichiers clés |
|-------|------|---------------|
| [`zentrix-core`](crates/zentrix-core) | Couleurs, chronomètre de frame, maths 3D (`glam`), `Transform`, `Image`. | `color.rs`, `time.rs`, `math.rs`, `transform.rs`, `image.rs` |
| [`zentrix-platform`](crates/zentrix-platform) | Fenêtre, boucle d'événements, état du clavier et de la souris (`Input`). Définit le trait `Application`. | `application.rs`, `runner.rs`, `input.rs` |
| [`zentrix-rhi`](crates/zentrix-rhi) | Connexion au GPU : instance, adapter, device, queue, surface. | `context.rs`, `error.rs` |
| [`zentrix-render`](crates/zentrix-render) | Maillages, textures, caméra, depth buffer, pipelines, shaders, passes de rendu. | `renderer.rs`, `mesh.rs`, `texture.rs`, `camera.rs`, `pipeline.rs`, `shaders/mesh.wgsl` |
| [`zentrix-assets`](crates/zentrix-assets) | Chargement des fichiers (images PNG/JPEG) en données du noyau. | `lib.rs` |
| [`zentrix-app`](crates/zentrix-app) | Point d'entrée `main()` et démo. | `main.rs` |

```text
ZENTRIX-moteur-graphique/
├── Cargo.toml              ← déclaration du workspace + versions des dépendances
├── crates/
│   ├── zentrix-core/
│   ├── zentrix-platform/
│   ├── zentrix-rhi/
│   ├── zentrix-render/
│   ├── zentrix-assets/
│   └── zentrix-app/
├── assets/                 ← ressources du jeu (textures...)
├── journal-de-travail/     ← historique daté des changements
└── documentation/
    ├── README.md           ← index de la documentation (commencer ici)
    └── guide-debutant/     ← le guide pas à pas
```

---

## Documentation

👉 **Commencez par [documentation/README.md](documentation/README.md).**

📓 L'historique daté des changements est dans le [journal de travail](journal-de-travail/README.md).

Le [guide du débutant](documentation/guide-debutant/) explique, dans l'ordre :

1. [Installer Rust et lancer le projet](documentation/guide-debutant/01-installer-et-lancer.md)
2. [Les bases de Rust, illustrées par le code de ZENTRIX](documentation/guide-debutant/02-bases-de-rust.md)
3. [Cargo et le workspace](documentation/guide-debutant/03-cargo-et-workspace.md)
4. [Comprendre le GPU et wgpu](documentation/guide-debutant/04-comprendre-le-gpu.md)
5. [Visite guidée du code, de `main()` au pixel](documentation/guide-debutant/05-visite-du-code.md)
6. [Exercices progressifs](documentation/guide-debutant/06-exercices.md)
7. [Feuille de route : la suite](documentation/guide-debutant/07-feuille-de-route.md)
8. [Glossaire](documentation/guide-debutant/08-glossaire.md)

