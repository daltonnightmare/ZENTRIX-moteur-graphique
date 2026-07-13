ZENTRIX-moteur-graphique/
├── Cargo.toml                 # Workspace principal
├── .cargo/
│   └── config.toml            # Config éventuelle (build, cibles)
├── assets/                    # Ressources (modèles, textures, shaders)
│   ├── shaders/
│   │   ├── shader.wgsl        # (votre shader actuel)
│   │   └── ...
│   ├── models/
│   └── textures/
├── crates/                    # Sous-crates (modules indépendants)
│   ├── zentrix-core/          # Fondations mathématiques, types communs
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── math/          # Vecteurs, matrices, quaternions
│   │       ├── color.rs
│   │       ├── error.rs
│   │       └── handle.rs      # Gestion des ressources via IDs
│   ├── zentrix-platform/      # Fenêtrage, entrées (winit)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── window.rs
│   │       ├── input.rs
│   │       └── event_loop.rs
│   ├── zentrix-rhi/           # Render Hardware Interface (wgpu)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── device.rs
│   │       ├── surface.rs
│   │       ├── buffer.rs
│   │       ├── texture.rs
│   │       └── pipeline.rs
│   ├── zentrix-render/        # Rendu haut niveau (passes, shaders, matériaux)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── render_graph/  # Architecture des passes (ex: forward, deferred)
│   │       ├── material.rs    # Système de matériaux
│   │       ├── shader_manager.rs
│   │       ├── camera.rs
│   │       └── light.rs
│   ├── zentrix-scene/         # ECS, gestion des entités et composants
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entity.rs
│   │       ├── component/     # Composants (Transform, Mesh, Light, Camera...)
│   │       ├── system/        # Systèmes de mise à jour
│   │       └── world.rs
│   ├── zentrix-assets/        # Chargement et gestion des ressources
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── loader.rs      # glTF, PNG, etc.
│   │       ├── asset_cache.rs
│   │       └── hot_reload.rs
│   └── zentrix-app/           # Point d'entrée et boucle principale
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── app.rs         # ApplicationHandler (winit)
│           └── main.rs        # Fonction main (démarrage)
├── target/                    # Généré par cargo
└── README.md