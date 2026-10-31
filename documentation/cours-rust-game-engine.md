# Cours complet : de débutant en Rust à créateur de game engine

> **Pour qui ?** Une personne qui débute en Rust (et éventuellement en programmation système) et qui veut, au bout du parcours, être capable de concevoir et d'écrire un **moteur de jeu polyvalent et performant** — en l'occurrence ZENTRIX.
>
> **Comment ?** 24 modules en 4 parties. Chaque module contient : les **notions** expliquées, des **exemples** commentés, des **exercices** corrigés, et un **lien avec le moteur**. Chaque partie se termine par un **projet** qui valide les acquis.

---

## Table des matières

- [Mode d'emploi](#mode-demploi)
- **Partie I — Les fondations de Rust** (≈ 6 semaines)
  - [Module 1 — Premiers pas](#module-1--premiers-pas)
  - [Module 2 — Propriété et emprunts](#module-2--propriété-et-emprunts)
  - [Module 3 — Structs, enums et `match`](#module-3--structs-enums-et-match)
  - [Module 4 — Collections et chaînes de caractères](#module-4--collections-et-chaînes-de-caractères)
  - [Module 5 — La gestion des erreurs](#module-5--la-gestion-des-erreurs)
  - [Module 6 — Modules, crates, tests et documentation](#module-6--modules-crates-tests-et-documentation)
  - [Projet I — Le jeu de la vie](#projet-i--le-jeu-de-la-vie)
- **Partie II — Rust intermédiaire** (≈ 6 semaines)
  - [Module 7 — Génériques et traits](#module-7--génériques-et-traits)
  - [Module 8 — Les durées de vie](#module-8--les-durées-de-vie)
  - [Module 9 — Closures et itérateurs](#module-9--closures-et-itérateurs)
  - [Module 10 — Pointeurs intelligents](#module-10--pointeurs-intelligents)
  - [Module 11 — Traits avancés et surcharge d'opérateurs](#module-11--traits-avancés-et-surcharge-dopérateurs)
  - [Module 12 — Les macros déclaratives](#module-12--les-macros-déclaratives)
  - [Projet II — Une bibliothèque mathématique 3D](#projet-ii--une-bibliothèque-mathématique-3d)
- **Partie III — Rust pour la performance** (≈ 6 semaines)
  - [Module 13 — Comment la machine exécute votre code](#module-13--comment-la-machine-exécute-votre-code)
  - [Module 14 — Mesurer avant d'optimiser](#module-14--mesurer-avant-doptimiser)
  - [Module 15 — La concurrence et le parallélisme](#module-15--la-concurrence-et-le-parallélisme)
  - [Module 16 — `unsafe` et FFI](#module-16--unsafe-et-ffi)
  - [Module 17 — Conception orientée données et SIMD](#module-17--conception-orientée-données-et-simd)
  - [Projet III — 100 000 particules](#projet-iii--100-000-particules)
- **Partie IV — Architecture d'un game engine** (≈ 12 semaines et plus)
  - [Module 18 — La boucle de jeu](#module-18--la-boucle-de-jeu)
  - [Module 19 — Le rendu avec wgpu](#module-19--le-rendu-avec-wgpu)
  - [Module 20 — L'ECS (Entity Component System)](#module-20--lecs-entity-component-system)
  - [Module 21 — Les assets et les handles](#module-21--les-assets-et-les-handles)
  - [Module 22 — Entrées, audio, physique](#module-22--entrées-audio-physique)
  - [Module 23 — La polyvalence : plugins, sérialisation, scripts](#module-23--la-polyvalence--plugins-sérialisation-scripts)
  - [Module 24 — La performance du rendu](#module-24--la-performance-du-rendu)
  - [Projet final — Une tranche verticale jouable](#projet-final--une-tranche-verticale-jouable)
- [Annexes](#annexes)

---

## Mode d'emploi

### Le rythme

| Partie | Durée indicative (5–8 h/semaine) | À la fin, vous savez... |
|--------|----------------------------------|--------------------------|
| I | 6 semaines | écrire des programmes Rust corrects et lire le code de ZENTRIX |
| II | 6 semaines | concevoir des API propres, génériques et réutilisables |
| III | 6 semaines | mesurer et rendre votre code rapide, l'exécuter sur plusieurs cœurs |
| IV | 12 semaines et plus | construire et faire évoluer chaque sous-système d'un moteur |

Ne cherchez pas à aller vite. **Un module n'est terminé que lorsque vous savez faire ses exercices sans regarder la solution.**

### La méthode

1. **Lisez** la partie « Notions » avec un terminal ouvert.
2. **Tapez** les exemples vous-même (pas de copier-coller : les doigts apprennent aussi).
3. **Cassez-les** volontairement pour lire les erreurs du compilateur.
4. **Faites les exercices.** Les solutions sont repliées (`▶ Solution`) : ouvrez-les seulement après avoir essayé au moins 20 minutes.
5. **Notez** dans un fichier ce qui vous a surpris. Relisez-le chaque semaine.

### Le terrain d'entraînement

Créez un dossier d'exercices **séparé** du moteur :

```bash
cargo new exercices-rust
cd exercices-rust
cargo run
```

Pour chaque module, vous pouvez créer un fichier dans `src/bin/` (par exemple `src/bin/module01.rs`) et le lancer avec `cargo run --bin module01`.

Pour tester un bout de code très vite, sans rien installer : <https://play.rust-lang.org>.

### Les ressources compagnons (gratuites)

| Ressource | Usage |
|-----------|-------|
| [Le livre Rust en français](https://jimskapt.github.io/rust-book-fr/) | La référence. Ce cours indique quel chapitre lire à chaque module. |
| [Rustlings](https://github.com/rust-lang/rustlings) | Petits exercices à corriger dans le terminal. Excellent complément de la partie I. |
| [Rust by Example](https://doc.rust-lang.org/rust-by-example/) | Exemples courts pour chaque notion (anglais). |
| [docs.rs](https://docs.rs) | La documentation de toutes les crates. |
| [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) | Le tutoriel de référence pour le rendu (partie IV). |
| [Game Programming Patterns](https://gameprogrammingpatterns.com/) | Les patrons de conception des moteurs de jeu, lisible gratuitement en ligne. |

> ⚙️ Les solutions de ce cours sont écrites pour **Rust 1.85 ou plus récent** et l'**édition 2024**, comme ZENTRIX.

---

# Partie I — Les fondations de Rust

**Objectif de la partie** : écrire des programmes corrects, comprendre et accepter les règles du compilateur, lire tout le code actuel de ZENTRIX.

---

## Module 1 — Premiers pas

📖 Livre : chapitres 1 à 3.

### Notions

#### Le programme minimal

```rust
fn main() {
    println!("Bonjour, ZENTRIX !");
}
```

- `fn main()` : le point d'entrée de tout exécutable.
- `println!` : une **macro** (le `!`) qui affiche une ligne.
- Les instructions se terminent par `;`.

#### Variables et mutabilité

```rust
let vies = 3;          // immuable : ne peut plus changer
let mut score = 0;     // `mut` : modifiable
score += 10;

let vitesse: f32 = 4.5; // type explicite (souvent facultatif : il est déduit)

const GRAVITE: f32 = 9.81;  // constante : type obligatoire, NOMS_EN_MAJUSCULES
```

**Pourquoi immuable par défaut ?** Parce que la plupart des valeurs ne changent pas, et le savoir rend le code plus simple à lire et à optimiser. Vous devez *demander* la mutabilité : c'est une information pour le lecteur.

#### Le masquage (*shadowing*)

```rust
let entree = "42";             // &str
let entree: i32 = entree.parse().unwrap(); // nouvelle variable, nouveau type
```

#### Les types scalaires

| Type | Taille | Plage / usage |
|------|--------|---------------|
| `i8`, `i16`, `i32`, `i64`, `i128` | 1 à 16 octets | entiers signés (`i32` par défaut) |
| `u8`, `u16`, `u32`, `u64`, `u128` | 1 à 16 octets | entiers non signés (≥ 0) |
| `isize`, `usize` | taille d'un pointeur (8 octets en 64 bits) | `usize` sert pour les **index** et les **tailles** |
| `f32`, `f64` | 4 / 8 octets | nombres à virgule. **Les jeux utilisent surtout `f32`** (le GPU aussi). |
| `bool` | 1 octet | `true` / `false` |
| `char` | 4 octets | un caractère Unicode : `'é'`, `'🎮'` |

Les conversions sont **toujours explicites** :

```rust
let n: i32 = 7;
let x: f32 = n as f32 / 2.0;  // 3.5 ; sans `as f32`, erreur de compilation
```

#### Les types composés

```rust
let position: (f32, f32) = (1.0, 2.5);   // tuple
let (x, y) = position;                    // déstructuration
let premier = position.0;

let couleurs: [u8; 3] = [255, 128, 0];    // tableau de taille fixe
let zeros = [0.0_f32; 16];                // 16 fois 0.0
println!("{}", couleurs[1]);              // 128
```

Un accès hors limites (`couleurs[10]`) **arrête le programme** (*panic*) au lieu de lire de la mémoire au hasard comme en C.

#### Les fonctions

```rust
fn aire_rectangle(largeur: f32, hauteur: f32) -> f32 {
    largeur * hauteur   // pas de `;` : c'est la valeur renvoyée
}
```

Une **expression** produit une valeur ; une **instruction** (terminée par `;`) n'en produit pas. Les blocs `{}` sont des expressions :

```rust
let carre = {
    let c = 4;
    c * c       // le bloc vaut 16
};
```

#### Le contrôle de flux

```rust
let pv = 30;
let etat = if pv > 50 { "en forme" } else if pv > 0 { "blessé" } else { "mort" };

for i in 0..5 { print!("{i} "); }        // 0 1 2 3 4   (5 exclu)
for i in 0..=5 { print!("{i} "); }       // 0 1 2 3 4 5 (5 inclus)

let mut n = 3;
while n > 0 { n -= 1; }

let mut tentatives = 0;
let resultat = loop {          // boucle infinie...
    tentatives += 1;
    if tentatives == 10 {
        break tentatives * 2;  // ...dont on sort avec une valeur
    }
};
```

> 🎮 **Lien moteur** : une boucle de jeu, c'est un `loop` qui tourne jusqu'à ce que le joueur quitte. Dans ZENTRIX, c'est winit qui la fait tourner pour nous.

### Exercices

**1.1 — Conversion de température.** Écrivez `fn celsius_vers_fahrenheit(c: f64) -> f64` (formule : `F = C × 9/5 + 32`) et affichez les conversions de -10 à 40 °C par pas de 10.

**1.2 — FizzBuzz.** Pour les nombres de 1 à 30 : affichez `Fizz` si divisible par 3, `Buzz` si par 5, `FizzBuzz` si par les deux, sinon le nombre. Utilisez `%` (reste de la division).

**1.3 — Fibonacci.** Écrivez `fn fibonacci(n: u32) -> u64` **sans récursion** (avec une boucle). `fibonacci(0) = 0`, `fibonacci(1) = 1`, `fibonacci(10) = 55`.

**1.4 — Distance.** Écrivez `fn distance(a: (f32, f32), b: (f32, f32)) -> f32`. Indice : `f32::sqrt`, ou la méthode `.sqrt()`.

<details>
<summary>▶ Solutions</summary>

```rust
fn celsius_vers_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn fizzbuzz() {
    for n in 1..=30 {
        if n % 15 == 0 {
            println!("FizzBuzz");
        } else if n % 3 == 0 {
            println!("Fizz");
        } else if n % 5 == 0 {
            println!("Buzz");
        } else {
            println!("{n}");
        }
    }
}

fn fibonacci(n: u32) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let suivant = a + b;
        a = b;
        b = suivant;
    }
    a
}

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    (dx * dx + dy * dy).sqrt()
}

fn main() {
    for c in (-10..=40).step_by(10) {
        println!("{c} °C = {} °F", celsius_vers_fahrenheit(c as f64));
    }
    fizzbuzz();
    assert_eq!(fibonacci(10), 55);
    println!("{}", distance((0.0, 0.0), (3.0, 4.0))); // 5
}
```

Remarquez `for _ in 0..n` : `_` signifie « je n'utilise pas la variable ».

</details>

---

## Module 2 — Propriété et emprunts

📖 Livre : chapitre 4. **Le module le plus important du cours.** Prenez le temps.

### Notions

#### La pile (*stack*) et le tas (*heap*)

- **La pile** : rapide, organisée, pour les valeurs de taille connue à la compilation (`i32`, `[f32; 3]`, une struct simple). Libérée automatiquement à la fin de la fonction.
- **Le tas** : pour les données de taille variable ou longue durée (`String`, `Vec`). Il faut demander de la mémoire (allocation) puis la rendre (libération).

En C, oublier de libérer = fuite ; libérer deux fois = crash ; utiliser après libération = faille de sécurité. **Rust résout ça à la compilation, sans ramasse-miettes**, grâce à la propriété.

#### Les trois règles de la propriété

1. Chaque valeur a **un propriétaire**.
2. Il n'y a **qu'un seul propriétaire** à la fois.
3. Quand le propriétaire sort de portée, la valeur est **libérée** (`drop`).

```rust
{
    let nom = String::from("Zentrix"); // alloue sur le tas
    println!("{nom}");
} // <- `nom` sort de portée : la mémoire est libérée ICI, automatiquement
```

#### Le déplacement (*move*)

```rust
let a = String::from("épée");
let b = a;                // la PROPRIÉTÉ passe à `b`
// println!("{a}");       // ❌ erreur E0382 : value borrowed here after move
```

Pourquoi ? Si `a` et `b` possédaient tous deux la chaîne, elle serait libérée deux fois. Rust interdit donc d'utiliser `a`.

Même chose avec les fonctions :

```rust
fn prendre(s: String) { println!("{s}"); } // `s` est libérée à la fin

let objet = String::from("bouclier");
prendre(objet);           // `objet` est déplacée dans la fonction
// prendre(objet);        // ❌ déjà déplacée
```

#### `Copy` et `Clone`

- Les types simples (`i32`, `f32`, `bool`, `char`, tuples/tableaux de ces types) sont **`Copy`** : ils sont copiés bit à bit, l'original reste utilisable.
- Pour dupliquer une valeur sur le tas, on appelle explicitement **`.clone()`** — c'est visible et donc coûteux *en connaissance de cause*.

```rust
let x = 5;
let y = x;                // copie : x reste utilisable

let s1 = String::from("arc");
let s2 = s1.clone();      // copie profonde explicite
println!("{s1} {s2}");    // ✅
```

> 🎮 **Lien moteur** : `zentrix_core::Color` est `Copy` (16 octets, aucune ressource). Un `wgpu::Buffer` ne l'est pas : il représente une ressource GPU unique.

#### Les emprunts (références)

Plutôt que donner la propriété, on peut **prêter** :

```rust
fn longueur(s: &String) -> usize {  // `&` : emprunt en lecture
    s.len()
}

let nom = String::from("Zentrix");
let n = longueur(&nom);   // on prête `nom`
println!("{nom} fait {n} caractères"); // ✅ toujours propriétaire
```

Emprunt **mutable** :

```rust
fn ajouter_suffixe(s: &mut String) {
    s.push_str("_v2");
}

let mut nom = String::from("niveau");
ajouter_suffixe(&mut nom);
```

#### La règle d'or des emprunts

> À tout instant, vous pouvez avoir **soit** autant de références `&T` que vous voulez, **soit** exactement **une** référence `&mut T`. Jamais les deux.

```rust
let mut v = vec![1, 2, 3];
let premier = &v[0];      // emprunt en lecture
v.push(4);                // ❌ emprunt mutable pendant qu'un emprunt en lecture existe
println!("{premier}");
```

Pourquoi c'est une **vraie** erreur : `push` peut réallouer le vecteur ailleurs en mémoire, et `premier` pointerait alors vers de la mémoire libérée. En C++, ce bug (« itérateur invalidé ») compile et plante au hasard. En Rust, il ne compile pas.

Une référence ne doit jamais survivre à la valeur : pas de **pointeur pendant** (*dangling pointer*).

```rust
fn creer() -> &String {           // ❌ ne compile pas
    let s = String::from("temp");
    &s                             // `s` est détruite à la fin de la fonction
}
// Solution : renvoyer la String elle-même (transfert de propriété).
```

#### Les slices (tranches)

Une **slice** est une vue empruntée sur une partie d'une collection :

```rust
let tableau = [10, 20, 30, 40, 50];
let milieu: &[i32] = &tableau[1..4];   // [20, 30, 40]

let texte = String::from("bonjour monde");
let mot: &str = &texte[0..7];          // "bonjour"
```

**Conseil** : préférez `&str` à `&String` et `&[T]` à `&Vec<T>` en paramètre : votre fonction acceptera plus de types d'entrée.

> 🎮 **Lien moteur** : `bytemuck::cast_slice(&TRIANGLE)` prend une slice `&[Vertex]` et renvoie une slice `&[u8]` — même mémoire, vue différemment, sans copie.

### Exercices

**2.1 — Corriger sans `clone`.** Ce code ne compile pas. Corrigez-le **sans** utiliser `.clone()` :

```rust
fn afficher(s: String) { println!("{s}"); }

fn main() {
    let titre = String::from("ZENTRIX");
    afficher(titre);
    afficher(titre);
}
```

**2.2 — Premier mot.** Écrivez `fn premier_mot(s: &str) -> &str` qui renvoie le texte jusqu'au premier espace (ou tout le texte s'il n'y en a pas). Indice : `s.find(' ')` renvoie une `Option<usize>`, ou parcourez `s.char_indices()`.

**2.3 — Prédire.** Pour chaque bloc, dites s'il compile, puis vérifiez :

```rust
// A
let mut a = String::from("x");
let r1 = &a;
let r2 = &a;
println!("{r1} {r2}");
let r3 = &mut a;
r3.push('y');

// B
let mut b = vec![1, 2];
let r = &mut b;
r.push(3);
println!("{:?}", b);
r.push(4);

// C
let c = vec![1.0, 2.0];
let d = c;
println!("{}", c.len());
```

**2.4 — Doubler sur place.** Écrivez `fn doubler(valeurs: &mut [f32])` qui multiplie chaque élément par 2. Indice : `for v in valeurs.iter_mut() { *v *= 2.0; }` — `*` *déréférence* : il accède à la valeur derrière la référence.

<details>
<summary>▶ Solutions</summary>

**2.1** — emprunter au lieu de déplacer :

```rust
fn afficher(s: &str) { println!("{s}"); }

fn main() {
    let titre = String::from("ZENTRIX");
    afficher(&titre);
    afficher(&titre);
}
```

`&String` se convertit automatiquement en `&str` (*deref coercion*).

**2.2**

```rust
fn premier_mot(s: &str) -> &str {
    match s.find(' ') {
        Some(index) => &s[..index],
        None => s,
    }
}
```

**2.3**
- **A compile** : `r1` et `r2` ne sont plus utilisés après le `println!`. Le compilateur sait que leurs emprunts sont terminés (*non-lexical lifetimes*) avant la création de `r3`.
- **B ne compile pas** : `println!("{:?}", b)` emprunte `b` en lecture alors que `r` (mutable) est encore utilisé à la ligne suivante.
- **C ne compile pas** : `c` a été déplacé dans `d`.

**2.4**

```rust
fn doubler(valeurs: &mut [f32]) {
    for v in valeurs.iter_mut() {
        *v *= 2.0;
    }
}

let mut donnees = vec![1.0, 2.5];
doubler(&mut donnees);
assert_eq!(donnees, vec![2.0, 5.0]);
```

</details>

---

## Module 3 — Structs, enums et `match`

📖 Livre : chapitres 5 et 6.

### Notions

#### Les structs

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    // Fonction associée (pas de self) : appelée avec Vec2::new(...)
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    // Méthode en lecture
    fn longueur(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    // Méthode en écriture
    fn mettre_a_l_echelle(&mut self, facteur: f32) {
        self.x *= facteur;
        self.y *= facteur;
    }

    // Méthode qui consomme (Vec2 est Copy, donc l'appelant garde sa copie)
    fn normaliser(self) -> Vec2 {
        let l = self.longueur();
        Vec2::new(self.x / l, self.y / l)
    }
}

let mut v = Vec2::new(3.0, 4.0);
println!("{:?} longueur {}", v, v.longueur()); // Vec2 { x: 3.0, y: 4.0 } longueur 5
v.mettre_a_l_echelle(2.0);
```

Variantes :

```rust
struct Metres(f32);       // « tuple struct » : un type distinct d'un simple f32
struct Joueur;            // « unit struct » : aucun champ (utile comme marqueur)
```

> 💡 Le patron *newtype* (`struct Metres(f32)`) empêche d'additionner par erreur des mètres et des secondes. Zéro coût à l'exécution.

#### Les enums

```rust
#[derive(Debug)]
enum Forme {
    Cercle { rayon: f32 },
    Rectangle { largeur: f32, hauteur: f32 },
    Triangle(Vec2, Vec2, Vec2),
}

impl Forme {
    fn aire(&self) -> f32 {
        match self {
            Forme::Cercle { rayon } => std::f32::consts::PI * rayon * rayon,
            Forme::Rectangle { largeur, hauteur } => largeur * hauteur,
            Forme::Triangle(a, b, c) => {
                ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs() / 2.0
            }
        }
    }
}
```

`match` est **exhaustif** : ajoutez une variante `Forme::Hexagone` et le compilateur vous montrera chaque `match` à mettre à jour. C'est un outil de refactorisation formidable.

#### Les motifs (*patterns*)

```rust
let n = 7;
match n {
    0 => println!("zéro"),
    1 | 2 => println!("un ou deux"),
    3..=9 => println!("entre 3 et 9"),
    x if x % 2 == 0 => println!("{x} est pair"),   // garde (`if`)
    _ => println!("autre"),
}

// Ne traiter qu'un cas :
if let Forme::Cercle { rayon } = forme {
    println!("cercle de rayon {rayon}");
}

// Extraire ou sortir :
let Forme::Cercle { rayon } = forme else {
    return;
};
```

#### `Option<T>` : l'absence de valeur

```rust
fn trouver_index(noms: &[&str], cible: &str) -> Option<usize> {
    for (i, nom) in noms.iter().enumerate() {
        if *nom == cible {
            return Some(i);
        }
    }
    None
}

match trouver_index(&["orc", "elfe"], "elfe") {
    Some(i) => println!("trouvé à {i}"),
    None => println!("absent"),
}

let i = trouver_index(&["orc"], "nain").unwrap_or(0);  // valeur de repli
```

### Exercices

**3.1 — Rectangle.** Créez `struct Rect { x: f32, y: f32, w: f32, h: f32 }` avec les méthodes `aire`, `contient(&self, px: f32, py: f32) -> bool` et `intersecte(&self, autre: &Rect) -> bool`.

> 🎮 C'est la base de la détection de collision 2D (AABB : *Axis-Aligned Bounding Box*).

**3.2 — États d'un ennemi.** Modélisez une IA simple :

```rust
enum EtatEnnemi {
    Patrouille { point: usize },
    Poursuite { cible_x: f32, cible_y: f32 },
    Attaque { cooldown: f32 },
    Mort,
}
```

Écrivez `fn suivant(etat: EtatEnnemi, joueur_visible: bool, distance: f32) -> EtatEnnemi` : un ennemi en patrouille passe en poursuite s'il voit le joueur ; en poursuite il attaque si `distance < 1.5` ; une attaque dont le cooldown est ≤ 0 revient en poursuite ; un mort reste mort.

**3.3 — Inventaire.** Écrivez `fn objet_le_plus_lourd(objets: &[(String, f32)]) -> Option<&String>` qui renvoie le nom de l'objet le plus lourd, ou `None` si la liste est vide.

<details>
<summary>▶ Solutions</summary>

**3.1**

```rust
struct Rect { x: f32, y: f32, w: f32, h: f32 }

impl Rect {
    fn aire(&self) -> f32 { self.w * self.h }

    fn contient(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    fn intersecte(&self, autre: &Rect) -> bool {
        self.x < autre.x + autre.w
            && self.x + self.w > autre.x
            && self.y < autre.y + autre.h
            && self.y + self.h > autre.y
    }
}
```

**3.2**

```rust
fn suivant(etat: EtatEnnemi, joueur_visible: bool, distance: f32) -> EtatEnnemi {
    match etat {
        EtatEnnemi::Patrouille { .. } if joueur_visible => {
            EtatEnnemi::Poursuite { cible_x: 0.0, cible_y: 0.0 }
        }
        EtatEnnemi::Poursuite { .. } if distance < 1.5 => EtatEnnemi::Attaque { cooldown: 1.0 },
        EtatEnnemi::Attaque { cooldown } if cooldown <= 0.0 => {
            EtatEnnemi::Poursuite { cible_x: 0.0, cible_y: 0.0 }
        }
        autre => autre, // tous les autres cas : l'état ne change pas (Mort compris)
    }
}
```

C'est une **machine à états finis**, le patron le plus courant de l'IA de jeu.

**3.3**

```rust
fn objet_le_plus_lourd(objets: &[(String, f32)]) -> Option<&String> {
    let mut meilleur: Option<&(String, f32)> = None;
    for objet in objets {
        match meilleur {
            Some(m) if m.1 >= objet.1 => {}
            _ => meilleur = Some(objet),
        }
    }
    meilleur.map(|(nom, _)| nom)
}
```

Version avec itérateurs (module 9) : `objets.iter().max_by(|a, b| a.1.total_cmp(&b.1)).map(|(nom, _)| nom)`.

</details>

---

## Module 4 — Collections et chaînes de caractères

📖 Livre : chapitre 8.

### Notions

#### `Vec<T>` : le tableau dynamique

```rust
let mut ennemis: Vec<String> = Vec::new();
ennemis.push("gobelin".to_string());
ennemis.push("troll".to_string());

let mut positions = vec![0.0_f32; 100];         // 100 zéros
let mut buffer = Vec::with_capacity(1024);       // réserve de la place à l'avance
buffer.push(1u8);

println!("{}", ennemis[0]);                      // panique si l'index est invalide
if let Some(e) = ennemis.get(5) { println!("{e}"); } // sûr : renvoie une Option

for e in &ennemis { println!("{e}"); }           // emprunt en lecture
for p in &mut positions { *p += 1.0; }           // emprunt en écriture
let dernier = ennemis.pop();                     // Option<String>
ennemis.retain(|e| e != "troll");                // garder ceux qui respectent la condition
ennemis.swap_remove(0);                          // retire en O(1) (ne garde pas l'ordre)
```

Un `Vec` est trois nombres sur la pile (pointeur, longueur, capacité) qui désignent un bloc contigu sur le tas. Quand la capacité est dépassée, il **réalloue** (en général en doublant). D'où l'intérêt de `with_capacity` dans les chemins critiques.

> 🎮 **Lien moteur** : `Vec` est LA structure de données des moteurs : contiguë en mémoire = parcours ultra-rapide grâce au cache du processeur (module 13). `swap_remove` est très utilisé pour supprimer des entités sans tout décaler.

#### `String` et `&str`

| | `String` | `&str` |
|---|---|---|
| Possède ses données | oui (sur le tas) | non (emprunt) |
| Modifiable | oui (si `mut`) | non |
| Exemple | `String::from("a")`, `format!(...)` | `"littéral"`, `&chaine[0..3]` |

```rust
let mut s = String::from("Niveau");
s.push(' ');
s.push_str("1");
let complet = format!("{s} - {}", "Forêt");

for c in "été".chars() { print!("{c},"); }   // é,t,é,
println!("{}", "été".len());                  // 5 : len() compte les OCTETS (UTF-8), pas les caractères !
```

⚠️ `s[0]` est interdit sur une chaîne : un caractère UTF-8 peut faire de 1 à 4 octets. Utilisez `.chars()`.

#### `HashMap<K, V>`

```rust
use std::collections::HashMap;

let mut scores: HashMap<String, u32> = HashMap::new();
scores.insert("Alice".to_string(), 10);
*scores.entry("Bob".to_string()).or_insert(0) += 5;   // créer ou modifier

if let Some(s) = scores.get("Alice") { println!("{s}"); }
for (nom, score) in &scores { println!("{nom}: {score}"); }
```

Autres collections utiles : `HashSet` (ensemble), `VecDeque` (file à double entrée), `BTreeMap` (map triée), `BinaryHeap` (file de priorité, pour l'algorithme A*).

### Exercices

**4.1 — Compter les mots.** `fn compter_mots(texte: &str) -> HashMap<String, usize>`. Indice : `texte.split_whitespace()`, et `to_lowercase()` pour ignorer la casse.

**4.2 — Statistiques.** Pour un `&[f32]`, renvoyez `(moyenne, médiane, max)` dans un `Option<(f32, f32, f32)>` (`None` si vide). Indice : copiez dans un `Vec`, puis `v.sort_by(|a, b| a.total_cmp(b))`.

**4.3 — File d'événements.** Avec un `VecDeque<Evenement>` (`enum Evenement { Saut, Tir, Pause }`), simulez une file : ajoutez 5 événements avec `push_back`, puis traitez-les avec `pop_front` dans une boucle `while let`.

<details>
<summary>▶ Solutions</summary>

```rust
use std::collections::{HashMap, VecDeque};

fn compter_mots(texte: &str) -> HashMap<String, usize> {
    let mut compte = HashMap::new();
    for mot in texte.split_whitespace() {
        *compte.entry(mot.to_lowercase()).or_insert(0) += 1;
    }
    compte
}

fn stats(valeurs: &[f32]) -> Option<(f32, f32, f32)> {
    if valeurs.is_empty() {
        return None;
    }
    let mut v = valeurs.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let moyenne = v.iter().sum::<f32>() / v.len() as f32;
    let mediane = if v.len() % 2 == 0 {
        (v[v.len() / 2 - 1] + v[v.len() / 2]) / 2.0
    } else {
        v[v.len() / 2]
    };
    Some((moyenne, mediane, v[v.len() - 1]))
}

#[derive(Debug)]
enum Evenement { Saut, Tir, Pause }

fn file() {
    let mut file = VecDeque::new();
    file.push_back(Evenement::Saut);
    file.push_back(Evenement::Tir);
    file.push_back(Evenement::Tir);
    file.push_back(Evenement::Pause);
    file.push_back(Evenement::Saut);
    while let Some(e) = file.pop_front() {
        println!("traitement de {e:?}");
    }
}
```

Pourquoi `total_cmp` ? Les `f32` ne sont pas totalement ordonnés (`NaN` n'est ni plus grand ni plus petit que rien), donc `sort()` est interdit sur eux. `total_cmp` définit un ordre total.

</details>

---

## Module 5 — La gestion des erreurs

📖 Livre : chapitre 9.

### Notions

#### Deux familles d'erreurs

| | Irrécupérable | Récupérable |
|---|---|---|
| Outil | `panic!`, `unwrap()`, `expect()` | `Result<T, E>` |
| Usage | un **bug** : une situation qui ne devrait jamais arriver | une situation **prévisible** : fichier absent, GPU incompatible, saisie invalide |
| Effet | arrête le thread | l'appelant décide |

#### `Result` et `?`

```rust
use std::fs;
use std::num::ParseIntError;

fn lire_niveau(texte: &str) -> Result<u32, ParseIntError> {
    let n: u32 = texte.trim().parse()?;   // si erreur : renvoyée immédiatement
    Ok(n * 10)
}

match lire_niveau("4") {
    Ok(v) => println!("valeur {v}"),
    Err(e) => println!("erreur : {e}"),
}
```

#### Créer ses propres erreurs

```rust
#[derive(Debug)]
enum ErreurConfig {
    Lecture(std::io::Error),
    LigneInvalide { numero: usize },
    ValeurInvalide(String),
}

impl std::fmt::Display for ErreurConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErreurConfig::Lecture(e) => write!(f, "lecture impossible : {e}"),
            ErreurConfig::LigneInvalide { numero } => write!(f, "ligne {numero} invalide"),
            ErreurConfig::ValeurInvalide(v) => write!(f, "valeur invalide : {v}"),
        }
    }
}

impl std::error::Error for ErreurConfig {}

// Permet à `?` de convertir automatiquement une io::Error en ErreurConfig
impl From<std::io::Error> for ErreurConfig {
    fn from(e: std::io::Error) -> Self {
        ErreurConfig::Lecture(e)
    }
}
```

C'est beaucoup de code répétitif : la crate **`thiserror`** le génère (voir `zentrix-rhi/src/error.rs`) :

```rust
#[derive(Debug, thiserror::Error)]
enum ErreurConfig {
    #[error("lecture impossible : {0}")]
    Lecture(#[from] std::io::Error),
    #[error("ligne {numero} invalide")]
    LigneInvalide { numero: usize },
    #[error("valeur invalide : {0}")]
    ValeurInvalide(String),
}
```

Et dans une **application**, `anyhow` accepte n'importe quelle erreur et permet d'ajouter du contexte :

```rust
use anyhow::Context;

fn charger() -> anyhow::Result<String> {
    let texte = std::fs::read_to_string("config.txt")
        .context("impossible de lire config.txt")?;
    Ok(texte)
}
```

> 📏 **Règle** : bibliothèque → `thiserror` (erreurs précises) ; application → `anyhow` (erreurs pratiques). C'est exactement ce que fait ZENTRIX.

### Exercices

**5.1 — Parser une configuration.** Écrivez `fn parser_config(texte: &str) -> Result<HashMap<String, String>, ErreurConfig>` pour un format `cle = valeur` (une par ligne, lignes vides et lignes commençant par `#` ignorées). Une ligne sans `=` donne `LigneInvalide { numero }` (numéro à partir de 1).

**5.2 — Configuration typée.** À partir de la map, construisez une `struct ConfigFenetre { largeur: u32, hauteur: u32, vsync: bool }`. Une valeur manquante ou non convertible donne `ValeurInvalide`.

**5.3 — Réflexion.** Dans ZENTRIX, trouvez un endroit où une erreur est propagée avec `?` et un endroit où elle est enrichie avec `.context(...)`. Pourquoi `GpuContext::new` ne fait-il jamais `unwrap()` ?

<details>
<summary>▶ Solutions</summary>

```rust
use std::collections::HashMap;

fn parser_config(texte: &str) -> Result<HashMap<String, String>, ErreurConfig> {
    let mut map = HashMap::new();
    for (i, ligne) in texte.lines().enumerate() {
        let ligne = ligne.trim();
        if ligne.is_empty() || ligne.starts_with('#') {
            continue;
        }
        let Some((cle, valeur)) = ligne.split_once('=') else {
            return Err(ErreurConfig::LigneInvalide { numero: i + 1 });
        };
        map.insert(cle.trim().to_string(), valeur.trim().to_string());
    }
    Ok(map)
}

struct ConfigFenetre { largeur: u32, hauteur: u32, vsync: bool }

fn lire<T: std::str::FromStr>(map: &HashMap<String, String>, cle: &str) -> Result<T, ErreurConfig> {
    map.get(cle)
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| ErreurConfig::ValeurInvalide(cle.to_string()))
}

fn config_fenetre(texte: &str) -> Result<ConfigFenetre, ErreurConfig> {
    let map = parser_config(texte)?;
    Ok(ConfigFenetre {
        largeur: lire(&map, "largeur")?,
        hauteur: lire(&map, "hauteur")?,
        vsync: lire(&map, "vsync")?,
    })
}
```

`lire` est **générique** (module 7) : elle fonctionne pour tout type qui sait se construire depuis du texte (`FromStr`), ici `u32` et `bool`.

**5.3** — `GpuContext::new` est dans une bibliothèque : l'absence de GPU compatible est une situation **prévisible** chez un utilisateur. La bibliothèque renvoie donc une erreur, et c'est l'application qui décide quoi faire (afficher un message, essayer un autre backend...).

</details>

---

## Module 6 — Modules, crates, tests et documentation

📖 Livre : chapitres 7 et 11. Relisez aussi le [chapitre 3 du guide ZENTRIX](guide-debutant/03-cargo-et-workspace.md).

### Notions

#### Les modules

```text
src/
├── main.rs          // mod math; mod physique;
├── math.rs          // le module `math`
└── physique/
    ├── mod.rs       // le module `physique` : pub mod collision;
    └── collision.rs // le sous-module `physique::collision`
```

```rust
// main.rs
mod math;
mod physique;

use math::Vec2;
use physique::collision::aabb;

fn main() { /* ... */ }
```

- Tout est **privé par défaut** ; `pub` rend visible, `pub(crate)` visible dans la crate seulement.
- `use` crée un raccourci ; `crate::`, `super::`, `self::` sont des chemins relatifs.
- **Encapsulation** : rendez les champs privés et exposez des méthodes. Vous pourrez changer l'implémentation sans casser le code qui l'utilise.

#### Bibliothèque + binaire

Un même package peut avoir `src/lib.rs` (la logique, testable) et `src/main.rs` (un petit programme qui l'utilise). C'est une excellente habitude.

#### Les tests

```rust
pub fn clamp01(x: f32) -> f32 { x.clamp(0.0, 1.0) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borne_en_bas() { assert_eq!(clamp01(-3.0), 0.0); }

    #[test]
    fn reste_identique() { assert_eq!(clamp01(0.4), 0.4); }

    #[test]
    #[should_panic]
    fn index_invalide() { let v: Vec<i32> = vec![]; let _ = v[0]; }
}
```

- `cargo test` : tests unitaires (dans `src/`), tests d'intégration (dans `tests/`, qui utilisent la crate comme un utilisateur externe) et doc-tests.
- ⚠️ Comparer des flottants avec `==` est risqué (`0.1 + 0.2 != 0.3`). Utilisez une tolérance : `assert!((a - b).abs() < 1e-5)`.

#### La documentation

```rust
/// Borne une valeur entre 0 et 1.
///
/// # Exemple
/// ```
/// assert_eq!(ma_crate::clamp01(2.0), 1.0);
/// ```
pub fn clamp01(x: f32) -> f32 { x.clamp(0.0, 1.0) }
```

`cargo doc --open` génère le site. Le bloc d'exemple est **exécuté** par `cargo test`.

#### Les outils à utiliser tout le temps

| Commande | Rôle |
|----------|------|
| `cargo fmt` | formate le code |
| `cargo clippy` | plus de 700 vérifications de qualité, très pédagogiques |
| `cargo check` | vérifie la compilation, très rapidement |

### Exercices

**6.1 — Réorganiser.** Prenez vos solutions des modules 3 à 5 et organisez-les dans un package `bac-a-sable` avec `src/lib.rs` et des modules `geometrie`, `inventaire`, `config`. Le `main.rs` ne fait qu'appeler la bibliothèque.

**6.2 — Tester.** Écrivez au moins 3 tests par module, dont un test d'intégration dans `tests/config.rs`.

**6.3 — Documenter.** Ajoutez un commentaire `///` avec un exemple exécutable sur chaque fonction publique. Vérifiez avec `cargo test` puis `cargo doc --open`.

**6.4 — Clippy.** Lancez `cargo clippy -- -W clippy::pedantic` et corrigez au moins 5 avertissements. Lisez l'explication de chacun (lien fourni dans le message).

*(Pas de solution unique : comparez votre organisation à celle de `crates/zentrix-core` dans ZENTRIX.)*

---

## Projet I — Le jeu de la vie

**Objectif** : réunir toute la partie I dans un programme complet. C'est votre première **simulation** : un état, une règle de mise à jour, un affichage, une boucle. Exactement la structure d'un jeu.

### Règles du jeu de la vie (Conway)

Une grille de cellules vivantes ou mortes. À chaque génération :
- une cellule vivante avec 2 ou 3 voisines vivantes survit, sinon elle meurt ;
- une cellule morte avec exactement 3 voisines vivantes naît.

### Cahier des charges

1. `struct Grille { largeur: usize, hauteur: usize, cellules: Vec<bool> }` — **un seul `Vec` à plat**, pas de `Vec<Vec<bool>>` (index = `y * largeur + x`). C'est plus rapide (module 13).
2. `fn voisins_vivants(&self, x: usize, y: usize) -> u8` avec bords qui « bouclent » (le voisin de droite du bord droit est au bord gauche).
3. `fn etape(&self) -> Grille` qui produit la génération suivante (ne modifiez **pas** la grille pendant que vous la lisez !).
4. Affichage dans le terminal avec `█` et `·`, en effaçant l'écran (`print!("\x1B[2J\x1B[H")`), une génération toutes les 100 ms (`std::thread::sleep`).
5. Chargement d'un motif depuis un fichier texte, avec gestion d'erreurs (`Result`).
6. Tests : un « clignotant » (3 cellules alignées) doit osciller avec une période de 2.

### Bonus

- Mesurez le temps de 1000 générations sur une grille 1000×1000 avec `std::time::Instant`. Gardez ce chiffre : vous l'optimiserez au module 15 avec le parallélisme.
- Évitez d'allouer une nouvelle grille à chaque étape : utilisez deux buffers et échangez-les (`std::mem::swap`). C'est le principe du **double buffering**, utilisé aussi par la swapchain du GPU.

<details>
<summary>▶ Squelette de solution</summary>

```rust
#[derive(Clone)]
struct Grille {
    largeur: usize,
    hauteur: usize,
    cellules: Vec<bool>,
}

impl Grille {
    fn new(largeur: usize, hauteur: usize) -> Self {
        Self { largeur, hauteur, cellules: vec![false; largeur * hauteur] }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.largeur + x
    }

    fn get(&self, x: usize, y: usize) -> bool {
        self.cellules[self.index(x, y)]
    }

    fn set(&mut self, x: usize, y: usize, vivante: bool) {
        let i = self.index(x, y);
        self.cellules[i] = vivante;
    }

    fn voisins_vivants(&self, x: usize, y: usize) -> u8 {
        let mut n = 0;
        for dy in [self.hauteur - 1, 0, 1] {
            for dx in [self.largeur - 1, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = (x + dx) % self.largeur; // + (largeur - 1) équivaut à -1 modulo largeur
                let ny = (y + dy) % self.hauteur;
                n += self.get(nx, ny) as u8;
            }
        }
        n
    }

    /// Écrit la génération suivante dans `sortie` (aucune allocation).
    fn etape_dans(&self, sortie: &mut Grille) {
        for y in 0..self.hauteur {
            for x in 0..self.largeur {
                let v = self.voisins_vivants(x, y);
                let vivante = matches!((self.get(x, y), v), (true, 2) | (true, 3) | (false, 3));
                sortie.set(x, y, vivante);
            }
        }
    }
}

fn main() {
    let mut actuelle = Grille::new(40, 20);
    let mut suivante = actuelle.clone();
    // planeur
    for (x, y) in [(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)] {
        actuelle.set(x, y, true);
    }
    loop {
        print!("\x1B[2J\x1B[H");
        for y in 0..actuelle.hauteur {
            let ligne: String = (0..actuelle.largeur)
                .map(|x| if actuelle.get(x, y) { '█' } else { '·' })
                .collect();
            println!("{ligne}");
        }
        actuelle.etape_dans(&mut suivante);
        std::mem::swap(&mut actuelle, &mut suivante); // double buffering
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
```

`matches!` est une macro qui renvoie `true` si la valeur correspond au motif.

</details>

### ✅ Checklist de fin de partie I

- [ ] Je sais expliquer la différence entre move, copie et emprunt.
- [ ] Je sais pourquoi on ne peut pas avoir `&mut` et `&` en même temps.
- [ ] J'utilise `Option` et `Result` sans `unwrap()` dans le code « sérieux ».
- [ ] J'ai organisé un projet en modules avec des tests.
- [ ] Je peux lire et expliquer chaque fichier de `crates/zentrix-core` et `crates/zentrix-app`.
- [ ] Bonus : j'ai fini les exercices Rustlings jusqu'à `error_handling`.

---

# Partie II — Rust intermédiaire

**Objectif de la partie** : concevoir des API **réutilisables** et **expressives**. C'est la clé de la *polyvalence* d'un moteur : le même code doit servir à des jeux très différents.

---

## Module 7 — Génériques et traits

📖 Livre : chapitre 10 (sections 1 et 2), chapitre 18.

### Notions

#### Les génériques

Une fonction générique fonctionne pour **plusieurs types** :

```rust
fn plus_grand<T: PartialOrd + Copy>(liste: &[T]) -> Option<T> {
    let mut max = *liste.first()?;
    for &x in liste {
        if x > max {
            max = x;
        }
    }
    Some(max)
}

plus_grand(&[3, 7, 2]);          // Some(7)
plus_grand(&[1.5, 0.2]);         // Some(1.5)
```

`T: PartialOrd + Copy` est une **contrainte** : `T` doit être comparable (`>`) et copiable. Le compilateur génère une version spécialisée par type utilisé (**monomorphisation**) : **aucun coût** à l'exécution, comme un template C++.

Structs génériques :

```rust
struct Paire<T> {
    a: T,
    b: T,
}

impl<T: std::ops::Add<Output = T> + Copy> Paire<T> {
    fn somme(&self) -> T { self.a + self.b }
}
```

#### Les traits

Un trait définit un **comportement partagé** :

```rust
trait Dessinable {
    fn nom(&self) -> String;

    // Méthode avec implémentation par défaut
    fn decrire(&self) -> String {
        format!("Je suis un {}", self.nom())
    }
}

struct Sprite { image: String }
struct Texte { contenu: String }

impl Dessinable for Sprite {
    fn nom(&self) -> String { format!("sprite ({})", self.image) }
}

impl Dessinable for Texte {
    fn nom(&self) -> String { "texte".to_string() }
    fn decrire(&self) -> String { format!("Texte : {}", self.contenu) } // redéfinition
}
```

> 🎮 **Lien moteur** : le trait `Application` de `zentrix-platform` est exactement ça. La plateforme ne connaît pas votre jeu, seulement le contrat.

#### Utiliser un trait : trois syntaxes

```rust
// 1. Contrainte générique (dispatch STATIQUE : une copie de la fonction par type)
fn afficher<T: Dessinable>(objet: &T) { println!("{}", objet.decrire()); }

// 2. `impl Trait` : même chose, syntaxe raccourcie
fn afficher2(objet: &impl Dessinable) { println!("{}", objet.decrire()); }

// 3. Objet trait `dyn` (dispatch DYNAMIQUE : une seule fonction, appel via une table)
fn afficher_tous(objets: &[Box<dyn Dessinable>]) {
    for o in objets { println!("{}", o.decrire()); }
}

let scene: Vec<Box<dyn Dessinable>> = vec![
    Box::new(Sprite { image: "hero.png".into() }),
    Box::new(Texte { contenu: "Score".into() }),
];
afficher_tous(&scene);
```

#### Statique vs dynamique : le choix crucial pour la performance

| | Générique `<T: Trait>` | `dyn Trait` |
|---|---|---|
| Résolution | à la compilation | à l'exécution (via une *vtable*) |
| Coût d'appel | nul (peut être *inliné*) | un appel indirect (~1–3 ns, empêche l'inlining) |
| Taille du binaire | plus gros (une copie par type) | plus petit |
| Collections hétérogènes | ❌ impossible (`Vec<T>` = un seul type) | ✅ `Vec<Box<dyn Trait>>` |
| Usage dans un moteur | chemins **chauds** (maths, boucles par entité) | systèmes **extensibles** (plugins, passes de rendu, éditeur) |

**Règle pratique** : générique par défaut ; `dyn` quand vous avez vraiment besoin de mélanger des types, et **jamais** pour 100 000 appels par image dans une boucle serrée.

#### Les traits standard à connaître

| Trait | Rôle | Dérivable ? |
|-------|------|-------------|
| `Debug` | affichage `{:?}` | oui |
| `Clone` / `Copy` | duplication | oui |
| `PartialEq` / `Eq` | `==` | oui |
| `PartialOrd` / `Ord` | `<`, tri | oui |
| `Hash` | clé de `HashMap` | oui |
| `Default` | valeur par défaut | oui |
| `Display` | affichage `{}` | non |
| `From` / `Into` | conversions | non |
| `Iterator` | parcours | non |
| `Drop` | code exécuté à la destruction | non |
| `Send` / `Sync` | sûreté entre threads (module 15) | automatiques |

#### Types associés

```rust
trait Ressource {
    type Handle;             // chaque implémentation choisit son type
    fn charger(&mut self, chemin: &str) -> Self::Handle;
}
```

C'est le cas de `Iterator` : `type Item;`.

### Exercices

**7.1 — Composants.** Définissez `trait Composant { fn nom(&self) -> &'static str; fn mettre_a_jour(&mut self, dt: f32); }`. Implémentez-le pour `Position { x, y, vx, vy }` (avance selon la vitesse) et `Minuteur { restant: f32 }` (décrémente). Stockez-les dans un `Vec<Box<dyn Composant>>` et mettez-les tous à jour.

**7.2 — Générique.** Écrivez `fn somme_ponderee<T>(valeurs: &[T], poids: &[f32]) -> T` où `T` supporte `Mul<f32, Output = T>`, `Add<Output = T>`, `Default` et `Copy`. Testez avec `f32` (et avec votre `Vec3` du module 11).

**7.3 — Mesurer la différence.** Écrivez une même fonction qui somme l'aire de 10 millions de formes, une fois avec `Vec<Box<dyn Forme>>`, une fois avec un `enum Forme` et `match`, une fois avec trois `Vec` séparés (un par type). Mesurez avec `Instant` en `--release`. Qu'en concluez-vous ?

<details>
<summary>▶ Solutions</summary>

**7.1**

```rust
trait Composant {
    fn nom(&self) -> &'static str;
    fn mettre_a_jour(&mut self, dt: f32);
}

struct Position { x: f32, y: f32, vx: f32, vy: f32 }
struct Minuteur { restant: f32 }

impl Composant for Position {
    fn nom(&self) -> &'static str { "Position" }
    fn mettre_a_jour(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }
}

impl Composant for Minuteur {
    fn nom(&self) -> &'static str { "Minuteur" }
    fn mettre_a_jour(&mut self, dt: f32) {
        self.restant = (self.restant - dt).max(0.0);
    }
}

fn main() {
    let mut composants: Vec<Box<dyn Composant>> = vec![
        Box::new(Position { x: 0.0, y: 0.0, vx: 1.0, vy: 2.0 }),
        Box::new(Minuteur { restant: 3.0 }),
    ];
    for c in composants.iter_mut() {
        c.mettre_a_jour(0.016);
        println!("{} mis à jour", c.nom());
    }
}
```

⚠️ Ce design (« chaque objet est un `Box<dyn>` ») est simple mais **lent à grande échelle**. Le module 20 (ECS) montre la bonne approche.

**7.2**

```rust
use std::ops::{Add, Mul};

fn somme_ponderee<T>(valeurs: &[T], poids: &[f32]) -> T
where
    T: Mul<f32, Output = T> + Add<Output = T> + Default + Copy,
{
    let mut total = T::default();
    for (&v, &p) in valeurs.iter().zip(poids) {
        total = total + v * p;
    }
    total
}
```

La clause `where` rend les contraintes longues plus lisibles.

**7.3** — Résultats typiques : l'`enum` est 2 à 4 fois plus rapide que `Box<dyn>`, et les `Vec` séparés encore plus rapides. Raisons : pas d'appel indirect, et surtout des données **contiguës** en mémoire au lieu d'objets éparpillés sur le tas (module 13).

</details>

---

## Module 8 — Les durées de vie

📖 Livre : chapitre 10.3.

### Notions

Une **durée de vie** est la portion de code pendant laquelle une référence est valide. Le compilateur les vérifie toutes ; la plupart du temps il les **déduit** (*élision*). Il faut les écrire quand il ne peut pas deviner.

```rust
// ❌ Le compilateur ne sait pas si le résultat emprunte `a` ou `b`
// fn plus_long(a: &str, b: &str) -> &str

// ✅ « Le résultat vit au plus aussi longtemps que a ET b »
fn plus_long<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}
```

`'a` ne **change** aucune durée de vie : c'est une **contrainte** que le compilateur vérifie chez l'appelant.

```rust
let s1 = String::from("long texte");
let resultat;
{
    let s2 = String::from("court");
    resultat = plus_long(&s1, &s2);
} // s2 détruite ici
// println!("{resultat}"); // ❌ resultat pourrait référencer s2
```

#### Les règles d'élision (quand on n'écrit rien)

1. Chaque paramètre référence reçoit sa propre durée de vie.
2. S'il n'y a **qu'un** paramètre référence, le résultat prend sa durée de vie.
3. S'il y a `&self` ou `&mut self`, le résultat prend la durée de vie de `self`.

#### Les structs qui contiennent des références

```rust
struct Analyseur<'a> {
    source: &'a str,   // l'analyseur ne peut pas survivre au texte
    position: usize,
}

impl<'a> Analyseur<'a> {
    fn prochain_mot(&mut self) -> Option<&'a str> {
        let source: &'a str = self.source;   // copie de la référence (pas du texte)
        let reste = source[self.position..].trim_start();
        if reste.is_empty() {
            return None;
        }
        let debut = source.len() - reste.len();
        let fin = reste.find(' ').map_or(source.len(), |i| debut + i);
        self.position = fin;
        Some(&source[debut..fin])   // emprunte le TEXTE ('a), pas l'analyseur
    }
}
```

> 🎮 **Lien moteur** : les descripteurs wgpu sont pleins de durées de vie (`RenderPassDescriptor<'a>`, `VertexBufferLayout<'static>`) : ils **empruntent** des données le temps d'un appel.

#### `'static`

- `&'static str` : une référence valable pendant tout le programme (les littéraux `"..."`).
- `T: 'static` (comme contrainte) : `T` ne contient **aucune référence temporaire**. Un `String` est `'static` en ce sens. C'est exigé par exemple pour envoyer une valeur dans un autre thread.

#### Conseil de conception

> Dans un moteur, **évitez de stocker des références dans les structs à longue durée de vie**. Préférez posséder les données, ou stocker des **index / handles** (module 21). Les références sont parfaites pour les paramètres de fonction et les objets temporaires.

### Exercices

**8.1** — Écrivez `fn plus_long_mot<'a>(texte: &'a str) -> &'a str`. Le `'a` est-il nécessaire ? (Testez sans.)

**8.2** — Ajoutez un champ `mots_lus: usize` à `Analyseur` et implémentez `Iterator` pour lui (`type Item = &'a str;`).

**8.3** — Expliquez pourquoi ZENTRIX a remplacé `Box::leak` par `Arc<Window>` pour obtenir une `Surface<'static>` (relisez `crates/zentrix-rhi/src/context.rs`).

<details>
<summary>▶ Solutions</summary>

**8.1** — Pas nécessaire : un seul paramètre référence, la règle 2 s'applique.

```rust
fn plus_long_mot(texte: &str) -> &str {
    texte.split_whitespace().max_by_key(|m| m.len()).unwrap_or("")
}
```

**8.2**

```rust
impl<'a> Iterator for Analyseur<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let mot = self.prochain_mot()?;
        self.mots_lus += 1;
        Some(mot)
    }
}
```

**8.3** — `Box::leak` rend la mémoire de la fenêtre immortelle (elle n'est jamais libérée). Avec `Arc`, la surface **possède une part** de la fenêtre : la fenêtre vit donc au moins aussi longtemps que la surface, ce qui satisfait `'static`, et elle est libérée proprement quand plus personne ne l'utilise.

</details>

---

## Module 9 — Closures et itérateurs

📖 Livre : chapitre 13.

### Notions

#### Les closures

```rust
let gravite = 9.81;
let chute = |t: f32| 0.5 * gravite * t * t;   // capture `gravite` de l'environnement
println!("{}", chute(2.0));
```

Selon ce qu'elle fait de ses captures, une closure implémente :

| Trait | Capture | Peut être appelée |
|-------|---------|-------------------|
| `Fn` | par référence `&` | plusieurs fois, en parallèle |
| `FnMut` | par référence `&mut` | plusieurs fois, une à la fois |
| `FnOnce` | par valeur (consomme) | une seule fois |

```rust
fn appliquer<F: Fn(f32) -> f32>(f: F, x: f32) -> f32 { f(x) }

let mut compteur = 0;
let mut incrementer = || compteur += 1;   // FnMut
incrementer();

let nom = String::from("Zentrix");
let dire = move || println!("{nom}");    // `move` : prend la propriété de `nom`
dire();
```

`move` est indispensable pour envoyer une closure dans un autre thread.

#### Les itérateurs

Un itérateur produit une suite de valeurs à la demande. Il est **paresseux** : rien ne se passe tant qu'on ne le consomme pas.

```rust
let pv = vec![100, 0, 35, 0, 80];

let vivants: Vec<i32> = pv.iter().copied().filter(|&p| p > 0).collect();
let total: i32 = pv.iter().sum();
let nb_morts = pv.iter().filter(|&&p| p == 0).count();
let doubles: Vec<i32> = pv.iter().map(|p| p * 2).collect();
let premier_blesse = pv.iter().position(|&p| p > 0 && p < 50);   // Some(2)
let tous_vivants = pv.iter().all(|&p| p > 0);                    // false

for (i, p) in pv.iter().enumerate() { println!("entité {i}: {p} PV"); }

let noms = ["a", "b"];
let positions = [1.0, 2.0];
for (nom, pos) in noms.iter().zip(positions.iter()) { println!("{nom} en {pos}"); }
```

| Méthode | Produit |
|---------|---------|
| `.iter()` | des `&T` (emprunt) |
| `.iter_mut()` | des `&mut T` |
| `.into_iter()` | des `T` (consomme la collection) |

**Performance** : les itérateurs sont des **abstractions à coût nul**. Une chaîne `filter().map().sum()` compile en une boucle aussi rapide qu'à la main, souvent plus (pas de vérification de bornes, vectorisation automatique).

#### Écrire son propre itérateur

```rust
struct Grille2D { largeur: u32, hauteur: u32, courant: u32 }

impl Iterator for Grille2D {
    type Item = (u32, u32);
    fn next(&mut self) -> Option<(u32, u32)> {
        if self.courant >= self.largeur * self.hauteur {
            return None;
        }
        let i = self.courant;
        self.courant += 1;
        Some((i % self.largeur, i / self.largeur))
    }
}

// Toutes les méthodes (map, filter, sum...) sont disponibles gratuitement :
let cases_paires = Grille2D { largeur: 4, hauteur: 4, courant: 0 }
    .filter(|(x, y)| (x + y) % 2 == 0)
    .count();
```

### Exercices

**9.1** — Avec un `Vec<(String, u32, f32)>` (nom, niveau, PV) : (a) les noms des personnages de niveau ≥ 10, triés ; (b) la moyenne des PV ; (c) le personnage avec le plus de PV (`max_by`) ; (d) regrouper les noms par niveau dans une `HashMap<u32, Vec<String>>` (`fold` ou boucle).

**9.2** — Écrivez `fn composer<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C`.

**9.3** — Réécrivez le comptage de voisins du jeu de la vie avec des itérateurs (`flat_map`, `filter`, `count`).

**9.4** — Écrivez un système d'événements : `struct Bus { abonnes: Vec<Box<dyn Fn(&str)>> }` avec `abonner` et `publier`.

<details>
<summary>▶ Solutions</summary>

**9.1**

```rust
use std::collections::HashMap;

fn analyse(persos: &[(String, u32, f32)]) {
    let mut experts: Vec<&String> = persos.iter().filter(|p| p.1 >= 10).map(|p| &p.0).collect();
    experts.sort();

    let moyenne = persos.iter().map(|p| p.2).sum::<f32>() / persos.len() as f32;

    let plus_robuste = persos.iter().max_by(|a, b| a.2.total_cmp(&b.2));

    let mut par_niveau: HashMap<u32, Vec<String>> = HashMap::new();
    for (nom, niveau, _) in persos {
        par_niveau.entry(*niveau).or_default().push(nom.clone());
    }
    println!("{experts:?} {moyenne} {plus_robuste:?} {par_niveau:?}");
}
```

**9.2**

```rust
fn composer<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}

let double_puis_plus_un = composer(|x: i32| x * 2, |x| x + 1);
assert_eq!(double_puis_plus_un(5), 11);
```

**9.3**

```rust
fn voisins_vivants(&self, x: usize, y: usize) -> usize {
    let (l, h) = (self.largeur, self.hauteur);
    [h - 1, 0, 1]
        .iter()
        .flat_map(|&dy| [l - 1, 0, 1].into_iter().map(move |dx| (dx, dy)))
        .filter(|&(dx, dy)| !(dx == 0 && dy == 0))
        .filter(|&(dx, dy)| self.get((x + dx) % l, (y + dy) % h))
        .count()
}
```

**9.4**

```rust
struct Bus {
    abonnes: Vec<Box<dyn Fn(&str)>>,
}

impl Bus {
    fn abonner(&mut self, f: impl Fn(&str) + 'static) {
        self.abonnes.push(Box::new(f));
    }
    fn publier(&self, message: &str) {
        for f in &self.abonnes {
            f(message);
        }
    }
}
```

`+ 'static` : la closure ne doit pas emprunter de données temporaires, puisqu'elle sera stockée.

</details>

---

## Module 10 — Pointeurs intelligents

📖 Livre : chapitre 15.

### Notions

| Type | Rôle | Coût |
|------|------|------|
| `Box<T>` | valeur sur le tas, un seul propriétaire | une allocation |
| `Rc<T>` | plusieurs propriétaires, **un seul thread** | compteur |
| `Arc<T>` | plusieurs propriétaires, **multi-thread** | compteur atomique (un peu plus cher) |
| `RefCell<T>` | mutabilité intérieure, vérifiée **à l'exécution**, un thread | vérification à chaque emprunt |
| `Mutex<T>` / `RwLock<T>` | mutabilité intérieure **multi-thread** | verrou |
| `Weak<T>` | référence non propriétaire vers un `Rc`/`Arc` (casse les cycles) | — |
| `Cell<T>` | mutabilité intérieure pour les types `Copy`, sans emprunt | quasi nul |

#### `Box<T>`

Utile pour : les types récursifs, les objets trait (`Box<dyn Trait>`), déplacer une grosse valeur sans la copier.

```rust
enum Expr {
    Nombre(f32),
    Addition(Box<Expr>, Box<Expr>),   // sans Box : taille infinie
}
```

#### `Rc<T>` + `RefCell<T>` : partager et modifier

```rust
use std::cell::RefCell;
use std::rc::Rc;

let texture = Rc::new(RefCell::new(String::from("herbe.png")));
let sol = Rc::clone(&texture);
let mur = Rc::clone(&texture);

sol.borrow_mut().push_str(" (v2)");   // emprunt mutable vérifié à l'exécution
println!("{}", mur.borrow());          // herbe.png (v2)
println!("{}", Rc::strong_count(&texture)); // 3
```

⚠️ `RefCell` déplace la vérification des emprunts à l'exécution : deux `borrow_mut()` simultanés **paniquent**.

#### Le graphe de scène et les cycles

Un parent possède ses enfants ; un enfant connaît son parent. Si les deux liens sont des `Rc`, le compteur n'atteint jamais 0 : **fuite mémoire**. Solution : `Weak` pour le lien vers le parent.

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Noeud {
    nom: String,
    parent: RefCell<Weak<Noeud>>,
    enfants: RefCell<Vec<Rc<Noeud>>>,
}

let racine = Rc::new(Noeud {
    nom: "monde".into(),
    parent: RefCell::new(Weak::new()),
    enfants: RefCell::new(vec![]),
});
let joueur = Rc::new(Noeud {
    nom: "joueur".into(),
    parent: RefCell::new(Rc::downgrade(&racine)),
    enfants: RefCell::new(vec![]),
});
racine.enfants.borrow_mut().push(Rc::clone(&joueur));

if let Some(p) = joueur.parent.borrow().upgrade() {   // Weak -> Option<Rc>
    println!("parent de {} : {}", joueur.nom, p.nom);
}
```

> ⚠️ **Avertissement de conception** : ce style « objets reliés par des pointeurs » (courant en C++/Java) se bat contre Rust **et** est lent (données éparpillées, compteurs, vérifications). Les moteurs Rust modernes stockent les objets dans des `Vec` et les relient par des **index** (ECS, module 20 ; handles, module 21). Connaissez `Rc<RefCell<T>>`, mais utilisez-le avec parcimonie.

### Exercices

**10.1** — Implémentez `fn evaluer(e: &Expr) -> f32` et ajoutez `Multiplication`.

**10.2** — Créez un cache de textures partagé : `struct Cache { textures: HashMap<String, Rc<Texture>> }` avec `fn obtenir(&mut self, chemin: &str) -> Rc<Texture>` qui ne « charge » (crée) une texture qu'une fois. Affichez `Rc::strong_count` pour vérifier le partage.

**10.3** — Réécrivez le graphe de scène sans `Rc` : `struct Scene { noeuds: Vec<Noeud> }` où `Noeud { nom, parent: Option<usize>, enfants: Vec<usize> }`. Écrivez `fn chemin(&self, i: usize) -> String` qui renvoie `"monde/joueur/arme"`. Comparez la simplicité des deux versions.

<details>
<summary>▶ Solutions</summary>

**10.1**

```rust
enum Expr {
    Nombre(f32),
    Addition(Box<Expr>, Box<Expr>),
    Multiplication(Box<Expr>, Box<Expr>),
}

fn evaluer(e: &Expr) -> f32 {
    match e {
        Expr::Nombre(n) => *n,
        Expr::Addition(a, b) => evaluer(a) + evaluer(b),
        Expr::Multiplication(a, b) => evaluer(a) * evaluer(b),
    }
}
```

**10.2**

```rust
use std::collections::HashMap;
use std::rc::Rc;

struct Texture { chemin: String }

struct Cache { textures: HashMap<String, Rc<Texture>> }

impl Cache {
    fn obtenir(&mut self, chemin: &str) -> Rc<Texture> {
        let tex = self
            .textures
            .entry(chemin.to_string())
            .or_insert_with(|| {
                println!("chargement de {chemin}");
                Rc::new(Texture { chemin: chemin.to_string() })
            });
        Rc::clone(tex)
    }
}
```

**10.3**

```rust
struct Noeud { nom: String, parent: Option<usize>, enfants: Vec<usize> }
struct Scene { noeuds: Vec<Noeud> }

impl Scene {
    fn ajouter(&mut self, nom: &str, parent: Option<usize>) -> usize {
        let id = self.noeuds.len();
        self.noeuds.push(Noeud { nom: nom.to_string(), parent, enfants: vec![] });
        if let Some(p) = parent {
            self.noeuds[p].enfants.push(id);
        }
        id
    }

    fn chemin(&self, mut i: usize) -> String {
        let mut parties = vec![self.noeuds[i].nom.as_str()];
        while let Some(p) = self.noeuds[i].parent {
            parties.push(&self.noeuds[p].nom);
            i = p;
        }
        parties.reverse();
        parties.join("/")
    }
}
```

Plus de `Rc`, plus de `RefCell`, plus de cycles possibles, et les nœuds sont contigus en mémoire. Inconvénient à régler plus tard : supprimer un nœud invalide les index (→ index générationnels, module 21).

</details>

---

## Module 11 — Traits avancés et surcharge d'opérateurs

📖 Livre : chapitre 20.2 (« Traits avancés »).

### Notions

#### Surcharger les opérateurs

Les opérateurs (`+`, `*`, `-`, `[]`...) sont des traits de `std::ops` :

```rust
use std::ops::{Add, Mul, Neg, Sub, AddAssign, Index};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, o: Vec3) -> Vec3 { Vec3 { x: self.x + o.x, y: self.y + o.y, z: self.z + o.z } }
}

impl Mul<f32> for Vec3 {            // Vec3 * f32
    type Output = Vec3;
    fn mul(self, s: f32) -> Vec3 { Vec3 { x: self.x * s, y: self.y * s, z: self.z * s } }
}

impl Mul<Vec3> for f32 {            // f32 * Vec3
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 { v * self }
}

impl AddAssign for Vec3 {           // v += w
    fn add_assign(&mut self, o: Vec3) { *self = *self + o; }
}

impl Index<usize> for Vec3 {        // v[0]
    type Output = f32;
    fn index(&self, i: usize) -> &f32 {
        match i { 0 => &self.x, 1 => &self.y, 2 => &self.z, _ => panic!("index {i} hors de Vec3") }
    }
}

let position = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
let vitesse = Vec3 { x: 0.0, y: 2.0, z: 0.0 };
let dt = 0.5;
let nouvelle = position + vitesse * dt;
```

#### Les conversions : `From` / `Into`

```rust
impl From<[f32; 3]> for Vec3 {
    fn from(a: [f32; 3]) -> Self { Vec3 { x: a[0], y: a[1], z: a[2] } }
}

let v: Vec3 = [1.0, 2.0, 3.0].into();   // `Into` est fourni automatiquement
let w = Vec3::from([0.0; 3]);

fn deplacer(cible: impl Into<Vec3>) { let _c: Vec3 = cible.into(); }
deplacer([1.0, 1.0, 1.0]);              // API souple pour l'utilisateur
```

#### `Display`

```rust
impl std::fmt::Display for Vec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.x, self.y, self.z)
    }
}
```

#### `Drop` : le destructeur

```rust
struct Chrono { nom: &'static str, debut: std::time::Instant }

impl Drop for Chrono {
    fn drop(&mut self) {
        println!("{} : {:?}", self.nom, self.debut.elapsed());
    }
}

fn charger_niveau() {
    let _c = Chrono { nom: "chargement", debut: std::time::Instant::now() };
    // ... travail ...
} // affiche automatiquement la durée ici
```

Ce patron (**RAII** : une ressource est libérée quand son propriétaire disparaît) est partout : fichiers, verrous, ressources GPU, zones de profilage.

#### Les traits « marqueurs » et les super-traits

```rust
trait Composant: 'static + Send + Sync {}   // tout Composant doit être 'static, Send et Sync
impl<T: 'static + Send + Sync> Composant for T {}  // implémentation « blanket » : pour TOUS les T
```

> 🎮 C'est la technique de `bevy_ecs` : n'importe quelle struct peut devenir un composant.

#### La règle de l'orphelin

Vous pouvez implémenter un trait pour un type **seulement si** le trait ou le type est défini dans votre crate. Impossible donc d'implémenter `Display` pour `wgpu::Color`. Contournement : un *newtype* `struct MaCouleur(wgpu::Color);`.

### Exercices

**11.1** — Complétez `Vec3` : `Sub`, `Neg`, `SubAssign`, `MulAssign<f32>`, `Div<f32>`, `IndexMut`, et les méthodes `dot`, `cross`, `length`, `normalize`, `lerp`. Constantes `ZERO`, `ONE`, `X`, `Y`, `Z`.

**11.2** — Écrivez des tests : `X.cross(Y) == Z`, `v.normalize().length() ≈ 1`, `a.dot(b) == b.dot(a)`.

**11.3** — Implémentez `From<Vec3> for [f32; 3]` puis ajoutez `#[repr(C)]` et `bytemuck::Pod`/`Zeroable` pour pouvoir envoyer des `Vec3` au GPU.

<details>
<summary>▶ Solutions (extraits)</summary>

```rust
impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const X: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }

    pub fn dot(self, o: Vec3) -> f32 { self.x * o.x + self.y * o.y + self.z * o.z }

    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    pub fn length(self) -> f32 { self.dot(self).sqrt() }

    pub fn normalize(self) -> Vec3 {
        let l = self.length();
        if l > 0.0 { self * (1.0 / l) } else { Vec3::ZERO }
    }

    pub fn lerp(self, o: Vec3, t: f32) -> Vec3 { self + (o - self) * t }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, o: Vec3) -> Vec3 { Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z) }
}

impl Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 { Vec3::new(-self.x, -self.y, -self.z) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produit_vectoriel() { assert_eq!(Vec3::X.cross(Vec3::Y), Vec3::Z); }

    #[test]
    fn normalisation() {
        let v = Vec3::new(3.0, 4.0, 12.0).normalize();
        assert!((v.length() - 1.0).abs() < 1e-6);
    }
}
```

</details>

---

## Module 12 — Les macros déclaratives

📖 Livre : chapitre 20.5.

### Notions

Une macro `macro_rules!` transforme du code **avant** la compilation. Utile pour éliminer la répétition que les fonctions et génériques ne peuvent pas éliminer.

```rust
macro_rules! vec3 {
    ($x:expr, $y:expr, $z:expr) => {
        Vec3::new($x, $y, $z)
    };
    ($v:expr) => {
        Vec3::new($v, $v, $v)
    };
}

let a = vec3!(1.0, 2.0, 3.0);
let b = vec3!(0.5);
```

| Fragment | Correspond à |
|----------|--------------|
| `$x:expr` | une expression |
| `$t:ty` | un type |
| `$n:ident` | un identifiant (nom) |
| `$($x:expr),*` | une liste répétée, séparée par des virgules |

Répétitions :

```rust
macro_rules! impl_operateur {
    ($($trait:ident, $methode:ident, $op:tt);* $(;)?) => {
        $(
            impl std::ops::$trait for Vec3 {
                type Output = Vec3;
                fn $methode(self, o: Vec3) -> Vec3 {
                    Vec3::new(self.x $op o.x, self.y $op o.y, self.z $op o.z)
                }
            }
        )*
    };
}

impl_operateur! {
    Add, add, +;
    Sub, sub, -;
    Mul, mul, *;
    Div, div, /;
}
```

Les **macros procédurales** (`#[derive(MonTrait)]`, `#[mon_attribut]`) sont plus puissantes mais demandent une crate dédiée (`proc-macro = true`) et les crates `syn`/`quote`. Les moteurs s'en servent pour la réflexion (`#[derive(Component, Reflect)]` dans Bevy). À aborder **plus tard**, quand le besoin sera réel.

> ⚠️ Règle : une macro est plus dure à lire, à déboguer et à documenter. **Fonction d'abord, générique ensuite, macro en dernier.**

### Exercices

**12.1** — Écrivez `hashmap!{ "a" => 1, "b" => 2 }` qui construit une `HashMap`.

**12.2** — Utilisez `impl_operateur!` pour générer aussi les versions `AddAssign`, `SubAssign`... de votre `Vec3`.

**12.3** — Écrivez `chrono!("nom", { bloc })` qui exécute le bloc, affiche sa durée, et renvoie sa valeur.

<details>
<summary>▶ Solutions</summary>

```rust
macro_rules! hashmap {
    ($($cle:expr => $valeur:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $( map.insert($cle, $valeur); )*
        map
    }};
}

macro_rules! chrono {
    ($nom:expr, $bloc:block) => {{
        let debut = std::time::Instant::now();
        let resultat = $bloc;
        println!("{} : {:?}", $nom, debut.elapsed());
        resultat
    }};
}

let somme = chrono!("somme", { (0..1_000_000u64).sum::<u64>() });
```

Les doubles accolades `{{ ... }}` : la macro produit un **bloc**, utilisable comme expression.

</details>

---

## Projet II — Une bibliothèque mathématique 3D

**Objectif** : écrire `zentrix-math`, votre propre mini-`glam`. Vous comprendrez ce que fait une caméra, et vous aurez une bibliothèque testée et mesurée.

### Cahier des charges

1. Un package bibliothèque (`cargo new --lib zentrix-math`).
2. `Vec2`, `Vec3`, `Vec4` avec opérateurs, `dot`, `length`, `normalize`, `lerp` ; `cross` pour `Vec3`.
3. `Mat4` (4×4, stockée **par colonnes** comme en WGSL/OpenGL : `cols: [Vec4; 4]`) avec :
   - `IDENTITY`, `Mul<Mat4>`, `Mul<Vec4>` ;
   - `from_translation(Vec3)`, `from_scale(Vec3)`, `from_rotation_y(angle)` ;
   - `look_at_rh(eye, target, up)` (matrice de vue) ;
   - `perspective_rh(fov_y, aspect, near, far)` (projection, profondeur 0..1 comme wgpu) ;
   - `transpose`.
4. `#[repr(C)]` + `bytemuck::Pod` sur tous les types.
5. Tests unitaires pour chaque opération, y compris : `M * IDENTITY == M`, une translation déplace un point mais pas une direction (`w = 1` vs `w = 0`).
6. Documentation avec exemples, `cargo doc` sans avertissement.
7. Benchmark (module 14) : comparez votre `Mat4 * Mat4` à celui de `glam`.

### Ressources

- [Learn Wgpu — Uniforms et caméra](https://sotrh.github.io/learn-wgpu/beginner/tutorial6-uniforms/)
- [Scratchapixel — Geometry](https://www.scratchapixel.com/) (les maths du rendu, très pédagogique)
- Le code source de `glam` sur docs.rs (cliquez sur « source »)

<details>
<summary>▶ Aide : look_at et perspective (main droite, profondeur 0..1)</summary>

```rust
impl Mat4 {
    pub fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        let f = (target - eye).normalize();      // direction du regard
        let s = f.cross(up).normalize();         // droite
        let u = s.cross(f);                      // haut corrigé
        Mat4 {
            cols: [
                Vec4::new(s.x, u.x, -f.x, 0.0),
                Vec4::new(s.y, u.y, -f.y, 0.0),
                Vec4::new(s.z, u.z, -f.z, 0.0),
                Vec4::new(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
            ],
        }
    }

    pub fn perspective_rh(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let h = 1.0 / (fov_y * 0.5).tan();
        let w = h / aspect;
        let r = far / (near - far);
        Mat4 {
            cols: [
                Vec4::new(w, 0.0, 0.0, 0.0),
                Vec4::new(0.0, h, 0.0, 0.0),
                Vec4::new(0.0, 0.0, r, -1.0),
                Vec4::new(0.0, 0.0, r * near, 0.0),
            ],
        }
    }
}
```

Vérifiez vos résultats contre `glam::Mat4::look_at_rh` et `glam::Mat4::perspective_rh` dans vos tests : c'est la meilleure façon de valider des maths.

</details>

> 💡 **Et dans ZENTRIX ?** Le moteur utilisera `glam` (optimisé SIMD, très testé). Ce projet sert à **comprendre** ; ne réinventez pas la roue en production.

### ✅ Checklist de fin de partie II

- [ ] Je sais choisir entre générique et `dyn Trait`, et expliquer le coût de chacun.
- [ ] Je comprends les annotations de durée de vie dans les signatures de wgpu.
- [ ] J'écris naturellement des chaînes d'itérateurs.
- [ ] Je sais quand `Rc<RefCell<T>>` est approprié et pourquoi les index sont souvent préférables.
- [ ] J'ai une bibliothèque mathématique testée, avec une matrice de caméra qui fonctionne.
- [ ] Je peux lire et expliquer tout `crates/zentrix-platform` et `crates/zentrix-render`.

---

# Partie III — Rust pour la performance

**Objectif de la partie** : comprendre **pourquoi** un code est rapide ou lent, savoir le **mesurer**, et exploiter tous les cœurs du processeur. C'est ce qui distingue un moteur « qui marche » d'un moteur « très performant ».

> 📏 **La règle d'or de toute la partie** : *on ne devine pas, on mesure.* Toute optimisation commence par une mesure et se termine par une mesure.

---

## Module 13 — Comment la machine exécute votre code

📖 Lecture : [Rust Performance Book](https://nnethercote.github.io/perf-book/) (anglais, gratuit, court). Vidéo culte : *Data-Oriented Design and C++*, Mike Acton (CppCon 2014).

### Notions

#### La hiérarchie mémoire

Le processeur est **beaucoup** plus rapide que la mémoire. Pour compenser, il utilise des **caches** :

| Niveau | Taille typique | Latence approximative |
|--------|----------------|-----------------------|
| Registres | quelques centaines d'octets | 0 cycle |
| Cache L1 | 32–64 Ko par cœur | ~1 ns (4 cycles) |
| Cache L2 | 256 Ko – 2 Mo par cœur | ~4 ns |
| Cache L3 | 8–64 Mo partagés | ~10–40 ns |
| RAM | Go | **~80–100 ns** |

Lire une valeur en RAM coûte autant que **des centaines** d'additions. Dans un moteur, le facteur limitant est très souvent **l'accès à la mémoire**, pas le calcul.

#### Les lignes de cache

La mémoire n'est jamais chargée octet par octet, mais par **lignes de 64 octets**. Conséquences :

- Parcourir un tableau **contigu** dans l'ordre est idéal : chaque ligne chargée sert pour plusieurs éléments, et le processeur **anticipe** (*prefetch*) les lignes suivantes.
- Sauter d'un objet à l'autre via des pointeurs (`Vec<Box<T>>`, listes chaînées, `Rc`) provoque un **défaut de cache** (*cache miss*) presque à chaque élément.

```text
Vec<Particule> (contigu)       │p0│p1│p2│p3│p4│p5│...   → lecture en flux, très rapide
Vec<Box<Particule>> (éparpillé)  ptr ──▶ p0 ......... ptr ──▶ p1 ....  → un saut par élément
```

#### Taille et alignement

Chaque type a une **taille** et un **alignement** (sa position en mémoire doit être un multiple de l'alignement).

```rust
use std::mem::{align_of, size_of};

struct A { a: u8, b: u32, c: u8 }          // Rust réordonne les champs : 8 octets
#[repr(C)]
struct B { a: u8, b: u32, c: u8 }          // ordre imposé : 1 + 3 (trou) + 4 + 1 + 3 (trou) = 12 octets

println!("{} {}", size_of::<A>(), size_of::<B>()); // 8 12
println!("{}", align_of::<u32>());                  // 4
println!("{}", size_of::<Option<Box<u64>>>());      // 8 : `None` utilise le pointeur nul (optimisation « niche »)
```

- Par défaut, Rust **réordonne** les champs pour minimiser les trous.
- `#[repr(C)]` garde l'ordre écrit : indispensable pour le GPU et le C, mais attention aux trous.

#### Pile vs tas, et le coût des allocations

Une allocation (`Box::new`, `Vec::push` qui dépasse la capacité, `String::from`, `format!`, `collect()`) appelle l'allocateur : quelques dizaines à centaines de nanosecondes, et des données dispersées. **Dans la boucle de jeu, visez zéro allocation par image** :

```rust
// ❌ alloue un nouveau Vec à chaque image
fn visibles(entites: &[Entite]) -> Vec<usize> { /* ... */ }

// ✅ réutilise un buffer
fn visibles_dans(entites: &[Entite], sortie: &mut Vec<usize>) {
    sortie.clear();               // garde la capacité, ne libère rien
    for (i, e) in entites.iter().enumerate() {
        if e.visible { sortie.push(i); }
    }
}
```

#### Les branches et la prédiction

Le processeur **devine** le résultat des `if` pour continuer à travailler. Une condition imprévisible (50 % / 50 % au hasard) coûte ~10–20 cycles à chaque erreur. Trier les données ou séparer les cas en plusieurs listes (module 17) supprime ces branches.

#### Debug vs release

```bash
cargo run            # debug : pas d'optimisation, vérifications de dépassement d'entier
cargo run --release  # release : optimisé, 10 à 100 fois plus rapide
```

**Ne mesurez jamais les performances en mode debug.**

### Exercices

**13.1** — Affichez `size_of` et `align_of` de : `u8`, `f32`, `(u8, f64)`, `[f32; 3]`, `Option<u32>`, `Option<&u8>`, `String`, `Vec<u8>`, `Box<[u8]>`, `&str`, `&[u8]`, `Box<dyn Fn()>`. Expliquez chaque résultat.

**13.2** — Créez un tableau de 10 millions de `f32` et sommez-le (a) dans l'ordre, (b) avec un pas de 16 (`for start in 0..16 { for i in (start..n).step_by(16) ... }`). Même nombre d'additions : mesurez la différence en `--release`.

**13.3** — Sommez un champ de 1 million de structs `Particule { pos: [f32; 3], vel: [f32; 3], couleur: [f32; 4], age: f32 }` stockées dans `Vec<Particule>` puis dans `Vec<Box<Particule>>` (allouées à des moments différents). Mesurez.

**13.4** — Une matrice 4000×4000 dans un `Vec<f32>` à plat : sommez-la ligne par ligne, puis colonne par colonne. Expliquez l'écart.

<details>
<summary>▶ Éléments de réponse</summary>

**13.1** — `Option<u32>` = 8 (il faut stocker le « discriminant ») ; `Option<&u8>` = 8 comme `&u8` (niche : une référence n'est jamais nulle) ; `String` et `Vec<u8>` = 24 (pointeur + longueur + capacité) ; `Box<[u8]>`, `&str`, `&[u8]` = 16 (pointeur + longueur : *fat pointer*) ; `Box<dyn Fn()>` = 16 (pointeur + pointeur de vtable).

**13.2 / 13.4** — L'accès séquentiel est typiquement 3 à 10 fois plus rapide : avec un pas de 16 `f32` (64 octets), chaque accès charge une nouvelle ligne de cache dont on n'utilise qu'un élément.

**13.3** — La version `Box` est souvent 2 à 5 fois plus lente, et pire si les allocations ont été mélangées avec d'autres.

Squelette de mesure :

```rust
use std::hint::black_box;
use std::time::Instant;

let donnees = vec![1.0_f32; 10_000_000];
let t = Instant::now();
let s: f32 = donnees.iter().sum();
println!("{:?} (résultat {})", t.elapsed(), black_box(s));
```

`black_box` empêche le compilateur de supprimer un calcul dont il penserait le résultat inutile.

</details>

---

## Module 14 — Mesurer avant d'optimiser

### Notions

#### Trois échelles de mesure

| Échelle | Outil | Question |
|---------|-------|----------|
| Une fonction | **benchmark** (`criterion`, `divan`) | « Cette version est-elle plus rapide que l'autre ? » |
| Le programme entier | **profileur échantillonneur** (`samply`, Superluminal, VTune, `perf`) | « Où passe le temps ? » |
| Une image du jeu | **profileur instrumenté** (Tracy, `puffin`) | « Pourquoi l'image 4512 a-t-elle pris 40 ms ? » |
| Le GPU | **RenderDoc**, PIX, Nsight, requêtes de timestamp wgpu | « Quelle passe de rendu est lente ? » |

#### Les benchmarks avec `criterion`

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.8"

[[bench]]
name = "maths"
harness = false
```

```rust
// benches/maths.rs
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn somme(v: &[f32]) -> f32 { v.iter().sum() }

fn bench(c: &mut Criterion) {
    let v = vec![1.0_f32; 100_000];
    c.bench_function("somme 100k", |b| b.iter(|| somme(black_box(&v))));
}

criterion_group!(benches, bench);
criterion_main!(benches);
```

`cargo bench` exécute la fonction des milliers de fois, élimine le bruit statistique et compare automatiquement avec la mesure précédente (« +12 % : régression détectée »).

> ⚙️ Vérifiez la dernière version de `criterion` sur crates.io avant de l'ajouter.

#### Le profilage du programme

[`samply`](https://github.com/mstange/samply) fonctionne sous Windows, Linux et macOS :

```bash
cargo install samply
cargo build --profile profiling
samply record target/profiling/zentrix-app.exe
```

Il ouvre un *flame graph* : la largeur de chaque bloc = le temps passé dans cette fonction. Cherchez les blocs larges.

Pour que les noms de fonctions apparaissent en release, ajoutez un profil au `Cargo.toml` racine :

```toml
[profile.profiling]
inherits = "release"
debug = true          # symboles de débogage, sans perte de vitesse
```

#### Le profilage par image (indispensable pour un moteur)

On place des « zones » dans le code ; un outil affiche une frise chronologique par image et par thread.

```rust
// avec la crate `puffin` (ou `tracy-client`, ou la façade `profiling`)
fn update(&mut self) {
    puffin::profile_function!();
    {
        puffin::profile_scope!("physique");
        self.physique.step();
    }
    {
        puffin::profile_scope!("ia");
        self.ia.update();
    }
}
```

La crate [`profiling`](https://crates.io/crates/profiling) offre une façade unique pour Tracy, puffin, Superluminal... : c'est ce que ZENTRIX devrait utiliser.

#### Le budget d'image

| Cible | Budget par image |
|-------|------------------|
| 30 FPS | 33,3 ms |
| 60 FPS | 16,6 ms |
| 144 FPS | 6,9 ms |

Raisonnez en **millisecondes**, pas en FPS : passer de 1000 à 500 FPS ne coûte que 1 ms, alors que de 60 à 30 FPS en coûte 16,7.

#### Le profil release d'un moteur

```toml
[profile.release]
lto = "thin"          # optimisation entre crates
codegen-units = 1     # meilleure optimisation, compilation plus lente
panic = "abort"       # binaire plus petit et un peu plus rapide (pas de déroulement de pile)
```

Et pour cibler le processeur de la machine (ne distribuez pas ce binaire à d'autres machines) :

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

#### Les optimisations classiques, par ordre de rentabilité

1. **Meilleur algorithme** (O(n²) → O(n log n)) : rien ne bat ça.
2. **Ne pas faire le travail** : cache, calcul paresseux, culling (ne pas traiter ce qui est invisible).
3. **Disposition mémoire** (module 17).
4. **Éliminer les allocations** dans les chemins chauds.
5. **Parallélisme** (module 15).
6. **SIMD** et micro-optimisations (module 17).

### Exercices

**14.1** — Ajoutez un benchmark `criterion` à votre bibliothèque mathématique (projet II) : `Mat4 * Mat4`, `Vec3::normalize`, comparés à `glam`.

**14.2** — Profilez votre jeu de la vie (projet I) sur une grille 2000×2000 avec `samply`. Quelle fonction domine ? Optimisez-la (indice : le modulo `%` est coûteux ; traitez les bords à part) et mesurez le gain.

**14.3** — Ajoutez au `Cargo.toml` de ZENTRIX le profil `profiling`, lancez l'application sous `samply` et identifiez combien de temps prend `Renderer::render` côté CPU.

---

## Module 15 — La concurrence et le parallélisme

📖 Livre : chapitre 16. Puis [Rust Atomics and Locks](https://marabos.nl/atomics/) (Mara Bos, gratuit en ligne) pour aller loin.

### Notions

#### Les threads

```rust
use std::thread;

let handle = thread::spawn(|| {
    (0..1_000_000u64).sum::<u64>()
});
let resultat = handle.join().unwrap();  // attendre la fin
```

Les **threads à portée limitée** peuvent emprunter des données locales, car Rust garantit qu'ils se terminent avant la fin du bloc :

```rust
let mut donnees = vec![1.0_f32; 1_000_000];
let (gauche, droite) = donnees.split_at_mut(500_000);  // deux emprunts mutables disjoints

thread::scope(|s| {
    s.spawn(|| gauche.iter_mut().for_each(|x| *x *= 2.0));
    s.spawn(|| droite.iter_mut().for_each(|x| *x *= 2.0));
}); // les deux threads sont terminés ici
```

#### La sûreté garantie : `Send` et `Sync`

- `Send` : le type peut être **déplacé** vers un autre thread.
- `Sync` : le type peut être **partagé** (`&T`) entre threads.

Le compilateur les calcule automatiquement. `Rc` n'est ni `Send` ni `Sync` (son compteur n'est pas atomique) : essayer de l'envoyer dans un thread **ne compile pas**. Les *data races* sont donc **impossibles** en Rust sûr. C'est la « fearless concurrency ».

#### Partager un état modifiable

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicU64, Ordering};

let compteur = Arc::new(Mutex::new(0));
let mut handles = vec![];
for _ in 0..8 {
    let c = Arc::clone(&compteur);
    handles.push(thread::spawn(move || {
        *c.lock().unwrap() += 1;     // verrou libéré à la fin de l'instruction (RAII)
    }));
}
for h in handles { h.join().unwrap(); }

// Pour un simple compteur, un atomique est bien plus rapide qu'un Mutex :
let frames = AtomicU64::new(0);
frames.fetch_add(1, Ordering::Relaxed);
```

`RwLock` : plusieurs lecteurs **ou** un écrivain (comme les emprunts, mais à l'exécution).

#### Les canaux (*channels*)

Communiquer en envoyant des messages plutôt qu'en partageant de la mémoire :

```rust
use std::sync::mpsc;

let (envoi, reception) = mpsc::channel::<String>();
thread::spawn(move || {
    // thread de chargement d'assets
    envoi.send("texture chargée : herbe.png".to_string()).unwrap();
});
for message in reception { println!("{message}"); }
```

> 🎮 **Lien moteur** : le chargement d'assets en arrière-plan (module 21) utilise exactement ce patron : un thread charge, le thread principal reçoit les résultats sans jamais bloquer la boucle de jeu.

#### `rayon` : le parallélisme de données en une ligne

```toml
rayon = "1"
```

```rust
use rayon::prelude::*;

let mut particules: Vec<Particule> = creer_particules(1_000_000);

// séquentiel
particules.iter_mut().for_each(|p| p.avancer(dt));
// parallèle : changez juste iter_mut en par_iter_mut
particules.par_iter_mut().for_each(|p| p.avancer(dt));

let energie: f32 = particules.par_iter().map(|p| p.energie()).sum();

// par paquets : moins de surcoût par élément
particules.par_chunks_mut(4096).for_each(|paquet| {
    for p in paquet { p.avancer(dt); }
});
```

`rayon` répartit le travail sur tous les cœurs par **vol de tâches** (*work stealing*). Le compilateur garantit qu'il n'y a pas de conflit : si votre closure modifiait une variable partagée sans protection, elle ne compilerait pas.

#### Les pièges de performance

- **Trop petit** : paralléliser 100 éléments coûte plus cher que de les traiter. Mesurez.
- **Faux partage** (*false sharing*) : deux threads qui écrivent dans la même ligne de cache de 64 octets se ralentissent mutuellement.
- **Contention** : un `Mutex` pris par tous les threads à chaque élément annule le parallélisme. Préférez : chaque thread calcule un résultat local, puis on fusionne (`map` + `reduce`, `fold`).

#### Dans un moteur

Les moteurs modernes organisent chaque image en **tâches** (*jobs*) avec des dépendances : « animation » et « physique » en parallèle, puis « transformations », puis « préparation du rendu ». Un ECS comme `bevy_ecs` calcule automatiquement quels systèmes peuvent tourner en même temps à partir des composants qu'ils lisent et écrivent (module 20).

### Exercices

**15.1** — Parallélisez votre jeu de la vie avec `rayon` : chaque ligne de la grille de sortie est calculée indépendamment (`sortie.cellules.par_chunks_mut(largeur).enumerate()`). Mesurez sur 2000×2000 avec 1, 2, 4, 8 threads (variable d'environnement `RAYON_NUM_THREADS`).

**15.2** — Écrivez un « chargeur » : un thread qui simule le chargement de 10 fichiers (`thread::sleep` de 100 ms chacun) et envoie chaque résultat par un canal ; la boucle principale tourne à 60 Hz, affiche « image N » et traite les messages arrivés avec `try_recv()` **sans jamais bloquer**.

**15.3** — Calculez la somme de 100 millions de nombres avec 8 threads de trois manières : `Mutex<u64>` mis à jour à chaque nombre, `AtomicU64` à chaque nombre, somme locale par thread puis fusion. Classez-les.

<details>
<summary>▶ Solutions</summary>

**15.1**

```rust
use rayon::prelude::*;

fn etape_parallele(entree: &Grille, sortie: &mut Grille) {
    let largeur = entree.largeur;
    sortie
        .cellules
        .par_chunks_mut(largeur)
        .enumerate()
        .for_each(|(y, ligne)| {
            for (x, cellule) in ligne.iter_mut().enumerate() {
                let v = entree.voisins_vivants(x, y);
                *cellule = matches!((entree.get(x, y), v), (true, 2) | (true, 3) | (false, 3));
            }
        });
}
```

Remarquez : `entree` est partagée en lecture (`&`) par tous les threads, `sortie` est découpée en lignes disjointes (`&mut` différents). Le compilateur est satisfait, sans un seul verrou.

**15.2**

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for i in 0..10 {
            thread::sleep(Duration::from_millis(100));
            tx.send(format!("fichier_{i}.png")).unwrap();
        }
    });

    let mut charges = 0;
    let mut image = 0;
    while charges < 10 {
        while let Ok(nom) = rx.try_recv() {
            charges += 1;
            println!("  -> {nom} prêt");
        }
        image += 1;
        println!("image {image}");
        thread::sleep(Duration::from_millis(16));
    }
}
```

**15.3** — Du plus lent au plus rapide : `Mutex` (contention massive) ≫ `AtomicU64` (la ligne de cache fait la navette entre cœurs) ≫ somme locale (chaque thread travaille seul, une seule fusion à la fin). Les écarts peuvent dépasser un facteur 100.

</details>

---

## Module 16 — `unsafe` et FFI

📖 Livre : chapitre 20.1. Pour aller loin : [The Rustonomicon](https://doc.rust-lang.org/nomicon/).

### Notions

#### Ce que `unsafe` permet

Un bloc `unsafe` n'éteint **pas** les vérifications de Rust. Il autorise seulement cinq choses de plus :

1. déréférencer un pointeur brut (`*const T`, `*mut T`) ;
2. appeler une fonction `unsafe` (dont les fonctions C) ;
3. accéder à une variable `static mut` ;
4. implémenter un trait `unsafe` (`Send`, `Sync`...) ;
5. accéder aux champs d'une `union`.

Avec `unsafe`, **vous** promettez au compilateur que les règles sont respectées. Si vous vous trompez : comportement indéfini (*undefined behavior*), comme en C.

```rust
let v = vec![10, 20, 30];

// Sûr : vérifie les bornes (une comparaison, souvent éliminée par l'optimiseur)
let a = v[1];

// unsafe : aucune vérification. La responsabilité est à vous.
// SAFETY: 1 < v.len() == 3
let b = unsafe { *v.get_unchecked(1) };
```

#### Les bonnes pratiques

- **Commentaire `// SAFETY:`** obligatoire au-dessus de chaque bloc : expliquez pourquoi c'est correct.
- **Encapsulez** : un petit bloc `unsafe` derrière une API sûre (c'est ainsi que sont écrits `Vec`, `wgpu`, `bytemuck`).
- **Testez avec Miri** : `cargo +nightly miri test` détecte beaucoup de comportements indéfinis.
- **Préférez les crates éprouvées** : `bytemuck` plutôt que `std::mem::transmute`, `rayon` plutôt que des pointeurs partagés entre threads.

> 📏 Dans ZENTRIX, il y a **zéro** `unsafe` à ce jour, et c'est voulu. Ne l'introduisez que si une mesure prouve un gain réel et qu'aucune alternative sûre n'existe.

#### FFI : appeler du C

Les moteurs ont parfois besoin de bibliothèques C/C++ (PhysX, FMOD, une API de plateforme...).

```rust
unsafe extern "C" {
    fn abs(x: i32) -> i32;     // fonction de la bibliothèque C standard
}

fn main() {
    // SAFETY: `abs` n'a pas de précondition pour les valeurs différentes de i32::MIN
    let n = unsafe { abs(-5) };
    println!("{n}");
}
```

En pratique, on génère les déclarations avec `bindgen` et on écrit une couche Rust sûre par-dessus. Pour exposer **votre** moteur à d'autres langages (éditeur en C#, scripts...) : `#[unsafe(no_mangle)] pub extern "C" fn ...` et `cbindgen`.

### Exercices

**16.1** — Écrivez `fn somme_sans_bornes(v: &[f32]) -> f32` avec `get_unchecked`, comparez-la au benchmark de `v.iter().sum()`. Il y a de bonnes chances qu'il n'y ait **aucune** différence : expliquez pourquoi (indice : les itérateurs n'ont pas de vérification de bornes).

**16.2** — Écrivez `fn as_bytes(v: &[Vertex]) -> &[u8]` avec `std::slice::from_raw_parts`, en documentant les conditions de sûreté, puis remplacez-la par `bytemuck::cast_slice` et comparez la lisibilité.

<details>
<summary>▶ Solution 16.2</summary>

```rust
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex { position: [f32; 3], color: [f32; 3] }

fn as_bytes(v: &[Vertex]) -> &[u8] {
    // SAFETY:
    // - `Vertex` est `repr(C)`, composé uniquement de f32 : aucun octet de remplissage
    //   non initialisé, toute valeur est lisible comme des octets ;
    // - le pointeur et la longueur viennent d'une slice valide ;
    // - la durée de vie du résultat est liée à celle de `v`.
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}
```

`bytemuck` vérifie ces conditions **à la compilation** via le trait `Pod` : c'est pour ça que ZENTRIX l'utilise.

</details>

---

## Module 17 — Conception orientée données et SIMD

📖 Lecture : [Data-Oriented Design](https://www.dataorienteddesign.com/dodbook/) (Richard Fabian, gratuit en ligne).

### Notions

#### Penser en données, pas en objets

La programmation orientée objet pense « un ennemi a des PV, une position, un sprite, et une méthode `update` ». La **conception orientée données** (*DOD*) pense : « quelles données sont transformées, combien y en a-t-il, et dans quel ordre sont-elles lues ? »

> « Là où il y en a un, il y en a plusieurs. » — on optimise pour le **lot**, pas pour l'élément.

#### AoS vs SoA

```rust
// AoS : Array of Structures (le réflexe objet)
struct Particule { pos: [f32; 3], vel: [f32; 3], couleur: [f32; 4], age: f32, vie: f32 } // 52 octets
let particules: Vec<Particule> = Vec::new();

// SoA : Structure of Arrays
struct Particules {
    pos: Vec<[f32; 3]>,
    vel: Vec<[f32; 3]>,
    couleur: Vec<[f32; 4]>,
    age: Vec<f32>,
    vie: Vec<f32>,
}
```

Le système « avancer les particules » ne lit que `pos` et `vel` (24 octets utiles). En AoS, chaque ligne de cache transporte aussi `couleur`, `age`, `vie` : **plus de la moitié de la bande passante est gaspillée**. En SoA, chaque octet chargé sert.

```rust
impl Particules {
    fn avancer(&mut self, dt: f32) {
        for (p, v) in self.pos.iter_mut().zip(&self.vel) {
            p[0] += v[0] * dt;
            p[1] += v[1] * dt;
            p[2] += v[2] * dt;
        }
    }
}
```

| | AoS | SoA |
|---|---|---|
| Accès à un élément complet | ✅ simple | plusieurs tableaux |
| Système qui lit peu de champs | ❌ gaspille le cache | ✅ optimal |
| Vectorisation SIMD | difficile | ✅ naturelle |
| Ajouter/supprimer | simple | tous les tableaux à la fois |

Un **ECS à archétypes** (module 20) est, en substance, du SoA automatisé.

#### Séparer chaud et froid (*hot/cold splitting*)

Les champs lus à chaque image (position, vitesse) dans une struct ; les champs rarement lus (nom, description, statistiques) dans une autre, référencée par index.

#### Supprimer les branches par le tri

```rust
// ❌ une branche imprévisible par élément
for e in &mut entites {
    if e.actif { e.mettre_a_jour(dt); }
}

// ✅ deux listes : on ne parcourt que les actifs, sans condition
for e in &mut actifs { e.mettre_a_jour(dt); }
```

#### Le SIMD

**SIMD** (*Single Instruction, Multiple Data*) : une instruction traite 4, 8 ou 16 nombres à la fois (SSE, AVX sur x86 ; NEON sur ARM).

Trois façons d'en profiter, de la plus simple à la plus pointue :

1. **Vectorisation automatique** : écrivez des boucles simples sur des slices contiguës, sans branche ni dépendance entre itérations. Le compilateur fait le reste en `--release`.
   ```rust
   fn ajouter(a: &mut [f32], b: &[f32]) {
       for (x, y) in a.iter_mut().zip(b) { *x += *y; }   // vectorisée automatiquement
   }
   ```
   Aidez-le avec `chunks_exact(8)` et `-C target-cpu=native`.
2. **Bibliothèques** : `glam` utilise déjà SIMD (`Vec4`, `Mat4`, `Vec3A`, `Quat`) ; la crate `wide` fournit des types `f32x8` portables.
3. **Intrinsèques** : `std::arch::x86_64::*` (`unsafe`, spécifiques au processeur) ou `std::simd` (portable, encore en *nightly*).

> 📌 `glam::Vec3A` est un `Vec3` aligné sur 16 octets (avec un 4ᵉ flottant inutilisé) : plus gros, mais calculs SIMD. Encore un compromis mémoire / calcul à **mesurer**.

#### Vérifier ce que génère le compilateur

[Compiler Explorer](https://godbolt.org) (langage Rust, option `-C opt-level=3`) montre l'assembleur. Les instructions `vaddps`, `vmulps` (registres `ymm`) indiquent du SIMD AVX.

### Exercices

**17.1** — Implémentez la même simulation de particules en AoS et en SoA. Mesurez `avancer` sur 1 million de particules. Puis ajoutez un système `vieillir` qui ne touche que `age` et `vie`, et mesurez encore.

**17.2** — Écrivez `fn produit_scalaire(a: &[f32], b: &[f32]) -> f32` de trois façons : boucle indexée, `zip().map().sum()`, et avec `chunks_exact(8)` et 8 accumulateurs. Regardez l'assembleur sur Compiler Explorer et mesurez.

**17.3** — Réorganisez une liste de 100 000 entités `{ actif: bool, ... }` en deux `Vec` (actives / inactives) et mesurez la mise à jour avec 50 % d'entités actives réparties au hasard.

<details>
<summary>▶ Indice 17.2</summary>

```rust
fn produit_scalaire_8(a: &[f32], b: &[f32]) -> f32 {
    let mut acc = [0.0_f32; 8];
    let (ca, cb) = (a.chunks_exact(8), b.chunks_exact(8));
    let (ra, rb) = (ca.remainder(), cb.remainder());
    for (x, y) in ca.zip(cb) {
        for i in 0..8 {
            acc[i] += x[i] * y[i];
        }
    }
    let reste: f32 = ra.iter().zip(rb).map(|(x, y)| x * y).sum();
    acc.iter().sum::<f32>() + reste
}
```

Pourquoi 8 accumulateurs ? Les flottants ne sont pas associatifs (`(a+b)+c ≠ a+(b+c)` au dernier bit près), donc le compilateur n'a **pas le droit** de réordonner une somme unique. En écrivant vous-même 8 sommes indépendantes, vous lui donnez cette permission.

</details>

---

## Projet III — 100 000 particules

**Objectif** : appliquer toute la partie III sur un vrai cas de moteur, et l'afficher avec ZENTRIX.

### Étape 1 — Simulation (sans affichage)

1. `Particules` en SoA : position, vitesse, couleur, âge, durée de vie.
2. Systèmes : `emettre(n)`, `appliquer_gravite(dt)`, `avancer(dt)`, `vieillir(dt)`, `supprimer_mortes()` (avec `swap_remove` sur tous les tableaux en même temps).
3. Zéro allocation par image une fois le régime établi (vérifiez avec `Vec::capacity`).
4. Benchmarks `criterion` de chaque système à 10k, 100k, 1M particules.

### Étape 2 — Parallélisme

5. Version `rayon` de `appliquer_gravite`, `avancer` et `vieillir`. À partir de combien de particules est-ce rentable ?
6. Tableau de résultats : AoS / SoA, séquentiel / parallèle, debug / release.

### Étape 3 — Affichage dans ZENTRIX

7. Chaque particule est un petit carré dessiné avec le **rendu instancié** : un seul maillage (le carré), et un *buffer d'instances* (`VertexStepMode::Instance`) contenant position et couleur de chaque particule, mis à jour chaque image avec `queue.write_buffer`.
8. Un seul `draw_indexed(0..6, 0, 0..nombre_particules)` pour tout dessiner.
9. Mesurez le temps CPU de la simulation et de la préparation du rendu ; visez moins de 4 ms pour 100 000 particules.

### ✅ Checklist de fin de partie III

- [ ] Je raisonne en lignes de cache et en millisecondes par image.
- [ ] J'ai écrit des benchmarks `criterion` et profilé un programme avec `samply` ou Tracy.
- [ ] Je parallélise une boucle avec `rayon` et je sais quand ça ne vaut pas le coup.
- [ ] Je comprends pourquoi `Send`/`Sync` empêchent les data races.
- [ ] Je sais ce que garantit (et ne garantit pas) un bloc `unsafe`.
- [ ] J'ai mesuré moi-même l'écart AoS / SoA.

---

# Partie IV — Architecture d'un game engine

**Objectif de la partie** : construire, dans ZENTRIX, chaque sous-système d'un moteur moderne, en comprenant les compromis de conception. Cette partie suit la [feuille de route du moteur](guide-debutant/07-feuille-de-route.md) et les phases de [GAME_ENGINE_DEVELOPMENT_PHASES.md](GAME_ENGINE_DEVELOPMENT_PHASES.md).

> 📌 À partir d'ici, **les exercices se font dans ZENTRIX**, sur une branche git par module (`git switch -c module-18-boucle`). Chaque module se termine par une démo visible et des tests là où c'est possible.

Les deux qualités visées guident chaque choix :

| Qualité | Ce que ça veut dire concrètement |
|---------|----------------------------------|
| **Polyvalent** | Le même moteur sert à un jeu 2D, un jeu 3D, un outil, un éditeur. Les sous-systèmes sont **découplés**, **optionnels** (features Cargo, plugins) et pilotés par les **données**. |
| **Très performant** | Données contiguës, zéro allocation par image, travail réparti sur tous les cœurs, GPU alimenté efficacement, et **tout est mesuré**. |

---

## Module 18 — La boucle de jeu

📖 Lecture : [Game Programming Patterns — Game Loop](https://gameprogrammingpatterns.com/game-loop.html) et l'article *Fix Your Timestep!* de Glenn Fiedler.

### Notions

#### Le problème du pas de temps variable

ZENTRIX appelle actuellement `on_update(dt)` avec un `dt` qui varie à chaque image. C'est très bien pour les animations visuelles, mais **dangereux pour la physique** :

- un gros `dt` (une saccade de 200 ms) fait traverser les murs ;
- le résultat dépend du nombre d'images par seconde : deux joueurs ne voient pas la même simulation ;
- impossible de rejouer une partie ou de synchroniser un jeu en réseau (non **déterministe**).

#### La solution : le pas fixe avec accumulateur

```rust
const PAS_FIXE: f32 = 1.0 / 60.0;   // la simulation avance toujours de 1/60 s

struct Boucle {
    accumulateur: f32,
}

impl Boucle {
    fn image(&mut self, dt: f32, monde: &mut Monde) {
        // 1. On limite dt pour éviter la « spirale de la mort » : si une image a
        //    pris 2 secondes, on ne veut pas simuler 120 pas d'un coup.
        self.accumulateur += dt.min(0.25);

        // 2. On simule autant de pas fixes que le temps écoulé le permet.
        while self.accumulateur >= PAS_FIXE {
            monde.etat_precedent = monde.etat_actuel.clone();
            monde.simuler(PAS_FIXE);
            self.accumulateur -= PAS_FIXE;
        }

        // 3. Le rendu interpole entre les deux derniers états pour rester fluide
        //    même si l'écran tourne à 144 Hz et la simulation à 60 Hz.
        let alpha = self.accumulateur / PAS_FIXE;
        monde.rendre(alpha);   // position affichée = lerp(precedent, actuel, alpha)
    }
}
```

```text
temps réel  : |──image──|──image──|──image──|──image──|   (144 Hz, irrégulier)
simulation  : |────pas────|────pas────|────pas────|        (60 Hz, régulier)
rendu       : interpolé entre les deux derniers pas
```

#### L'ordre d'une image dans un moteur complet

```text
1. Événements (OS)       → entrées clavier/souris/manette
2. Mise à jour variable  → animations, caméra, UI           (dt réel)
3. Pas fixes (0..n)      → physique, gameplay déterministe   (PAS_FIXE)
4. Préparation du rendu  → extraire ce qu'il faut dessiner
5. Rendu GPU             → encoder, soumettre, présenter
6. Fin d'image           → statistiques, nettoyage, hot-reload
```

### Exercices

**18.1** — Dans `zentrix-platform`, ajoutez une méthode `on_fixed_update(&mut self, dt: f32)` au trait `Application` (avec une implémentation par défaut vide) et l'accumulateur dans `PlatformRunner`. Le pas fixe est configurable dans `WindowConfig` (renommez-le éventuellement `AppConfig`).

**18.2** — Dans la démo, faites rebondir une balle sous gravité dans `on_fixed_update`, et affichez-la avec interpolation. Vérifiez que la hauteur du rebond est identique avec la V-Sync (`Fifo`) et sans (`Immediate`).

**18.3** — Ajoutez une struct `FrameInfo { dt, temps_total, numero_image, fps }` passée à `on_update` (cela résout aussi la question de l'exercice 5 du guide).

---

## Module 19 — Le rendu avec wgpu

📖 Le [guide ZENTRIX, chapitres 4 à 6](guide-debutant/04-comprendre-le-gpu.md) puis [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) en entier.

### Notions

Ce module correspond aux étapes A à G de la [feuille de route](guide-debutant/07-feuille-de-route.md). Les notions sont détaillées dans le guide ; voici les points d'architecture à respecter en les implémentant.

#### Séparer les couches

```text
zentrix-rhi     : « comment parler au GPU »  → GpuContext, création de buffers/textures génériques
zentrix-render  : « quoi dessiner »          → Mesh, Material, Camera, passes, Renderer
le jeu          : « ce qui existe »          → entités, composants, logique
```

Le jeu ne voit **jamais** un `wgpu::Buffer`. Il manipule des **handles** (`MeshHandle`, `TextureHandle`) et soumet une liste de commandes de dessin.

#### Le cycle de vie des ressources GPU

| Ressource | Créée | Mise à jour | Fréquence |
|-----------|-------|-------------|-----------|
| Pipeline | au démarrage (ou au chargement d'un matériau) | jamais (ou hot-reload) | rare |
| Maillage statique | au chargement | jamais | rare |
| Texture | au chargement | rarement | rare |
| Uniform caméra | au démarrage | `queue.write_buffer` | chaque image |
| Buffer d'instances | au démarrage (taille max) | `queue.write_buffer` | chaque image |
| Depth buffer | au démarrage | recréé au redimensionnement | rare |

**Règle** : ne **jamais** créer de pipeline, de buffer ou de texture dans la boucle de rendu normale. Tout ce qui change par image passe par des buffers réutilisés.

#### Les étapes à réaliser (rappel)

| Étape | Résultat | Notions Rust mobilisées |
|-------|----------|------------------------|
| A. `Mesh` + index buffer | plusieurs maillages | structs, `Vec`, modules |
| B. Uniforms | animation dans le shader | `repr(C)`, `bytemuck`, alignement (module 13) |
| C. Caméra 3D | perspective | votre projet II, puis `glam` |
| D. Depth buffer | occlusion correcte | gestion du redimensionnement |
| E. Entrées | caméra libre | `HashSet<KeyCode>`, traits |
| F. Textures | images | crate `image`, erreurs (module 5) |
| G. Debug lines | grille, axes | buffers dynamiques, zéro allocation |

### Exercices

**19.1** — Réalisez les étapes A à D. Démo : une grille de 10×10 cubes en perspective, vue d'une caméra qui tourne autour.

**19.2** — Instanciez : dessinez 10 000 cubes en **un seul** `draw_indexed` avec un buffer d'instances (matrice modèle par instance). Mesurez le temps CPU de `render` avant/après (comparé à 10 000 draw calls).

**19.3** — Gérez la perte de surface et le redimensionnement du depth buffer ; testez en minimisant/restaurant la fenêtre et en la passant d'un écran à un autre.

---

## Module 20 — L'ECS (Entity Component System)

📖 Lecture : [Game Programming Patterns — Component](https://gameprogrammingpatterns.com/component.html), la documentation de [`hecs`](https://docs.rs/hecs) et le livre non officiel [Bevy Cheat Book — ECS](https://bevy-cheatbook.github.io/programming/ecs-intro.html).

### Notions

#### Pourquoi pas l'héritage ?

En POO classique : `Entite` → `Personnage` → `Ennemi` → `EnnemiVolant`... Puis on veut un ennemi volant **et** nageur, un objet qui a des PV mais ne bouge pas : la hiérarchie explose. Et Rust n'a pas d'héritage de toute façon.

L'**ECS** remplace l'héritage par la **composition de données** :

| Concept | Ce que c'est | Exemple |
|---------|--------------|---------|
| **Entité** | un simple identifiant (un nombre) | `Entity(42)` |
| **Composant** | une donnée **sans logique** attachée à une entité | `Position`, `Vitesse`, `Sante`, `MeshRenderer` |
| **Système** | une fonction qui traite toutes les entités ayant certains composants | `physique(Position, &Vitesse)` |

```text
            Position   Vitesse   Sante   MeshRenderer   Joueur
Entité 0       ✔          ✔        ✔          ✔            ✔      ← le joueur
Entité 1       ✔          ✔        ✔          ✔                   ← un ennemi
Entité 2       ✔                              ✔                   ← un décor
Entité 3       ✔          ✔                                       ← une particule invisible
```

Le système « physique » parcourt toutes les entités ayant `Position` **et** `Vitesse` : 0, 1 et 3. Ajouter un comportement = ajouter un composant. **Polyvalence maximale.**

Et pour la **performance** : les composants d'un même type sont stockés de façon **contiguë** (c'est du SoA, module 17), et les systèmes qui touchent des composants différents peuvent tourner **en parallèle** (module 15).

#### Un mini-ECS fait main

Le meilleur moyen de comprendre est d'en écrire un, simple (stockage « clairsemé » : un `Vec<Option<T>>` par type de composant) :

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity(u32);

#[derive(Default)]
pub struct World {
    prochaine: u32,
    // Un stockage par type de composant. `Box<dyn Any>` contient en réalité
    // un `Vec<Option<T>>` ; `TypeId` identifie le type T de façon unique.
    stockages: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    pub fn spawn(&mut self) -> Entity {
        let e = Entity(self.prochaine);
        self.prochaine += 1;
        e
    }

    fn stockage_mut<T: 'static>(&mut self) -> &mut Vec<Option<T>> {
        self.stockages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Vec::<Option<T>>::new()))
            .downcast_mut::<Vec<Option<T>>>()   // « je sais que c'est un Vec<Option<T>> »
            .expect("stockage de type incohérent")
    }

    pub fn insert<T: 'static>(&mut self, e: Entity, composant: T) {
        let s = self.stockage_mut::<T>();
        let i = e.0 as usize;
        if s.len() <= i {
            s.resize_with(i + 1, || None);
        }
        s[i] = Some(composant);
    }

    pub fn get<T: 'static>(&self, e: Entity) -> Option<&T> {
        self.stockages
            .get(&TypeId::of::<T>())?
            .downcast_ref::<Vec<Option<T>>>()?
            .get(e.0 as usize)?
            .as_ref()
    }

    /// Toutes les entités qui ont un composant T.
    pub fn iter<T: 'static>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.stockages
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<Vec<Option<T>>>())
            .into_iter()
            .flat_map(|v| v.iter().enumerate())
            .filter_map(|(i, c)| c.as_ref().map(|c| (Entity(i as u32), c)))
    }

    /// Sort temporairement un stockage du monde (pour le modifier pendant
    /// qu'on lit un autre stockage), voir le système `physique` ci-dessous.
    pub fn take<T: 'static>(&mut self) -> Vec<Option<T>> {
        std::mem::take(self.stockage_mut::<T>())
    }

    pub fn put_back<T: 'static>(&mut self, stockage: Vec<Option<T>>) {
        *self.stockage_mut::<T>() = stockage;
    }
}

// --- Composants : de simples structs ---
#[derive(Debug)]
struct Position { x: f32, y: f32 }
struct Vitesse { x: f32, y: f32 }

// --- Système : une simple fonction ---
fn physique(world: &mut World, dt: f32) {
    // On ne peut pas emprunter `world` en écriture (positions) ET en lecture
    // (vitesses) en même temps : on sort les positions, on travaille, on les remet.
    let mut positions = world.take::<Position>();
    for (e, v) in world.iter::<Vitesse>() {
        if let Some(Some(p)) = positions.get_mut(e.0 as usize) {
            p.x += v.x * dt;
            p.y += v.y * dt;
        }
    }
    world.put_back(positions);
}

fn main() {
    let mut world = World::default();
    let joueur = world.spawn();
    world.insert(joueur, Position { x: 0.0, y: 0.0 });
    world.insert(joueur, Vitesse { x: 1.0, y: 0.5 });

    let decor = world.spawn();
    world.insert(decor, Position { x: 5.0, y: 5.0 }); // pas de vitesse : immobile

    for _ in 0..60 {
        physique(&mut world, 1.0 / 60.0);
    }
    println!("{:?}", world.get::<Position>(joueur)); // ≈ (1.0, 0.5)
    println!("{:?}", world.get::<Position>(decor));  // (5.0, 5.0)
}
```

La gymnastique `take` / `put_back` illustre **le** problème difficile d'un ECS en Rust : donner à un système plusieurs accès simultanés (mutable et partagé) à des stockages différents du même monde. Les vrais ECS le résolvent avec des requêtes typées (`Query<(&mut Position, &Vitesse)>`) et un peu d'`unsafe` soigneusement encapsulé.

#### Clairsemé vs archétypes

| Stockage | Principe | Forces | Faiblesses |
|----------|----------|--------|------------|
| **Clairsemé** (*sparse set*) | un tableau par type de composant | ajout/retrait de composant très rapide | itérer sur plusieurs composants demande des recherches |
| **Archétypes** | les entités ayant **exactement** les mêmes composants sont rangées ensemble, en SoA | itération ultra-rapide et contiguë | ajouter/retirer un composant déplace l'entité dans une autre table |

`hecs` et `bevy_ecs` utilisent les archétypes (Bevy propose aussi le stockage clairsemé par composant).

#### Utiliser un vrai ECS

Pour ZENTRIX, ne réécrivez pas tout : écrivez un mini-ECS pour **comprendre**, puis utilisez **`hecs`** (minimal, facile à intégrer dans votre propre architecture) ou **`bevy_ecs`** (le plus complet : planificateur parallèle automatique, événements, ressources, mais plus imposant).

```rust
// Avec hecs : un tuple de composants = une entité
let mut world = hecs::World::new();
let joueur = world.spawn((Position { x: 0.0, y: 0.0 }, Vitesse { x: 1.0, y: 0.5 }));

for (pos, vel) in world.query_mut::<(&mut Position, &Vitesse)>() {
    pos.x += vel.x * dt;
    pos.y += vel.y * dt;
}
```

> ⚙️ Les API des ECS évoluent vite : vérifiez toujours la documentation de la version que vous utilisez sur docs.rs.

#### Le rendu et l'ECS : l'extraction

Le moteur de rendu ne doit **pas** dépendre de l'ECS (voir [moteurGraphique.md, partie 2](moteurGraphique.md)). À chaque image, un système d'**extraction** parcourt les entités ayant `Transform` + `MeshRenderer` et remplit une liste de commandes de dessin simple, que le `Renderer` consomme :

```rust
pub struct DrawCommand { pub mesh: MeshHandle, pub material: MaterialHandle, pub model: glam::Mat4 }

fn extraire(world: &hecs::World, sortie: &mut Vec<DrawCommand>) {
    sortie.clear(); // réutilise la mémoire de l'image précédente
    for (t, r) in world.query::<(&Transform, &MeshRenderer)>().iter() {
        sortie.push(DrawCommand { mesh: r.mesh, material: r.material, model: t.matrice() });
    }
}
```

### Exercices

**20.1** — Tapez et faites fonctionner le mini-ECS. Ajoutez `remove<T>(e)`, `despawn(e)` (retire tous les composants) et `iter_mut<T>()`.

**20.2** — Créez la crate `zentrix-scene` avec `hecs`, les composants `Transform` (position, rotation `Quat`, échelle), `MeshRenderer` et `Camera`, et le système d'extraction. La démo du module 19 devient pilotée par l'ECS.

**20.3** — Benchmark : 100 000 entités `Position` + `Vitesse` ; comparez votre mini-ECS, `hecs`, et un simple `Vec<(Position, Vitesse)>`. Expliquez les écarts.

**20.4** — Hiérarchie : ajoutez un composant `Parent(Entity)` et un système qui calcule la matrice monde de chaque entité (`monde = monde_du_parent × locale`). Indice : traitez les parents avant les enfants (tri par profondeur).

---

## Module 21 — Les assets et les handles

### Notions

#### Le problème

Un modèle 3D est utilisé par 500 entités. Il faut : le charger **une seule fois**, le partager, savoir quand il n'est plus utilisé, le recharger à chaud quand l'artiste le modifie, et ne **jamais** avoir de référence vers une ressource détruite.

Les références Rust (`&Mesh`) ne conviennent pas (durées de vie impossibles à exprimer sur toute une partie), `Rc` non plus (lent, pas `Send`, et ne permet pas le rechargement).

#### Les index générationnels

Un **handle** est un petit identifiant copiable : un index dans un tableau, **plus** un numéro de **génération** qui détecte les handles périmés.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Handle {
    index: u32,
    generation: u32,
}

struct Slot<T> {
    generation: u32,
    valeur: Option<T>,
}

pub struct Arena<T> {
    slots: Vec<Slot<T>>,
    libres: Vec<u32>, // index réutilisables
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { slots: Vec::new(), libres: Vec::new() }
    }

    pub fn insert(&mut self, valeur: T) -> Handle {
        if let Some(index) = self.libres.pop() {
            let slot = &mut self.slots[index as usize];
            slot.valeur = Some(valeur);
            Handle { index, generation: slot.generation }
        } else {
            self.slots.push(Slot { generation: 0, valeur: Some(valeur) });
            Handle { index: (self.slots.len() - 1) as u32, generation: 0 }
        }
    }

    pub fn remove(&mut self, h: Handle) -> Option<T> {
        let slot = self.slots.get_mut(h.index as usize)?;
        if slot.generation != h.generation {
            return None; // handle périmé
        }
        let valeur = slot.valeur.take()?;
        slot.generation = slot.generation.wrapping_add(1); // invalide les anciens handles
        self.libres.push(h.index);
        Some(valeur)
    }

    pub fn get(&self, h: Handle) -> Option<&T> {
        let slot = self.slots.get(h.index as usize)?;
        if slot.generation == h.generation { slot.valeur.as_ref() } else { None }
    }
}
```

```text
insert(A) → Handle{0, gen 0}
remove(Handle{0, gen 0})          slot 0 : génération passe à 1
insert(B) → Handle{0, gen 1}      le slot 0 est réutilisé
get(Handle{0, gen 0})  → None     l'ancien handle est détecté comme périmé ✔
```

Avantages : 8 octets, `Copy`, `Send`, aucune durée de vie, stockage contigu, détection des erreurs. **C'est la structure de base des moteurs Rust** (les `Entity` de `hecs`/`bevy_ecs` fonctionnent ainsi). La crate `slotmap` en fournit une version éprouvée.

Pour la sûreté des types, rendez le handle générique : `Handle<Mesh>` et `Handle<Texture>` ne doivent pas être confondus (`PhantomData<T>`).

#### Le gestionnaire d'assets

```text
jeu : assets.load::<Texture>("herbe.png")  ──▶  Handle<Texture> immédiatement (état : Chargement)
                                                    │
thread de chargement : lit le fichier, décode ──────┘ envoie le résultat par un canal (module 15)
                                                    │
début d'image : le gestionnaire reçoit, crée la texture GPU, état → Prêt
```

- **Chemin → handle** dans une `HashMap` : charger deux fois le même fichier renvoie le même handle.
- **Asynchrone** : le jeu ne se fige jamais pendant un chargement ; il affiche une texture par défaut en attendant.
- **Hot-reload** : la crate `notify` surveille le dossier `assets/` ; à chaque modification, le fichier est rechargé **dans le même slot** : tous les handles existants voient la nouvelle version. Idéal pour les shaders (reconstruire le pipeline) et les textures.

### Exercices

**21.1** — Tapez `Arena<T>`, rendez `Handle` générique avec `PhantomData`, ajoutez `get_mut`, `iter`, `len`, et écrivez les tests du scénario ci-dessus.

**21.2** — Créez `zentrix-assets` : `AssetServer` avec chargement d'images en arrière-plan (crate `image`), cache par chemin, état `Chargement / Prêt / Erreur`.

**21.3** — Hot-reload des shaders : surveillez `crates/zentrix-render/src/shaders/` avec `notify` en mode debug ; à chaque modification, rechargez le WGSL depuis le disque (au lieu de `include_str!`), validez-le, reconstruisez le pipeline. Une erreur de shader doit être **affichée dans les logs sans faire planter** le moteur (on garde l'ancien pipeline).

---

## Module 22 — Entrées, audio, physique

### Notions

Pour ces sous-systèmes, l'enjeu n'est pas de tout réécrire, mais de **bien intégrer** des crates spécialisées derrière vos propres abstractions.

#### Entrées (Phase 5)

Séparez trois niveaux :

```text
Matériel (winit)      : touche KeyW enfoncée, souris déplacée de (3, -1), bouton A de manette
       ↓
État (Input)          : quelles touches sont enfoncées / viennent d'être enfoncées cette image
       ↓
Actions (gameplay)    : « Avancer » = W ou Flèche haut ou stick gauche vers le haut
```

```rust
use std::collections::HashSet;

#[derive(Default)]
pub struct Input {
    enfoncees: HashSet<KeyCode>,
    juste_enfoncees: HashSet<KeyCode>,
    juste_relachees: HashSet<KeyCode>,
}

impl Input {
    pub fn appui(&mut self, k: KeyCode) {
        if self.enfoncees.insert(k) { self.juste_enfoncees.insert(k); }
    }
    pub fn relache(&mut self, k: KeyCode) {
        if self.enfoncees.remove(&k) { self.juste_relachees.insert(k); }
    }
    pub fn fin_image(&mut self) {
        self.juste_enfoncees.clear();
        self.juste_relachees.clear();
    }
    pub fn est_enfoncee(&self, k: KeyCode) -> bool { self.enfoncees.contains(&k) }
    pub fn vient_d_etre_enfoncee(&self, k: KeyCode) -> bool { self.juste_enfoncees.contains(&k) }
}
```

Le niveau « actions » (configurable dans un fichier) rend le moteur **polyvalent** : le même jeu fonctionne au clavier, à la manette, et avec des touches redéfinies. Manettes : crate `gilrs`.

#### Audio (Phase 8)

Crate recommandée : **`kira`** (conçue pour les jeux : mixage, pistes, transitions, son spatialisé). Encapsulez-la dans `zentrix-audio` avec des `Handle<Son>` gérés par votre système d'assets.

#### Physique (Phase 7)

Crate recommandée : **`rapier`** (`rapier2d` / `rapier3d`), écrite en Rust, déterministe, multithreadée. Intégration :

1. le monde physique vit dans une ressource ;
2. composant `CorpsRigide` = handle vers un corps rapier ;
3. à chaque **pas fixe** (module 18) : `pipeline.step(...)` ;
4. un système recopie les positions rapier dans les `Transform` de l'ECS.

> 📌 Écrire son propre moteur physique simple (AABB, gravité, rebonds) est un excellent exercice. Pour un vrai jeu, utilisez rapier.

#### Interface de debug

Crate **`egui`** (et `egui-wgpu`, `egui-winit`) : une interface en mode immédiat, dessinée par-dessus le rendu, parfaite pour afficher les FPS, les temps par système, l'inspecteur d'entités. C'est la base du futur **éditeur** (Phase 10).

### Exercices

**22.1** — Ajoutez `Input` à `zentrix-platform` et une caméra libre (ZQSD + souris capturée) dans la démo.

**22.2** — Intégrez `egui` : une fenêtre de debug avec les FPS, le temps CPU de l'update et du rendu, le nombre d'entités, et un curseur pour la vitesse de la caméra.

**22.3** — Intégrez `rapier3d` : des cubes tombent sur un sol et s'empilent. Espace en fait apparaître un nouveau.

---

## Module 23 — La polyvalence : plugins, sérialisation, scripts

### Notions

#### Un moteur modulaire : les plugins

Chaque sous-système s'enregistre lui-même auprès de l'application. Le jeu choisit ce qu'il utilise.

```rust
pub type Systeme = Box<dyn FnMut(&mut World, f32)>;

pub struct App {
    pub world: World,
    systemes: Vec<Systeme>,
}

pub trait Plugin {
    fn build(&self, app: &mut App);
}

impl App {
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    pub fn add_system(&mut self, s: impl FnMut(&mut World, f32) + 'static) -> &mut Self {
        self.systemes.push(Box::new(s));
        self
    }

    pub fn update(&mut self, dt: f32) {
        for s in &mut self.systemes {
            s(&mut self.world, dt);
        }
    }
}

// Un sous-système = un plugin
pub struct PhysiquePlugin;
impl Plugin for PhysiquePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(physique);
    }
}

// Le jeu compose son moteur
// app.add_plugin(PhysiquePlugin).add_plugin(AudioPlugin).add_plugin(MonJeuPlugin);
```

Ici, `dyn` est le bon choix (module 7) : quelques dizaines de systèmes appelés une fois par image, et le jeu doit pouvoir en ajouter de nouveaux.

#### Les *features* Cargo : ne compiler que ce qu'on utilise

```toml
# crates/zentrix/Cargo.toml (une crate « façade » qui ré-exporte le moteur)
[features]
default = ["render", "audio"]
render = ["dep:zentrix-render"]
audio = ["dep:zentrix-audio"]
physics-3d = ["dep:rapier3d"]
editor = ["dep:egui"]

[dependencies]
zentrix-render = { workspace = true, optional = true }
zentrix-audio = { workspace = true, optional = true }
```

```rust
#[cfg(feature = "audio")]
pub use zentrix_audio as audio;
```

Un serveur de jeu sans fenêtre : `default-features = false`. Polyvalence **et** binaire plus léger.

#### La sérialisation des scènes : `serde`

```toml
serde = { version = "1", features = ["derive"] }
ron = "0.12"
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct EntiteFichier {
    nom: String,
    position: [f32; 3],
    mesh: Option<String>,   // chemin de l'asset
}

#[derive(Serialize, Deserialize, Debug)]
struct SceneFichier {
    entites: Vec<EntiteFichier>,
}

fn sauver(scene: &SceneFichier) -> anyhow::Result<String> {
    Ok(ron::ser::to_string_pretty(scene, ron::ser::PrettyConfig::default())?)
}

fn charger(texte: &str) -> anyhow::Result<SceneFichier> {
    Ok(ron::from_str(texte)?)
}
```

Fichier `.ron` produit (format lisible, proche de Rust) :

```ron
(
    entites: [
        (nom: "joueur", position: (0.0, 1.0, 0.0), mesh: Some("models/hero.glb")),
        (nom: "lumiere", position: (5.0, 10.0, 2.0), mesh: None),
    ],
)
```

Les scènes, les configurations, les préfabriqués et les réglages de l'éditeur passent tous par `serde`. Le même `derive` fonctionne aussi avec JSON (`serde_json`), TOML, ou des formats binaires rapides (`bincode`, `postcard`) pour les fichiers de production.

#### Les scripts

Pour que des non-programmeurs (ou des moddeurs) écrivent du gameplay sans recompiler :

| Option | Crate | Profil |
|--------|-------|--------|
| Rhai | `rhai` | langage de script embarqué, syntaxe proche de Rust, 100 % Rust, simple à intégrer |
| Lua | `mlua` | standard de l'industrie, très rapide (LuaJIT) |
| WebAssembly | `wasmtime` | scripts dans n'importe quel langage, isolés et sûrs (mods) |

Commencez **sans** script : en Rust pur, le hot-reload d'assets suffit longtemps. Ajoutez-en quand un vrai besoin apparaît.

#### Multiplateforme

wgpu et winit ciblent Windows, Linux, macOS, Android, iOS **et le web** (WebAssembly + WebGPU/WebGL). Pour garder cette porte ouverte : pas d'appel bloquant sur le thread principal (pas de `thread::sleep`, pas de `pollster` sur le web), et des chemins de fichiers abstraits par le gestionnaire d'assets.

### Exercices

**23.1** — Créez la crate façade `zentrix` avec des features ; compilez la démo avec et sans `audio`.

**23.2** — Transformez chaque sous-système (rendu, physique, entrées, debug UI) en plugin.

**23.3** — Sauvegarde et chargement de scène en RON ; un raccourci `F5` sauvegarde, `F9` recharge.

**23.4** *(avancé)* — Intégrez `rhai` : un fichier `scripts/ennemi.rhai` définit une fonction `update(dt)` qui modifie la position d'une entité ; il est rechargé à chaud.

---

## Module 24 — La performance du rendu

📖 Lecture : *A trip through the Graphics Pipeline* (Fabian Giesen, blog, 2011 — toujours la meilleure introduction au fonctionnement interne d'un GPU), et les conférences GDC/SIGGRAPH sur les moteurs (*Destiny's Multithreaded Rendering*, *Rendering the Hellscape of Doom Eternal*, *GPU-Driven Rendering Pipelines* d'Ubisoft).

### Notions

#### Où part le temps ?

| Goulot | Symptôme | Remèdes |
|--------|----------|---------|
| **CPU** : trop de draw calls / changements d'état | le GPU attend ; le temps de `render` côté CPU est élevé | instanciation, regroupement, tri, multithreading |
| **GPU — sommets** | beaucoup de triangles | LOD, culling, meshlets |
| **GPU — pixels** (*fill rate*) | le temps dépend de la résolution | moins de surdessin, shaders plus simples, résolution dynamique |
| **Bande passante** | grosses textures, nombreux render targets | compression de textures (BC7, KTX2), mipmaps, formats compacts |

Mesurez d'abord : temps CPU avec Tracy/puffin, temps GPU avec RenderDoc ou les **requêtes de timestamp** wgpu (`Features::TIMESTAMP_QUERY`).

#### Les techniques, par ordre d'impact typique

1. **Ne pas dessiner l'invisible : le *frustum culling*.** Tester chaque objet (sa sphère ou sa boîte englobante) contre les 6 plans du champ de la caméra, avant de générer une commande de dessin.
   ```rust
   /// Un plan : normale (a, b, c) et distance d. Point p devant si a·x + b·y + c·z + d >= 0.
   fn sphere_visible(plans: &[glam::Vec4; 6], centre: glam::Vec3, rayon: f32) -> bool {
       plans.iter().all(|p| p.truncate().dot(centre) + p.w >= -rayon)
   }
   ```
   Les plans se calculent à partir de la matrice `view_projection`. Parallélisable avec `rayon` sur des milliers d'objets.
2. **Instanciation** : un draw call pour N objets identiques (module 19).
3. **Trier les commandes** : par pipeline, puis matériau, puis maillage, pour minimiser les changements d'état. Opaques de l'avant vers l'arrière (moins de surdessin grâce au depth test), transparents de l'arrière vers l'avant. Astuce : encoder tout ça dans une **clé de tri de 64 bits** et trier un `Vec<(u64, index)>`.
4. **Niveaux de détail (LOD)** : un maillage simplifié pour les objets lointains.
5. **Préparer les données en parallèle** : extraction, culling et tri sur plusieurs threads ; wgpu permet aussi d'encoder plusieurs `CommandEncoder` (ou `RenderBundle`) en parallèle.
6. **Réduire les transferts** : un grand buffer partagé pour les données par objet plutôt que de nombreux petits, mis à jour en un seul `write_buffer`.
7. **Rendu piloté par le GPU** (*GPU-driven*) : le culling se fait dans un **compute shader**, qui écrit directement les commandes de dessin dans un buffer (`draw_indexed_indirect`). Le CPU ne touche presque plus aux objets. C'est le socle des techniques de type Nanite (voir [nanites_et_lumen.md](nanites_et_lumen.md)).

#### Le *render graph*

Quand les passes se multiplient (ombres, G-buffer, éclairage, post-traitement, UI), on les décrit comme un **graphe** : chaque passe déclare les textures qu'elle lit et écrit. Le graphe en déduit l'ordre d'exécution, alloue et réutilise les textures intermédiaires, et supprime les passes inutiles (voir [moteurGraphique.md §1.4](moteurGraphique.md)). À introduire quand vous avez **au moins 4 ou 5 passes** réelles, pas avant.

### Exercices

**24.1** — Ajoutez les requêtes de timestamp GPU et affichez dans `egui` le temps de chaque passe.

**24.2** — Frustum culling : une scène de 50 000 cubes, dont la plupart hors champ. Mesurez le temps CPU et GPU avant/après, puis parallélisez le culling avec `rayon`.

**24.3** — Clé de tri 64 bits : (pipeline: 8 bits | matériau: 16 bits | profondeur: 24 bits | ...). Triez les commandes et comptez les changements de pipeline/matériau par image avant et après.

**24.4** *(avancé)* — Déplacez le frustum culling dans un compute shader qui remplit un buffer d'arguments indirects.

---

## Projet final — Une tranche verticale jouable

**Objectif** : prouver que ZENTRIX est un vrai moteur en livrant un **petit jeu complet** (Phase 6 de la feuille de route). Exemple : une arène 3D où le joueur ramasse des objets en évitant des ennemis.

### Cahier des charges

| Domaine | Exigence |
|---------|----------|
| Boucle | pas fixe pour le gameplay et la physique, rendu interpolé |
| Rendu | caméra 3D, maillages glTF texturés (crate `gltf`), éclairage simple (une lumière directionnelle, Lambert), ombres (bonus), instanciation, frustum culling |
| Scène | ECS (`hecs` ou `bevy_ecs`), scène chargée depuis un fichier RON |
| Assets | handles générationnels, chargement asynchrone, hot-reload des shaders et textures |
| Entrées | actions configurables, clavier et manette |
| Audio | musique + effets sonores (`kira`) |
| Physique | `rapier3d` |
| Outils | interface de debug `egui` : FPS, temps CPU/GPU par système, inspecteur d'entités |
| Qualité | `cargo clippy` sans avertissement, tests sur `core`/`assets`/`scene`, benchmarks des systèmes chauds |
| Performance | 60 FPS stables sur la machine de développement avec 10 000 objets visibles ; temps CPU d'image < 8 ms ; zéro allocation par image en régime établi |

### Livrables

- Le jeu, lancé par `cargo run --release -p zentrix-demo`.
- Un document `documentation/bilan-performance.md` : mesures, goulots identifiés, optimisations et leurs gains chiffrés.
- Une mise à jour de la documentation d'architecture.

### ✅ Checklist finale

- [ ] Je sais expliquer et justifier l'architecture en crates de ZENTRIX.
- [ ] J'ai écrit un mini-ECS et je comprends les compromis clairsemé / archétypes.
- [ ] J'utilise des handles générationnels plutôt que des références pour les ressources.
- [ ] La boucle de jeu est déterministe (pas fixe).
- [ ] Le moteur est modulaire (plugins, features) et piloté par les données (scènes RON).
- [ ] Chaque optimisation que j'ai faite est justifiée par une mesure.
- [ ] Un jeu complet tourne sur ZENTRIX. 🎉

---

# Annexes

## Annexe A — Calendrier type

| Semaines | Contenu | Jalon |
|----------|---------|-------|
| 1–2 | Modules 1–2 + Rustlings | Je comprends la propriété |
| 3–4 | Modules 3–5 | J'écris des programmes sans `unwrap()` |
| 5–6 | Module 6 + Projet I | Jeu de la vie testé |
| 7–9 | Modules 7–9 | J'écris des API génériques |
| 10–12 | Modules 10–12 + Projet II | Ma bibliothèque mathématique fonctionne |
| 13–15 | Modules 13–15 | Je mesure et je parallélise |
| 16–18 | Modules 16–17 + Projet III | 100 000 particules dans ZENTRIX |
| 19–22 | Modules 18–19 | Scène 3D avec caméra et textures |
| 23–26 | Modules 20–21 | ECS et assets |
| 27–30 | Modules 22–24 | Moteur complet et optimisé |
| 31+ | Projet final | Jeu jouable |

Si vous prenez du retard : c'est normal. La partie I et le module 2 en particulier demandent souvent deux fois le temps prévu. **Ne sautez pas la partie I pour « arriver au graphisme » plus vite** : tout le reste repose dessus.

## Annexe B — Les erreurs du compilateur les plus fréquentes

| Code | Message (résumé) | Cause habituelle | Correction habituelle |
|------|------------------|------------------|-----------------------|
| E0382 | *use of moved value* | valeur déplacée puis réutilisée | emprunter (`&x`), cloner, ou réorganiser |
| E0499 | *cannot borrow as mutable more than once* | deux `&mut` en même temps | réduire la portée, `split_at_mut`, découper la struct |
| E0502 | *cannot borrow as mutable because it is also borrowed as immutable* | lecture et écriture simultanées | finir d'utiliser la lecture avant d'écrire, copier la valeur lue |
| E0505 | *cannot move out because it is borrowed* | déplacer une valeur encore empruntée | terminer l'emprunt avant |
| E0106 | *missing lifetime specifier* | référence renvoyée sans origine claire | annoter `'a`, ou renvoyer une valeur possédée |
| E0597 | *does not live long enough* | une référence survit à sa valeur | allonger la durée de vie de la valeur, ou posséder |
| E0277 | *the trait bound is not satisfied* | un type n'implémente pas le trait requis | `#[derive(...)]`, `impl`, ou ajouter la contrainte |
| E0308 | *mismatched types* | types différents (souvent `;` en trop en fin de fonction) | conversion `as`/`into`, retirer le `;` |
| E0599 | *no method named ... found* | trait non importé ou mauvais type | `use le::Trait;`, vérifier le type |

`rustc --explain E0382` donne une explication complète avec exemples.

## Annexe C — Idiomes à adopter, pièges à éviter

**À faire**

- Paramètres en `&str` et `&[T]` plutôt que `&String` et `&Vec<T>`.
- `impl Default` et `new()` cohérents ; `#[derive(Debug)]` partout.
- Renvoyer des `Result`, propager avec `?`, ajouter du contexte.
- Préférer les itérateurs aux boucles indexées.
- Posséder les données dans les structs à longue durée de vie ; emprunter dans les paramètres.
- Index / handles plutôt que pointeurs entre objets.
- Réutiliser les buffers (`clear()` plutôt que `Vec::new()`) dans la boucle de jeu.
- `cargo clippy` et `cargo fmt` avant chaque commit.

**À éviter**

- `.clone()` partout pour « faire taire » le borrow checker : c'est souvent le signe d'une conception à revoir (mais un `clone` occasionnel et réfléchi est parfaitement acceptable).
- `Rc<RefCell<T>>` comme réflexe de conception.
- `unwrap()` dans le code du moteur.
- Les hiérarchies de traits qui imitent l'héritage.
- Les abstractions « au cas où » : on généralise au **deuxième** cas d'usage réel, pas avant.
- Optimiser sans mesurer.

## Annexe D — Bibliothèque de référence

**Rust**

- [Le livre Rust (fr)](https://jimskapt.github.io/rust-book-fr/) · [Rust by Example](https://doc.rust-lang.org/rust-by-example/) · [Rustlings](https://github.com/rust-lang/rustlings)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/) · [Rust Atomics and Locks](https://marabos.nl/atomics/) · [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) — pour concevoir des API publiques propres
- *Rust for Rustaceans* (Jon Gjengset, livre payant) — le meilleur livre de niveau intermédiaire/avancé

**Graphisme**

- [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) · [LearnOpenGL](https://learnopengl.com/) (concepts) · [Scratchapixel](https://www.scratchapixel.com/)
- [WebGPU Fundamentals](https://webgpufundamentals.org/) — mêmes concepts que wgpu
- *Real-Time Rendering* (Akenine-Möller et al.) — la bible, à consulter par chapitre
- [RenderDoc](https://renderdoc.org/) — débogueur GPU indispensable

**Architecture de moteurs**

- [Game Programming Patterns](https://gameprogrammingpatterns.com/) (gratuit en ligne)
- *Game Engine Architecture* (Jason Gregory) — la référence complète
- [Le code source de Bevy](https://github.com/bevyengine/bevy) — un moteur Rust moderne à lire module par module
- [Data-Oriented Design](https://www.dataorienteddesign.com/dodbook/) (gratuit en ligne)

**Communauté**

- [Forum des utilisateurs Rust](https://users.rust-lang.org/) · [Are We Game Yet?](https://arewegameyet.rs/) (catalogue des crates de jeu)

---

⬅️ **Retour** : [Index de la documentation](README.md) · [Guide du débutant ZENTRIX](guide-debutant/01-installer-et-lancer.md)
