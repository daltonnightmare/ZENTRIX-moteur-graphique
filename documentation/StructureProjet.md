Voici le contenu complet de votre fichier `STRUCTURE.md` à copier-coller dans votre projet.

```markdown
# Structure du Moteur Graphique (Vulkan / Rust)

Cette documentation définit l'architecture des dossiers pour le moteur de jeu, en suivant la feuille de route découpée en phases. Le projet est organisé en **Cargo Workspace** pour permettre une compilation modulaire et des temps de build optimisés.

## Arborescence Générale

```text
my_engine/
├── Cargo.toml                 # Fichier manifeste du workspace
├── Cargo.lock
├── .cargo/
│   └── config.toml            # Optimisations du linker (LLD / Mold)
├── crates/                    # 🧩 Coeur du moteur (workspace members)
│   ├── core/                  # Phase 1 : Boucle de vie, fenêtrage (winit), logging
│   ├── render/                # Phase 2 & 12 : Cœur graphique (Vulkan via wgpu/ash)
│   ├── ecs/                   # Phase 3 : ECS, Scène, Transformations
│   ├── assets/                # Phase 4 : Pipeline d'import, hot-reload, gestionnaire
│   ├── input/                 # Phase 5 : Abstraction clavier/souris/manette
│   ├── physics/               # Phase 7 : Intégration de la physique (Rapier)
│   ├── audio/                 # Phase 8 : Intégration audio (Kira / Rodio)
│   ├── ui/                    # Phase 9 : UI de debug (egui) et UI runtime
│   ├── macros/                # Macros procédurales (sérialisation, IDs)
│   └── editor/                # Phase 10 : Application Éditeur (interface graphique)
├── assets/                    # 🎨 Actifs sources (données brutes)
│   ├── shaders/
│   │   ├── vulkan/            # Shaders sources (.glsl, .hlsl ou .wgsl)
│   │   └── spirv/             # Binaires compilés (.spv) pour Vulkan
│   ├── textures/              # .png, .jpg, .ktx2 (textures compressées)
│   ├── models/                # .gltf, .glb (modèles 3D)
│   ├── fonts/                 # .ttf, .otf (polices)
│   ├── audio/                 # .wav, .ogg, .mp3
│   └── scenes/                # Scènes sérialisées (.ron, .json, .toml)
├── tools/                     # 🔧 Outils en ligne de commande externes
│   └── asset_processor/       # Outil de packaging des actifs en binaire
├── examples/                  # 🎯 Démos de validation (workspace members)
│   ├── empty_window/          # Jalon 1 : Fenêtre vide
│   ├── triangle/              # Phase 2 : Premier rendu Vulkan
│   ├── scene_loader/          # Phase 3 : Chargement d'une scène
│   └── game_slice/            # Phase 6 : Tranche verticale jouable
└── target/                    # 📦 Dossier de compilation (ignoré par Git)
```

---

## Détail du Crate `render` (Cœur Vulkan)

Ce dossier contient toute la logique bas niveau liée au GPU. La structure est conçue pour être utilisée aussi bien avec l'abstraction `wgpu` (recommandée dans la roadmap) qu'avec les bindings natifs `ash`.

```text
crates/render/
├── Cargo.toml
├── build.rs                  # Script de compilation : convertit les .glsl en .spv
└── src/
    ├── lib.rs                # Exporte les modules publics
    ├── device/               # Initialisation du GPU et de la Swapchain
    │   ├── mod.rs
    │   ├── adapter.rs        # Sélection du périphérique physique (Vulkan)
    │   ├── surface.rs        # Création de la surface de rendu (via winit)
    │   └── swapchain.rs      # Gestion des images, format, présentation
    ├── pipeline/             # États du pipeline graphique
    │   ├── mod.rs
    │   ├── layout.rs         # Descriptor set layouts & Push constants
    │   └── builder.rs        # Construction des pipelines (graphique/compute)
    ├── resources/            # Mémoire GPU (Buffers, Images, Samplers)
    │   ├── mod.rs
    │   ├── buffer.rs         # Vertex, Index, Uniform, Storage buffers
    │   ├── image.rs          # Textures 2D, Cubemaps, Depth/Stencil
    │   └── descriptor.rs     # Gestion des Descriptor Sets (pools, écritures)
    ├── shaders/              # Module Rust contenant les shaders compilés
    │   ├── mod.rs
    │   ├── include.rs        # Inclut les binaires SPIR-V via include_bytes!
    │   └── hot_reload.rs     # Surveille les .spv pour rechargement à chaud (Phase 4)
    ├── geometry/             # Maillages et formats de sommets
    │   ├── mod.rs
    │   ├── mesh.rs           # Chargement des données GPU depuis un modèle CPU
    │   └── vertex.rs         # Définition des layouts de sommets (bytemuck)
    ├── passes/               # Graphique de rendu (Render Graph / Command Buffers)
    │   ├── mod.rs
    │   ├── clear.rs          # Passe d'effacement de l'écran
    │   ├── forward.rs        # Passe de rendu principal (Forward Shading)
    │   ├── shadow.rs         # Passe des ombres (cascades)
    │   └── debug_overlay.rs  # Passe pour les lignes/grilles de debug (Phase 2)
    ├── camera/               # Modèles de projection et vue
    │   └── mod.rs
    └── debug/                # Outils de validation et marqueurs GPU
        └── mod.rs
```

---

## Configuration recommandée (`Cargo.toml` du workspace)

À la racine du projet, le fichier `Cargo.toml` doit définir les membres du workspace et les dépendances partagées pour éviter les duplications.

```toml
[workspace]
resolver = "2"
members = [
    "crates/core",
    "crates/render",
    "crates/ecs",
    "crates/assets",
    "crates/input",
    "crates/physics",
    "crates/audio",
    "crates/ui",
    "crates/macros",
    "crates/editor",
    "examples/empty_window",
    "examples/triangle",
    "examples/scene_loader",
    "examples/game_slice",
]

[workspace.dependencies]
# Noyau et fenêtrage
winit = "0.30"
tracing = "0.1"
anyhow = "1.0"
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
ron = "0.8"

# Graphismes (Vulkan / wgpu)
wgpu = "0.20"                # Abstraction recommandée pour Vulkan/Metal/DX12
glam = "0.25"                # Mathématiques
bytemuck = "1.14"            # Casts de mémoire sûrs
image = "0.24"               # Chargement d'images

# ECS
bevy_ecs = "0.13"            # Ou hecs / shipyard

# Physique
rapier3d = "0.18"

# Audio
kira = "0.9"

# UI
egui = "0.27"
cosmic-text = "0.11"

# Outils
notify = "6.1"               # Hot-reload
```

---

## Optimisation du Linking (`.cargo/config.toml`)

Pour accélérer les compilations en développement (crucial avec Vulkan et les dépendances lourdes), utilisez un linker rapide :

```toml
[build]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.x86_64-pc-windows-msvc]
linker = "lld-link.exe"
```

---

## Conseils d'implémentation pour Vulkan

1. **Compilation des shaders** : Dans le `build.rs` de `crates/render`, utilisez `glslang` ou `naga` pour transformer automatiquement les fichiers `.glsl` du dossier `assets/shaders/vulkan/` en `.spv` dans `assets/shaders/spirv/`.
2. **Hot-Reload** : La Phase 4 utilise le crate `notify` pour surveiller les changements dans `assets/shaders/spirv/`. Lorsqu'un fichier change, le moteur doit reconstruire le `Pipeline` et le `Descriptor Set Layout` associé.
3. **Validation Layers** : Activez les couches de validation Vulkan (`VK_LAYER_KHRONOS_validation`) uniquement en mode debug via une feature flag dans `Cargo.toml` (`debug_validation`). Cela alourdit les performances mais est indispensable pour le débogage.
4. **Mémoire** : Pour une gestion optimale, le dossier `resources/` doit séparer les tampons par type d'utilisation :
   - `BufferType::Vertex` (mappable ou non).
   - `BufferType::Uniform` (mappable, mis à jour chaque frame).
   - `BufferType::Storage` (Compute shaders).

## Ordre d'implémentation suggéré

1. **`crates/core`** + **`crates/render/device`** → Phase 1 (Fenêtre + boucle vide).
2. **`crates/render/passes/clear`** + **`crates/render/geometry/triangle`** → Phase 2 (Premier rendu).
3. **`crates/ecs`** + **`crates/render/passes/forward`** → Phase 3 (Rendu d'entités).
4. **`crates/assets`** + **`crates/render/shaders/hot_reload`** → Phase 4 (Pipeline d'actifs).
```

Ce fichier Markdown peut être sauvegardé directement sous `docs/STRUCTURE.md` ou `STRUCTURE.md` à la racine de votre projet.