// =============================================================================
//  sky.wgsl — dessine le ciel (l'environnement) en arrière-plan
// =============================================================================
//
// Le ciel n'est pas un objet : c'est ce qu'on voit « à l'infini », dans toutes
// les directions. On dessine donc UN triangle qui couvre tout l'écran, et
// chaque pixel lit la cube map de l'environnement dans la direction de son
// rayon de vue. Les reflets des objets montrent ainsi exactement ce ciel.
//
// Le triangle est placé à la profondeur maximale (1.0) et dessiné APRÈS les
// objets, avec le test `LessEqual` : seuls les pixels où rien n'a été dessiné
// (profondeur encore à 1.0) reçoivent le ciel. Les autres ne coûtent rien.
// =============================================================================

// Même struct que dans mesh.wgsl (bind group n°0).
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

// L'environnement : le même bind group que le n°3 de mesh.wgsl, placé ici
// en n°1 (seule sa FORME compte, pas son numéro).
@group(1) @binding(0)
var env_specular: texture_cube<f32>;
@group(1) @binding(1)
var env_irradiance: texture_cube<f32>;
@group(1) @binding(2)
var env_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // Coordonnées de l'écran (NDC), de -1 à 1.
    @location(0) ndc: vec2<f32>,
};

// Pas de vertex buffer : les 3 sommets sont calculés à partir de leur numéro.
// Le triangle (-1, -1), (3, -1), (-1, 3) déborde de l'écran, et le GPU découpe
// ce qui dépasse : il reste exactement le rectangle de l'écran.
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let ndc = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u)) * 2.0 - 1.0;
    var out: VertexOutput;
    // z = w : après la division par w, la profondeur vaut 1 (le plus loin).
    out.clip_position = vec4<f32>(ndc, 1.0, 1.0);
    out.ndc = ndc;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Le point du plan lointain sous ce pixel, ramené dans le monde par
    // l'inverse de la caméra ; le rayon de vue va de l'œil vers ce point.
    let far = frame.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
    let direction = normalize(far.xyz / far.w - frame.camera_position.xyz);
    var color = textureSampleLevel(env_specular, env_sampler, direction, 0.0).rgb
        * frame.environment.y;
    // Une couleur HDR : le tone mapping est fait après, sur toute l'image
    // (post.wgsl).
    return vec4<f32>(color, 1.0);
}
