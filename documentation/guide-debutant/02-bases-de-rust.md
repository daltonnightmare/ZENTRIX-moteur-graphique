# Chapitre 2 — Les bases de Rust, illustrées par ZENTRIX

**Objectif** : comprendre **chaque construction Rust** utilisée dans le moteur. Chaque notion est reliée à un endroit précis du code pour que vous puissiez la voir « en vrai ».

Ce chapitre ne remplace pas le livre officiel, gratuit et traduit en français : [**Le langage de programmation Rust**](https://jimskapt.github.io/rust-book-fr/). Lisez-le en parallèle ; ici, on se concentre sur ce dont ZENTRIX a besoin.

> 🧪 Pour tester un petit bout de code sans toucher au projet : <https://play.rust-lang.org>

---

## Sommaire

1. [Variables et mutabilité](#1-variables-et-mutabilité)
2. [Les types de base](#2-les-types-de-base)
3. [Fonctions et expressions](#3-fonctions-et-expressions)
4. [Les structs et les blocs `impl`](#4-les-structs-et-les-blocs-impl)
5. [Les enums et `match`](#5-les-enums-et-match)
6. [`Option` : une valeur qui peut être absente](#6-option--une-valeur-qui-peut-être-absente)
7. [`Result` et l'opérateur `?` : gérer les erreurs](#7-result-et-lopérateur---gérer-les-erreurs)
8. [La propriété (ownership)](#8-la-propriété-ownership)
9. [Les emprunts (`&` et `&mut`)](#9-les-emprunts--et-mut)
10. [Les durées de vie (`'a`, `'static`)](#10-les-durées-de-vie-a-static)
11. [Les traits](#11-les-traits)
12. [Les génériques](#12-les-génériques)
13. [`Arc` : partager une valeur](#13-arc--partager-une-valeur)
14. [Modules et visibilité](#14-modules-et-visibilité)
15. [Les macros](#15-les-macros)
16. [Les attributs `#[...]`](#16-les-attributs-)
17. [Closures et itérateurs](#17-closures-et-itérateurs)
18. [`async` / `.await`](#18-async--await)
19. [Les tests](#19-les-tests)
20. [Lire les erreurs du compilateur](#20-lire-les-erreurs-du-compilateur)

---

## 1. Variables et mutabilité

En Rust, une variable est **immuable par défaut** : une fois une valeur donnée, on ne peut plus la changer.

```rust
let x = 5;
x = 6;        // ❌ erreur : cannot assign twice to immutable variable `x`

let mut y = 5; // `mut` = mutable (modifiable)
y = 6;         // ✅
```

**Pourquoi ?** Quand vous lisez `let size = window.inner_size();`, vous savez que `size` ne changera jamais plus bas. Le code est plus facile à comprendre, et le compilateur peut mieux optimiser.

**Dans ZENTRIX** — [renderer.rs](../../crates/zentrix-render/src/renderer.rs) : l'encoder doit être `mut` car on lui ajoute des commandes :

```rust
let mut encoder = gpu.device().create_command_encoder(/* ... */);
```

### Le masquage (shadowing)

On peut redéclarer une variable du même nom avec `let`. C'est une **nouvelle** variable qui cache l'ancienne :

```rust
// crates/zentrix-core/src/color.rs, fonction lerp
let t = t.clamp(0.0, 1.0);   // le nouveau `t` est borné entre 0 et 1
```

---

## 2. Les types de base

Rust est **statiquement typé** : chaque valeur a un type connu à la compilation. Souvent, le compilateur le **devine** (on dit qu'il l'*infère*) et vous n'avez pas à l'écrire.

| Type | Description | Exemple dans ZENTRIX |
|------|-------------|----------------------|
| `i32`, `i64` | Entiers signés (positifs ou négatifs) sur 32/64 bits | — |
| `u32`, `u64` | Entiers **non signés** (≥ 0) | `width: u32` (une taille n'est jamais négative) |
| `f32`, `f64` | Nombres à virgule | `Color { r: f32, ... }` |
| `bool` | `true` / `false` | `paused: bool` |
| `[T; N]` | Tableau de taille **fixe** N | `position: [f32; 3]` |
| `Vec<T>` | Tableau de taille **variable** (alloué sur le tas) | `view_formats: vec![]` |
| `&str` | Texte emprunté (souvent écrit en dur dans le code) | `"ZENTRIX"` |
| `String` | Texte possédé et modifiable | `title: String` |
| `()` | Le type « vide » (« unit ») : rien | `anyhow::Result<()>` |

### Pas de conversion implicite

Rust ne convertit **jamais** un nombre d'un type à un autre sans qu'on le demande, avec `as` :

```rust
// crates/zentrix-render/src/renderer.rs
r: c.r as f64,                     // f32 → f64, wgpu veut des f64
vertex_count: TRIANGLE.len() as u32, // usize → u32
```

Cela évite toute une famille de bugs silencieux (dépassements, pertes de précision).

---

## 3. Fonctions et expressions

```rust
// crates/zentrix-core/src/color.rs
pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
    Self { r, g, b, a: 1.0 }
}
```

- `fn` déclare une fonction ; les paramètres ont **toujours** un type explicite.
- `-> Self` indique le type de retour.
- **Pas de `return` ni de `;` sur la dernière ligne** : en Rust, presque tout est une *expression* qui produit une valeur, et la dernière expression d'un bloc est sa valeur de retour. Si vous ajoutez un `;`, elle devient une *instruction* et la fonction renverrait `()` → erreur de type.

`if` et `match` sont aussi des expressions :

```rust
// crates/zentrix-app/src/main.rs
log::info!("Animation {}", if self.paused { "en pause" } else { "reprise" });
```

`return` existe quand même pour **sortir plus tôt** :

```rust
// crates/zentrix-rhi/src/context.rs
if width == 0 || height == 0 {
    return;
}
```

---

## 4. Les structs et les blocs `impl`

Une **struct** regroupe des données. Un bloc **`impl`** y attache des fonctions.

```rust
// crates/zentrix-core/src/color.rs
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self { /* ... */ }      // fonction associée
    pub fn lerp(self, other: Color, t: f32) -> Color { /* ... */ }     // méthode
}
```

- Une **fonction associée** (pas de `self`) s'appelle avec `::` → `Color::rgb(1.0, 0.0, 0.0)`. C'est l'équivalent d'un « constructeur ». Par convention, on appelle `new` le constructeur principal (`FrameTimer::new()`).
- Une **méthode** (premier paramètre `self`, `&self` ou `&mut self`) s'appelle avec `.` → `couleur.lerp(autre, 0.5)`.

| Premier paramètre | Signification | Exemple |
|-------------------|---------------|---------|
| `&self` | Emprunt en **lecture** | `FrameTimer::fps(&self)` |
| `&mut self` | Emprunt en **écriture** | `FrameTimer::tick(&mut self)` |
| `self` | Prend la **propriété** (consomme la valeur) | `Color::lerp(self, ...)` |

### Syntaxe utile

```rust
// Raccourci : si une variable s'appelle comme le champ
Self { r, g, b, a: 1.0 }           // au lieu de Self { r: r, g: g, b: b, a: 1.0 }

// « Tous les autres champs par défaut »
let config = WindowConfig {
    title: "ZENTRIX".to_string(),
    ..WindowConfig::default()      // crates/zentrix-app/src/main.rs
};
```

---

## 5. Les enums et `match`

Un **enum** est un type qui vaut **une** valeur parmi plusieurs possibles. Contrairement au C, chaque variante peut **contenir des données**.

```rust
// crates/zentrix-rhi/src/error.rs (simplifié)
pub enum RhiError {
    Surface(wgpu::CreateSurfaceError),  // variante qui contient une erreur wgpu
    NoSurfaceFormat,                    // variante sans donnée
    SurfaceLost,
}
```

On examine un enum avec **`match`**, qui oblige à traiter **tous** les cas :

```rust
// crates/zentrix-rhi/src/context.rs (simplifié)
let texture = match self.surface.get_current_texture() {
    wgpu::CurrentSurfaceTexture::Success(texture) => texture,   // on extrait la texture
    wgpu::CurrentSurfaceTexture::Timeout
    | wgpu::CurrentSurfaceTexture::Occluded => return Ok(None), // `|` = « ou »
    wgpu::CurrentSurfaceTexture::Outdated => { /* ... */ return Ok(None); }
    other => return Err(RhiError::SurfaceLost),                 // tous les autres cas
};
```

Si wgpu ajoute un jour une nouvelle variante et qu'on ne l'a pas traitée, **le code ne compile plus** : impossible d'oublier un cas. `_` (ou un nom comme `other`) attrape « tout le reste ».

### Les motifs (patterns) peuvent être profonds

```rust
// crates/zentrix-platform/src/runner.rs
WindowEvent::KeyboardInput {
    event: KeyEvent {
        physical_key: PhysicalKey::Code(key),  // extrait `key`
        state,                                 // extrait `state`
        repeat: false,                         // ne correspond QUE si repeat == false
        ..                                     // ignore les autres champs
    },
    ..
} => { /* on peut utiliser `key` et `state` ici */ }
```

---

## 6. `Option` : une valeur qui peut être absente

Rust n'a **pas de `null`**. Une valeur qui peut manquer est représentée par l'enum standard :

```rust
enum Option<T> {
    Some(T),  // il y a une valeur
    None,     // il n'y en a pas
}
```

Le compilateur vous **force** à gérer le cas `None` : fini les « null pointer exceptions ».

**Dans ZENTRIX** — [main.rs](../../crates/zentrix-app/src/main.rs) : les graphismes n'existent pas tant que la fenêtre n'est pas créée.

```rust
struct ZentrixApp {
    graphics: Option<GraphicsState>,   // None au départ
    // ...
}

// Plus tard : « s'il y a des graphismes, emprunte-les sous le nom `g` »
if let Some(g) = &mut self.graphics {
    g.gpu.resize(width, height);
}
```

Façons courantes de manipuler une `Option` :

| Code | Effet |
|------|-------|
| `if let Some(x) = opt { ... }` | Exécute le bloc seulement s'il y a une valeur |
| `let Some(x) = opt else { return ... };` | Extrait la valeur, ou sort de la fonction (voir `renderer.rs`) |
| `opt.is_some()` / `opt.is_none()` | Teste la présence |
| `opt.take()` | Sort la valeur et laisse `None` à la place (voir `runner.rs`) |
| `opt.as_ref()` | `&Option<T>` → `Option<&T>` : emprunter le contenu |
| `opt.unwrap()` | Extrait la valeur ou **plante** si `None`. ⚠️ À éviter hors des tests et prototypes. |

---

## 7. `Result` et l'opérateur `?` : gérer les erreurs

Une opération qui peut échouer renvoie un `Result` :

```rust
enum Result<T, E> {
    Ok(T),   // succès, avec la valeur produite
    Err(E),  // échec, avec l'erreur
}
```

### L'opérateur `?`

Écrire `?` après un `Result` signifie : **« si c'est `Err`, renvoie l'erreur immédiatement à l'appelant ; sinon, donne-moi la valeur. »**

```rust
// crates/zentrix-rhi/src/context.rs
let surface = instance.create_surface(Arc::clone(&window))?;
```

équivaut à :

```rust
let surface = match instance.create_surface(Arc::clone(&window)) {
    Ok(s) => s,
    Err(e) => return Err(e.into()),   // `.into()` convertit l'erreur au bon type
};
```

Le `.into()` caché est important : `create_surface` renvoie une `wgpu::CreateSurfaceError`, mais notre fonction renvoie un `RhiError`. La conversion existe grâce à `#[from]` dans [error.rs](../../crates/zentrix-rhi/src/error.rs).

### Deux styles d'erreurs dans ZENTRIX

| Crate | Style | Pourquoi |
|-------|-------|----------|
| `zentrix-rhi` (bibliothèque) | Enum **`RhiError`** avec `thiserror` | L'appelant peut distinguer précisément les cas (`match`) |
| `zentrix-app`, `zentrix-platform` | **`anyhow::Result`** | On veut juste propager et afficher l'erreur, avec du contexte |

```rust
// crates/zentrix-app/src/main.rs
let gpu = pollster::block_on(GpuContext::new(Arc::clone(&window)))
    .context("échec de l'initialisation du GPU")?;
```

`.context(...)` (de `anyhow`) ajoute une explication. En cas d'échec, vous verrez :

```text
Erreur fatale : échec de l'initialisation du GPU: aucune carte graphique compatible trouvée : ...
```

### Et `panic!` ?

`panic!` (et `unwrap()`, `expect()`) **arrête le programme** brutalement. On le réserve aux bugs « impossibles » (une invariante violée), jamais aux erreurs prévisibles comme « fichier introuvable » ou « pas de GPU ». La v1 du projet utilisait beaucoup `.expect(...)` ; la nouvelle base propage proprement les erreurs avec `?`.

---

## 8. La propriété (ownership)

C'est **LA** notion qui rend Rust unique. Trois règles :

1. Chaque valeur a **un seul propriétaire** (une variable).
2. Quand le propriétaire sort de portée (fin du bloc `}`), la valeur est **détruite** automatiquement.
3. On peut **transférer** (déplacer, *move*) la propriété à une autre variable ou à une fonction.

```rust
let a = String::from("bonjour");
let b = a;            // la propriété passe de `a` à `b` (move)
println!("{a}");      // ❌ erreur : borrow of moved value: `a`
```

**Pourquoi c'est génial pour un moteur graphique** : la mémoire et les ressources GPU sont libérées **exactement** quand il le faut, sans ramasse-miettes (*garbage collector*) qui provoquerait des saccades, et sans `free()` manuel qu'on pourrait oublier.

**Dans ZENTRIX** — [renderer.rs](../../crates/zentrix-render/src/renderer.rs) : le `Renderer` possède son pipeline et son buffer. Quand le `Renderer` est détruit, eux aussi, et wgpu libère la mémoire GPU correspondante.

**Exemple concret de move qui protège** — [context.rs](../../crates/zentrix-rhi/src/context.rs) :

```rust
pub fn present(&self, frame: Frame) { /* ... */ }
```

`present` prend la `Frame` **par valeur** : elle est déplacée dans la fonction. Après `gpu.present(frame);`, la variable `frame` n'est plus utilisable. Le compilateur **interdit** donc de dessiner dans une image déjà envoyée à l'écran. Une règle de l'API wgpu devient une règle vérifiée à la compilation !

### `Copy` : l'exception des petits types

Les types simples (`i32`, `f32`, `bool`...) et ceux marqués `#[derive(Clone, Copy)]` comme notre `Color` sont **copiés** au lieu d'être déplacés. C'est pour ça que `Color::lerp(self, ...)` ne « détruit » pas votre couleur.

---

## 9. Les emprunts (`&` et `&mut`)

Déplacer la propriété à chaque appel serait pénible. On peut donc **prêter** une valeur :

| Syntaxe | Nom | Règle |
|---------|-----|-------|
| `&valeur` | Emprunt partagé (lecture) | On peut en avoir **autant qu'on veut** en même temps |
| `&mut valeur` | Emprunt exclusif (écriture) | **Un seul** à la fois, et aucun emprunt de lecture pendant ce temps |

> **Règle d'or : soit plusieurs lecteurs, soit un seul écrivain.**

Cette règle, vérifiée par le **borrow checker**, élimine à la compilation les bugs de données modifiées « dans le dos » et les *data races* entre threads.

**Dans ZENTRIX** :

```rust
// crates/zentrix-render/src/renderer.rs
pub fn new(gpu: &GpuContext) -> Self              // emprunte le GPU en lecture
pub fn render(&self, gpu: &mut GpuContext) -> ... // emprunte le GPU en écriture
```

`render` a besoin de `&mut` car acquérir une image peut **reconfigurer** la surface.

### Le cas de la render pass : une portée dédiée

```rust
// crates/zentrix-render/src/renderer.rs
{
    let mut render_pass = encoder.begin_render_pass(/* ... */); // emprunte `encoder` en &mut
    render_pass.draw(/* ... */);
}   // ← render_pass est détruite ici : l'emprunt se termine

gpu.queue().submit(std::iter::once(encoder.finish())); // ✅ encoder est de nouveau libre
```

Sans les accolades, `encoder.finish()` serait refusé car `render_pass` emprunterait encore `encoder`.

---

## 10. Les durées de vie (`'a`, `'static`)

Une **durée de vie** (*lifetime*) est le nom qu'on donne à « combien de temps un emprunt reste valide ». Le compilateur les déduit presque toujours seul. Vous les verrez surtout sous deux formes :

- `'a` dans une signature : « cette référence ne doit pas survivre à la donnée d'origine ».
- `'static` : « valable pendant toute la durée du programme ».

**Dans ZENTRIX** — [context.rs](../../crates/zentrix-rhi/src/context.rs) :

```rust
surface: wgpu::Surface<'static>,
```

Une surface dessine **dans** une fenêtre : elle ne doit pas lui survivre. Pour obtenir `'static`, on lui donne un `Arc<Window>` (voir §13) : la surface devient **copropriétaire** de la fenêtre, qui ne peut donc pas disparaître avant elle.

> 📜 **Historique** : la v1 utilisait `Box::leak(Box::new(window))` pour obtenir un `&'static Window`. Ça fonctionne, mais la mémoire de la fenêtre n'est jamais libérée (c'est une fuite volontaire). `Arc` est la solution propre.

---

## 11. Les traits

Un **trait** décrit un **comportement** : une liste de méthodes qu'un type s'engage à fournir. C'est l'équivalent des *interfaces* d'autres langages.

```rust
// crates/zentrix-platform/src/application.rs (simplifié)
pub trait Application {
    fn on_start(&mut self, window: Arc<Window>) -> anyhow::Result<()>; // obligatoire
    fn on_render(&mut self) -> anyhow::Result<()>;                      // obligatoire
    fn on_update(&mut self, _dt: f32) {}        // facultatif : corps par défaut vide
    fn should_exit(&self) -> bool { false }     // facultatif
}
```

On « signe le contrat » avec `impl Trait for Type` :

```rust
// crates/zentrix-app/src/main.rs
impl Application for ZentrixApp {
    fn on_start(&mut self, window: Arc<Window>) -> anyhow::Result<()> { /* ... */ }
    fn on_render(&mut self) -> anyhow::Result<()> { /* ... */ }
    // on_update, on_key... redéfinis aussi
}
```

**Pourquoi c'est utile ici** : `zentrix-platform` ne connaît **pas** `ZentrixApp`. Il sait seulement manipuler « quelque chose qui implémente `Application` ». Demain, un jeu ou un éditeur pourra implémenter le même trait et réutiliser toute la plateforme.

ZENTRIX implémente aussi des traits de la bibliothèque standard ou de winit :

| Trait | Où | Rôle |
|-------|-----|------|
| `Default` | `Color`, `FrameTimer`, `WindowConfig` | Fournir une valeur par défaut |
| `ApplicationHandler` (winit) | `PlatformRunner` | Recevoir les événements de la fenêtre |
| `Debug`, `Clone`, `Copy`, `PartialEq` | via `#[derive]` | Affichage, copie, comparaison |
| `std::error::Error` | `RhiError` via `thiserror` | Être une « vraie » erreur |

---

## 12. Les génériques

Un **générique** est un type « à trou », rempli plus tard. Vous en utilisez déjà : `Option<T>`, `Vec<T>`, `Result<T, E>`.

```rust
// crates/zentrix-platform/src/runner.rs
pub fn run<A: Application>(config: WindowConfig, app: A) -> anyhow::Result<()>
```

Se lit : « `run` accepte n'importe quel type `A`, **à condition** qu'il implémente `Application` ». La partie `: Application` s'appelle une **contrainte de trait** (*trait bound*).

Le compilateur génère une copie spécialisée de `run` pour chaque type utilisé (*monomorphisation*) : **aucun coût** à l'exécution, c'est aussi rapide que si on avait écrit `run` spécialement pour `ZentrixApp`.

---

## 13. `Arc` : partager une valeur

Parfois, une valeur doit vraiment avoir **plusieurs propriétaires**. Ici, la fenêtre est utilisée par :

- la plateforme (pour demander de redessiner),
- la surface wgpu (pour afficher dedans),
- l'application (pour changer le titre).

`Arc<T>` (*Atomically Reference Counted*) est un pointeur qui **compte** ses propriétaires :

```rust
let window = Arc::new(window);              // compteur = 1
let pour_gpu = Arc::clone(&window);         // compteur = 2 (ne copie PAS la fenêtre)
drop(pour_gpu);                             // compteur = 1
// ... quand le dernier Arc disparaît, compteur = 0 → la fenêtre est détruite
```

- `Arc::clone(&x)` est très peu coûteux : il incrémente juste un nombre.
- « Atomic » : le compteur fonctionne même entre plusieurs threads. (`Rc` est la version mono-thread.)
- Le contenu d'un `Arc` est en **lecture seule**. Pour le modifier, on combine avec un `Mutex` ; on n'en a pas besoin ici car les méthodes de `Window` prennent `&self`.

---

## 14. Modules et visibilité

Le code d'une crate est découpé en **modules**.

```rust
// crates/zentrix-core/src/lib.rs
mod color;              // charge le fichier color.rs comme module privé
mod time;

pub use color::Color;   // ré-exporte Color à la racine de la crate
pub use time::FrameTimer;
```

- `mod nom;` → Rust cherche `nom.rs` (ou `nom/mod.rs`). **Un fichier non déclaré avec `mod` est ignoré !**
- Tout est **privé par défaut**.

| Mot-clé | Visible depuis |
|---------|----------------|
| *(rien)* | Le module courant et ses enfants |
| `pub(crate)` | Toute la crate, mais pas les autres crates (ex. `create_triangle_pipeline`) |
| `pub` | Partout |

Chemins : `crate::` part de la racine de la crate courante, `super::` remonte d'un module, et `zentrix_core::` désigne une autre crate (le `-` du nom de crate devient `_` dans le code).

---

## 15. Les macros

Une **macro** se reconnaît au `!`. C'est du code qui **génère du code** à la compilation.

| Macro | Rôle |
|-------|------|
| `println!("x = {x}")` | Afficher dans le terminal |
| `format!("...")` | Construire une `String` (voir le titre de la fenêtre) |
| `log::info!(...)`, `log::error!(...)` | Écrire un log |
| `vec![a, b]` | Créer un `Vec` |
| `include_str!("fichier")` | Insérer le contenu d'un fichier à la compilation (le shader) |
| `env!("CARGO_PKG_VERSION")` | Lire une variable d'environnement de compilation (la version) |
| `wgpu::vertex_attr_array![...]` | Générer la description des attributs de sommet |
| `assert!`, `assert_eq!` | Vérifier une condition (dans les tests) |

Dans `format!` et les logs : `{}` affiche avec `Display` (lisible), `{:?}` avec `Debug` (technique), `{:.0}` arrondit à 0 décimale, `{x}` insère directement la variable `x`.

---

## 16. Les attributs `#[...]`

Les **attributs** donnent des instructions au compilateur.

| Attribut | Rôle |
|----------|------|
| `#[derive(Debug, Clone, Copy)]` | Générer automatiquement l'implémentation de traits |
| `#[repr(C)]` | Garder l'ordre des champs en mémoire (obligatoire pour les données envoyées au GPU, voir `Vertex`) |
| `#[cfg(test)]` | Ne compiler ce bloc que pendant `cargo test` |
| `#[test]` | Cette fonction est un test |
| `#[error("...")]`, `#[from]` | Attributs propres à `thiserror` |

---

## 17. Closures et itérateurs

Une **closure** est une petite fonction anonyme : `|paramètres| corps`.

```rust
// crates/zentrix-rhi/src/context.rs
let format = capabilities
    .formats
    .iter()                                         // parcourt la liste
    .find(|f| f.is_srgb())                          // premier format sRGB → Option
    .copied()                                       // Option<&Format> → Option<Format>
    .or_else(|| capabilities.formats.first().copied()) // sinon le premier tout court
    .ok_or(RhiError::NoSurfaceFormat)?;             // None → erreur
```

On enchaîne des **adaptateurs** (`find`, `map`, `filter`...) sur un **itérateur**. C'est aussi rapide qu'une boucle `for` écrite à la main, mais plus expressif.

---

## 18. `async` / `.await`

Certaines opérations prennent du temps (trouver un GPU, ouvrir le device). wgpu les rend **asynchrones** :

```rust
// crates/zentrix-rhi/src/context.rs
pub async fn new(window: Arc<Window>) -> Result<Self, RhiError> {
    let adapter = instance.request_adapter(/* ... */).await?;
    // ...
}
```

- Une fonction `async` ne s'exécute pas immédiatement : elle renvoie un **`Future`**, une « promesse » de résultat.
- `.await` attend ce résultat.
- Quelqu'un doit « faire tourner » le `Future`. Les gros programmes réseau utilisent `tokio`. Nous, on veut juste attendre : `pollster::block_on(...)` suffit.

```rust
// crates/zentrix-app/src/main.rs
let gpu = pollster::block_on(GpuContext::new(Arc::clone(&window)))?;
```

---

## 19. Les tests

```rust
// crates/zentrix-core/src/color.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_au_milieu() {
        let gris = Color::BLACK.lerp(Color::WHITE, 0.5);
        assert_eq!(gris, Color::rgb(0.5, 0.5, 0.5));
    }
}
```

Lancer : `cargo test --workspace`. Les exemples de code dans les commentaires `///` sont **aussi** exécutés comme tests (*doc-tests*) : la documentation ne peut pas devenir fausse sans qu'on le sache.

Tester du code GPU est difficile (il faut une carte graphique). C'est une des raisons de garder `zentrix-core` sans aucune dépendance GPU : toute la logique pure y est facilement testable.

---

## 20. Lire les erreurs du compilateur

Le compilateur Rust est strict, mais c'est votre meilleur professeur. Ses messages sont détaillés :

```text
error[E0382]: borrow of moved value: `frame`
  --> crates/zentrix-render/src/renderer.rs:98:9
   |
95 |         gpu.present(frame);
   |                     ----- value moved here
...
98 |         frame.view;
   |         ^^^^^^^^^^ value borrowed here after move
```

Méthode :

1. Lisez la **première** erreur seulement (les suivantes en découlent souvent).
2. Regardez le fichier et la ligne (`-->`).
3. Lisez les annotations sous le code : elles disent *où* et *pourquoi*.
4. Lisez les lignes `help:` : elles proposent souvent la correction exacte.
5. Pour plus de détails : `rustc --explain E0382`.

> 💬 « Lutter contre le borrow checker » est une étape normale. Au bout de quelques semaines, vous écrirez naturellement du code qu'il accepte — et vous remarquerez que ce code est aussi plus clair.

---

➡️ **Chapitre suivant** : [Cargo et le workspace](03-cargo-et-workspace.md)
