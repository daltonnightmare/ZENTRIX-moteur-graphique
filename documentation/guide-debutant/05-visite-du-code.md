# Chapitre 5 — Visite guidée du code : de `main()` au pixel

**Objectif** : suivre l'exécution du programme dans l'ordre réel, fichier par fichier. Ouvrez chaque fichier mentionné à côté de ce guide.

---

## Vue d'ensemble

```text
main()                                         zentrix-app/src/main.rs
 └─ zentrix_platform::run(config, app)         zentrix-platform/src/runner.rs
     └─ boucle d'événements winit
         ├─ resumed()        ──▶ crée la fenêtre ──▶ app.on_start(window)
         │                                           ├─ GpuContext::new(window)   zentrix-rhi
         │                                           └─ Renderer::new(&gpu)       zentrix-render
         ├─ Resized          ──▶ app.on_resize(w, h) ──▶ gpu.resize(w, h)
         ├─ KeyboardInput    ──▶ app.on_key(touche, appuyée)
         ├─ RedrawRequested  ──▶ timer.tick()
         │                       app.on_update(dt)
         │                       app.should_exit() ?
         │                       app.on_render() ──▶ renderer.render(&mut gpu)
         ├─ about_to_wait()  ──▶ window.request_redraw()   (relance la boucle)
         └─ CloseRequested   ──▶ event_loop.exit() ──▶ app.on_exit()
```

---

## Étape 1 — Le point d'entrée : `main()`

📄 [crates/zentrix-app/src/main.rs](../../crates/zentrix-app/src/main.rs)

```rust
fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn,naga=warn"),
    )
    .init();

    log::info!("Démarrage de ZENTRIX v{}", env!("CARGO_PKG_VERSION"));

    let config = WindowConfig { title: "ZENTRIX".to_string(), ..WindowConfig::default() };

    zentrix_platform::run(config, ZentrixApp::new())
}
```

1. **Les logs** sont activés en premier, pour voir tout ce qui suit. Le filtre par défaut montre `info` pour nos crates mais seulement `warn` pour wgpu, très bavard.
2. **La configuration** de la fenêtre est créée (taille par défaut 1280×720, voir `WindowConfig::default()`).
3. **`run`** reçoit la config et l'application, et **ne rend la main qu'à la fermeture**. Son résultat (`Ok` ou `Err`) devient celui de `main`.

`ZentrixApp::new()` crée l'état initial : `graphics: None` (pas encore de fenêtre, donc pas de GPU), `time: 0.0`, `paused: false`.

---

## Étape 2 — La boucle d'événements

📄 [crates/zentrix-platform/src/runner.rs](../../crates/zentrix-platform/src/runner.rs)

```rust
pub fn run<A: Application>(config: WindowConfig, app: A) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = PlatformRunner { config, app, window: None, timer: FrameTimer::new(), error: None };
    event_loop.run_app(&mut runner)?;
    // ...
}
```

- `ControlFlow::Poll` : la boucle tourne en continu (un jeu redessine sans arrêt).
- Le `PlatformRunner` implémente le trait `ApplicationHandler` de winit : c'est **winit qui appelle ses méthodes** quand un événement survient. On appelle ce modèle l'**inversion de contrôle** : on ne demande pas les événements, on est appelé.
- Si une erreur fatale survient pendant la boucle, elle est rangée dans `runner.error` et renvoyée à la fin.

---

## Étape 3 — Création de la fenêtre : `resumed`

📄 [runner.rs](../../crates/zentrix-platform/src/runner.rs), méthode `resumed`

winit impose de créer la fenêtre **dans** `resumed` (et pas avant), car sur mobile l'application peut être suspendue et reprise.

```rust
let window = match event_loop.create_window(attributes) {
    Ok(window) => Arc::new(window),
    Err(e) => { self.fail(event_loop, e.into()); return; }
};

if let Err(e) = self.app.on_start(Arc::clone(&window)) { /* ... */ }

self.timer = FrameTimer::new();
window.request_redraw();
self.window = Some(window);
```

- La fenêtre est emballée dans un `Arc` pour être partagée (plateforme + GPU + application).
- `on_start` est appelée : c'est le moment d'initialiser le GPU.
- Le chronomètre est remis à zéro, sinon la première image aurait un delta énorme (le temps d'initialisation du GPU, parfois une seconde).

---

## Étape 4 — Initialisation de l'application : `on_start`

📄 [main.rs](../../crates/zentrix-app/src/main.rs), méthode `on_start`

```rust
fn on_start(&mut self, window: Arc<Window>) -> anyhow::Result<()> {
    let gpu = pollster::block_on(GpuContext::new(Arc::clone(&window)))
        .context("échec de l'initialisation du GPU")?;

    let renderer = Renderer::new(&gpu);

    let info = gpu.adapter_info();
    window.set_title(&format!("ZENTRIX — {} ({:?})", info.name, info.backend));

    self.graphics = Some(GraphicsState { window, gpu, renderer });
    Ok(())
}
```

Les trois objets qui n'existent qu'avec une fenêtre (`window`, `gpu`, `renderer`) sont regroupés dans **une** struct `GraphicsState`, stockée dans **une** `Option`. Soit tout est prêt, soit rien : pas d'état à moitié initialisé.

---

## Étape 5 — Connexion au GPU : `GpuContext::new`

📄 [crates/zentrix-rhi/src/context.rs](../../crates/zentrix-rhi/src/context.rs)

Les cinq étapes, détaillées au [chapitre 4](04-comprendre-le-gpu.md#43-les-objets-wgpu-étape-par-étape) :

| # | Code | Résultat |
|---|------|----------|
| 1 | `wgpu::Instance::new(...)` | Point d'entrée wgpu |
| 2 | `instance.create_surface(window)?` | Lien avec la fenêtre |
| 3 | `instance.request_adapter(...).await?` | Choix du GPU |
| 4 | `adapter.request_device(...).await?` | `Device` + `Queue` |
| 5 | `surface.configure(&device, &config)` | Format, taille, V-Sync |

Chaque `?` peut faire échouer la fonction avec un `RhiError` précis (défini dans [error.rs](../../crates/zentrix-rhi/src/error.rs)). Dans `main.rs`, `.context(...)` y ajoute « échec de l'initialisation du GPU ».

---

## Étape 6 — Préparation du rendu : `Renderer::new`

📄 [crates/zentrix-render/src/renderer.rs](../../crates/zentrix-render/src/renderer.rs) et [pipeline.rs](../../crates/zentrix-render/src/pipeline.rs)

```rust
pub fn new(gpu: &GpuContext) -> Self {
    let pipeline = create_triangle_pipeline(gpu.device(), gpu.surface_format());
    let vertex_buffer = gpu.device().create_buffer_init(/* les 3 sommets */);
    Self { clear_color: Color::default(), pipeline, vertex_buffer, vertex_count: 3 }
}
```

`create_triangle_pipeline` :

1. charge le shader avec `include_str!("shaders/triangle.wgsl")` (le texte est intégré dans l'exécutable à la compilation) ;
2. crée un *pipeline layout* vide (pas encore de textures ni d'uniforms) ;
3. assemble le pipeline : vertex shader + format des sommets (`Vertex::layout()`), triangles en liste, culling des faces arrière, fragment shader qui écrit dans le format de la surface.

Toutes ces créations coûteuses ont lieu **une seule fois**.

---

## Étape 7 — La boucle de jeu : `RedrawRequested`

📄 [runner.rs](../../crates/zentrix-platform/src/runner.rs)

Chaque image suit ce cycle :

```rust
WindowEvent::RedrawRequested => {
    if self.timer.tick() { log::debug!("{:.0} FPS", self.timer.fps()); }  // a. temps
    self.app.on_update(self.timer.delta_seconds());                       // b. logique
    if self.app.should_exit() { event_loop.exit(); return; }              // c. quitter ?
    if let Err(e) = self.app.on_render() { self.fail(event_loop, e); }    // d. dessin
}
```

Puis `about_to_wait` appelle `window.request_redraw()`, ce qui provoque le prochain `RedrawRequested`. Avec la V-Sync (`PresentMode::Fifo`), la boucle se cale naturellement sur la fréquence de l'écran (60 Hz → 60 FPS).

> 📌 **Séparer `update` et `render`** est une règle fondamentale des moteurs de jeu (voir [moteurGraphique.md](../moteurGraphique.md), partie 2) : la logique ne dessine pas, et le rendu ne modifie pas la logique.

---

## Étape 8 — La logique : `on_update`

📄 [main.rs](../../crates/zentrix-app/src/main.rs)

```rust
fn on_update(&mut self, dt: f32) {
    if !self.paused {
        self.time += dt;
    }
    if let Some(g) = &mut self.graphics {
        let t = (self.time * 0.8).sin() * 0.5 + 0.5;  // oscille entre 0 et 1
        g.renderer.clear_color = Color::ZENTRIX_NIGHT.lerp(Color::rgb(0.15, 0.05, 0.25), t);
    }
}
```

- `time` avance de `dt` secondes à chaque image : l'animation a **la même vitesse** sur un écran 60 Hz ou 144 Hz.
- `sin` oscille entre -1 et 1 ; `* 0.5 + 0.5` ramène entre 0 et 1.
- `lerp` mélange deux couleurs selon `t` (voir [color.rs](../../crates/zentrix-core/src/color.rs)).

---

## Étape 9 — Le dessin : `Renderer::render`

📄 [renderer.rs](../../crates/zentrix-render/src/renderer.rs)

```rust
pub fn render(&self, gpu: &mut GpuContext) -> Result<(), RhiError> {
    let Some(frame) = gpu.acquire_frame()? else { return Ok(()); };   // 1

    let mut encoder = gpu.device().create_command_encoder(/* ... */); // 2
    {
        let mut render_pass = encoder.begin_render_pass(/* Clear(clear_color) */); // 3
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
    gpu.queue().submit(std::iter::once(encoder.finish()));            // 4
    gpu.present(frame);                                               // 5
    Ok(())
}
```

1. **Acquérir** une image. `acquire_frame` peut renvoyer `None` (fenêtre minimisée, surface en cours de reconfiguration) : on saute simplement l'image.
2. **Ouvrir** un carnet de commandes.
3. **Enregistrer** la passe : effacer avec la couleur de fond, choisir le pipeline, brancher les sommets, dessiner.
4. **Soumettre** : le GPU commence à travailler.
5. **Présenter** : l'image est affichée dès qu'elle est prête.

C'est ici que le **pixel** est né : le fragment shader a écrit sa couleur dans `frame.view`, et `present` l'envoie à l'écran. 🎉

---

## Étape 10 — Redimensionnement

📄 [rhi/context.rs](../../crates/zentrix-rhi/src/context.rs), méthode `resize`

Quand l'utilisateur redimensionne la fenêtre : `WindowEvent::Resized` → `app.on_resize(w, h)` → `gpu.resize(w, h)`, qui reconfigure la surface avec la nouvelle taille. Une taille de 0 (fenêtre minimisée) est ignorée car wgpu l'interdit.

---

## Étape 11 — Fermeture

Deux chemins mènent à la sortie :

- **Clic sur la croix** → `WindowEvent::CloseRequested` → `event_loop.exit()`.
- **Touche Échap** → `on_key` met `quit_requested = true` → `should_exit()` renvoie `true` → `event_loop.exit()`.

Puis winit appelle `exiting` → `app.on_exit()`, `run_app` se termine, `run` renvoie `Ok(())`, et `main` aussi.

En sortant de portée, `runner` est détruit, donc `app`, donc `graphics`, donc `renderer`, `gpu` et `window` : **toutes les ressources GPU et la fenêtre sont libérées automatiquement**, dans le bon ordre, grâce à la propriété de Rust. Aucune ligne de nettoyage à écrire.

---

## Récapitulatif : qui fait quoi ?

| Question | Réponse | Fichier |
|----------|---------|---------|
| Où est `main` ? | `zentrix-app` | `crates/zentrix-app/src/main.rs` |
| Qui crée la fenêtre ? | `PlatformRunner::resumed` | `crates/zentrix-platform/src/runner.rs` |
| Qui décide de l'ordre update/render ? | `PlatformRunner::window_event` | `crates/zentrix-platform/src/runner.rs` |
| Qui choisit le GPU ? | `GpuContext::new` | `crates/zentrix-rhi/src/context.rs` |
| Où sont les sommets des formes ? | `MeshData::triangle` / `cube` / `plane` | `crates/zentrix-render/src/mesh.rs` |
| Où est le shader ? | `mesh.wgsl` | `crates/zentrix-render/src/shaders/` |
| Où est la caméra ? | `Camera` | `crates/zentrix-render/src/camera.rs` |
| Où est construite la scène ? | `ZentrixApp::on_update` | `crates/zentrix-app/src/main.rs` |

> ℹ️ Ce chapitre décrit la v1 (un triangle codé en dur). Depuis, les étapes A à D de la
> [feuille de route](07-feuille-de-route.md) ont été réalisées : `const TRIANGLE` est devenu
> `MeshData::triangle()`, `triangle.wgsl` est devenu `mesh.wgsl` et `create_triangle_pipeline`
> est devenu `create_mesh_pipeline`. Le principe expliqué ici reste identique.

---

➡️ **Chapitre suivant** : [Exercices](06-exercices.md)
