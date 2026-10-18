# Chapitre 6 — Exercices progressifs

**Objectif** : apprendre en modifiant le moteur. Les exercices vont du plus simple (changer une valeur) au plus ambitieux (envoyer des données au shader).

**Méthode conseillée** :

1. Créez une branche git pour expérimenter sans risque : `git switch -c exercices`.
2. Essayez **seul** d'abord, avec l'indice.
3. Lancez `cargo check` souvent : c'est rapide et le compilateur vous guide.
4. Ne regardez la solution (dépliable) qu'ensuite.

| # | Exercice | Niveau | Notions |
|---|----------|--------|---------|
| 1 | Changer les couleurs de fond | ⭐ | constantes, `Color` |
| 2 | Jouer avec le triangle | ⭐ | tableaux, coordonnées, culling |
| 3 | Nouvelles touches clavier | ⭐⭐ | `match`, état de l'application |
| 4 | Un carré avec un index buffer | ⭐⭐ | buffers, `draw_indexed` |
| 5 | Afficher les FPS dans le titre | ⭐⭐ | delta time, `format!`, `Arc<Window>` |
| 6 | `Color::from_rgb8` piloté par les tests | ⭐⭐ | tests, conversions numériques |
| 7 | Faire tourner le triangle (uniform buffer) | ⭐⭐⭐ | uniforms, bind groups, WGSL |

---

## Exercice 1 ⭐ — Changer les couleurs de fond

**But** : le fond doit osciller entre du vert foncé et du bleu, deux fois plus vite.

**Indice** : tout se passe dans `on_update` de [main.rs](../../crates/zentrix-app/src/main.rs). La vitesse est le `0.8` dans `self.time * 0.8`.

<details>
<summary>Solution</summary>

```rust
let t = (self.time * 1.6).sin() * 0.5 + 0.5;
g.renderer.clear_color = Color::rgb(0.0, 0.15, 0.05).lerp(Color::rgb(0.0, 0.1, 0.4), t);
```

**Bonus** : ajoutez une constante `Color::ZENTRIX_FOREST` dans [color.rs](../../crates/zentrix-core/src/color.rs) et utilisez-la.

</details>

---

## Exercice 2 ⭐ — Jouer avec le triangle

**But** :

1. Rendre le triangle tout jaune.
2. Le rendre plus grand (il doit presque toucher les bords).
3. Échanger deux sommets et observer ce qui se passe. Pourquoi ?

**Indice** : `const TRIANGLE` dans [renderer.rs](../../crates/zentrix-render/src/renderer.rs). Relisez [§4.7 du chapitre 4](04-comprendre-le-gpu.md#47-les-coordonnées-à-lécran).

<details>
<summary>Solution</summary>

```rust
const TRIANGLE: [Vertex; 3] = [
    Vertex { position: [-0.9, -0.9, 0.0], color: [1.0, 1.0, 0.0] },
    Vertex { position: [0.9, -0.9, 0.0], color: [1.0, 1.0, 0.0] },
    Vertex { position: [0.0, 0.9, 0.0], color: [1.0, 1.0, 0.0] },
];
```

3\. Le triangle **disparaît** : ses sommets sont maintenant dans le sens des aiguilles d'une montre, il est donc considéré comme « vu de dos » et éliminé par `cull_mode: Some(wgpu::Face::Back)` dans [pipeline.rs](../../crates/zentrix-render/src/pipeline.rs). Mettez `cull_mode: None` pour le voir réapparaître.

</details>

---

## Exercice 3 ⭐⭐ — Nouvelles touches clavier

**But** :

- `R` remet l'animation à zéro (`time = 0`).
- `Flèche haut` / `Flèche bas` accélèrent / ralentissent l'animation (×2 / ÷2).

**Indices** :

- Ajoutez un champ `speed: f32` à `ZentrixApp` (initialisé à `1.0` dans `new`).
- Les noms des touches sont dans l'enum `KeyCode` : `KeyCode::KeyR`, `KeyCode::ArrowUp`, `KeyCode::ArrowDown`.
- Utilisez `speed` dans `on_update`.

<details>
<summary>Solution</summary>

```rust
// Dans la struct :
speed: f32,

// Dans ZentrixApp::new :
speed: 1.0,

// Dans on_key, à l'intérieur du match :
KeyCode::KeyR => self.time = 0.0,
KeyCode::ArrowUp => self.speed *= 2.0,
KeyCode::ArrowDown => self.speed /= 2.0,

// Dans on_update :
if !self.paused {
    self.time += dt * self.speed;
}
```

**Remarquez** : si vous oubliez d'initialiser `speed` dans `new`, le compilateur refuse de compiler (`missing field speed`). En Rust, une struct est toujours entièrement initialisée.

</details>

---

## Exercice 4 ⭐⭐ — Un carré avec un index buffer

**But** : dessiner un **carré** au lieu du triangle.

Un carré, ce sont **deux triangles**. Avec 6 sommets, deux d'entre eux seraient en double. Un **index buffer** évite ça : on donne 4 sommets uniques, puis une liste d'indices qui dit comment les relier.

```text
 3 ─────── 2        triangle 1 : 0, 1, 2
 │       ╱ │        triangle 2 : 0, 2, 3
 │     ╱   │        (sens anti-horaire pour les deux !)
 │   ╱     │
 0 ─────── 1
```

**Indices** :

1. Remplacez `TRIANGLE` par `QUAD: [Vertex; 4]` et ajoutez `const QUAD_INDICES: [u16; 6]`.
2. Créez un deuxième buffer avec `usage: wgpu::BufferUsages::INDEX`.
3. Dans la passe : `set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16)`, puis `draw_indexed(0..6, 0, 0..1)` au lieu de `draw`.

<details>
<summary>Solution</summary>

```rust
const QUAD: [Vertex; 4] = [
    Vertex { position: [-0.5, -0.5, 0.0], color: [1.0, 0.0, 0.0] }, // 0 bas-gauche
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },  // 1 bas-droite
    Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 0.0, 1.0] },   // 2 haut-droite
    Vertex { position: [-0.5, 0.5, 0.0], color: [1.0, 1.0, 0.0] },  // 3 haut-gauche
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

pub struct Renderer {
    pub clear_color: Color,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

// Dans Renderer::new :
let vertex_buffer = gpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Quad Vertex Buffer"),
    contents: bytemuck::cast_slice(&QUAD),
    usage: wgpu::BufferUsages::VERTEX,
});
let index_buffer = gpu.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Quad Index Buffer"),
    contents: bytemuck::cast_slice(&QUAD_INDICES),
    usage: wgpu::BufferUsages::INDEX,
});
Self {
    clear_color: Color::default(),
    pipeline,
    vertex_buffer,
    index_buffer,
    index_count: QUAD_INDICES.len() as u32,
}

// Dans render, à la place de `draw` :
render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
render_pass.draw_indexed(0..self.index_count, 0, 0..1);
```

`u16` permet jusqu'à 65 536 sommets par maillage ; au-delà, on utilise `u32` et `IndexFormat::Uint32`.

**Pour aller plus loin** : créez une struct `Mesh { vertex_buffer, index_buffer, index_count }` dans un nouveau fichier `crates/zentrix-render/src/mesh.rs`, avec une fonction `Mesh::new(device, &[Vertex], &[u16])`. C'est la première brique du futur système de maillages.

</details>

---

## Exercice 5 ⭐⭐ — Afficher les FPS dans le titre

**But** : le titre de la fenêtre affiche `ZENTRIX — 60 FPS`, mis à jour une fois par seconde.

**Indices** :

- `on_update` reçoit `dt` : comptez les images et additionnez les `dt` dans deux nouveaux champs.
- Quand la somme dépasse `1.0` seconde : FPS = images / somme. Puis remettez à zéro.
- La fenêtre est accessible via `g.window` (c'est un `Arc<Window>`) : `g.window.set_title(...)`.

<details>
<summary>Solution</summary>

```rust
// Champs :
fps_frames: u32,
fps_time: f32,

// Dans new : fps_frames: 0, fps_time: 0.0,

// Dans on_update :
self.fps_frames += 1;
self.fps_time += dt;
if self.fps_time >= 1.0 {
    let fps = self.fps_frames as f32 / self.fps_time;
    if let Some(g) = &self.graphics {
        g.window.set_title(&format!("ZENTRIX — {fps:.0} FPS"));
    }
    self.fps_frames = 0;
    self.fps_time = 0.0;
}
```

**Réflexion** : `zentrix_core::FrameTimer` calcule déjà les FPS dans la plateforme. Comment pourriez-vous les transmettre à l'application sans dupliquer le calcul ? (Piste : ajouter un paramètre à `on_update`, ou passer une struct `FrameInfo { dt, fps, frame }`.)

</details>

---

## Exercice 6 ⭐⭐ — `Color::from_rgb8`, piloté par les tests

**But** : pouvoir écrire `Color::from_rgb8(255, 128, 0)` avec des valeurs 0–255, comme dans un logiciel de dessin.

On pratique le **TDD** (*Test-Driven Development*) : on écrit d'abord le test, on le voit échouer, puis on écrit le code.

1. Dans le module `tests` de [color.rs](../../crates/zentrix-core/src/color.rs), ajoutez :
   ```rust
   #[test]
   fn from_rgb8_convertit_en_0_1() {
       assert_eq!(Color::from_rgb8(255, 0, 0), Color::RED);
       assert_eq!(Color::from_rgb8(0, 0, 0), Color::BLACK);
   }
   ```
2. `cargo test -p zentrix-core` → erreur de compilation (la fonction n'existe pas). C'est normal !
3. Écrivez la fonction.

**Indice** : `u8` est un entier 0–255. Divisez par `255.0` après une conversion `as f32`.

<details>
<summary>Solution</summary>

```rust
/// Crée une couleur opaque à partir de composantes 0–255.
pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
    Self::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}
```

**Bonus** : `from_hex(0xFF8000)` qui prend un `u32`. Indice : `((hex >> 16) & 0xFF) as u8` extrait le rouge.

</details>

---

## Exercice 7 ⭐⭐⭐ — Faire tourner le triangle (uniform buffer)

**But** : le triangle tourne sur lui-même. Le CPU envoie le temps au shader à chaque image, et c'est le **vertex shader** qui calcule la rotation.

C'est l'exercice le plus important : le même mécanisme servira à envoyer la **matrice de caméra**.

### Les notions nouvelles

| Notion | Rôle |
|--------|------|
| **Uniform buffer** | Petit buffer dont la valeur est la même pour tous les sommets/pixels d'un dessin |
| **Bind group layout** | Décrit *quelles* ressources le shader attend (« un uniform au binding 0, lu par le vertex shader ») |
| **Bind group** | Le lot de ressources *concrètes* branchées sur ce layout (« ce buffer-là ») |
| `queue.write_buffer` | Copie des octets du CPU vers un buffer GPU |
| `@group(0) @binding(0)` | Côté WGSL : où trouver la ressource |

### Étapes

1. **Shader** : déclarez l'uniform et utilisez-le pour tourner les positions.
2. **Rust** : une struct `Globals` (`#[repr(C)]`, `Pod`) contenant le temps.
3. **Pipeline** : créez le bind group layout et passez-le au pipeline layout.
4. **Renderer** : créez le buffer uniform et le bind group ; ajoutez une méthode `set_time`.
5. **Rendu** : `set_bind_group(0, &self.bind_group, &[])` avant `draw`.
6. **Application** : appelez `set_time` dans `on_update`.

⚠️ **Piège d'alignement** : les uniforms doivent avoir une taille multiple de **16 octets**. Un `f32` seul fait 4 octets : il faut ajouter 12 octets de remplissage (*padding*).

<details>
<summary>Solution</summary>

**1. Shader** — `triangle.wgsl`, ajouter en haut et modifier `vs_main` :

```wgsl
struct Globals {
    time: f32,
    // 12 octets de remplissage côté Rust : WGSL les ajoute automatiquement ici.
};

@group(0) @binding(0)
var<uniform> globals: Globals;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let c = cos(globals.time);
    let s = sin(globals.time);
    // Rotation 2D autour de l'origine.
    let p = vec2<f32>(
        in.position.x * c - in.position.y * s,
        in.position.x * s + in.position.y * c,
    );
    var out: VertexOutput;
    out.clip_position = vec4<f32>(p, in.position.z, 1.0);
    out.color = in.color;
    return out;
}
```

**2. Struct Rust** — dans `renderer.rs` :

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    time: f32,
    _padding: [f32; 3], // pour atteindre 16 octets
}
```

**3. Pipeline** — `pipeline.rs`, la fonction reçoit le layout en paramètre :

```rust
pub(crate) fn create_triangle_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    globals_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    // ...
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Triangle Pipeline Layout"),
        bind_group_layouts: &[Some(globals_layout)], // le groupe 0
        immediate_size: 0,
    });
    // ... le reste ne change pas
}
```

**4. Renderer** — nouveaux champs et création dans `new` :

```rust
pub struct Renderer {
    pub clear_color: Color,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    globals_buffer: wgpu::Buffer,
    globals_bind_group: wgpu::BindGroup,
}

// Dans Renderer::new, AVANT la création du pipeline :
let device = gpu.device();

let globals_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("Globals Layout"),
    entries: &[wgpu::BindGroupLayoutEntry {
        binding: 0,                                 // @binding(0)
        visibility: wgpu::ShaderStages::VERTEX,     // lu par le vertex shader
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }],
});

let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Globals Buffer"),
    contents: bytemuck::bytes_of(&Globals { time: 0.0, _padding: [0.0; 3] }),
    // UNIFORM : lisible comme uniform ; COPY_DST : on pourra écrire dedans.
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
});

let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("Globals Bind Group"),
    layout: &globals_layout,
    entries: &[wgpu::BindGroupEntry {
        binding: 0,
        resource: globals_buffer.as_entire_binding(),
    }],
});

let pipeline = create_triangle_pipeline(device, gpu.surface_format(), &globals_layout);
```

Et une nouvelle méthode :

```rust
/// Envoie le temps au GPU (à appeler une fois par image).
pub fn set_time(&self, gpu: &GpuContext, time: f32) {
    let globals = Globals { time, _padding: [0.0; 3] };
    gpu.queue().write_buffer(&self.globals_buffer, 0, bytemuck::bytes_of(&globals));
}
```

**5. Rendu** — dans `render`, avant `draw` :

```rust
render_pass.set_pipeline(&self.pipeline);
render_pass.set_bind_group(0, &self.globals_bind_group, &[]); // @group(0)
```

**6. Application** — dans `on_update` de `main.rs` :

```rust
if let Some(g) = &mut self.graphics {
    g.renderer.set_time(&g.gpu, self.time);
    // ...
}
```

Le triangle tourne, et `Espace` met aussi la rotation en pause, puisque `time` n'avance plus.

**Pour aller plus loin** : remplacez `time` par une matrice `glam::Mat4` (`[[f32; 4]; 4]` dans la struct, `mat4x4<f32>` en WGSL). Vous venez de poser les bases de la **caméra** !

</details>

---

➡️ **Chapitre suivant** : [Feuille de route](07-feuille-de-route.md)
