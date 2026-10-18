# Chapitre 4 — Comprendre le GPU et wgpu

**Objectif** : comprendre ce qui se passe *réellement* quand ZENTRIX affiche un triangle, sans prérequis en programmation graphique.

---

## 4.1 CPU et GPU : deux processeurs très différents

| | CPU (processeur) | GPU (carte graphique) |
|---|---|---|
| Nombre de cœurs | 4 à 32, très puissants | Des milliers, simples |
| Spécialité | Tâches variées, logique complexe, décisions | **La même opération sur énormément de données en parallèle** |
| Analogie | Quelques professeurs d'université | Une armée d'élèves qui font chacun une addition |

Calculer la couleur de 2 millions de pixels (une image 1920×1080), 60 fois par seconde, c'est exactement le travail pour lequel le GPU est conçu.

**Conséquence clé** : le CPU et le GPU ont chacun **leur mémoire** et travaillent **en parallèle**. Le CPU ne dessine pas : il **prépare des données** (sommets, textures), les **envoie** au GPU, puis lui **envoie une liste de commandes**. Le GPU exécute ensuite tout ça de son côté.

```text
         CPU (notre code Rust)                        GPU
  ┌───────────────────────────────┐          ┌──────────────────────┐
  │ 1. créer buffers & pipelines  │ ───────▶ │ mémoire vidéo (VRAM) │
  │    (une fois au démarrage)    │          │                      │
  │                               │          │                      │
  │ 2. chaque image :             │          │                      │
  │    enregistrer des commandes  │          │                      │
  │    puis submit()  ────────────┼────────▶ │ exécute les commandes│
  │                               │          │ → pixels dans l'image│
  │ 3. present()  ────────────────┼────────▶ │ → affichage écran    │
  └───────────────────────────────┘          └──────────────────────┘
```

---

## 4.2 Les API graphiques et le rôle de wgpu

Pour parler au GPU, on passe par une **API graphique** fournie par le pilote :

| API | Plateformes |
|-----|-------------|
| **Vulkan** | Windows, Linux, Android |
| **DirectX 12** | Windows, Xbox |
| **Metal** | macOS, iOS |
| **OpenGL** | Partout (ancienne) |
| **WebGPU** | Navigateurs web |

Écrire un moteur pour chacune serait énorme. **wgpu** est une bibliothèque Rust qui offre **une seule API** (calquée sur le standard WebGPU) et la traduit vers Vulkan, DirectX 12, Metal ou OpenGL selon la machine.

Sur la machine de développement, les logs montrent : `GPU sélectionné : Intel(R) Iris(R) Xe Graphics (IntegratedGpu, API Vulkan)`. Le même code tournera sous DirectX 12 ou Metal sans changement.

> 📌 [StructureProjet.md](../StructureProjet.md) évoque aussi `ash` (Vulkan « brut »). C'est beaucoup plus bas niveau et verbeux : ~1000 lignes pour un triangle. wgpu est le bon choix pour apprendre et pour la majorité des moteurs. La crate `zentrix-rhi` isole wgpu : si un jour on veut passer à `ash`, seule cette crate change.

---

## 4.3 Les objets wgpu, étape par étape

Voici les objets créés dans [`GpuContext::new`](../../crates/zentrix-rhi/src/context.rs), dans l'ordre :

```text
 Instance ──▶ Surface (liée à la fenêtre)
    │
    └──▶ Adapter (un GPU physique) ──▶ Device + Queue
                                          │
                     SurfaceConfiguration ┘ (taille, format, V-Sync)
```

### `Instance` — l'annuaire

Le point d'entrée. Il connaît les API disponibles (Vulkan, DX12...) et les GPU installés.

### `Surface` — la toile

Le lien entre le GPU et **notre fenêtre**. C'est là que les images finales sont affichées.

### `Adapter` — le GPU choisi

Une carte graphique précise. On demande `PowerPreference::HighPerformance` pour préférer un GPU dédié à un GPU intégré, et `compatible_surface` pour qu'il sache afficher dans notre fenêtre.

### `Device` et `Queue` — la connexion

- Le **`Device`** sert à **créer** des ressources : buffers, textures, shaders, pipelines.
- La **`Queue`** sert à **envoyer** du travail : listes de commandes, copies de données, présentation à l'écran.

### `SurfaceConfiguration` — le format de la toile

| Champ | Valeur dans ZENTRIX | Explication |
|-------|---------------------|-------------|
| `format` | un format **sRGB** (ex. `Bgra8UnormSrgb`) | 8 bits par canal, ordre Bleu-Vert-Rouge-Alpha, conversion sRGB automatique |
| `width`, `height` | taille de la fenêtre | à mettre à jour à chaque redimensionnement |
| `present_mode` | `Fifo` | **V-Sync** : on attend le rafraîchissement de l'écran (60 Hz → 60 FPS) |
| `desired_maximum_frame_latency` | 2 | le CPU peut avoir 2 images d'avance sur le GPU |

### La « swapchain »

Derrière la surface, il y a plusieurs images qui tournent (*swap chain*) : pendant que l'écran affiche l'image A, on dessine dans l'image B. Puis on échange. Dans le code :

```rust
let frame = gpu.acquire_frame()?;  // « donne-moi une image libre »
// ... dessiner dans frame.view ...
gpu.present(frame);                // « affiche-la »
```

---

## 4.4 Le pipeline de rendu

Le **pipeline** est la chaîne de traitement qui transforme des **sommets** en **pixels** :

```text
  Vertex buffer          ┌───────────────┐      ┌────────────────┐      ┌─────────────────┐
  (3 sommets) ─────────▶ │ VERTEX SHADER │ ───▶ │ RASTÉRISATION  │ ───▶ │ FRAGMENT SHADER │ ───▶ image
  position + couleur     │ 1× par sommet │      │ (fixe, GPU)    │      │ 1× par pixel    │
                         │ « où ? »      │      │ quels pixels ? │      │ « quelle couleur ? »
                         └───────────────┘      └────────────────┘      └─────────────────┘
          programmable ▲                               fixe                     ▲ programmable
```

1. **Vertex shader** (programmable) : exécuté une fois **par sommet**. Il calcule la position du sommet à l'écran. Plus tard, c'est ici qu'on appliquera la caméra.
2. **Rastérisation** (fixe, faite par le matériel) : détermine quels **pixels** sont couverts par chaque triangle, et **interpole** les valeurs des sommets. C'est ce qui produit le dégradé : un pixel au centre du triangle reçoit un mélange du rouge, du vert et du bleu.
3. **Fragment shader** (programmable) : exécuté une fois **par pixel** couvert. Il renvoie la couleur finale.

En wgpu, tout cela est figé dans un objet `RenderPipeline`, créé dans [pipeline.rs](../../crates/zentrix-render/src/pipeline.rs). Le créer coûte cher (le pilote compile les shaders) : on le fait **une seule fois**.

---

## 4.5 Les shaders et WGSL

Un **shader** est un petit programme exécuté par le GPU. ZENTRIX les écrit en **WGSL** (*WebGPU Shading Language*), dont la syntaxe rappelle Rust.

Notre shader complet : [triangle.wgsl](../../crates/zentrix-render/src/shaders/triangle.wgsl).

```wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
```

| Élément WGSL | Signification |
|--------------|---------------|
| `vec3<f32>` | Vecteur de 3 flottants (x, y, z) ou (r, g, b) |
| `@location(n)` | Numéro d'un « tuyau » de données. Doit correspondre côté Rust (`Vertex::ATTRIBUTES`) |
| `@builtin(position)` | Valeur spéciale exigée : la position du sommet en *clip space* |
| `@vertex`, `@fragment` | Marquent les points d'entrée |
| `var` / `let` | Variable modifiable / constante |

⚠️ **Le lien Rust ↔ WGSL est fragile** : si la struct `Vertex` en Rust, le `VertexBufferLayout` et la struct `VertexInput` en WGSL ne correspondent pas exactement, le pipeline échoue (heureusement avec un message clair de wgpu). Le test `taille_du_vertex` dans [vertex.rs](../../crates/zentrix-render/src/vertex.rs) aide à détecter un changement accidentel.

---

## 4.6 Les buffers : envoyer des données au GPU

Un **buffer** est un bloc de mémoire **sur le GPU**. Pour notre triangle :

```rust
// crates/zentrix-render/src/renderer.rs
const TRIANGLE: [Vertex; 3] = [
    Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] }, // rouge
    Vertex { position: [ 0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] }, // vert
    Vertex { position: [ 0.0,  0.5, 0.0], color: [0.0, 0.0, 1.0] }, // bleu
];

let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
    contents: bytemuck::cast_slice(&TRIANGLE), // nos structs vues comme des octets
    usage: wgpu::BufferUsages::VERTEX,
    // ...
});
```

Le GPU ne connaît pas les types Rust : il ne voit que des **octets**. Il faut donc :

1. que la struct ait une disposition mémoire **prévisible** → `#[repr(C)]` ;
2. pouvoir la voir comme des octets **sans `unsafe`** → `bytemuck::Pod` + `cast_slice` ;
3. **décrire** au GPU comment lire ces octets → `Vertex::layout()` (24 octets par sommet, position à l'octet 0, couleur à l'octet 12).

Types de buffers que vous rencontrerez :

| Usage | Contenu | État dans ZENTRIX |
|-------|---------|-------------------|
| `VERTEX` | Les sommets | ✅ utilisé |
| `INDEX` | Les indices (réutiliser des sommets entre triangles) | exercice 4 |
| `UNIFORM` | Petites données constantes pour tout un dessin (matrice de caméra, temps) | prévu |
| `STORAGE` | Grandes données lisibles/écrivables par les shaders | plus tard |

---

## 4.7 Les coordonnées à l'écran

Sans caméra, les positions sont en **coordonnées normalisées** (*NDC*) :

```text
            y = +1
              ▲
              │
 x = -1 ◀─────┼─────▶ x = +1
              │
              ▼
            y = -1
```

- Le centre de la fenêtre est `(0, 0)`, quelle que soit sa taille.
- Conséquence : si la fenêtre n'est pas carrée, le triangle est **étiré**. Une matrice de projection (caméra) corrigera ça plus tard.

### L'ordre des sommets compte (culling)

Le pipeline utilise `front_face: Ccw` et `cull_mode: Some(Face::Back)` : seuls les triangles dont les sommets apparaissent dans le sens **inverse des aiguilles d'une montre** sont dessinés. En 3D, cela évite de dessiner l'arrière des objets (invisible). Inversez deux sommets de `TRIANGLE` : il disparaît !

---

## 4.8 Une image, du début à la fin

C'est le contenu de [`Renderer::render`](../../crates/zentrix-render/src/renderer.rs) :

```text
1. acquire_frame()            → récupérer une image libre de la swapchain
2. create_command_encoder()   → ouvrir un « carnet de commandes »
3. begin_render_pass()        → commencer une passe :
     LoadOp::Clear(couleur)       • effacer l'image avec la couleur de fond
     set_pipeline(...)            • choisir la recette de dessin
     set_vertex_buffer(...)       • brancher les sommets
     draw(0..3, 0..1)             • dessiner 3 sommets, 1 instance
   (fin de la passe)
4. queue.submit(...)          → envoyer le carnet au GPU (c'est là qu'il travaille)
5. present(frame)             → afficher l'image
```

> 💡 Les étapes 2 et 3 n'exécutent **rien** : elles *enregistrent*. C'est ce qui rend les API modernes rapides : le pilote reçoit tout le travail d'un coup et peut l'optimiser.

---

## 4.9 Couleurs : pourquoi sRGB ?

Nos yeux sont plus sensibles aux variations dans les tons sombres. Les écrans utilisent donc un espace de couleur non linéaire, **sRGB**. Les calculs d'éclairage, eux, doivent se faire en espace **linéaire** pour être corrects.

En choisissant un format de surface **sRGB**, on laisse le GPU convertir automatiquement : on calcule en linéaire, il convertit à l'écriture. C'est pourquoi `[0.5, 0.5, 0.5]` apparaît plus clair qu'un « gris 50 % » dans un logiciel de dessin. C'est normal et correct.

---

## Pour aller plus loin

- [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) — **le** tutoriel de référence, en anglais, qui suit à peu près le même chemin que ZENTRIX.
- [Exemples officiels de wgpu](https://github.com/gfx-rs/wgpu/tree/trunk/examples)
- [Spécification WGSL](https://www.w3.org/TR/WGSL/) (référence, à consulter au besoin)
- [LearnOpenGL](https://learnopengl.com/) (en anglais) — les concepts (caméra, éclairage...) sont les mêmes quelle que soit l'API.

---

➡️ **Chapitre suivant** : [Visite guidée du code](05-visite-du-code.md)
