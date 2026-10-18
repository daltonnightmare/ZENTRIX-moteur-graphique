# Chapitre 3 — Cargo et le workspace

**Objectif** : comprendre comment le projet est organisé en plusieurs crates, comment elles dépendent les unes des autres, et savoir en ajouter une.

---

## 3.1 Qu'est-ce qu'une crate ?

Une **crate** est l'unité de compilation de Rust : un projet avec son propre `Cargo.toml`. Il en existe deux sortes :

| Sorte | Fichier racine | Produit | Exemple |
|-------|----------------|---------|---------|
| **Bibliothèque** (*lib*) | `src/lib.rs` | Du code réutilisable par d'autres crates | `zentrix-core`, `zentrix-rhi`... |
| **Binaire** (*bin*) | `src/main.rs` | Un exécutable avec une fonction `main()` | `zentrix-app` |

Les crates téléchargées depuis [crates.io](https://crates.io) (`wgpu`, `winit`...) sont aussi des crates bibliothèques.

> 📌 **Package** vs **crate** : un *package* est un dossier avec un `Cargo.toml` ; il contient une ou plusieurs crates. Dans ZENTRIX, chaque package contient exactement une crate, donc les deux mots sont interchangeables.

---

## 3.2 Pourquoi un workspace ?

La v1 tenait dans un seul `main.rs` de 300 lignes. Ça marche pour un prototype, mais un moteur grandit vite (fenêtre, GPU, rendu, scène, assets, audio, physique...). Le **workspace** découpe le projet en crates indépendantes qui partagent :

- **un seul dossier `target/`** : une dépendance n'est compilée qu'une fois pour toutes les crates ;
- **un seul `Cargo.lock`** : toutes les crates utilisent exactement les mêmes versions ;
- **les versions déclarées une seule fois** dans `[workspace.dependencies]`.

Avantages du découpage :

1. **Frontières claires** : `zentrix-core` ne *peut pas* utiliser wgpu, puisqu'il ne l'a pas dans ses dépendances. L'architecture est garantie par le compilateur, pas seulement par la bonne volonté.
2. **Compilation incrémentale** : modifier `zentrix-app` ne recompile pas `zentrix-rhi`.
3. **Tests ciblés** : `cargo test -p zentrix-core`.
4. **Réutilisation** : un futur éditeur pourra réutiliser `zentrix-render` sans `zentrix-app`.

---

## 3.3 Le graphe de dépendances

```text
zentrix-app ──────┬──────────────┬──────────────┬──────────────┐
                  ▼              ▼              ▼              ▼
          zentrix-platform  zentrix-render  zentrix-rhi   zentrix-core
                  │              │   │          │
                  │              │   └────▶ zentrix-rhi ──▶ wgpu, winit
                  │              └────────▶ zentrix-core
                  ├──▶ winit
                  └──▶ zentrix-core
```

Règle importante : **les flèches vont toujours vers le bas**. Une crate de bas niveau (`zentrix-core`, `zentrix-rhi`) ne dépend jamais d'une crate de plus haut niveau. Cargo interdit d'ailleurs les dépendances circulaires.

| Crate | Dépend de | Ne doit **pas** connaître |
|-------|-----------|---------------------------|
| `zentrix-core` | rien | le GPU, la fenêtre |
| `zentrix-platform` | `winit`, `zentrix-core` | le GPU |
| `zentrix-rhi` | `wgpu`, `winit` | les triangles, caméras, matériaux |
| `zentrix-render` | `zentrix-rhi`, `zentrix-core` | la fenêtre, le clavier, la logique de jeu |
| `zentrix-app` | tout | — |

---

## 3.4 Lire le `Cargo.toml` racine

```toml
[workspace]
resolver = "3"
members = [
    "crates/zentrix-core",
    "crates/zentrix-platform",
    # ...
]
```

- `members` : la liste des crates du workspace.
- `resolver = "3"` : version moderne de l'algorithme qui choisit les versions des dépendances.

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
```

Des métadonnées communes. Chaque crate écrit `version.workspace = true` pour en hériter.

> 📌 **L'édition** (`2015`, `2018`, `2021`, `2024`) est une version du *langage*, pas du compilateur. Elle permet à Rust d'évoluer sans casser l'ancien code : chaque crate choisit son édition, et des crates d'éditions différentes peuvent se mélanger.

```toml
[workspace.dependencies]
zentrix-core = { path = "crates/zentrix-core" }   # une crate locale
wgpu = "30"                                       # une crate de crates.io
bytemuck = { version = "1", features = ["derive"] }
```

Le catalogue des dépendances. Une crate qui en a besoin écrit simplement :

```toml
# crates/zentrix-render/Cargo.toml
[dependencies]
zentrix-core = { workspace = true }
bytemuck = { workspace = true }
```

### Les numéros de version (SemVer)

`wgpu = "30"` signifie « n'importe quelle version **compatible** avec 30.x.y ». Cargo suit le *Semantic Versioning* :

- `MAJEUR.MINEUR.CORRECTIF` (ex. `30.0.1`) ;
- changement de **majeur** = l'API peut casser ;
- pour les versions `0.x` (comme `winit = "0.30"`), c'est le **mineur** qui joue ce rôle : `0.30` → `0.31` peut casser.

La version **exacte** retenue est inscrite dans `Cargo.lock`. Ce fichier est versionné dans git pour que tout le monde compile exactement la même chose. `cargo update` le met à jour dans les limites autorisées.

### Les *features*

`features = ["derive"]` active une partie optionnelle d'une crate. Ici, cela permet d'écrire `#[derive(bytemuck::Pod)]`. Les features évitent de compiler du code dont on n'a pas besoin.

### Le profil de compilation

```toml
[profile.dev.package."*"]
opt-level = 2
```

En mode debug (`cargo run`), le code n'est pas optimisé : compilation rapide, débogage facile... mais wgpu deviendrait très lent. Cette section optimise **uniquement les dépendances** (`"*"`), pas notre code. On profite des deux mondes.

---

## 3.5 Ajouter une dépendance externe

Exemple : on veut utiliser `glam` (mathématiques, déjà déclarée dans le catalogue) dans `zentrix-render`.

1. Vérifier qu'elle est dans `[workspace.dependencies]` du `Cargo.toml` racine (sinon l'y ajouter).
2. Dans `crates/zentrix-render/Cargo.toml` :
   ```toml
   [dependencies]
   glam = { workspace = true }
   ```
3. L'utiliser dans le code : `use glam::Mat4;`

Raccourci en ligne de commande pour une **nouvelle** dépendance :

```bash
cargo add nom_de_la_crate -p zentrix-render
```

(Pensez ensuite à déplacer la version dans `[workspace.dependencies]` pour garder un catalogue unique.)

> 🔎 Pour trouver une crate : <https://crates.io> et <https://lib.rs>. Sa documentation est sur `https://docs.rs/nom_de_la_crate`.

---

## 3.6 Ajouter une nouvelle crate au moteur

Exemple : créer `zentrix-scene` (prévue dans [crates.md](../crates.md)).

```bash
cargo new --lib crates/zentrix-scene
```

Puis :

1. **Remplacer** le contenu de `crates/zentrix-scene/Cargo.toml` par :
   ```toml
   [package]
   name = "zentrix-scene"
   version.workspace = true
   edition.workspace = true
   authors.workspace = true

   [dependencies]
   zentrix-core = { workspace = true }
   ```
2. Vérifier que `"crates/zentrix-scene"` apparaît dans `members` du `Cargo.toml` racine (`cargo new` l'ajoute normalement tout seul).
3. L'ajouter au catalogue :
   ```toml
   [workspace.dependencies]
   zentrix-scene = { path = "crates/zentrix-scene" }
   ```
4. L'utiliser depuis `zentrix-app` : `zentrix-scene = { workspace = true }`.
5. `cargo check --workspace` pour vérifier.

---

## 3.7 Où sont les fichiers compilés ?

```text
target/
├── debug/            ← cargo build / cargo run
│   └── zentrix-app.exe
├── release/          ← cargo build --release
└── doc/              ← cargo doc
```

`target/` peut peser plusieurs gigaoctets. Il est ignoré par git (`.gitignore`). En cas de souci étrange, `cargo clean` le supprime (la prochaine compilation sera longue).

---

➡️ **Chapitre suivant** : [Comprendre le GPU](04-comprendre-le-gpu.md)
