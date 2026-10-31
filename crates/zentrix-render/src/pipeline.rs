//! Création du **pipeline de rendu** : la « recette » complète qui dit au GPU
//! comment transformer des sommets en pixels.

use zentrix_rhi::wgpu;

use crate::depth::DEPTH_FORMAT;
use crate::vertex::{InstanceRaw, Vertex};

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
/// `camera_layout` décrit le bind group n°0 (la matrice de la caméra) : le
/// pipeline doit connaître sa *forme* à l'avance, le contenu viendra à chaque image.
pub(crate) fn create_mesh_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    camera_layout: &wgpu::BindGroupLayout,
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
    // shader : ici un seul groupe, celui de la caméra.
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Mesh Pipeline Layout"),
        bind_group_layouts: &[Some(camera_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        // Les labels apparaissent dans les messages d'erreur et les outils de
        // débogage GPU (RenderDoc...). Toujours en mettre !
        label: Some("Mesh Pipeline"),
        layout: Some(&layout),

        // Étape 1 : le vertex shader et les buffers qu'il lit.
        // Slot 0 : les sommets du maillage ; slot 1 : les matrices des instances.
        // Ces numéros sont ceux de `render_pass.set_vertex_buffer(slot, ...)`.
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"), // nom de la fonction `@vertex` dans le .wgsl
            buffers: &[Some(Vertex::layout()), Some(InstanceRaw::layout())],
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
            // on ne voit jamais l'intérieur d'un objet fermé).
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },

        // Le test de profondeur (voir depth.rs) : un pixel n'est dessiné que
        // s'il est plus proche (`Less`) que ce qui est déjà dans le depth
        // buffer, et il y écrit alors sa propre profondeur.
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
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
                // REPLACE : la nouvelle couleur écrase l'ancienne (pas de transparence).
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),

        multiview_mask: None,
        cache: None,
    })
}
