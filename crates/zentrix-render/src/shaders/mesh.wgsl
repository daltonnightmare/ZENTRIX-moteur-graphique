// =============================================================================
//  mesh.wgsl — dessine des maillages 3D colorés, vus par une caméra
// =============================================================================
//
// Un SHADER est un petit programme exécuté par le GPU, des milliers de fois
// EN PARALLÈLE. Il est écrit en WGSL (WebGPU Shading Language), un langage
// dont la syntaxe ressemble volontairement à Rust.
//
// Le pipeline de rendu appelle deux shaders, l'un après l'autre :
//
//   sommets (vertex buffer) + matrices des instances (instance buffer)
//        │
//        ▼
//   [ vs_main ]  VERTEX SHADER   : exécuté 1 fois par SOMMET et par INSTANCE.
//        │                         Calcule où le sommet apparaît à l'écran.
//        ▼
//   rastérisation (fixe, faite par le GPU) : trouve tous les PIXELS couverts
//        │                                    par le triangle, interpole les
//        │                                    valeurs, et fait le TEST DE
//        ▼                                    PROFONDEUR (depth buffer).
//   [ fs_main ]  FRAGMENT SHADER : exécuté 1 fois par PIXEL du triangle.
//        │                         Calcule la couleur finale du pixel.
//        ▼
//   image de la fenêtre
// =============================================================================

// Les données de la caméra, identiques pour tous les objets de l'image.
// Côté Rust : `CameraUniform` dans renderer.rs.
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

// `@group(0) @binding(0)` : la ressource n°0 du bind group n°0, fourni par
// `render_pass.set_bind_group(0, ...)`. `var<uniform>` : lecture seule, la
// même valeur pour toutes les exécutions du shader.
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Données d'ENTRÉE par sommet. Les `@location(n)` correspondent EXACTEMENT
// à `Vertex::ATTRIBUTES` côté Rust (crates/zentrix-render/src/vertex.rs).
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

// Données d'ENTRÉE par instance : les 4 colonnes de la matrice modèle
// (`InstanceRaw::ATTRIBUTES`).
struct InstanceInput {
    @location(2) model_0: vec4<f32>,
    @location(3) model_1: vec4<f32>,
    @location(4) model_2: vec4<f32>,
    @location(5) model_3: vec4<f32>,
};

// Données transmises du vertex shader au fragment shader.
struct VertexOutput {
    // `@builtin(position)` : valeur spéciale exigée par le GPU, la position du
    // sommet en « clip space » (x, y, z, w). Le GPU divise ensuite par w :
    // c'est cette division qui crée l'effet de perspective.
    @builtin(position) clip_position: vec4<f32>,
    // Une valeur libre : la couleur. Le GPU l'INTERPOLE automatiquement entre les
    // 3 sommets du triangle.
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(
        instance.model_0,
        instance.model_1,
        instance.model_2,
        instance.model_3,
    );

    // objet → monde (model) → écran (view_proj). Comme en Rust avec glam, les
    // matrices s'appliquent de droite à gauche.
    let world_position = model * vec4<f32>(vertex.position, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.color = vertex.color;
    return out;
}

// `@location(0)` sur le retour : on écrit dans la cible de couleur n°0
// (l'image de la fenêtre, voir `targets` dans pipeline.rs).
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0); // alpha = 1 : opaque
}
