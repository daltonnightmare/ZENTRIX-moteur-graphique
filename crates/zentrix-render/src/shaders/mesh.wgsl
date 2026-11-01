// =============================================================================
//  mesh.wgsl — dessine des maillages 3D texturés, éclairés (PBR + environnement) et ombrés
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

// `@group(0) @binding(0)` : la ressource n°0 du bind group n°0, fourni par
// `render_pass.set_bind_group(0, ...)`. `var<uniform>` : lecture seule, la
// même valeur pour toutes les exécutions du shader.
@group(0) @binding(0)
var<uniform> frame: FrameUniform;

// Une lampe ou un projecteur (`LightRaw` dans lighting.rs).
struct LocalLight {
    // xyz : position ; w : portée.
    position_range: vec4<f32>,
    // rgb : couleur × intensité ; w : 1 pour un projecteur.
    color_kind: vec4<f32>,
    // xyz : direction du projecteur ; w : cosinus de l'angle extérieur.
    direction_cos_outer: vec4<f32>,
    // x : cosinus de l'angle intérieur.
    cos_inner: vec4<f32>,
};

// Les lumières locales de l'image : un STORAGE buffer, lu comme un tableau.
// Seules les `frame.params.w` premières sont valides.
@group(0) @binding(1)
var<storage, read> local_lights: array<LocalLight>;

// Les textures du MATÉRIAU de l'objet (bind group n°1, voir texture.rs).
// Une texture absente est remplacée par une texture neutre (blanche, ou
// « plate » pour la normal map) : le shader n'a jamais de cas particulier.
@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;
// Le relief (voir `surface_normal`).
@group(1) @binding(1)
var normal_texture: texture_2d<f32>;
// Convention glTF : vert = rugosité, bleu = métal (rouge inutilisé).
@group(1) @binding(2)
var metallic_roughness_texture: texture_2d<f32>;
// Rouge : quelle part de la lumière ambiante atteint ce point (1 = toute,
// 0 = aucune, au fond d'une rainure).
@group(1) @binding(3)
var occlusion_texture: texture_2d<f32>;
// La lumière émise par l'objet lui-même (écran, néon, lave...).
@group(1) @binding(4)
var emissive_texture: texture_2d<f32>;
// Le sampler décide COMMENT lire les textures : filtrage, répétition, mipmaps.
@group(1) @binding(5)
var material_sampler: sampler;

// La shadow map et son sampler de comparaison (bind group n°2, voir shadow.rs).
// Une couche par tranche d'ombre.
@group(2) @binding(0)
var shadow_map: texture_depth_2d_array;
@group(2) @binding(1)
var shadow_sampler: sampler_comparison;

// L'ENVIRONNEMENT (bind group n°3, voir environment.rs) : deux « cube maps »,
// des textures à 6 faces que l'on lit avec une DIRECTION (et non des UV).
// `env_specular` : le ciel, de plus en plus flou à chaque niveau de mipmap
// (pour les reflets des surfaces de plus en plus rugueuses).
@group(3) @binding(0)
var env_specular: texture_cube<f32>;
// `env_irradiance` : la lumière totale reçue du ciel par une surface
// tournée dans chaque direction (pour l'éclairage diffus).
@group(3) @binding(1)
var env_irradiance: texture_cube<f32>;
@group(3) @binding(2)
var env_sampler: sampler;

// Données d'ENTRÉE par sommet. Les `@location(n)` correspondent EXACTEMENT
// à `Vertex::ATTRIBUTES` côté Rust (crates/zentrix-render/src/vertex.rs).
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) normal: vec3<f32>,
    // xyz : direction des u croissants ; w : ±1 (texture en miroir ou non).
    @location(10) tangent: vec4<f32>,
};

// Données d'ENTRÉE par instance : les 4 colonnes de la matrice modèle,
// puis le matériau (`InstanceRaw::ATTRIBUTES`).
struct InstanceInput {
    @location(4) model_0: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
    @location(8) base_color: vec4<f32>,
    // x : métal, y : rugosité, z : intensité du relief ; w : mode alpha
    // (0 = opaque, > 0 = seuil de découpe, -1 = mélange).
    @location(9) material: vec4<f32>,
    // rgb : émission ; w : intensité de l'occlusion.
    @location(11) emissive: vec4<f32>,
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
    // Le matériau de l'instance, le même pour tous les pixels :
    // `@interpolate(flat)` évite une interpolation inutile.
    @location(5) @interpolate(flat) base_color: vec4<f32>,
    @location(6) @interpolate(flat) material: vec4<f32>,
    @location(7) world_tangent: vec4<f32>,
    @location(8) @interpolate(flat) emissive: vec4<f32>,
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
    // La tangente tourne avec l'objet, comme la normale ; w ne change pas.
    out.world_tangent = vec4<f32>((model * vec4<f32>(vertex.tangent.xyz, 0.0)).xyz, vertex.tangent.w);
    out.base_color = instance.base_color;
    out.material = instance.material;
    out.emissive = instance.emissive;
    return out;
}

// =============================================================================
//  Les OMBRES EN CASCADE
// =============================================================================

// Dans quelle tranche tombe un point situé à `depth` mètres devant la
// caméra ? 3 = au-delà de la dernière : pas d'ombre.
fn cascade_index(depth: f32) -> i32 {
    var index = 0;
    if depth > frame.cascade_far.x {
        index = 1;
    }
    if depth > frame.cascade_far.y {
        index = 2;
    }
    if depth > frame.cascade_far.z {
        index = 3;
    }
    return index;
}

// Quelle part de la lumière directe atteint ce point, d'après la tranche
// `index` ? 1 = pleine lumière, 0 = dans l'ombre, entre les deux sur les bords.
fn cascade_shadow(index: i32, world_position: vec3<f32>, normal: vec3<f32>) -> f32 {
    // « Normal offset » : on teste un point légèrement décollé de la
    // surface, d'environ un texel. Sans cela, une surface se ferait de
    // l'ombre à elle-même par endroits (« acné »), d'autant plus que les
    // texels des tranches lointaines sont grands.
    let position = world_position + normal * frame.cascade_texel[index] * 1.5;
    let light_clip = frame.cascades[index] * vec4<f32>(position, 1.0);
    // Coordonnées « écran » de la lumière (NDC) → coordonnées de texture.
    // L'axe Y est inversé : +Y monte en NDC, mais v descend dans une texture.
    let ndc = light_clip.xyz / light_clip.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || ndc.z > 1.0 {
        return 1.0;
    }

    // PCF (« Percentage-Closer Filtering ») : 9 tests autour du point, dont
    // on fait la moyenne. Chaque `textureSampleCompareLevel` renvoie déjà la
    // moyenne de 4 tests (sampler `Linear`) : les bords d'ombre sont doux.
    let texel = frame.params.y;
    var lit = 0.0;
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel;
            lit += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + offset, index, ndc.z);
        }
    }
    return lit / 9.0;
}

// L'ombre d'un point : on choisit sa tranche selon sa profondeur. Sur les
// 10 derniers % d'une tranche, on MÉLANGE avec la suivante : sans ce
// fondu, on verrait une ligne nette là où la précision change. Après la
// dernière tranche, l'ombre s'efface doucement.
fn shadow_factor(world_position: vec3<f32>, normal: vec3<f32>) -> f32 {
    if frame.params.x < 0.5 {
        return 1.0; // ombres désactivées
    }
    let depth = dot(world_position - frame.camera_position.xyz, frame.camera_forward.xyz);
    let index = cascade_index(depth);
    if index >= 3 {
        return 1.0;
    }
    var shadow = cascade_shadow(index, world_position, normal);

    let far = frame.cascade_far[index];
    var near = 0.0;
    if index > 0 {
        near = frame.cascade_far[index - 1];
    }
    let fade_start = far - 0.1 * (far - near);
    if depth > fade_start {
        var next = 1.0;
        if index < 2 {
            next = cascade_shadow(index + 1, world_position, normal);
        }
        shadow = mix(shadow, next, (depth - fade_start) / (far - fade_start));
    }
    return shadow;
}

// =============================================================================
//  Le modèle d'éclairage PBR (« Physically Based Rendering »)
// =============================================================================
//
// La lumière qui frappe une surface se partage en deux :
//
//   - une partie REBONDIT aussitôt à la surface : c'est le REFLET (spéculaire).
//     Sa couleur est celle de la lumière... sauf pour les métaux, dont les
//     reflets sont colorés (l'or reflète du jaune) ;
//   - le reste PÉNÈTRE dans la matière, y est en partie absorbé, puis ressort
//     dans toutes les directions : c'est la couleur DIFFUSE de l'objet. Un
//     métal absorbe tout ce qui pénètre : il n'a PAS de couleur diffuse.
//
// Le reflet est calculé avec le modèle de COOK-TORRANCE, qui imagine la
// surface comme une foule de minuscules miroirs (« micro-facettes ») plus ou
// moins désordonnés selon la RUGOSITÉ :
//
//   reflet = D × G × F / (4 · (n·l) · (n·v))
//
//   D : combien de micro-facettes sont tournées pile pour renvoyer la lumière
//       vers l'œil (distribution GGX) → la taille et la forme du reflet ;
//   G : combien ne sont pas cachées par leurs voisines (Smith) → assombrit
//       les surfaces rugueuses vues de biais ;
//   F : la part de lumière réfléchie plutôt que transmise (Fresnel) → tout
//       devient plus réfléchissant vu de biais (une route au soleil couchant).

const PI: f32 = 3.14159265;

// D — distribution GGX (Trowbridge-Reitz).
fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    // Le carré de la rugosité donne un réglage plus « régulier » à l'œil
    // (convention de Disney, reprise par glTF).
    let a = roughness * roughness;
    let a2 = a * a;
    let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * d * d);
}

// G — masquage géométrique de Smith (approximation de Schlick-GGX), pour la
// lumière (n·l) et pour l'œil (n·v).
fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = r * r / 8.0;
    let g_v = n_dot_v / (n_dot_v * (1.0 - k) + k);
    let g_l = n_dot_l / (n_dot_l * (1.0 - k) + k);
    return g_v * g_l;
}

// F — approximation de Schlick. `f0` est la couleur du reflet vu de face :
// environ 4 % de gris pour un isolant, la couleur du métal pour un métal.
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

// Ce qu'il faut savoir d'un point de surface pour l'éclairer.
struct Surface {
    // La normale et la direction vers l'œil (unitaires).
    n: vec3<f32>,
    v: vec3<f32>,
    n_dot_v: f32,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    // Le reflet vu de face : 4 % pour un isolant, la couleur pour un métal.
    f0: vec3<f32>,
};

// La lumière renvoyée vers l'œil, en deux parts (voir l'alpha prémultiplié
// dans `fs_main`).
struct Reflected {
    diffuse: vec3<f32>,
    specular: vec3<f32>,
};

// L'éclairage DIRECT d'une source de lumière : `l` est la direction VERS la
// lumière, `radiance` la lumière qui arrive (déjà atténuée par l'ombre, la
// distance...). Le soleil et chaque lampe passent par ici.
fn direct_light(s: Surface, l: vec3<f32>, radiance: vec3<f32>) -> Reflected {
    // `h` est à mi-chemin entre la lumière et l'œil : c'est l'orientation
    // qu'une micro-facette doit avoir pour renvoyer la lumière vers l'œil.
    let h = normalize(l + s.v);
    // `dot` de deux vecteurs unitaires = cosinus de leur angle. Négatif =
    // tourné à l'opposé : on borne (pas de lumière « négative »).
    let n_dot_l = max(dot(s.n, l), 0.0);
    let n_dot_h = max(dot(s.n, h), 0.0);
    let h_dot_v = max(dot(h, s.v), 0.0);

    let f = fresnel_schlick(h_dot_v, s.f0);
    let d = distribution_ggx(n_dot_h, s.roughness);
    let g = geometry_smith(s.n_dot_v, n_dot_l, s.roughness);
    let specular = d * g * f / (4.0 * s.n_dot_v * max(n_dot_l, 0.0001));

    // Ce qui n'est pas réfléchi pénètre (1 - F) ; un métal l'absorbe entièrement.
    let k_diffuse = (vec3<f32>(1.0) - f) * (1.0 - s.metallic);
    // Lambert : divisé par π pour que la surface ne renvoie jamais plus de
    // lumière qu'elle n'en reçoit.
    let diffuse = k_diffuse * s.albedo / PI;

    var out: Reflected;
    out.diffuse = diffuse * radiance * n_dot_l;
    out.specular = specular * radiance * n_dot_l;
    return out;
}

// La lumière qu'une lampe ou un projecteur envoie jusqu'au point `position`,
// et sa direction. Renvoie une radiance nulle hors de portée.
//
// L'atténuation physique est en 1 / distance² (la lumière s'étale sur une
// sphère). On la multiplie par une « fenêtre » qui la fait descendre
// doucement à zéro à la portée : sans elle, la lumière s'arrêterait net
// en dessinant un cercle (méthode d'Unreal Engine 4).
fn local_light(light: LocalLight, position: vec3<f32>) -> vec4<f32> {
    let to_light = light.position_range.xyz - position;
    let distance2 = max(dot(to_light, to_light), 1e-4);
    let ratio2 = distance2 / (light.position_range.w * light.position_range.w);
    let window = clamp(1.0 - ratio2 * ratio2, 0.0, 1.0);
    // Au plus près, on fait comme si le point était à 10 cm : pas de
    // division par presque zéro au contact de la lampe.
    var attenuation = window * window / max(distance2, 0.01);
    let l = to_light * inverseSqrt(distance2);
    if light.color_kind.w > 0.5 {
        // Un projecteur : plein dans le cône intérieur, rien hors du cône
        // extérieur, un fondu entre les deux (en comparant des cosinus).
        let cos_angle = dot(-l, light.direction_cos_outer.xyz);
        attenuation *= smoothstep(light.direction_cos_outer.w, light.cos_inner.x, cos_angle);
    }
    // xyz : direction vers la lampe ; w : atténuation.
    return vec4<f32>(l, attenuation);
}

// La normale du pixel, avec le relief de la normal map.
//
// La normal map donne une normale dans le repère de la TEXTURE (rouge =
// droite de l'image, vert = haut, bleu = hors de la surface). La matrice
// TBN (Tangente, Bitangente, Normale) la transporte dans le repère du MONDE :
//
//   normale du monde = T × rouge + B × vert + N × bleu
//
// `sampled` est le pixel de la normal map, lu par l'appelant AVANT tout `if` :
// le GPU choisit le niveau de mipmap en comparant des pixels voisins, qui
// doivent donc tous faire la lecture (règle d'« uniformité », vérifiée par le
// compilateur WGSL).
//
// `facing` vaut -1 quand on voit la face ARRIÈRE du triangle (objets
// « double face ») : la normale est retournée vers l'œil, sinon cette face
// serait éclairée comme si on la regardait de dos.
fn surface_normal(in: VertexOutput, sampled: vec3<f32>, facing: f32) -> vec3<f32> {
    let n = normalize(in.world_normal) * facing;
    // Maillage sans tangentes : on ne sait pas orienter la normal map.
    if dot(in.world_tangent.xyz, in.world_tangent.xyz) < 1e-8 {
        return n;
    }
    // L'interpolation a pu écarter T de la perpendiculaire à N : on l'y
    // ramène (Gram-Schmidt), puis B complète le repère.
    let t = normalize(in.world_tangent.xyz - n * dot(n, in.world_tangent.xyz));
    let b = cross(n, t) * in.world_tangent.w;

    // Les octets 0..255 sont lus comme 0..1 ; on revient à -1..1.
    var m = sampled * 2.0 - 1.0;
    // L'intensité du relief (`Material::normal_scale`) incline plus ou moins
    // la normale ; elle ne touche pas à la composante « hors de la surface ».
    m = vec3<f32>(m.xy * in.material.z, m.z);
    return normalize(t * m.x + b * m.y + n * m.z);
}

// =============================================================================
//  La lumière de l'ENVIRONNEMENT (« IBL », Image-Based Lighting)
// =============================================================================
//
// Le soleil n'est pas la seule source de lumière : le ciel entier éclaire la
// scène, et se reflète dans les objets brillants. On le précalcule une fois
// pour toutes (environment.rs) dans deux cube maps, et on y lit :
//
//   - la lumière DIFFUSE : dans la direction de la normale, la moyenne du
//     ciel vu depuis la surface (`env_irradiance`) ;
//   - le REFLET : dans la direction du REFLET de l'œil sur la surface, le
//     ciel plus ou moins flou selon la rugosité (`env_specular`, un niveau de
//     mipmap par rugosité). C'est l'approximation « split sum » d'Unreal.

// Fresnel pour une lumière venue de partout : sur une surface rugueuse, le
// renforcement des reflets vus de biais est plus faible.
fn fresnel_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
    let f90 = max(vec3<f32>(1.0 - roughness), f0);
    return f0 + (f90 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

// La part du ciel renvoyée par le reflet, selon f0, la rugosité et l'angle.
// La méthode exacte lit une petite table précalculée (la « BRDF LUT ») ;
// cette formule en est une approximation (Karis, « Unreal Engine 4 mobile »).
fn env_brdf(f0: vec3<f32>, roughness: f32, n_dot_v: f32) -> vec3<f32> {
    let c0 = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let r = roughness * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * n_dot_v)) * r.x + r.y;
    let ab = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
    return f0 * ab.x + ab.y;
}

// `@location(0)` sur le retour : on écrit dans la cible de couleur n°0
// (l'image de la fenêtre, voir `targets` dans pipeline.rs).
//
// `@builtin(front_facing)` : vrai si le triangle est vu de face. Toujours
// vrai pour un objet simple face (les faces arrière ne sont pas dessinées).
@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
    // Toutes les textures du matériau sont lues d'abord, avant tout `if`
    // (règle d'uniformité, voir `surface_normal`). `textureSample` applique
    // le filtrage et choisit le bon niveau de mipmap selon la distance.
    let base_texel = textureSample(base_color_texture, material_sampler, in.uv);
    let normal_texel = textureSample(normal_texture, material_sampler, in.uv).xyz;
    let mr_texel = textureSample(metallic_roughness_texture, material_sampler, in.uv);
    let occlusion_texel = textureSample(occlusion_texture, material_sampler, in.uv).r;
    let emissive_texel = textureSample(emissive_texture, material_sampler, in.uv).rgb;

    // L'ALPHA : celui de la texture × celui du matériau.
    let alpha = base_texel.a * in.base_color.a;
    // Mode DÉCOUPE : sous le seuil, le pixel n'existe pas. `discard` abandonne
    // le pixel : rien n'est écrit, ni couleur ni profondeur. On peut donc
    // voir à travers les trous, et l'objet reste un opaque comme les autres.
    // (Après les lectures de textures : la règle d'uniformité est respectée.)
    let cutoff = in.material.w;
    if cutoff > 0.0 && alpha < cutoff {
        discard;
    }
    // Mode MÉLANGE (w < 0) : la « couverture » du pixel est son alpha. Sinon
    // le pixel couvre entièrement ce qui est derrière.
    let coverage = select(1.0, alpha, cutoff < 0.0);

    // La couleur du sommet et celle du matériau TEINTENT la texture
    // (multiplication composante par composante). Avec la texture blanche
    // par défaut, on retrouve la couleur seule.
    let albedo = base_texel.rgb * in.color * in.base_color.rgb;
    // Les textures multiplient les valeurs du matériau (convention glTF) :
    // sans texture (blanche = 1), seules les valeurs du matériau comptent.
    let metallic = clamp(in.material.x * mr_texel.b, 0.0, 1.0);
    // Une rugosité nulle donnerait un reflet infiniment petit (invisible,
    // ou des pixels qui scintillent) : on garde un minimum.
    let roughness = clamp(in.material.y * mr_texel.g, 0.04, 1.0);
    // L'intensité de l'occlusion (w) mélange « pas d'occlusion » (1) et la texture.
    let occlusion = mix(1.0, occlusion_texel, in.emissive.w);
    let emissive = in.emissive.rgb * emissive_texel;

    // La normale, avec le relief éventuel de la normal map.
    let n = surface_normal(in, normal_texel, select(-1.0, 1.0, front_facing));
    // La direction vers l'œil. Un minimum pour n·v évite une division par
    // zéro sur les bords vus de profil.
    let v = normalize(frame.camera_position.xyz - in.world_position);
    let n_dot_v = max(dot(n, v), 0.0001);
    // Le reflet vu de face : 4 % pour un isolant, la couleur pour un métal.
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    let surface = Surface(n, v, n_dot_v, albedo, metallic, roughness, f0);

    // Le SOLEIL. Convention de ZENTRIX : une intensité de 1 éclaire une
    // surface blanche mate, vue de face, exactement en blanc. D'où la
    // multiplication par π (qui compense la division du terme diffuse).
    // L'ombre utilise la normale GÉOMÉTRIQUE (sans le relief de la normal
    // map) pour décoller le point testé de la surface.
    let geometric_normal = normalize(in.world_normal) * select(-1.0, 1.0, front_facing);
    let shadow = shadow_factor(in.world_position, geometric_normal);
    let sun = direct_light(surface, -frame.light_direction.xyz, frame.light_color.rgb * PI * shadow);
    var direct_diffuse = sun.diffuse;
    var direct_specular = sun.specular;

    // Les LAMPES et PROJECTEURS : on les additionne toutes. (Toutes, pour
    // chaque pixel : simple, et suffisant pour quelques dizaines de lampes.)
    let light_count = u32(frame.params.w);
    for (var i = 0u; i < light_count; i++) {
        let light = local_lights[i];
        let incoming = local_light(light, in.world_position);
        if incoming.w <= 0.0 {
            continue; // hors de portée ou hors du cône
        }
        let lit = direct_light(surface, incoming.xyz, light.color_kind.rgb * PI * incoming.w);
        direct_diffuse += lit.diffuse;
        direct_specular += lit.specular;
    }

    // La lumière ambiante, venue « de partout », éclaire aussi les zones
    // d'ombre. L'occlusion l'atténue dans les creux.
    var ambient_diffuse: vec3<f32>;
    var ambient_specular = vec3<f32>(0.0);
    if frame.environment.x > 0.5 {
        // Avec l'environnement (IBL). `textureSampleLevel` choisit lui-même
        // le niveau de mipmap : il est permis après un `if`.
        let f_env = fresnel_schlick_roughness(n_dot_v, f0, roughness);
        let kd_env = (vec3<f32>(1.0) - f_env) * (1.0 - metallic);
        let irradiance = textureSampleLevel(env_irradiance, env_sampler, n, 0.0).rgb;
        // Le reflet de l'œil sur la surface ; plus la surface est rugueuse,
        // plus on lit un niveau de mipmap flou.
        let r = reflect(-v, n);
        let lod = roughness * frame.environment.z;
        let prefiltered = textureSampleLevel(env_specular, env_sampler, r, lod).rgb;
        ambient_diffuse = kd_env * albedo * irradiance * frame.environment.y;
        ambient_specular = prefiltered * env_brdf(f0, roughness, n_dot_v) * frame.environment.y;
    } else {
        // Sans environnement : une couleur ambiante uniforme. Les métaux,
        // sans couleur diffuse, en prennent une part réduite teintée par
        // leur reflet (sinon ils seraient noirs dans l'ombre).
        let ambient_tint = mix(albedo, f0 * 0.5, metallic);
        ambient_diffuse = frame.ambient.rgb * ambient_tint;
    }

    // L'ALPHA PRÉMULTIPLIÉ. Pour du verre, seule la lumière DIFFUSE (celle
    // qui ressort de la matière) est affaiblie par la transparence : le
    // REFLET naît à la surface, il reste entier. Une vitre presque invisible
    // montre donc encore le reflet du ciel. On multiplie nous-mêmes la
    // partie diffuse par l'alpha ; le mélange (pipeline.rs) fait ensuite
    //   image = couleur + fond × (1 - alpha)
    // Pour un objet opaque, `coverage` vaut 1 : rien ne change.
    let diffuse_light = ambient_diffuse * occlusion + direct_diffuse;
    let specular_light = ambient_specular * occlusion + direct_specular;
    // Une couleur HDR (elle peut dépasser 1) : le tone mapping est fait
    // après, sur toute l'image (post.wgsl). Le mélange du verre se fait donc
    // sur de vraies intensités lumineuses.
    var color = diffuse_light * coverage + specular_light + emissive;

    // Debug : chaque tranche d'ombre teintée (rouge, vert, bleu).
    if frame.params.z > 0.5 {
        let depth = dot(in.world_position - frame.camera_position.xyz, frame.camera_forward.xyz);
        let tints = array<vec3<f32>, 4>(
            vec3<f32>(1.0, 0.3, 0.3),
            vec3<f32>(0.3, 1.0, 0.3),
            vec3<f32>(0.3, 0.4, 1.0),
            vec3<f32>(1.0, 1.0, 1.0),
        );
        color *= tints[cascade_index(depth)];
    }
    return vec4<f32>(color, coverage);
}
