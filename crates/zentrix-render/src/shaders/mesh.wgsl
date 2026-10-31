// =============================================================================
//  triangle.wgsl — le premier shader de ZENTRIX
// =============================================================================
//
// Un SHADER est un petit programme exécuté par le GPU, des milliers de fois
// EN PARALLÈLE. Il est écrit en WGSL (WebGPU Shading Language), un langage
// dont la syntaxe ressemble volontairement à Rust.
//
// Le pipeline de rendu appelle deux shaders, l'un après l'autre :
//
//   sommets (Vertex buffer)
//        │
//        ▼
//   [ vs_main ]  VERTEX SHADER   : exécuté 1 fois par SOMMET (3 fois ici).
//        │                         Calcule où le sommet apparaît à l'écran.
//        ▼
//   rastérisation (fixe, faite par le GPU) : trouve tous les PIXELS couverts
//        │                                    par le triangle et interpole
//        ▼                                    les valeurs entre les sommets.
//   [ fs_main ]  FRAGMENT SHADER : exécuté 1 fois par PIXEL du triangle.
//        │                         Calcule la couleur finale du pixel.
//        ▼
//   image de la fenêtre
// =============================================================================

// Données d'ENTRÉE du vertex shader. Les `@location(n)` correspondent EXACTEMENT
// à `Vertex::ATTRIBUTES` côté Rust (crates/zentrix-render/src/vertex.rs).
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

// Données transmises du vertex shader au fragment shader.
struct VertexOutput {
    // `@builtin(position)` : valeur spéciale exigée par le GPU, la position du
    // sommet en « clip space » (x, y, z, w). Avec w = 1.0, x et y vont de -1 à 1.
    @builtin(position) clip_position: vec4<f32>,
    // Une valeur libre : la couleur. Le GPU l'INTERPOLE automatiquement entre les
    // 3 sommets, ce qui produit le joli dégradé à l'intérieur du triangle.
    @location(0) color: vec3<f32>,
};

// `@vertex` marque le point d'entrée du vertex shader.
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

// `@fragment` marque le point d'entrée du fragment shader.
// `@location(0)` sur le retour : on écrit dans la cible de couleur n°0
// (l'image de la fenêtre, voir `targets` dans pipeline.rs).
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0); // alpha = 1 : opaque
}
