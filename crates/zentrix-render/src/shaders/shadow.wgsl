// =============================================================================
//  shadow.wgsl — la passe d'ombre : la scène vue depuis la lumière
// =============================================================================
//
// Pas de couleur, pas de texture, pas d'éclairage : seulement la POSITION de
// chaque sommet vue depuis la lumière. Le GPU écrit la profondeur dans la
// shadow map tout seul (il n'y a même pas de fragment shader).
// =============================================================================

// Même structure que dans mesh.wgsl (bind group n°0, même buffer).
// La matrice « vue depuis la lumière » de la tranche dessinée par cette
// passe (une passe par tranche : voir cascades.rs et renderer.rs).
struct Cascade {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> cascade: Cascade;

// On ne lit que la position du sommet (location 0) et la matrice de
// l'instance (locations 4 à 7) : les autres attributs du buffer (couleur,
// matériau...) sont ignorés.
// --- Les objets à DÉCOUPE (feuillage, grillage) ---------------------------
//
// Leurs trous doivent laisser passer la lumière : la passe d'ombre lit alors
// la texture de couleur et abandonne (`discard`) les pixels transparents,
// comme le fait mesh.wgsl. Seuls ces objets paient ce coût : les autres
// utilisent `vs_main`, sans fragment shader (voir pipeline.rs).
@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(5)
var material_sampler: sampler;

struct MaskedOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    // L'alpha du matériau et le seuil de découpe.
    @location(1) @interpolate(flat) alpha_cutoff: vec2<f32>,
};

@vertex
fn vs_masked(
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
    @location(8) base_color: vec4<f32>,
    @location(9) material: vec4<f32>,
) -> MaskedOutput {
    let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
    var out: MaskedOutput;
    out.clip_position = cascade.view_proj * model * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.alpha_cutoff = vec2<f32>(base_color.a, material.w);
    return out;
}

// Aucune couleur à écrire : le fragment shader ne sert qu'à ABANDONNER des pixels.
@fragment
fn fs_masked(in: MaskedOutput) {
    let alpha = textureSample(base_color_texture, material_sampler, in.uv).a * in.alpha_cutoff.x;
    if alpha < in.alpha_cutoff.y {
        discard;
    }
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
) -> @builtin(position) vec4<f32> {
    let model = mat4x4<f32>(model_0, model_1, model_2, model_3);
    return cascade.view_proj * model * vec4<f32>(position, 1.0);
}
