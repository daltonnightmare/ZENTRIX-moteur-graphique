//! L'**inspecteur** : une fenêtre egui pour observer et modifier la scène
//! pendant qu'elle tourne. C'est un premier pas vers un éditeur.

use zentrix_core::math::{EulerRot, Quat, Vec3};
use zentrix_core::{AlphaMode, Color, FrameProfiler, Material, Transform};
use zentrix_render::{GpuTimings, RenderStats};
use zentrix_scene::{DirectionalLight, Entity, MeshRenderer, Name, PointLight, Scene, SpotLight};
use zentrix_ui::egui;

use crate::demo::Spin;
use crate::editor::{GizmoMode, SpawnKind};

/// Les réglages de l'application que l'inspecteur peut modifier.
/// Des références `&mut` : l'inspecteur agit directement sur les vraies valeurs.
pub struct Toggles<'a> {
    pub show_grid: &'a mut bool,
    pub show_bounds: &'a mut bool,
    pub shadows: &'a mut bool,
    pub tone_mapping: &'a mut bool,
    pub bloom: &'a mut bool,
    pub bloom_intensity: &'a mut f32,
    pub bloom_threshold: &'a mut f32,
    pub fxaa: &'a mut bool,
    pub show_cascades: &'a mut bool,
    pub shadow_distance: &'a mut f32,
    pub environment: &'a mut bool,
    pub environment_intensity: &'a mut f32,
    pub volume: &'a mut f32,
    pub sound_off: &'a mut bool,
    pub paused: &'a mut bool,
    pub culling: &'a mut bool,
    pub freeze_culling: &'a mut bool,
    pub multithreading: &'a mut bool,
    pub vsync: &'a mut bool,
    pub gpu_timing: &'a mut bool,
    /// Ce que fait le gizmo de l'éditeur.
    pub gizmo_mode: &'a mut GizmoMode,
    /// Y a-t-il quelque chose à annuler, à refaire ?
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Les mesures de performance à afficher. `Copy` : de simples lectures,
/// que la fermeture de l'interface peut reprendre à chaque passe.
#[derive(Clone, Copy)]
pub struct Perf<'a> {
    pub fps: f32,
    pub stats: RenderStats,
    pub profiler: &'a FrameProfiler,
    /// Le temps GPU de chaque étape ; `None` si le GPU ne permet pas de le mesurer.
    pub gpu: Option<GpuTimings>,
    /// Le champ de stress est-il présent ?
    pub stress: bool,
}

/// Ce que l'utilisateur a demandé pendant cette image, et que l'inspecteur
/// ne peut pas faire lui-même (il n'a pas accès aux fichiers ni au GPU).
#[derive(Debug, Default)]
pub struct Requests {
    pub save: bool,
    pub load: bool,
    /// Ajouter ou retirer le champ de stress.
    pub toggle_stress: bool,
    /// Éditeur : annuler, refaire, ajouter un objet, supprimer la sélection.
    pub undo: bool,
    pub redo: bool,
    pub spawn: Option<SpawnKind>,
    pub delete: bool,
}

/// L'état de l'inspecteur (ce qui doit survivre d'une image à l'autre).
#[derive(Debug)]
pub struct Inspector {
    pub visible: bool,
    selected: Option<Entity>,
    /// Le texte du champ de recherche.
    filter: String,
    /// La liste des entités affichée, gardée d'une image à l'autre (voir
    /// `entity_list`), avec ce qui oblige à la refaire.
    entries: Vec<(Entity, String)>,
    entries_key: (u32, String),
    entries_age: u32,
}

/// La liste des entités est refaite au moins toutes les 30 images (0,5 s).
const ENTRIES_MAX_AGE: u32 = 30;

impl Default for Inspector {
    fn default() -> Self {
        Self {
            visible: true,
            selected: None,
            filter: String::new(),
            entries: Vec::new(),
            entries_key: (0, String::new()),
            entries_age: ENTRIES_MAX_AGE,
        }
    }
}

impl Inspector {
    /// L'entité sélectionnée, si elle existe encore.
    pub fn selected(&self) -> Option<Entity> {
        self.selected
    }

    /// Sélectionne une entité (clic dans la scène), ou aucune.
    pub fn select(&mut self, entity: Option<Entity>) {
        self.selected = entity;
    }

    /// Décrit la fenêtre de l'inspecteur pour cette image (mode immédiat).
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        scene: &mut Scene,
        mut toggles: Toggles,
        perf: Perf,
    ) -> Requests {
        let mut requests = Requests::default();
        if !self.visible {
            return requests;
        }
        // Une entité supprimée (F9, suppression...) n'est plus sélectionnable.
        if self.selected.is_some_and(|e| !scene.world.contains(e)) {
            self.selected = None;
        }

        // Le nom de la sélection, affiché en haut avec le bouton « Supprimer ».
        let selection = self.selected.map(|e| {
            scene
                .world
                .get::<&Name>(e)
                .map_or_else(|_| format!("{e:?}"), |n| n.0.clone())
        });

        egui::Window::new("Inspecteur (F4)")
            .default_pos([12.0, 12.0])
            .default_width(300.0)
            .show(ctx, |ui| {
                editor_section(ui, &mut toggles, &mut requests, selection.as_deref());
                ui.separator();
                self.engine_section(ui, scene, &mut toggles, perf.fps, &mut requests);
                ui.separator();
                performance_section(ui, toggles, &perf, &mut requests);
                ui.separator();
                light_section(ui, scene);
                ui.separator();
                self.entity_list(ui, scene);
                ui.separator();
                self.selection_section(ui, scene, &mut requests);
            });
        requests
    }

    fn engine_section(
        &self,
        ui: &mut egui::Ui,
        scene: &Scene,
        toggles: &mut Toggles,
        fps: f32,
        requests: &mut Requests,
    ) {
        ui.label(format!("{fps:.0} FPS · {} entités", scene.entity_count()));
        // `horizontal` : les éléments se suivent sur une ligne.
        ui.horizontal(|ui| {
            ui.checkbox(toggles.show_grid, "Grille (F1)");
            ui.checkbox(toggles.show_bounds, "Boîtes (F2)");
        });
        ui.horizontal(|ui| {
            ui.checkbox(toggles.shadows, "Ombres (F3)");
            ui.checkbox(toggles.paused, "Pause (P)");
        });
        if *toggles.shadows {
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(toggles.shadow_distance, 10.0..=150.0).text("m"))
                    .on_hover_text("Distance jusqu'à laquelle il y a des ombres");
                ui.checkbox(toggles.show_cascades, "Tranches")
                    .on_hover_text("Teinter chaque tranche d'ombre (rouge, vert, bleu)");
            });
        }
        ui.horizontal(|ui| {
            ui.checkbox(toggles.tone_mapping, "Tone mapping (F6)");
            ui.checkbox(toggles.environment, "Environnement (F7)");
        });
        ui.add(egui::Slider::new(toggles.environment_intensity, 0.0..=3.0).text("ciel"))
            .on_hover_text("Intensité de la lumière et des reflets de l'environnement");
        ui.horizontal(|ui| {
            ui.checkbox(toggles.bloom, "Bloom")
                .on_hover_text("Les zones très lumineuses débordent en halo");
            ui.checkbox(toggles.fxaa, "FXAA")
                .on_hover_text("Anticrénelage : adoucit les bords en escalier");
        });
        if *toggles.bloom {
            ui.add(egui::Slider::new(toggles.bloom_intensity, 0.0..=3.0).text("halo"))
                .on_hover_text("Force du bloom");
            ui.add(egui::Slider::new(toggles.bloom_threshold, 0.2..=4.0).text("seuil"))
                .on_hover_text("Luminosité à partir de laquelle un pixel produit un halo");
        }
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(toggles.volume, 0.0..=1.0).text("volume"));
            ui.checkbox(toggles.sound_off, "Muet (F8)");
        });
        ui.horizontal(|ui| {
            // `clicked()` : vrai seulement à l'image où le bouton est cliqué.
            requests.save = ui.button("Sauvegarder (F5)").clicked();
            requests.load = ui.button("Recharger (F9)").clicked();
        });
    }

    fn entity_list(&mut self, ui: &mut egui::Ui, scene: &Scene) {
        ui.horizontal(|ui| {
            ui.label("Entités");
            ui.text_edit_singleline(&mut self.filter)
                .on_hover_text("Filtrer par nom");
        });

        // Construire la liste (un texte par entité, puis un tri) coûte cher
        // avec 4 000 entités : le profileur montrait l'interface plus lente
        // que tout le reste du moteur. On la garde donc d'une image à
        // l'autre, et on ne la refait que si le nombre d'entités ou le filtre
        // change, ou au bout de 30 images (pour voir les entités remplacées
        // sans changer le nombre, comme une balle qui en chasse une autre).
        let key = (scene.entity_count(), self.filter.to_lowercase());
        self.entries_age += 1;
        if key != self.entries_key || self.entries_age >= ENTRIES_MAX_AGE {
            self.entries.clear();
            self.entries.extend(
                scene
                    .world
                    .query::<(Entity, Option<&Name>)>()
                    .iter()
                    .map(|(entity, name)| {
                        let label = name.map_or_else(|| format!("{entity:?}"), |n| n.0.clone());
                        (entity, label)
                    })
                    .filter(|(_, label)| label.to_lowercase().contains(&key.1)),
            );
            self.entries.sort_by(|a, b| a.1.cmp(&b.1));
            self.entries_key = key;
            self.entries_age = 0;
        }

        // Une zone qui défile. `show_rows` ne construit QUE les lignes
        // visibles, au lieu de mettre en page les 4 000 à chaque image.
        let row_height = ui.spacing().interact_size.y;
        egui::ScrollArea::vertical()
            .max_height(180.0)
            .auto_shrink([false, true])
            .show_rows(ui, row_height, self.entries.len(), |ui, rows| {
                for (entity, label) in &self.entries[rows] {
                    let selected = self.selected == Some(*entity);
                    if ui.selectable_label(selected, label).clicked() {
                        self.selected = Some(*entity);
                    }
                }
            });
    }

    fn selection_section(&mut self, ui: &mut egui::Ui, scene: &mut Scene, requests: &mut Requests) {
        let Some(entity) = self.selected else {
            ui.weak("Cliquez sur un objet de la scène, ou sur une entité de la liste, pour la modifier.");
            return;
        };

        if let Ok(name) = scene.world.get::<&Name>(entity) {
            ui.strong(&name.0);
        }

        // Les blocs `{ }` limitent la durée des emprunts du monde : il faut
        // les avoir relâchés avant de pouvoir supprimer l'entité.
        if let Ok(mut transform) = scene.world.get::<&mut Transform>(entity) {
            edit_transform(ui, &mut transform);
        }
        if let Ok(mut spin) = scene.world.get::<&mut Spin>(entity) {
            ui.horizontal(|ui| {
                ui.label("Rotation (rad/s)");
                ui.add(egui::DragValue::new(&mut spin.speed).speed(0.01));
            });
        }
        if let Ok(mut renderer) = scene.world.get::<&mut MeshRenderer>(entity) {
            edit_material(ui, &mut renderer.material);
        }
        if let Ok(mut light) = scene.world.get::<&mut PointLight>(entity) {
            ui.label("Lampe");
            // `&mut *light` : un emprunt de la lampe elle-même, pour en prendre
            // plusieurs champs à la fois.
            let PointLight {
                color,
                intensity,
                range,
            } = &mut *light;
            edit_light(ui, color, intensity, range);
        }
        if let Ok(mut spot) = scene.world.get::<&mut SpotLight>(entity) {
            ui.label("Projecteur");
            let SpotLight {
                color,
                intensity,
                range,
                inner_angle,
                outer_angle,
            } = &mut *spot;
            edit_light(ui, color, intensity, range);
            // Les angles s'affichent en degrés, plus parlants que des radians.
            ui.add(
                egui::Slider::new(outer_angle, 0.02..=1.5)
                    .text("cône")
                    .custom_formatter(|v, _| format!("{:.0}°", v.to_degrees())),
            );
            ui.add(egui::Slider::new(inner_angle, 0.0..=1.5).text("cœur").custom_formatter(|v, _| format!("{:.0}°", v.to_degrees())))
                .on_hover_text("Le cœur du cône, en pleine lumière ; au-delà, la lumière s'estompe jusqu'au bord du cône");
        }

        // La suppression passe par l'application, qui garde d'abord une
        // photo de la scène pour pouvoir l'annuler.
        requests.delete = ui.button("Supprimer, avec ses enfants (Suppr)").clicked();
    }
}

/// L'éditeur : le mode du gizmo, annuler / refaire, ajouter un objet, et
/// l'objet sélectionné avec le bouton pour le supprimer (en haut de
/// l'inspecteur : toujours visible, même quand la fenêtre dépasse de l'écran).
fn editor_section(
    ui: &mut egui::Ui,
    toggles: &mut Toggles,
    requests: &mut Requests,
    selection: Option<&str>,
) {
    ui.horizontal(|ui| match selection {
        Some(name) => {
            ui.label("Sélection :");
            ui.strong(name);
            if ui.button("Supprimer (Suppr)").clicked() {
                requests.delete = true;
            }
        }
        None => {
            ui.weak("Aucune sélection : cliquez sur un objet.");
        }
    });
    ui.horizontal(|ui| {
        for mode in GizmoMode::ALL {
            ui.selectable_value(toggles.gizmo_mode, mode, mode.label());
        }
    });
    ui.horizontal(|ui| {
        requests.undo = ui
            .add_enabled(toggles.can_undo, egui::Button::new("Annuler (Ctrl+Z)"))
            .clicked();
        requests.redo = ui
            .add_enabled(toggles.can_redo, egui::Button::new("Refaire (Ctrl+Y)"))
            .clicked();
    });
    ui.horizontal(|ui| {
        ui.label("Ajouter :");
        for kind in SpawnKind::ALL {
            if ui.button(kind.label()).clicked() {
                requests.spawn = Some(kind);
            }
        }
    });
    ui.weak("Clic : sélectionner · clic droit maintenu : piloter la caméra · Ctrl : par crans");
}

/// Position, rotation (en degrés) et taille d'un `Transform`.
fn edit_transform(ui: &mut egui::Ui, transform: &mut Transform) {
    // `Grid` : des colonnes alignées, comme un tableau.
    egui::Grid::new("transform").num_columns(4).show(ui, |ui| {
        ui.label("Position");
        for value in transform.translation.as_mut() {
            ui.add(egui::DragValue::new(value).speed(0.02));
        }
        ui.end_row();

        // Un quaternion est illisible pour un humain : on l'affiche sous
        // forme d'angles d'Euler (tangage X, lacet Y, roulis Z), en degrés.
        ui.label("Rotation °");
        let (y, x, z) = transform.rotation.to_euler(EulerRot::YXZ);
        let mut degrees = [x.to_degrees(), y.to_degrees(), z.to_degrees()];
        let mut changed = false;
        for value in &mut degrees {
            changed |= ui.add(egui::DragValue::new(value).speed(0.5)).changed();
        }
        // On ne réécrit la rotation que si l'utilisateur l'a modifiée : la
        // conversion aller-retour n'est pas parfaitement exacte.
        if changed {
            let [x, y, z] = degrees.map(f32::to_radians);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, y, x, z);
        }
        ui.end_row();

        ui.label("Taille");
        for value in transform.scale.as_mut() {
            ui.add(egui::DragValue::new(value).speed(0.01).range(0.01..=100.0));
        }
        ui.end_row();
    });
}

/// Couleur de base, métal et rugosité d'un matériau.
fn edit_material(ui: &mut egui::Ui, material: &mut Material) {
    ui.horizontal(|ui| {
        let c = material.base_color;
        let mut rgb = [c.r, c.g, c.b];
        if ui.color_edit_button_rgb(&mut rgb).changed() {
            material.base_color = Color::rgba(rgb[0], rgb[1], rgb[2], c.a);
        }
        ui.label("couleur de base");
    });
    ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0).text("métal"));
    ui.add(egui::Slider::new(&mut material.roughness, 0.0..=1.0).text("rugosité"));
    ui.add(egui::Slider::new(&mut material.normal_scale, 0.0..=3.0).text("relief"))
        .on_hover_text("Intensité de la normal map (sans effet si l'objet n'en a pas)");
    ui.add(egui::Slider::new(&mut material.occlusion_strength, 0.0..=1.0).text("occlusion"))
        .on_hover_text("Intensité de la texture d'occlusion (sans effet si l'objet n'en a pas)");
    // L'émission peut dépasser 1 (un néon très lumineux) : des valeurs
    // libres plutôt qu'un sélecteur de couleur limité à 0..1.
    ui.horizontal(|ui| {
        let e = &mut material.emissive;
        for value in [&mut e.r, &mut e.g, &mut e.b] {
            ui.add(egui::DragValue::new(value).speed(0.02).range(0.0..=20.0));
        }
        ui.label("émission (r, v, b)");
    });
    edit_alpha(ui, material);
}

/// La transparence : le mode alpha, l'opacité, le seuil de découpe, et
/// l'affichage des deux faces.
fn edit_alpha(ui: &mut egui::Ui, material: &mut Material) {
    ui.horizontal(|ui| {
        ui.label("alpha");
        let mode = &mut material.alpha_mode;
        // Le seuil est gardé si l'on repasse en découpe.
        let cutoff = match *mode {
            AlphaMode::Mask { cutoff } => cutoff,
            _ => 0.5,
        };
        ui.selectable_value(mode, AlphaMode::Opaque, "opaque");
        ui.selectable_value(mode, AlphaMode::Mask { cutoff }, "découpe");
        ui.selectable_value(mode, AlphaMode::Blend, "mélange");
    });
    if let AlphaMode::Mask { cutoff } = &mut material.alpha_mode {
        ui.add(egui::Slider::new(cutoff, 0.01..=1.0).text("seuil"))
            .on_hover_text("Les pixels dont l'alpha est sous ce seuil disparaissent");
    }
    if material.alpha_mode != AlphaMode::Opaque {
        ui.add(egui::Slider::new(&mut material.base_color.a, 0.0..=1.0).text("opacité"));
    }
    ui.checkbox(&mut material.double_sided, "Double face")
        .on_hover_text("Dessiner aussi l'arrière des triangles (intérieur visible)");
}

/// Les mesures : le temps de chaque étape de l'image, ce que le rendu a
/// dessiné, et les réglages qui changent ces chiffres.
fn performance_section(ui: &mut egui::Ui, toggles: Toggles, perf: &Perf, requests: &mut Requests) {
    egui::CollapsingHeader::new("Performance")
        .default_open(true)
        .show(ui, |ui| {
            let total = perf.profiler.average_frame_ms();
            ui.label(format!("{:.0} FPS · {total:.2} ms par image", perf.fps));
            frame_graph(ui, perf.profiler.history());

            // Le temps de chaque étape (moyenne glissante), avec une barre
            // proportionnelle à sa part de l'image.
            egui::Grid::new("profiler").num_columns(3).show(ui, |ui| {
                for section in perf.profiler.sections() {
                    ui.label(section.name);
                    ui.monospace(format!("{:6.2} ms", section.average_ms));
                    let share = if total > 0.0 {
                        section.average_ms / total
                    } else {
                        0.0
                    };
                    ui.add(egui::ProgressBar::new(share).desired_width(90.0));
                    ui.end_row();
                }
            });

            gpu_section(ui, perf.gpu, toggles.gpu_timing, total);

            let s = perf.stats;
            ui.label(format!(
                "Objets : {} · dessinés : {} (dont {} transparents) · ombres : {}",
                s.objects, s.visible, s.transparent, s.shadow_casters
            ));
            ui.label(format!(
                "Lampes et projecteurs éclairant la vue : {}",
                s.lights
            ));
            ui.label(format!(
                "Appels de dessin : {} + {} (ombre) · {} k triangles",
                s.draw_calls,
                s.shadow_draw_calls,
                s.triangles / 1000
            ));
            ui.label(format!(
                "Tri et envoi des objets : {:.2} ms",
                s.prepare_time.as_secs_f32() * 1000.0
            ));

            ui.horizontal(|ui| {
                ui.checkbox(toggles.culling, "Culling")
                    .on_hover_text("Ne pas dessiner ce qui est hors du champ de la caméra");
                ui.checkbox(toggles.freeze_culling, "Figer").on_hover_text(
                    "Garder la pyramide de vision actuelle, puis reculer pour la voir",
                );
            });
            ui.horizontal(|ui| {
                ui.checkbox(toggles.multithreading, "Plusieurs threads")
                    .on_hover_text(
                        "Répartir le tri des objets entre les cœurs (au-delà de 1 024 objets)",
                    );
                ui.checkbox(toggles.vsync, "V-Sync").on_hover_text(
                    "Sans V-Sync, les FPS ne sont plus limités par l'écran : utile pour mesurer",
                );
            });
            let label = if perf.stress {
                "Retirer le champ de stress (N)"
            } else {
                "Champ de stress : 4 096 cubes (N)"
            };
            requests.toggle_stress = ui.button(label).clicked();
        });
}

/// Le temps passé par le **GPU** sur chaque étape de l'image. Les barres
/// sont rapportées à la durée d'une image (`frame_ms`, mesurée côté CPU) :
/// si le total GPU s'en approche, c'est la carte graphique qui limite les FPS.
fn gpu_section(ui: &mut egui::Ui, gpu: Option<GpuTimings>, enabled: &mut bool, frame_ms: f32) {
    let Some(gpu) = gpu else {
        ui.label("GPU : ce GPU ne permet pas de mesurer son temps (pas d'horodatages).");
        return;
    };
    ui.horizontal(|ui| {
        ui.checkbox(enabled, "Mesures GPU").on_hover_text(
            "Le temps passé par la carte graphique sur chaque étape (horodatages du GPU)",
        );
        if *enabled && gpu.measured {
            ui.label(format!("total {:.2} ms", gpu.total.average_ms));
        }
    });
    if !*enabled || !gpu.measured {
        return;
    }
    egui::Grid::new("gpu").num_columns(3).show(ui, |ui| {
        for section in gpu.sections {
            ui.label(format!("GPU · {}", section.name));
            ui.monospace(format!("{:6.2} ms", section.average_ms));
            let share = if frame_ms > 0.0 {
                section.average_ms / frame_ms
            } else {
                0.0
            };
            ui.add(egui::ProgressBar::new(share.min(1.0)).desired_width(90.0));
            ui.end_row();
        }
    });
}

/// Le graphique des dernières images : une barre par image, d'autant plus
/// haute que l'image a été longue. La ligne marque 16,7 ms (60 images/s) :
/// une barre qui la dépasse est une image « en retard », un à-coup visible.
fn frame_graph(ui: &mut egui::Ui, history: &[f32]) {
    const BUDGET_60_FPS: f32 = 1000.0 / 60.0;
    let size = egui::vec2(ui.available_width().min(280.0), 44.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
    // L'échelle : au moins 2 × le budget, pour que la ligne reste visible.
    let max = history.iter().copied().fold(2.0 * BUDGET_60_FPS, f32::max);
    let y_of = |ms: f32| rect.bottom() - rect.height() * (ms / max).min(1.0);
    let bar_width = rect.width() / FrameProfiler::HISTORY_LEN as f32;
    for (i, &ms) in history.iter().enumerate() {
        let x = rect.left() + i as f32 * bar_width;
        let color = if ms > BUDGET_60_FPS * 1.05 {
            egui::Color32::from_rgb(230, 110, 60)
        } else {
            egui::Color32::from_rgb(90, 170, 120)
        };
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x, y_of(ms)),
                egui::pos2(x + bar_width, rect.bottom()),
            ),
            0.0,
            color,
        );
    }
    let y = y_of(BUDGET_60_FPS);
    painter.hline(
        rect.x_range(),
        y,
        egui::Stroke::new(1.0, egui::Color32::from_gray(160)),
    );
}

/// La couleur, l'intensité et la portée d'une lampe ou d'un projecteur.
fn edit_light(ui: &mut egui::Ui, color: &mut Color, intensity: &mut f32, range: &mut f32) {
    ui.horizontal(|ui| {
        let mut rgb = [color.r, color.g, color.b];
        if ui.color_edit_button_rgb(&mut rgb).changed() {
            *color = Color::rgb(rgb[0], rgb[1], rgb[2]);
        }
        ui.label("couleur");
    });
    ui.add(egui::Slider::new(intensity, 0.0..=30.0).text("intensité"));
    ui.add(egui::Slider::new(range, 0.5..=30.0).text("portée (m)"));
}

/// L'intensité du soleil et la lumière ambiante.
fn light_section(ui: &mut egui::Ui, scene: &mut Scene) {
    ui.label("Lumière");
    for light in scene.world.query_mut::<&mut DirectionalLight>() {
        ui.add(egui::Slider::new(&mut light.intensity, 0.0..=3.0).text("soleil"));
    }
    ui.horizontal(|ui| {
        let c = scene.ambient_light;
        let mut rgb = [c.r, c.g, c.b];
        // `color_edit_button_rgb` : couleur linéaire, comme dans le moteur.
        if ui.color_edit_button_rgb(&mut rgb).changed() {
            scene.ambient_light = Color::rgb(rgb[0], rgb[1], rgb[2]);
        }
        ui.label("ambiante");
    });
}

/// Le contour à dessiner autour de l'entité sélectionnée (en position monde).
pub fn selection_outline(scene: &Scene, entity: Entity) -> Option<zentrix_core::math::Mat4> {
    let global = scene
        .world
        .get::<&zentrix_scene::GlobalTransform>(entity)
        .ok()?;
    // Un peu plus grand que l'objet, pour ne pas se confondre avec ses arêtes.
    Some(global.0 * zentrix_core::math::Mat4::from_scale(Vec3::splat(1.08)))
}
