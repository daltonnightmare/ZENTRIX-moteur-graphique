# Chapitre 8 — Glossaire

Tous les termes techniques du projet, classés par ordre alphabétique. Revenez-y dès qu'un mot vous bloque.

---

## Rust

| Terme | Explication |
|-------|-------------|
| **`anyhow`** | Crate qui fournit un type d'erreur « universel » (`anyhow::Error`), pratique dans les applications. |
| **`Arc<T>`** | Pointeur partagé à comptage de références : plusieurs propriétaires pour une même valeur. Voir [chap. 2 §13](02-bases-de-rust.md#13-arc--partager-une-valeur). |
| **`async` / `await`** | Fonctions dont le résultat arrive « plus tard » (un `Future`) ; `.await` attend ce résultat. |
| **Borrow checker** | La partie du compilateur qui vérifie les règles d'emprunt (`&` / `&mut`). |
| **Cargo** | L'outil qui compile, teste, lance et gère les dépendances. |
| **`Cargo.lock`** | Fichier qui fige les versions exactes de toutes les dépendances. Versionné dans git. |
| **Closure** | Fonction anonyme : `|x| x + 1`. |
| **Crate** | Unité de compilation Rust : une bibliothèque (`lib.rs`) ou un exécutable (`main.rs`). |
| **crates.io** | Le registre public des crates Rust. |
| **`derive`** | Attribut qui génère automatiquement l'implémentation d'un trait : `#[derive(Debug, Clone)]`. |
| **Doc-test** | Exemple de code dans un commentaire `///`, exécuté par `cargo test`. |
| **Édition** | Version du langage (2015, 2018, 2021, 2024), choisie par crate. ZENTRIX utilise 2024. |
| **Emprunt** (*borrow*) | Accès temporaire à une valeur sans en prendre la propriété : `&x` (lecture) ou `&mut x` (écriture). |
| **Enum** | Type qui vaut une variante parmi plusieurs, chacune pouvant contenir des données. |
| **Feature** | Partie optionnelle d'une crate, activée dans `Cargo.toml`. |
| **Générique** | Type ou fonction paramétré par un autre type : `Vec<T>`, `fn run<A: Application>`. |
| **`impl`** | Bloc qui définit les méthodes d'un type, ou l'implémentation d'un trait pour ce type. |
| **Durée de vie** (*lifetime*) | Annotation (`'a`, `'static`) qui exprime combien de temps une référence reste valide. |
| **Macro** | Code qui génère du code, reconnaissable au `!` : `println!`, `vec!`. |
| **`match`** | Expression qui examine une valeur et traite **tous** ses cas possibles. |
| **Module** | Sous-partie d'une crate, généralement un fichier, déclaré avec `mod`. |
| **Move** | Transfert de propriété d'une variable à une autre. L'ancienne devient inutilisable. |
| **`Option<T>`** | `Some(valeur)` ou `None` : remplace `null`. |
| **Propriété** (*ownership*) | Chaque valeur a un unique propriétaire ; elle est détruite quand il sort de portée. |
| **`?`** | Opérateur qui propage une erreur à l'appelant. |
| **`Result<T, E>`** | `Ok(valeur)` ou `Err(erreur)` : le résultat d'une opération qui peut échouer. |
| **rust-analyzer** | Extension d'éditeur qui analyse le code en direct. |
| **rustup** | Gestionnaire des versions de Rust. |
| **Struct** | Type qui regroupe plusieurs champs nommés. |
| **`thiserror`** | Crate qui simplifie la création de types d'erreur précis pour les bibliothèques. |
| **Trait** | Ensemble de méthodes qu'un type s'engage à fournir (comme une interface). |
| **`unwrap()`** | Extrait la valeur d'une `Option`/`Result`, ou plante. À éviter hors des tests. |
| **Workspace** | Ensemble de crates compilées ensemble, avec un `target/` et un `Cargo.lock` communs. |

---

## Graphisme et GPU

| Terme | Explication |
|-------|-------------|
| **Adapter** | Dans wgpu : une carte graphique physique précise. |
| **Backend** | L'API native utilisée par wgpu : Vulkan, DirectX 12, Metal, OpenGL. |
| **Bind group** | Lot de ressources (buffers, textures) branché sur un shader. |
| **Bind group layout** | Description du type de ressources qu'attend un bind group. |
| **Buffer** | Bloc de mémoire sur le GPU (sommets, indices, uniforms...). |
| **Clear color** | Couleur utilisée pour effacer l'image au début d'une passe. |
| **Clip space / NDC** | Système de coordonnées du GPU après le vertex shader : x et y de -1 à +1. |
| **Command encoder** | Objet qui enregistre des commandes GPU avant de les soumettre. |
| **Culling** | Élimination des triangles invisibles (par exemple vus de dos). |
| **Delta time** (`dt`) | Temps écoulé depuis l'image précédente. Rend les mouvements indépendants des FPS. |
| **Depth buffer** | Texture qui stocke la profondeur de chaque pixel, pour que les objets proches cachent les lointains. |
| **Device** | Dans wgpu : la connexion logique au GPU, qui sert à créer des ressources. |
| **Draw call** | Une commande de dessin (`draw`, `draw_indexed`). |
| **ECS** | *Entity Component System* : architecture où les objets du jeu (entités) sont des identifiants auxquels on attache des données (composants), traitées par des systèmes. |
| **Fragment shader** | Programme GPU exécuté pour chaque pixel couvert par un triangle ; calcule sa couleur. |
| **Frame** | Une image complète affichée à l'écran. |
| **FPS** | *Frames Per Second* : images par seconde. |
| **Index buffer** | Buffer d'indices qui permet de réutiliser des sommets entre plusieurs triangles. |
| **Instance** | Dans wgpu : le point d'entrée, qui liste les GPU et les API. (Ne pas confondre avec le *rendu instancié*.) |
| **Interpolation** | Le GPU mélange les valeurs des sommets pour chaque pixel intermédiaire (d'où le dégradé du triangle). |
| **Lerp** | *Linear interpolation* : mélange de deux valeurs selon un facteur `t` entre 0 et 1. |
| **Pipeline** | Objet qui fige tout l'état de rendu : shaders, format des sommets, culling, blending, format de sortie. |
| **Present mode** | Façon d'afficher les images : `Fifo` (V-Sync), `Mailbox`, `Immediate`. |
| **Queue** | Dans wgpu : la file qui envoie le travail au GPU. |
| **Rastérisation** | Étape fixe du GPU qui transforme des triangles en pixels. |
| **Render pass** | Séquence de dessins vers une même image cible. |
| **RHI** | *Render Hardware Interface* : couche d'abstraction bas niveau du GPU (`zentrix-rhi`). |
| **Sampler** | Objet qui décrit comment lire une texture (filtrage, répétition). |
| **Shader** | Programme exécuté par le GPU. |
| **sRGB** | Espace de couleur non linéaire des écrans. |
| **Surface** | Dans wgpu : la zone de la fenêtre où les images sont affichées. |
| **Swapchain** | Ensemble d'images qui tournent : on dessine dans l'une pendant que l'écran affiche l'autre. |
| **Texture** | Image stockée sur le GPU. |
| **Uniform** | Donnée constante pour tout un draw call (matrice de caméra, temps). |
| **UV** | Coordonnées 2D (0–1) qui indiquent quelle partie d'une texture appliquer sur un sommet. |
| **Vertex** | Un sommet : position, couleur, normale, UV... |
| **Vertex shader** | Programme GPU exécuté pour chaque sommet ; calcule sa position à l'écran. |
| **V-Sync** | Synchronisation avec le rafraîchissement de l'écran : évite le déchirement d'image. |
| **WGSL** | *WebGPU Shading Language* : le langage des shaders de wgpu. |
| **wgpu** | Bibliothèque Rust de graphisme portable (Vulkan/DX12/Metal/OpenGL/WebGPU). |
| **winit** | Bibliothèque Rust de création de fenêtres et de gestion des événements. |

---

⬅️ **Retour au sommaire** : [documentation/README.md](../README.md)
