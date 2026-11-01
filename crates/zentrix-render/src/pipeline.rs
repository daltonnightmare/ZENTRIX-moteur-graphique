//! Création du **pipeline de rendu** : la « recette » complète qui dit au GPU
//! comment transformer des sommets en pixels.

use zentrix_rhi::wgpu;

use crate::debug::LineVertex;
use crate::depth::DEPTH_FORMAT;
use crate::vertex::{InstanceRaw, vertex_layout};

/// Crée le pipeline qui dessine des maillages 3D colorés.
///
/// Un pipeline regroupe **tout l'état** du GPU pour un type de dessin :
/// quels shaders, quel format de sommets, comment assembler les triangles,
/// comment mélanger les couleurs, dans quel format d'image écrire...
///
/// Le créer est **coûteux** (le pilote compile les shaders pour votre GPU) :
/// on le fait une seule fois au démarrage, puis on le réutilise à chaque image.
///
/// `pub(crate)` : visible dans toute la crate `zentrix-render`, mais pas à
/// l'extérieur. C'est un détail d'implémentation.
///
/// `frame_layout` décrit le bind group n°0 (caméra et lumière) : le pipeline
/// doit connaître sa *forme* à l'avance, le contenu viendra à chaque image.
///
/// # Quatre variantes
///
/// Le mélange des couleurs et l'élimination des faces arrière font partie
/// du pipeline : on ne peut pas les changer entre deux dessins sans changer
/// de pipeline. Le rendu crée donc les 4 combinaisons de [`MeshVariant`]
/// au démarrage (même shader), et trie les objets par variante.
pub(crate) fn create_mesh_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    frame_layout: &wgpu::BindGroupLayout,
    material_layout: &wgpu::BindGroupLayout,
    shadow_layout: &wgpu::BindGroupLayout,
    environment_layout: &wgpu::BindGroupLayout,
    variant: MeshVariant,
) -> wgpu::RenderPipeline {
    // `include_str!` lit le fichier À LA COMPILATION et l'insère dans le
    // programme sous forme de texte : pas besoin de livrer le .wgsl à côté
    // de l'exécutable. (Inconvénient : modifier le shader impose de recompiler.
    // Le « hot-reload » des shaders est prévu plus tard dans la feuille de route.)
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Mesh Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mesh.wgsl").into()),
    });

    // Le « layout » décrit les ressources externes du shader (textures,
    // uniforms...). L'élément n°i de la liste correspond à `@group(i)` dans le
    // shader : le groupe 0 pour la caméra et la lumière (une fois par image), le
    // groupe 1 pour les textures du matériau (change entre deux objets), le
    // groupe 2 pour la shadow map, le groupe 3 pour l'environnement. Quatre
    // groupes : c'est le maximum garanti par wgpu sur tous les GPU.
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Mesh Pipeline Layout"),
        bind_group_layouts: &[
            Some(frame_layout),
            Some(material_layout),
            Some(shadow_layout),
            Some(environment_layout),
        ],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        // Les labels apparaissent dans les messages d'erreur et les outils de
        // débogage GPU (RenderDoc...). Toujours en mettre !
        label: Some(match (variant.blend, variant.double_sided) {
            (false, false) => "Mesh Pipeline",
            (false, true) => "Mesh Pipeline (double face)",
            (true, false) => "Mesh Pipeline (transparent)",
            (true, true) => "Mesh Pipeline (transparent, double face)",
        }),
        layout: Some(&layout),

        // Étape 1 : le vertex shader et les buffers qu'il lit.
        // Slot 0 : les sommets du maillage ; slot 1 : les matrices des instances.
        // Ces numéros sont ceux de `render_pass.set_vertex_buffer(slot, ...)`.
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"), // nom de la fonction `@vertex` dans le .wgsl
            buffers: &[Some(vertex_layout()), Some(InstanceRaw::layout())],
            compilation_options: Default::default(),
        },

        // Étape 2 : comment assembler les sommets en formes.
        primitive: wgpu::PrimitiveState {
            // Chaque groupe de 3 sommets forme un triangle indépendant.
            topology: wgpu::PrimitiveTopology::TriangleList,
            // Les sommets sont listés dans le sens inverse des aiguilles d'une
            // montre (Counter-ClockWise) pour la face « avant ».
            front_face: wgpu::FrontFace::Ccw,
            // Ne pas dessiner les faces arrière (optimisation classique en 3D :
            // on ne voit jamais l'intérieur d'un objet fermé)... sauf pour
            // les objets « double face ».
            cull_mode: if variant.double_sided {
                None
            } else {
                Some(wgpu::Face::Back)
            },
            ..Default::default()
        },

        // Le test de profondeur (voir depth.rs) : un pixel n'est dessiné que
        // s'il est plus proche (`Less`) que ce qui est déjà dans le depth
        // buffer, et il y écrit alors sa propre profondeur.
        //
        // Un objet TRANSPARENT passe le test (il est caché par les opaques
        // devant lui), mais n'écrit pas sa profondeur : sinon, un autre
        // objet transparent situé derrière, dessiné ensuite, serait
        // éliminé au lieu d'apparaître à travers.
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(!variant.blend),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        // Pas d'anticrénelage (MSAA) : 1 échantillon par pixel.
        multisample: wgpu::MultisampleState::default(),

        // Étape 3 : le fragment shader et l'image dans laquelle il écrit.
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                // Doit correspondre au format de la surface de la fenêtre.
                format: target_format,
                // REPLACE : la nouvelle couleur écrase l'ancienne.
                // PREMULTIPLIED_ALPHA_BLENDING : image = couleur + fond × (1 - alpha).
                // « Prémultiplié » : le shader a déjà multiplié par l'alpha
                // ce qui doit l'être (la lumière diffuse, pas les reflets).
                blend: Some(if variant.blend {
                    wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING
                } else {
                    wgpu::BlendState::REPLACE
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),

        multiview_mask: None,
        cache: None,
    })
}

/// Les réglages qui distinguent les pipelines des maillages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MeshVariant {
    /// Mélange avec le fond (objets transparents).
    pub blend: bool,
    /// Les faces arrière sont dessinées aussi.
    pub double_sided: bool,
}

impl MeshVariant {
    /// Les 4 combinaisons, dans l'ordre de leur numéro (`index`).
    pub const ALL: [MeshVariant; 4] = [
        MeshVariant::new(false, false),
        MeshVariant::new(false, true),
        MeshVariant::new(true, false),
        MeshVariant::new(true, true),
    ];

    pub const fn new(blend: bool, double_sided: bool) -> Self {
        Self {
            blend,
            double_sided,
        }
    }

    /// Le numéro du pipeline dans le tableau du rendu (0 à 3).
    pub const fn index(self) -> u8 {
        (self.blend as u8) * 2 + self.double_sided as u8
    }
}

/// Crée un pipeline de la **passe d'ombre** (voir shadow.rs) : la scène vue
/// depuis la lumière, profondeur seule.
///
/// - `masked = false` : pas de fragment shader ni de cible de couleur, seule
///   la profondeur compte ;
/// - `masked = true` : pour les objets à découpe, un fragment shader lit
///   l'alpha de la texture de couleur (bind group n°1) et abandonne les
///   pixels transparents : les trous laissent passer la lumière ;
/// - aucune face n'est éliminée : une feuille ou un grillage double face
///   doit faire de l'ombre quel que soit le côté éclairé ;
/// - un **biais de profondeur** : chaque profondeur écrite est légèrement
///   repoussée. Sans lui, une surface éclairée se ferait de l'ombre à
///   elle-même par endroits (« acné d'ombre » : des rayures sur tout le sol),
///   à cause de la résolution limitée de la shadow map. Le biais « pente »
///   (`slope_scale`) est plus fort sur les surfaces vues de biais par la
///   lumière, là où l'acné est la plus marquée.
pub(crate) fn create_shadow_pipeline(
    device: &wgpu::Device,
    cascade_layout: &wgpu::BindGroupLayout,
    material_layout: &wgpu::BindGroupLayout,
    masked: bool,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shadow Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shadow.wgsl").into()),
    });

    let layouts: &[Option<&wgpu::BindGroupLayout>] = if masked {
        &[Some(cascade_layout), Some(material_layout)]
    } else {
        &[Some(cascade_layout)]
    };
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Shadow Pipeline Layout"),
        bind_group_layouts: layouts,
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(if masked {
            "Shadow Pipeline (découpe)"
        } else {
            "Shadow Pipeline"
        }),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some(if masked { "vs_masked" } else { "vs_main" }),
            // Les mêmes buffers que la passe principale : aucune copie.
            buffers: &[Some(vertex_layout()), Some(InstanceRaw::layout())],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: 2,
                slope_scale: 2.0,
                clamp: 0.0,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: masked.then(|| wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_masked"),
            // Aucune cible de couleur : le fragment shader ne fait qu'abandonner des pixels.
            targets: &[],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// Crée le pipeline des lignes de debug (voir debug.rs).
///
/// Différences avec le pipeline des maillages :
///
/// - topologie `LineList` : les sommets sont reliés deux par deux en segments ;
/// - pas d'élimination des faces arrière (une ligne n'a pas de face) ;
/// - test de profondeur `LessEqual` **sans écriture** : une ligne derrière
///   un cube est cachée, mais une ligne ne cache jamais rien ;
/// - pas de texture ni d'éclairage : seul le bind group n°0 (la caméra) est utilisé.
pub(crate) fn create_line_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    frame_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Debug Line Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/debug.wgsl").into()),
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Debug Line Pipeline Layout"),
        bind_group_layouts: &[Some(frame_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Debug Line Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(LineVertex::layout())],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::LineList,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// Crée le pipeline du **ciel** (voir sky.wgsl) : un triangle qui couvre
/// l'écran, à la profondeur maximale.
///
/// - aucun vertex buffer : les sommets sont calculés dans le shader ;
/// - test de profondeur `LessEqual` sans écriture : le ciel n'apparaît que
///   là où aucun objet n'a été dessiné.
pub(crate) fn create_sky_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    frame_layout: &wgpu::BindGroupLayout,
    environment_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Sky Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sky.wgsl").into()),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Sky Pipeline Layout"),
        bind_group_layouts: &[Some(frame_layout), Some(environment_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Sky Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    /// Compile et valide un shader WGSL avec naga, exactement comme le fera
    /// wgpu au démarrage, mais sans GPU : une faute de frappe dans un .wgsl
    /// est ainsi détectée par `cargo test` au lieu de planter au lancement.
    fn valider(nom: &str, source: &str) {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|e| panic!("{nom} : {}", e.emit_to_string(source)));
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::default(),
        )
        .validate(&module)
        .unwrap_or_else(|e| panic!("{nom} invalide : {e:?}"));
    }

    #[test]
    fn le_shader_des_maillages_est_valide() {
        valider("mesh.wgsl", include_str!("shaders/mesh.wgsl"));
    }

    #[test]
    fn le_shader_des_ombres_est_valide() {
        valider("shadow.wgsl", include_str!("shaders/shadow.wgsl"));
    }

    #[test]
    fn le_shader_du_ciel_est_valide() {
        valider("sky.wgsl", include_str!("shaders/sky.wgsl"));
    }

    #[test]
    fn le_shader_du_post_traitement_est_valide() {
        valider("post.wgsl", include_str!("shaders/post.wgsl"));
    }

    #[test]
    fn le_shader_des_lignes_est_valide() {
        valider("debug.wgsl", include_str!("shaders/debug.wgsl"));
    }
}
