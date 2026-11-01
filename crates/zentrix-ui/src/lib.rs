//! # zentrix-ui
//!
//! Une interface graphique de debug, grâce à [`egui`] : des fenêtres, des
//! boutons, des curseurs... dessinés par-dessus la scène.
//!
//! ## Le mode immédiat
//!
//! egui est une interface en **mode immédiat** (comme `DebugLines`) : on ne
//! crée pas des boutons une fois pour toutes, on DÉCRIT l'interface à chaque
//! image, et la fonction qui dessine un bouton renvoie directement s'il a
//! été cliqué :
//!
//! ```ignore
//! ui_layer.run(&window, |ctx| {
//!     egui::Window::new("Moteur").show(ctx, |ui| {
//!         ui.label(format!("{fps:.0} FPS"));
//!         if ui.button("Sauvegarder").clicked() {
//!             save();
//!         }
//!     });
//! });
//! ```
//!
//! Pas de rappels (« callbacks ») ni d'état à synchroniser : c'est idéal
//! pour des outils de debug qui changent souvent.
//!
//! ## Le trajet
//!
//! ```text
//!  événements winit ──on_window_event──▶ egui-winit ──┐
//!                                                      ▼
//!  chaque image : run(|ctx| ...) ──▶ egui (formes) ──▶ triangles
//!                                                      │
//!  Renderer::render_with_overlay ──prepare/draw──▶ egui-wgpu ──▶ image de la fenêtre
//! ```

use zentrix_platform::{Window, WindowEvent};
use zentrix_render::RenderOverlay;
use zentrix_rhi::{GpuContext, wgpu};

// Ré-export : l'application écrit `zentrix_ui::egui` et utilise ainsi la
// même version qu'ici.
pub use egui;

/// La couche d'interface : le contexte egui, la traduction des événements
/// et le rendu GPU.
pub struct UiLayer {
    ctx: egui::Context,
    /// Traduit les événements winit en événements egui.
    state: egui_winit::State,
    /// Dessine les triangles d'egui avec wgpu.
    renderer: egui_wgpu::Renderer,
    /// Le résultat de la dernière description de l'interface, à dessiner.
    primitives: Vec<egui::ClippedPrimitive>,
    /// Les textures (polices, images) à créer ou libérer sur le GPU.
    textures_delta: egui::TexturesDelta,
    /// Le zoom de l'écran (1.0, 1.25, 1.5... selon les réglages Windows).
    pixels_per_point: f32,
    /// Taille de l'image en pixels, connue dans `prepare` et réutilisée
    /// dans `draw` (egui s'en sert pour découper les zones de dessin).
    size_in_pixels: (u32, u32),
}

impl UiLayer {
    pub fn new(window: &Window, gpu: &GpuContext) -> Self {
        let ctx = egui::Context::default();
        // Pas d'ombre sous les fenêtres et menus : sur certains GPU (constaté
        // sur Intel Iris Xe / Vulkan), le flou de l'ombre s'affiche en
        // escalier au lieu d'être doux. Cause non identifiée ; une interface
        // de debug n'en a pas besoin.
        ctx.all_styles_mut(|style| {
            style.visuals.window_shadow = egui::Shadow::NONE;
            style.visuals.popup_shadow = egui::Shadow::NONE;
        });
        let state = egui_winit::State::new(
            ctx.clone(),
            ctx.viewport_id(),
            window,
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(gpu.device().limits().max_texture_dimension_2d as usize),
        );
        // egui fait lui-même la conversion de couleurs : il dessine dans la
        // vue non-sRGB de l'image (`Frame::overlay_view`).
        let renderer = egui_wgpu::Renderer::new(
            gpu.device(),
            gpu.overlay_format(),
            egui_wgpu::RendererOptions::default(),
        );
        Self {
            ctx,
            state,
            renderer,
            primitives: Vec::new(),
            textures_delta: egui::TexturesDelta::default(),
            pixels_per_point: window.scale_factor() as f32,
            size_in_pixels: (1, 1),
        }
    }

    /// À appeler pour chaque événement de la fenêtre. Renvoie `true` si
    /// l'interface l'a « consommé » (clic sur un panneau, frappe dans un
    /// champ de texte) : le jeu ne doit alors pas y réagir.
    ///
    /// Une touche du clavier n'est consommée que si un **champ de texte** a
    /// le focus. egui-winit, lui, la consomme dès qu'un widget quelconque a
    /// le focus. Or cliquer sur un bouton (« Ajouter un cube ») lui donne le
    /// focus : ensuite, `Suppr`, `Ctrl+Z` ou `F1` n'arrivaient plus au moteur.
    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let consumed = self.state.on_window_event(window, event).consumed;
        match event {
            WindowEvent::KeyboardInput { .. } => consumed && self.ctx.text_edit_focused(),
            _ => consumed,
        }
    }

    /// Retire le focus clavier de l'interface (on a cliqué dans la scène) :
    /// sinon, `Espace` ou `Entrée` « recliqueraient » le dernier bouton utilisé.
    pub fn clear_focus(&self) {
        self.ctx.memory_mut(|memory| {
            if let Some(id) = memory.focused() {
                memory.surrender_focus(id);
            }
        });
    }

    /// Décrit l'interface pour cette image. `build` reçoit le contexte egui
    /// et y crée fenêtres, boutons, etc.
    pub fn run(&mut self, window: &Window, build: impl FnMut(&egui::Context)) {
        profiling::scope!("UiLayer::run");
        let input = self.state.take_egui_input(window);
        let output = self.ctx.run_pass(input, build);
        // Curseur de la souris, presse-papiers, liens... à appliquer à la fenêtre.
        self.state
            .handle_platform_output(window, output.platform_output);
        // Les formes (rectangles, texte...) deviennent des triangles.
        self.primitives = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        // `append` : si une image a été sautée, on ne perd pas ses textures.
        self.textures_delta.append(output.textures_delta);
        self.pixels_per_point = output.pixels_per_point;
    }

    /// La souris est-elle au-dessus de l'interface (ou en train de la manipuler) ?
    /// Utile pour ne pas déclencher une action du jeu en cliquant sur un bouton.
    pub fn wants_pointer(&self) -> bool {
        self.ctx.egui_wants_pointer_input()
    }

    /// Un champ de texte de l'interface a-t-il le focus clavier ?
    pub fn wants_keyboard(&self) -> bool {
        self.ctx.text_edit_focused()
    }

    fn screen_descriptor(&self, (width, height): (u32, u32)) -> egui_wgpu::ScreenDescriptor {
        egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: self.pixels_per_point,
        }
    }
}

impl RenderOverlay for UiLayer {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        size_in_pixels: (u32, u32),
    ) -> Vec<wgpu::CommandBuffer> {
        // Nouvelles textures (la police de caractères, au premier appel).
        for (id, deltas) in &self.textures_delta.set {
            for delta in deltas {
                self.renderer.update_texture(device, queue, *id, delta);
            }
        }
        self.textures_delta.set.clear();

        self.size_in_pixels = size_in_pixels;
        let screen = self.screen_descriptor(size_in_pixels);
        self.renderer
            .update_buffers(device, queue, encoder, &self.primitives, &screen)
    }

    fn draw(&mut self, pass: &mut wgpu::RenderPass<'static>) {
        let screen = self.screen_descriptor(self.size_in_pixels);
        self.renderer.render(pass, &self.primitives, &screen);

        // Les textures devenues inutiles ne sont libérées qu'APRÈS le dessin.
        for id in self.textures_delta.free.drain() {
            self.renderer.free_texture(&id);
        }
    }
}
