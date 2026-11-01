# ZENTRIX — moteur graphique en Rust

ZENTRIX est un moteur graphique écrit en **Rust**, construit pas à pas et pensé pour **apprendre** :
le code est commenté ligne par ligne, et un guide complet pour débutant accompagne le projet.

Il repose sur [`wgpu`](https://wgpu.rs) (une seule base de code pour Vulkan, DirectX 12, Metal et WebGPU)
et sur [`winit`](https://docs.rs/winit) pour la fenêtre.

> **État actuel : Phase 2 terminée, Phase 3 (scène et ECS) commencée.** Chaque objet est une entité d'un ECS,
> les entités peuvent être attachées les unes aux autres (hiérarchie : un « système solaire » flotte au-dessus des cubes),
> et des modèles 3D glTF sont chargés depuis le disque (des arbres entourent la scène). La scène se
> sauvegarde et se recharge au format texte RON (`F5` / `F9`). Un inspecteur (interface egui, `F4`)
> permet de régler le moteur et de modifier les entités pendant que la scène tourne. La physique (rapier3d)
> fait tomber et rebondir les objets : une pyramide de caisses attend les balles lancées avec `B`. Le son
> (kira) est spatialisé : chaque choc s'entend à l'endroit où il a lieu, et les caisses « sci-fi » bourdonnent. Une fenêtre s'ouvre et affiche une scène 3D :
> une grille de caisses en bois texturées et de cubes colorés qui tournent sur un sol en damier, vue en
> perspective et explorable avec une caméra libre (clavier + souris), avec test de profondeur, rendu
> instancié, textures avec mipmaps, éclairage PBR (lumière directionnelle, ciel et reflets de l'environnement, matériaux complets : métal / rugosité, normal maps, occlusion, émission ; tone mapping ; vitrine de sphères et caisses « sci-fi »), ombres portées et rendu de
> debug (grille, axes, boîtes, direction de la lumière, FPS dans le titre).

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
| `P`                   | Mettre en pause l'animation et la physique      |
| `F1`                  | Afficher / masquer la grille, les axes et la direction de la lumière |
| `F2`                  | Afficher / masquer les boîtes et les liens parent → enfant |
| `F3`                  | Activer / désactiver les ombres portées         |
| `F4`                  | Afficher / masquer l'inspecteur (interface egui) |
| `F5` / `F9`           | Sauvegarder / recharger la scène (`assets/scenes/demo.ron`) |
| `F6`                  | Activer / désactiver le tone mapping            |
| `F7`                  | Activer / désactiver l'environnement (ciel, reflets) |
| `B`                   | Lancer une balle droit devant                   |
| clic droit (souris capturée) | Pousser l'objet visé (lancer de rayon)   |
| `F8`                  | Couper / remettre le son                        |
| `N`                   | Ajouter / retirer le champ de stress (4 096 cubes) |
| `L`                   | Passer à la nuit (seules les lampes éclairent) / au jour |
| `Échap`               | Libérer la souris, puis quitter                 |

La première compilation prend quelques minutes (wgpu est une grosse dépendance). Les suivantes ne prennent que quelques secondes.

Commandes utiles :

```bash
cargo test --workspace          # lance tous les tests
cargo clippy --workspace        # conseils de qualité de code
cargo fmt --all                 # formate le code
cargo doc --open --no-deps      # génère et ouvre la documentation du code
cargo run --release             # version optimisée : pour mesurer les performances
cargo run --release --features tracy   # avec les mesures pour le profileur Tracy 0.14
```

Le panneau **Performance** de l'inspecteur (`F4`) montre le temps de chaque étape de l'image,
le nombre d'objets dessinés et d'appels de dessin. Il permet aussi de couper le culling, les
threads et la V-Sync pour comparer. Pour une analyse plus fine, lancez avec `--features tracy`
puis ouvrez l'application [Tracy](https://github.com/wolfpld/tracy) **0.14** : elle se connecte
au moteur pendant qu'il tourne.

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
| [`zentrix-core`](crates/zentrix-core) | Couleurs, chronomètre de frame, maths 3D (`glam`), `Transform`, `Image`, géométrie (`Vertex`, `MeshData`), boîtes englobantes et pyramide de vision (`Aabb`, `Frustum`), profileur d'image. | `color.rs`, `time.rs`, `math.rs`, `transform.rs`, `image.rs`, `mesh.rs`, `bounds.rs`, `profiler.rs` |
| [`zentrix-platform`](crates/zentrix-platform) | Fenêtre, boucle d'événements, état du clavier et de la souris (`Input`). Définit le trait `Application`. | `application.rs`, `runner.rs`, `input.rs` |
| [`zentrix-rhi`](crates/zentrix-rhi) | Connexion au GPU : instance, adapter, device, queue, surface. | `context.rs`, `error.rs` |
| [`zentrix-render`](crates/zentrix-render) | Maillages, textures, éclairage, ombres, caméra, depth buffer, pipelines, shaders, passes de rendu, ombres en cascade, lampes et projecteurs, culling (sur plusieurs threads), post-traitement (bloom, tone mapping, FXAA). | `renderer.rs`, `culling.rs`, `cascades.rs`, `postprocess.rs`, `mesh.rs`, `texture.rs`, `lighting.rs`, `shadow.rs`, `camera.rs`, `pipeline.rs`, `shaders/` |
| [`zentrix-assets`](crates/zentrix-assets) | Chargement des fichiers (images PNG/JPEG, modèles glTF) en données du noyau. | `lib.rs`, `model.rs` |
| [`zentrix-scene`](crates/zentrix-scene) | La scène : entités et composants (ECS `hecs`), hiérarchie, sauvegarde RON, extraction vers le rendu. | `lib.rs`, `components.rs`, `hierarchy.rs`, `serialization.rs` |
| [`zentrix-audio`](crates/zentrix-audio) | Son : sons positionnés dans la scène (kira), auditeur, émetteurs en boucle, synthèse de bruitages. | `lib.rs`, `components.rs`, `synth.rs` |
| [`zentrix-physics`](crates/zentrix-physics) | Physique : corps rigides, collisions, lancer de rayon (rapier3d), synchronisés avec la scène à pas fixe. | `lib.rs`, `components.rs` |
| [`zentrix-ui`](crates/zentrix-ui) | Interface graphique de debug : intégration d'egui (événements winit, rendu wgpu en overlay). | `lib.rs` |
| [`zentrix-app`](crates/zentrix-app) | Point d'entrée `main()` et démo (entités, composants et systèmes du jeu). | `main.rs`, `demo.rs`, `fly_camera.rs`, `model.rs`, `inspector.rs` |

```text
ZENTRIX-moteur-graphique/
├── Cargo.toml              ← déclaration du workspace + versions des dépendances
├── crates/
│   ├── zentrix-core/
│   ├── zentrix-platform/
│   ├── zentrix-rhi/
│   ├── zentrix-render/
│   ├── zentrix-assets/
│   ├── zentrix-scene/
│   ├── zentrix-audio/
│   ├── zentrix-physics/
│   ├── zentrix-ui/
│   └── zentrix-app/
├── assets/                 ← ressources du jeu (textures, modèles 3D)
├── tools/                  ← scripts (génération du modèle d'arbre)
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

