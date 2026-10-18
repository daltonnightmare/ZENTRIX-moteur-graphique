//! Création du **pipeline de rendu** : la « recette » complète qui dit au GPU
//! comment transformer des sommets en pixels.

use zentrix_rhi::wgpu;

use crate::vertex::Vertex;

/// Crée le pipeline qui dessine des triangles colorés.
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
pub(crate) fn create_triangle_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    // `include_str!` lit le fichier À LA COMPILATION et l'insère dans le
    // programme sous forme de texte : pas besoin de livrer le .wgsl à côté
    // de l'exécutable. (Inconvénient : modifier le shader impose de recompiler.
    // Le « hot-reload » des shaders est prévu plus tard dans la feuille de route.)
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Triangle Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/triangle.wgsl").into()),
    });

    // Le « layout » décrit les ressources externes du shader (textures,
    // uniforms comme la matrice de caméra...). Notre shader n'en a aucune pour
    // l'instant, d'où la liste vide.
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Triangle Pipeline Layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        // Les labels apparaissent dans les messages d'erreur et les outils de
        // débogage GPU (RenderDoc...). Toujours en mettre !
        label: Some("Triangle Pipeline"),
        layout: Some(&layout),

        // Étape 1 : le vertex shader et le format des sommets qu'il reçoit.
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"), // nom de la fonction `@vertex` dans le .wgsl
            buffers: &[Some(Vertex::layout())],
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

        // Pas de test de profondeur pour l'instant (nécessaire en 3D, plus tard).
        depth_stencil: None,
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
