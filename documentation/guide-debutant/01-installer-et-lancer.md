# Chapitre 1 — Installer Rust et lancer ZENTRIX

**Objectif** : à la fin de ce chapitre, la fenêtre de ZENTRIX s'ouvre sur votre machine et vous savez lire ce qu'elle raconte dans le terminal.

---

## 1.1 Installer Rust

Rust s'installe avec **rustup**, le gestionnaire officiel de versions.

### Windows

1. Téléchargez et lancez `rustup-init.exe` depuis <https://rustup.rs>.
2. Si l'installateur le demande, installez les **Visual Studio Build Tools** (option « Développement Desktop en C++ »). Rust en a besoin pour l'édition de liens (le *linker*) sous Windows.
3. Acceptez l'installation par défaut.
4. **Fermez et rouvrez** votre terminal.

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Sous Linux, installez aussi les en-têtes nécessaires à la fenêtre et à Vulkan (exemple Debian/Ubuntu) :

```bash
sudo apt install build-essential libvulkan1 mesa-vulkan-drivers libxkbcommon-dev libwayland-dev
```

### Vérifier

```bash
rustc --version   # le compilateur
cargo --version   # le gestionnaire de projet (on l'utilise tout le temps)
```

Les deux commandes doivent afficher un numéro de version (par exemple `1.97.1`). Le projet utilise l'**édition 2024** de Rust, qui demande Rust **1.85 ou plus récent**. Pour mettre à jour : `rustup update`.

### Les trois outils installés

| Outil | Rôle |
|-------|------|
| `rustc` | Le compilateur : transforme le code `.rs` en programme exécutable. On l'appelle rarement directement. |
| `cargo` | Le chef d'orchestre : compile, lance, teste, télécharge les dépendances. **C'est lui que vous utiliserez.** |
| `rustup` | Gère les versions de Rust et les composants (`clippy`, `rustfmt`...). |

---

## 1.2 Configurer l'éditeur (VS Code)

1. Installez l'extension **rust-analyzer** (indispensable) : complétion, erreurs en direct, types affichés au survol, « aller à la définition ».
2. Optionnel : **Even Better TOML** (coloration des `Cargo.toml`), **WGSL** (coloration des shaders), **CodeLLDB** (débogueur).

> 💡 Astuce rust-analyzer : maintenez `Ctrl` et survolez une variable pour voir son type. Faites `F12` sur une fonction pour sauter à son code — y compris dans `wgpu` ou `winit`. C'est le meilleur moyen d'apprendre.

---

## 1.3 Lancer ZENTRIX

Depuis la racine du projet (le dossier qui contient le `Cargo.toml` principal) :

```bash
cargo run
```

Ce qui se passe :

1. Cargo lit `Cargo.toml` et télécharge les dépendances depuis [crates.io](https://crates.io) (une seule fois).
2. Il compile toutes les crates, dans le bon ordre (`zentrix-core` d'abord, `zentrix-app` en dernier).
3. Il lance l'exécutable `target/debug/zentrix-app`.

La **première compilation est longue** (plusieurs minutes) : `wgpu` et ses dépendances représentent des centaines de milliers de lignes. Les compilations suivantes ne recompilent que ce qui a changé.

Vous devez voir une fenêtre avec un **triangle rouge/vert/bleu** sur un fond qui oscille doucement entre bleu nuit et violet.

- `Espace` : met l'animation du fond en pause.
- `Échap` : quitte.

---

## 1.4 Lire les logs

Le terminal affiche des messages comme :

```text
[INFO  zentrix_app] Démarrage de ZENTRIX v0.1.0
[INFO  zentrix_platform::runner] Fenêtre créée : PhysicalSize { width: 1600, height: 900 }
[INFO  zentrix_rhi::context] GPU sélectionné : Intel(R) Iris(R) Xe Graphics (IntegratedGpu, API Vulkan)
[INFO  zentrix_rhi::context] Surface configurée : 1600x900 Bgra8UnormSrgb
[INFO  zentrix_render::renderer] Renderer initialisé
```

Chaque ligne indique un **niveau** (`ERROR` > `WARN` > `INFO` > `DEBUG` > `TRACE`), le **module** qui parle, et le message. Ils viennent des macros `log::info!(...)`, `log::error!(...)` présentes dans le code.

Pour en voir plus, on règle la variable d'environnement `RUST_LOG` :

```bash
# Bash / Git Bash
RUST_LOG=debug cargo run

# PowerShell
$env:RUST_LOG="debug"; cargo run
```

Exemples de filtres :

| Valeur de `RUST_LOG` | Effet |
|----------------------|-------|
| `info` | Valeur par défaut du projet |
| `debug` | Tout en mode détaillé (wgpu devient **très** bavard) |
| `info,zentrix_platform=debug` | Affiche aussi les FPS une fois par seconde |
| `warn` | Seulement les avertissements et erreurs |

### Choisir l'API graphique

wgpu choisit automatiquement la meilleure API (Vulkan ou DirectX 12 sous Windows). Pour forcer un choix :

```bash
WGPU_BACKEND=dx12 cargo run      # ou vulkan, metal, gl
```

---

## 1.5 Dépannage

| Symptôme | Cause probable / solution |
|----------|---------------------------|
| `linker 'link.exe' not found` (Windows) | Installez les Visual Studio Build Tools avec « Développement Desktop en C++ ». |
| `aucune carte graphique compatible trouvée` | Pilotes graphiques trop anciens. Mettez-les à jour, ou essayez `WGPU_BACKEND=gl`. |
| Avertissement `Loader Message ... Registry lookup failed to get layer manifest files` | **Sans gravité.** Le chargeur Vulkan signale une entrée de registre incomplète (souvent laissée par un logiciel désinstallé). Le rendu fonctionne normalement. |
| La compilation est très lente | Normal la première fois. Le profil `[profile.dev.package."*"]` du `Cargo.toml` optimise déjà les dépendances pour que le programme reste fluide en mode debug. |
| `error: edition 2024 is unstable` | Votre Rust est trop ancien : `rustup update`. |

---

## 1.6 Les commandes Cargo à connaître

| Commande | Effet |
|----------|-------|
| `cargo run` | Compile et lance le programme |
| `cargo build` | Compile sans lancer |
| `cargo check` | Vérifie que le code compile, **sans** produire d'exécutable (beaucoup plus rapide : à utiliser souvent !) |
| `cargo test --workspace` | Lance tous les tests de toutes les crates |
| `cargo test -p zentrix-core` | Lance les tests d'une seule crate (`-p` = *package*) |
| `cargo clippy --workspace` | Analyse le code et suggère des améliorations |
| `cargo fmt --all` | Reformate tout le code selon le style officiel |
| `cargo doc --open --no-deps` | Génère la documentation HTML à partir des commentaires `///` |
| `cargo run --release` | Compile avec toutes les optimisations (lent à compiler, rapide à exécuter) |

---

➡️ **Chapitre suivant** : [Les bases de Rust](02-bases-de-rust.md)
