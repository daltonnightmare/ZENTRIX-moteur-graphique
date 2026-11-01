// =============================================================================
//  debug.wgsl — dessine les lignes de debug (grille, axes, boîtes...)
// =============================================================================
//
// Bien plus simple que mesh.wgsl : les positions sont déjà dans le monde (pas
// de matrice d'instance), et il n'y a ni texture ni éclairage.
// Le pipeline assemble les sommets deux par deux en segments (`LineList`).
// =============================================================================

// Les mêmes données que pour les maillages (bind group n°0, même buffer).
// Seule `view_proj` sert ici : pas d'éclairage pour les lignes de debug.
struct FrameUniform {
    view_proj: mat4x4<f32>,
    // L'inverse de `view_proj` : de l'écran vers le monde (pour le ciel).
    inv_view_proj: mat4x4<f32>,
    // La scène vue depuis la lumière, une matrice par tranche d'ombre
    // (ombres en cascade, voir cascades.rs).
    cascades: array<mat4x4<f32>, 3>,
    camera_position: vec4<f32>,
    // La direction du regard : la profondeur d'un point est sa distance
    // devant la caméra le long de cet axe.
    camera_forward: vec4<f32>,
    // Direction dans laquelle voyagent les rayons (normalisée).
    light_direction: vec4<f32>,
    // Couleur × intensité.
    light_color: vec4<f32>,
    ambient: vec4<f32>,
    // x : ombres activées (1) ou non (0) ; y : taille d'un texel de la shadow
    // map ; z : tranches d'ombre colorées (debug) ; w : nombre de lumières
    // ponctuelles et projecteurs.
    params: vec4<f32>,
    // x : environnement activé (1) ou non (0) ; y : son intensité ;
    // z : numéro du dernier niveau de mipmap de la carte des reflets.
    environment: vec4<f32>,
    // x, y, z : la profondeur où se termine chaque tranche d'ombre.
    cascade_far: vec4<f32>,
    // x, y, z : la taille d'un texel de chaque tranche, en mètres.
    cascade_texel: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> frame: FrameUniform;

// Côté Rust : `LineVertex::ATTRIBUTES` dans debug.rs.
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = frame.view_proj * vec4<f32>(in.position, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
