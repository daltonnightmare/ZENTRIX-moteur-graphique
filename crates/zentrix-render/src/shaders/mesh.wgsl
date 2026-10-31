// =============================================================================
//  mesh.wgsl — dessine des maillages 3D colorés, texturés et éclairés
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

// Les données communes à tous les objets de l'image : caméra et lumière.
// Côté Rust : `FrameUniform` dans renderer.rs (même ordre, mêmes tailles).
struct FrameUniform {
    view_proj: mat4x4<f32>,
    camera_position: vec4<f32>,
    // Direction dans laquelle voyagent les rayons (normalisée).
    light_direction: vec4<f32>,
    // Couleur × intensité.
    light_color: vec4<f32>,
    ambient: vec4<f32>,
};

// `@group(0) @binding(0)` : la ressource n°0 du bind group n°0, fourni par
// `render_pass.set_bind_group(0, ...)`. `var<uniform>` : lecture seule, la
// même valeur pour toutes les exécutions du shader.
@group(0) @binding(0)
var<uniform> frame: FrameUniform;

// La texture de l'objet et son sampler (bind group n°1, voir texture.rs).
// Le sampler décide COMMENT lire la texture : filtrage, répétition, mipmaps.
@group(1) @binding(0)
var color_texture: texture_2d<f32>;
@group(1) @binding(1)
var color_sampler: sampler;

// Données d'ENTRÉE par sommet. Les `@location(n)` correspondent EXACTEMENT
// à `Vertex::ATTRIBUTES` côté Rust (crates/zentrix-render/src/vertex.rs).
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) normal: vec3<f32>,
};

// Données d'ENTRÉE par instance : les 4 colonnes de la matrice modèle
// (`InstanceRaw::ATTRIBUTES`).
struct InstanceInput {
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
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
    // Les UV sont interpolées de la même façon : chaque pixel du triangle
    // sait ainsi quel point de la texture lui correspond.
    @location(1) uv: vec2<f32>,
    // Position et normale dans le MONDE, pour l'éclairage.
    @location(2) world_position: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
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
    out.clip_position = frame.view_proj * world_position;
    out.color = vertex.color;
    out.uv = vertex.uv;
    out.world_position = world_position.xyz;
    // La normale tourne avec l'objet. Avec w = 0, la translation est ignorée
    // (une direction ne se déplace pas). Limite connue : ce calcul n'est
    // exact que pour une échelle UNIFORME ; une échelle (2, 1, 1) fausserait
    // légèrement les normales (il faudrait la « matrice normale »).
    out.world_normal = (model * vec4<f32>(vertex.normal, 0.0)).xyz;
    return out;
}

// `@location(0)` sur le retour : on écrit dans la cible de couleur n°0
// (l'image de la fenêtre, voir `targets` dans pipeline.rs).
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // `textureSample` lit la texture au point `uv`, en appliquant le filtrage
    // et en choisissant le bon niveau de mipmap selon la distance.
    let texel = textureSample(color_texture, color_sampler, in.uv);
    // La couleur du sommet TEINTE la texture (multiplication composante par
    // composante). Avec la texture blanche par défaut, on retrouve la couleur seule.
    let albedo = texel.rgb * in.color;

    // L'interpolation entre les 3 sommets raccourcit les normales : on les
    // remet à la longueur 1.
    let n = normalize(in.world_normal);
    // Direction VERS la lumière (l'inverse du trajet des rayons).
    let l = -frame.light_direction.xyz;

    // --- Diffuse (Lambert) : cos(angle) entre la normale et la lumière -------
    // `dot` de deux vecteurs unitaires = cosinus de leur angle. Négatif =
    // face tournée à l'opposé : on borne à 0 (pas de lumière « négative »).
    let diffuse = max(dot(n, l), 0.0);

    // --- Spéculaire (Blinn-Phong) : le reflet brillant -----------------------
    // `h` est à mi-chemin entre la lumière et l'œil. Plus la normale est
    // proche de `h`, plus le reflet est fort ; l'exposant (32) règle sa
    // finesse. Pas de reflet sur une face à l'ombre (`select`).
    let v = normalize(frame.camera_position.xyz - in.world_position);
    let h = normalize(l + v);
    let shininess = 32.0;
    let specular_strength = 0.25;
    let specular = select(0.0, pow(max(dot(n, h), 0.0), shininess), diffuse > 0.0)
        * specular_strength;

    let light = frame.light_color.rgb;
    let color = albedo * (frame.ambient.rgb + light * diffuse) + light * specular;
    return vec4<f32>(color, 1.0); // alpha = 1 : opaque
}
