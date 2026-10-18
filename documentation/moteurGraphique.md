Voici une documentation complète en deux parties. La première détaille l'architecture interne du moteur graphique (le "Render Engine"). La seconde explique comment l'intégrer proprement dans un moteur de jeu complet (le "Game Engine").

---

# Documentation du Moteur Graphique et de son Intégration

## Partie 1 : Architecture du Moteur Graphique (Render Engine)

### 1.1. Introduction et Philosophie

Le moteur graphique est une bibliothèque spécialisée. Sa seule responsabilité est de transformer une scène décrite en données mathématiques (positions, couleurs, normales, textures) en une image 2D affichée à l'écran.

**Principes fondamentaux :**
- **Orientation Données (Data-Oriented)** : Les données GPU (buffers, textures) sont organisées pour le cache et le traitement parallèle du GPU.
- **Sans État (Stateless)** : Le moteur ne conserve pas d'état global caché. Toutes les informations de rendu sont explicitement passées à chaque frame.
- **Abstraction Matérielle** : Le code métier (jeu) ne manipule jamais directement le GPU. Il passe par des handles et des commandes.

### 1.2. Architecture Générale (Couches)

```text
+-----------------------------------------------------------+
|                  API Publique (RenderEngine)                |
+-----------------------------------------------------------+
|                    Render Graph / Passes                    |
|   (Shadow → GBuffer → Forward → Post-Process → UI)        |
+-----------------------------------------------------------+
|                   Pipeline & Shader Manager                 |
+-----------------------------------------------------------+
|                   Resource Manager (GPU)                    |
|   (Buffers, Textures, Samplers, Descriptor Sets)           |
+-----------------------------------------------------------+
|                   Device / Queue / Swapchain                |
+-----------------------------------------------------------+
|                         wgpu / Vulkan                       |
+-----------------------------------------------------------+
```

### 1.3. Le Cœur : WGPU et la Swapchain

Nous utilisons **`wgpu`** comme couche d'abstraction. Elle nous permet de cibler Vulkan, DirectX 12, Metal et WebGPU avec la même base de code.

```rust
// Structure principale du moteur de rendu
pub struct RenderEngine {
    // L'instance wgpu
    instance: wgpu::Instance,
    // L'adaptateur physique (GPU sélectionné)
    adapter: wgpu::Adapter,
    // Le périphérique logique (connexion au GPU)
    device: wgpu::Device,
    // La file de commandes (soumission des commandes au GPU)
    queue: wgpu::Queue,
    // La surface de rendu (liée à la fenêtre)
    surface: wgpu::Surface<'static>,
    // La configuration de la swapchain (format, dimensions)
    surface_config: wgpu::SurfaceConfiguration,
    // ...
}
```

**Cycle de vie d'une frame :**
1.  Obtenir une image de la swapchain (`surface.get_current_texture()`).
2.  Créer une `CommandEncoder`.
3.  Enregistrer toutes les commandes de rendu (draw calls, changements de pipeline).
4.  Soumettre le `CommandBuffer` à la `Queue`.
5.  Présenter l'image à l'écran (`surface.present()`).

### 1.4. Le Graphe de Rendu (Render Graph)

Plutôt que d'exécuter des passes dans un ordre codé en dur, nous utilisons un **Render Graph**. C'est un graphe acyclique dirigé (DAG) où chaque nœud est une passe de rendu.

**Avantages :**
- **Optimisation automatique** : Les passes qui ne produisent rien d'utilisé sont sautées.
- **Gestion mémoire** : Les textures intermédiaires (render targets) sont allouées et libérées automatiquement entre les frames.
- **Modularité** : On peut activer/désactiver des passes facilement.

**Exemple de passes :**
- `ClearPass` : Efface l'écran avec une couleur.
- `ShadowPass` : Rend la scène depuis le point de vue des lumières pour générer des shadow maps.
- `OpaquePass` : Rend tous les objets opaques (forward ou deferred).
- `TransparentPass` : Rend les objets transparents (back-to-front).
- `DebugOverlayPass` : Dessine les lignes, grilles et gizmos.
- `PostProcessPass` : Applique le bloom, le HDR, l'anti-aliasing (FXAA/TAA).

### 1.5. Gestion des Ressources GPU

Le moteur utilise un système de **handles** (identifiants) pour référencer les ressources. Le moteur de jeu ne manipule jamais un pointeur brut vers un buffer GPU.

```rust
// Handle vers une texture
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureHandle(u64);

// Handle vers un maillage
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeshHandle(u64);
```

Le `ResourceManager` gère le cycle de vie :
- **Création** : Chargement depuis un fichier ou création procédurale.
- **Upload** : Transfert des données CPU vers la mémoire GPU (staging buffer).
- **Référencement** : Compteur de références (ou génération d'index) pour éviter les double-free.
- **Destruction** : Nettoyage différé (quelques frames après la dernière utilisation).

**Types de Buffers :**
- `Vertex Buffer` : Contient les positions, normales, UV, couleurs.
- `Index Buffer` : Contient les indices pour le triangle strip/fan.
- `Uniform Buffer` : Contient les matrices de transformation, les données de caméra, les paramètres matériaux (mise à jour chaque frame).
- `Storage Buffer` : Pour les calculs complexes (particules, skinning).

### 1.6. Le Pipeline et les Shaders

Un `Pipeline` est l'état complet du GPU : vertex shader, fragment shader, layouts de sommets, états de rasterization, blend, depth test.

**Flux de compilation :**
1. **Source** : Fichiers `.wgsl` (recommandé) ou `.glsl` dans `assets/shaders/`.
2. **Compilation à la volée** : Dans le `build.rs` de la crate `render`, on utilise `naga` pour valider et compiler les shaders.
3. **Runtime** : Les shaders sont chargés en SPIR-V (ou WGSL natif). Le `ShaderManager` les transforme en `ShaderModule` WGPU.

**Hot-Reload (Phase 4 de la roadmap) :**
Le moteur graphique doit intégrer un watcher (`notify`) qui surveille le dossier des shaders. À chaque modification, il recompile le shader et reconstruit le pipeline. Cela permet d'itérer sur les effets visuels sans redémarrer le jeu.

### 1.7. Modèle de Caméra

Le moteur graphique expose une structure `Camera` qui encapsule la projection et la vue.

```rust
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn view_matrix(&self) -> Mat4 { /* ... */ }
    pub fn projection_matrix(&self) -> Mat4 { /* ... */ }
    pub fn view_projection(&self) -> Mat4 { /* ... */ }
}
```

Le moteur de jeu est responsable de mettre à jour les données de la caméra à chaque frame et de les envoyer au moteur graphique via une structure `Uniform` (alignée sur 16 octets pour Vulkan).

### 1.8. Mathématiques

Nous utilisons la crate **`glam`** pour les mathématiques vectorielles et matricielles. Elle est optimisée pour le SIMD et s'interface parfaitement avec `bytemuck` pour le casting de données.

```rust
use glam::{Mat4, Vec3, Vec4};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TransformUniform {
    pub model: Mat4,
    pub view: Mat4,
    pub projection: Mat4,
}
```

---

## Partie 2 : Implémentation dans un Moteur de Jeu (Integration)

### 2.1. Séparation des Responsabilités

La règle d'or est : **Le moteur de jeu orchestre, le moteur graphique exécute.**

- **Moteur de jeu** : Gère l'ECS, la physique, l'audio, les entrées, la logique métier.
- **Moteur graphique** : Reçoit une liste d'objets à dessiner (avec leurs transformations et matériaux) et une caméra, puis dessine.

**Le moteur graphique ne doit jamais :**
- Posséder les composants du jeu (ex: `Health`, `Player`, `Enemy`).
- Interroger directement l'ECS du jeu.
- Prendre de décisions de gameplay basées sur le rendu.

### 2.2. Architecture de Communication (Data Flow)

Voici comment les données circulent à chaque frame :

```text
+-------------------------------------+
| Game Engine Loop (Update)           |
| - Mise à jour physique              |
| - Mise à jour IA                    |
| - Mise à jour Transform             |
+------------------+------------------+
                   v
+-------------------------------------+
| Extraction des Données (Render Prep) |
| - Parcours des entités avec Mesh    |
|   et Transform components.          |
| - Construction d'une liste de draw  |
|   calls (Command Buffers "métier"). |
+------------------+------------------+
                   v
+-------------------------------------+
| Soumission au Render Engine         |
| - renderer.render(frame_data)       |
|   (caméra, lumières, draw_calls)    |
+------------------+------------------+
                   v
+-------------------------------------+
| Render Engine (Execution GPU)       |
| - Met à jour les uniform buffers    |
| - Enregistre les command buffers    |
| - Soumet au GPU                     |
| - Présente l'image                  |
+-------------------------------------+
```

### 2.3. Implémentation du `RenderContext` (Côté Jeu)

Le jeu expose une structure `RenderContext` qui fait le pont entre l'ECS et le moteur graphique.

```rust
// Dans le crate "game_engine"
pub struct RenderContext {
    renderer: RenderEngine,
    // Cache des handles pour éviter de les recréer chaque frame
    mesh_cache: HashMap<Entity, MeshHandle>,
    material_cache: HashMap<Entity, MaterialHandle>,
    // Frame data - alloué par frame (pour éviter les fuites de mémoire)
    frame_allocator: FrameAllocator,
}

impl RenderContext {
    pub fn prepare_frame(&mut self, ecs: &ECS, camera: &CameraComponent) -> FrameData {
        let mut draw_list = Vec::new();
        
        // Parcours des entités rendables
        for (entity, mesh, transform) in ecs.query::<(&MeshComponent, &Transform)>().iter() {
            let handle = self.mesh_cache.entry(entity).or_insert_with(|| {
                self.renderer.load_mesh(&mesh.data)
            });
            
            draw_list.push(DrawCommand {
                mesh: *handle,
                transform: transform.matrix,
                material: self.get_material_for_entity(entity),
            });
        }
        
        FrameData {
            camera: camera.to_uniform(),
            lights: self.extract_lights(ecs),
            draw_commands: draw_list,
        }
    }
}
```

### 2.4. Gestion des "In-Flight Frames" (Double/Triple Buffer)

Le GPU et le CPU travaillent en parallèle. Vous ne devez **jamais** modifier un buffer uniforme pendant que le GPU l'utilise.

**Solution :** Le moteur graphique implémente un système de *frame ring buffer* (généralement 2 ou 3 frames).

```rust
// Dans le RenderEngine
const FRAME_COUNT: usize = 3;

struct PerFrameResources {
    uniform_buffer: Buffer,      // Un buffer par frame
    command_encoder: CommandEncoder,
    // ...
}

pub struct RenderEngine {
    frames: [PerFrameResources; FRAME_COUNT],
    current_frame: usize,
}

impl RenderEngine {
    pub fn begin_frame(&mut self) -> &mut PerFrameResources {
        let idx = self.current_frame % FRAME_COUNT;
        // On attend que le GPU ait fini d'utiliser la frame idx-1
        self.frames[idx].wait_for_completion();
        &mut self.frames[idx]
    }
}
```

### 2.5. L'intégration des Actifs (Assets)

Le moteur graphique ne sait pas ouvrir un fichier `.png` ou `.gltf`. Il ne manipule que des données binaires.

**Flux d'intégration :**
1. **Asset Pipeline** (Phase 4) : Lit le fichier `.png`, le décode en pixels bruts.
2. **Transfert** : Le moteur de jeu demande au moteur graphique de créer une texture avec ces pixels bruts.
   ```rust
   // API du moteur graphique
   pub fn create_texture(&mut self, data: &[u8], width: u32, height: u32, format: TextureFormat) -> TextureHandle;
   ```
3. **Handle** : Le moteur de jeu stocke ce `TextureHandle` dans un composant `Material`.

### 2.6. Débogage et Gizmos (Phase 2 / 10)

Le moteur de jeu doit pouvoir dessiner des lignes, des sphères, des boîtes englobantes pour le débogage.

**Stratégie :** Le moteur graphique expose une API de rendu immédiat (immediate mode) ou un buffer de ligne dynamique.

```rust
// Dans l'API publique du moteur graphique
impl RenderEngine {
    pub fn draw_debug_line(&mut self, start: Vec3, end: Vec3, color: Vec4);
    pub fn draw_debug_aabb(&mut self, min: Vec3, max: Vec3, color: Vec4);
}
```

**Implémentation interne :** Ces appels remplissent un `DynamicVertexBuffer` (un buffer CPU-GPU avec `wgpu::BufferUsages::MAP_WRITE`). À la fin de la frame, ce buffer est vidé dans le `DebugOverlayPass`.

### 2.7. Exemple de Boucle de Jeu Complète

```rust
// main.rs du jeu
fn main() {
    let mut game = Game::new();
    let mut renderer = RenderEngine::new(&window);
    let mut ecs = ECS::new();
    
    // Création d'une caméra
    let camera_entity = ecs.spawn(Camera::new(...));
    
    loop {
        // 1. Entrées
        let input = window.poll_events();
        game.handle_input(input);
        
        // 2. Physique (pas fixe)
        physics.step();
        
        // 3. Mise à jour logique (Update)
        game.update();
        ecs.update();
        
        // 4. Préparation du rendu (Extraction des données)
        let camera = ecs.get::<Camera>(camera_entity);
        let frame_data = render_context.build_frame(&ecs, &camera);
        
        // 5. Rendu effectif
        renderer.render(frame_data);
        
        // 6. Swap buffers et gestion de la VSync
        window.swap_buffers();
    }
}
```

### 2.8. Résumé des Bonnes Pratiques

| Aspect | Moteur Graphique | Moteur de Jeu |
| :--- | :--- | :--- |
| **Propriété** | Possède les ressources GPU (buffers, textures). | Possède les données logiques (HP, position, nom). |
| **Dépendance** | Dépend de `winit`, `wgpu`, `glam`. | Dépend du moteur graphique, de l'ECS, de la physique. |
| **Temps** | Cadencé par le V-Sync (60/144Hz). | Peut tourner à un pas fixe (60 Hz) indépendant du rendu. |
| **État** | Possède un état GPU lourd (pipeline, descriptors). | Possède un état logique mutable. |
| **Communication** | Reçoit des commandes (Draw calls). | Envoie des commandes et interroge l'état du jeu. |

### 2.9. Prochaine Étape : Mise en Place

1. **Démarrage** : Créez le workspace Cargo avec `crates/render` et `crates/game_engine`.
2. **Fenêtre + Surface** : Utilisez `winit` et `wgpu` pour ouvrir une fenêtre et obtenir la surface.
3. **Premier Triangle** : Ne vous occupez pas de l'ECS ni de la physique pour l'instant. Hardcodez un triangle pour valider que votre moteur graphique écrit correctement dans la swapchain.
4. **Extraction** : Ajoutez un `Game` minimal qui possède une liste d'objets, et faites tourner votre Renderer en utilisant ces données.

Bonne programmation ! N'hésitez pas si vous avez besoin de détails plus techniques sur un point précis.