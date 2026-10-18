# ZENTRIX — moteur graphique en Rust

ZENTRIX est un moteur graphique écrit en **Rust**, construit pas à pas et pensé pour **apprendre** :
le code est commenté ligne par ligne, et un guide complet pour débutant accompagne le projet.

Il repose sur [`wgpu`](https://wgpu.rs) (une seule base de code pour Vulkan, DirectX 12, Metal et WebGPU)
et sur [`winit`](https://docs.rs/winit) pour la fenêtre.

> **État actuel : Phase 1–2 de la feuille de route.** Une fenêtre s'ouvre et affiche un triangle
> coloré sur un fond animé, à 60 images par seconde.

---

## Démarrage rapide

```bash
# 1. Installer Rust (une seule fois) : https://rustup.rs
# 2. Depuis la racine du projet :
cargo run
```

| Touche  | Action                          |
|---------|---------------------------------|
| `Échap` | Quitter                         |
| `Espace`| Mettre en pause l'animation du fond |

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
| [`zentrix-core`](crates/zentrix-core) | Couleurs, chronomètre de frame. Aucune dépendance. | `color.rs`, `time.rs` |
| [`zentrix-platform`](crates/zentrix-platform) | Fenêtre et boucle d'événements. Définit le trait `Application`. | `application.rs`, `runner.rs` |
| [`zentrix-rhi`](crates/zentrix-rhi) | Connexion au GPU : instance, adapter, device, queue, surface. | `context.rs`, `error.rs` |
| [`zentrix-render`](crates/zentrix-render) | Pipelines, shaders, vertex buffers, passes de rendu. | `renderer.rs`, `pipeline.rs`, `shaders/triangle.wgsl` |
| [`zentrix-app`](crates/zentrix-app) | Point d'entrée `main()` et démo. | `main.rs` |

```text
ZENTRIX-moteur-graphique/
├── Cargo.toml              ← déclaration du workspace + versions des dépendances
├── crates/
│   ├── zentrix-core/
│   ├── zentrix-platform/
│   ├── zentrix-rhi/
│   ├── zentrix-render/
│   └── zentrix-app/
└── documentation/
    ├── README.md           ← index de la documentation (commencer ici)
    └── guide-debutant/     ← le guide pas à pas
```

---

## Documentation

👉 **Commencez par [documentation/README.md](documentation/README.md).**

Le [guide du débutant](documentation/guide-debutant/) explique, dans l'ordre :

1. [Installer Rust et lancer le projet](documentation/guide-debutant/01-installer-et-lancer.md)
2. [Les bases de Rust, illustrées par le code de ZENTRIX](documentation/guide-debutant/02-bases-de-rust.md)
3. [Cargo et le workspace](documentation/guide-debutant/03-cargo-et-workspace.md)
4. [Comprendre le GPU et wgpu](documentation/guide-debutant/04-comprendre-le-gpu.md)
5. [Visite guidée du code, de `main()` au pixel](documentation/guide-debutant/05-visite-du-code.md)
6. [Exercices progressifs](documentation/guide-debutant/06-exercices.md)
7. [Feuille de route : la suite](documentation/guide-debutant/07-feuille-de-route.md)
8. [Glossaire](documentation/guide-debutant/08-glossaire.md)

