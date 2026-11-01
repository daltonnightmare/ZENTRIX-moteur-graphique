// =============================================================================
//  post.wgsl — le POST-TRAITEMENT : des passes « plein écran » sur l'image HDR
// =============================================================================
//
// La scène n'est plus dessinée directement dans la fenêtre, mais dans une
// image HDR (High Dynamic Range) : des nombres à virgule, qui peuvent
// dépasser 1. Un néon vaut 4, un reflet du soleil 20... Ces valeurs sont
// gardées telles quelles jusqu'ici, où elles deviennent des effets :
//
//   image HDR ──▶ BLOOM : ce qui dépasse le seuil « bave » autour de lui
//       │          prefilter → downsample × N → upsample × N (flou large)
//       ▼
//   composite : HDR + bloom, puis TONE MAPPING (ramené entre 0 et 1)
//       │
//       ▼
//   FXAA : adoucit les bords en escalier ──▶ fenêtre
//
// Chaque passe dessine UN triangle qui couvre tout l'écran (voir `vs_main`) :
// le fragment shader est alors appelé une fois par pixel de la cible.

// Les réglages, communs à toutes les passes (`PostUniform` dans postprocess.rs).
struct PostUniform {
    // x : seuil du bloom ; y : « genou » (transition douce autour du seuil) ;
    // z : intensité du bloom (0 = pas de bloom) ; w : nombre de niveaux.
    bloom: vec4<f32>,
    // x : tone mapping activé (1) ou non (0) ; y, z, w : inutilisés.
    options: vec4<f32>,
};

// L'image lue par la passe, une seconde image (le bloom, pour la passe de
// composition), le sampler et les réglages.
@group(0) @binding(0)
var source: texture_2d<f32>;
@group(0) @binding(1)
var bloom_texture: texture_2d<f32>;
@group(0) @binding(2)
var linear_sampler: sampler;
@group(0) @binding(3)
var<uniform> post: PostUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Le triangle plein écran : 3 sommets, sans vertex buffer. Les sommets
// (−1, −1), (3, −1) et (−1, 3) forment un triangle deux fois trop grand,
// découpé par les bords de l'écran : il couvre exactement tous les pixels.
// (Un seul triangle plutôt que deux : pas de diagonale traitée deux fois.)
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: VertexOutput;
    out.clip_position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    // Les coordonnées de texture vont vers le BAS : y est inversé.
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

// =============================================================================
//  Le BLOOM
// =============================================================================
//
// Méthode de « Call of Duty: Advanced Warfare » (Jorge Jimenez, 2014) :
// on réduit l'image de moitié plusieurs fois (chaque réduction floute un
// peu), puis on remonte en additionnant les niveaux. Un point lumineux
// devient une tache douce et LARGE pour un coût très faible : un flou de
// cette taille sur l'image entière coûterait des centaines de lectures par pixel.

// La luminance perçue d'une couleur linéaire (le vert compte le plus pour l'œil).
fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// Réduction « 13 lectures » : la moyenne pondérée d'un carré de 4 × 4
// pixels de la source, sans les artefacts en damier d'une simple moyenne de 4.
fn downsample13(uv: vec2<f32>) -> vec3<f32> {
    let px = 1.0 / vec2<f32>(textureDimensions(source));
    let a = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(-2.0, -2.0), 0.0).rgb;
    let b = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(0.0, -2.0), 0.0).rgb;
    let c = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(2.0, -2.0), 0.0).rgb;
    let d = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(-2.0, 0.0), 0.0).rgb;
    let e = textureSampleLevel(source, linear_sampler, uv, 0.0).rgb;
    let f = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(2.0, 0.0), 0.0).rgb;
    let g = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(-2.0, 2.0), 0.0).rgb;
    let h = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(0.0, 2.0), 0.0).rgb;
    let i = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(2.0, 2.0), 0.0).rgb;
    let j = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(-1.0, -1.0), 0.0).rgb;
    let k = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(1.0, -1.0), 0.0).rgb;
    let l = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(-1.0, 1.0), 0.0).rgb;
    let m = textureSampleLevel(source, linear_sampler, uv + px * vec2<f32>(1.0, 1.0), 0.0).rgb;
    return e * 0.125 + (a + c + g + i) * 0.03125 + (b + d + f + h) * 0.0625 + (j + k + l + m) * 0.125;
}

// Première réduction : on ne garde que ce qui dépasse le SEUIL. Le « genou »
// évite une coupure nette : un pixel juste sous le seuil brille un peu.
@fragment
fn fs_prefilter(in: VertexOutput) -> @location(0) vec4<f32> {
    // Un pixel isolé extrêmement brillant (un reflet du soleil sur un seul
    // pixel) ferait clignoter le bloom en bougeant : on borne la valeur.
    let color = min(downsample13(in.uv), vec3<f32>(64.0));
    let threshold = post.bloom.x;
    let knee = max(post.bloom.y, 1e-4);
    let brightness = max(color.r, max(color.g, color.b));
    // La « courbe douce » d'Unity / Bevy : 0 sous seuil − genou, puis une
    // parabole, puis linéaire au-delà du seuil + genou.
    var soft = clamp(brightness - threshold + knee, 0.0, 2.0 * knee);
    soft = soft * soft / (4.0 * knee);
    let contribution = max(soft, brightness - threshold) / max(brightness, 1e-4);
    return vec4<f32>(color * contribution, 1.0);
}

@fragment
fn fs_downsample(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(downsample13(in.uv), 1.0);
}

// Agrandissement avec un filtre « tente » 3 × 3 : le niveau plus petit est
// étalé et AJOUTÉ (mélange additif, voir postprocess.rs) au niveau au-dessus.
@fragment
fn fs_upsample(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = 1.0 / vec2<f32>(textureDimensions(source));
    var sum = textureSampleLevel(source, linear_sampler, in.uv, 0.0).rgb * 4.0;
    sum += textureSampleLevel(source, linear_sampler, in.uv + px * vec2<f32>(-1.0, 0.0), 0.0).rgb * 2.0;
    sum += textureSampleLevel(source, linear_sampler, in.uv + px * vec2<f32>(1.0, 0.0), 0.0).rgb * 2.0;
    sum += textureSampleLevel(source, linear_sampler, in.uv + px * vec2<f32>(0.0, -1.0), 0.0).rgb * 2.0;
    sum += textureSampleLevel(source, linear_sampler, in.uv + px * vec2<f32>(0.0, 1.0), 0.0).rgb * 2.0;
    sum += textureSampleLevel(source, linear_sampler, in.uv + px * vec2<f32>(-1.0, -1.0), 0.0).rgb;
    sum += textureSampleLevel(source, linear_sampler, in.uv + px * vec2<f32>(1.0, -1.0), 0.0).rgb;
    sum += textureSampleLevel(source, linear_sampler, in.uv + px * vec2<f32>(-1.0, 1.0), 0.0).rgb;
    sum += textureSampleLevel(source, linear_sampler, in.uv + px * vec2<f32>(1.0, 1.0), 0.0).rgb;
    return vec4<f32>(sum / 16.0, 1.0);
}

// =============================================================================
//  La COMPOSITION et le TONE MAPPING
// =============================================================================

// Le TONE MAPPING ramène des couleurs de toute intensité dans ce que l'écran
// sait afficher (0 à 1). Sans lui, tout ce qui dépasse 1 est écrêté : un
// reflet orange vif devient un aplat jaune-blanc sans nuance.
//
// Ici, « Khronos PBR Neutral » : il ne touche pas aux couleurs ordinaires
// (jusqu'à environ 0,76) et compresse doucement les plus intenses, en les
// désaturant un peu (comme un appareil photo surexposé).
fn tone_map_pbr_neutral(color_in: vec3<f32>) -> vec3<f32> {
    let start_compression = 0.8 - 0.04;
    let desaturation = 0.15;

    // Une petite courbe dans les noirs, pour compenser l'éclat de Fresnel.
    let x = min(color_in.r, min(color_in.g, color_in.b));
    let offset = select(0.04, x - 6.25 * x * x, x < 0.08);
    var color = color_in - offset;

    let peak = max(color.r, max(color.g, color.b));
    if peak < start_compression {
        return color;
    }
    let d = 1.0 - start_compression;
    let new_peak = 1.0 - d * d / (peak + d - start_compression);
    color *= new_peak / peak;
    let g = 1.0 - 1.0 / (desaturation * (peak - new_peak) + 1.0);
    return mix(color, vec3<f32>(new_peak), g);
}

@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSampleLevel(source, linear_sampler, in.uv, 0.0).rgb;
    // Le niveau 0 du bloom contient la SOMME de tous les niveaux : on en
    // fait la moyenne, puis on l'ajoute selon l'intensité choisie.
    let levels = max(post.bloom.w, 1.0);
    let bloom = textureSampleLevel(bloom_texture, linear_sampler, in.uv, 0.0).rgb / levels;
    color += bloom * post.bloom.z;
    if post.options.x > 0.5 {
        color = tone_map_pbr_neutral(color);
    }
    // La cible est au format sRGB : le GPU encode lui-même pour l'écran.
    return vec4<f32>(color, 1.0);
}

// =============================================================================
//  FXAA (« Fast Approximate Anti-Aliasing », Timothy Lottes, NVIDIA)
// =============================================================================
//
// Les bords des objets sont en ESCALIER : un pixel est ou n'est pas dans le
// triangle. Le FXAA repère, sur l'image FINALE, les pixels où la luminosité
// change brusquement (un bord), estime la direction du bord, et mélange le
// pixel avec ses voisins LE LONG de ce bord. Une seule passe, aucune
// modification du reste du rendu ; en contrepartie, un léger flou des détails
// fins (le MSAA, plus coûteux, n'a pas ce défaut).

// La luminosité PERÇUE : sur des valeurs linéaires, la racine carrée
// approche l'encodage de l'écran (les écarts dans les sombres comptent plus).
fn luma(c: vec3<f32>) -> f32 {
    return sqrt(luminance(c));
}

@fragment
fn fs_fxaa(in: VertexOutput) -> @location(0) vec4<f32> {
    let px = 1.0 / vec2<f32>(textureDimensions(source));
    let rgb_m = textureSampleLevel(source, linear_sampler, in.uv, 0.0).rgb;
    let luma_nw = luma(textureSampleLevel(source, linear_sampler, in.uv + vec2<f32>(-1.0, -1.0) * px, 0.0).rgb);
    let luma_ne = luma(textureSampleLevel(source, linear_sampler, in.uv + vec2<f32>(1.0, -1.0) * px, 0.0).rgb);
    let luma_sw = luma(textureSampleLevel(source, linear_sampler, in.uv + vec2<f32>(-1.0, 1.0) * px, 0.0).rgb);
    let luma_se = luma(textureSampleLevel(source, linear_sampler, in.uv + vec2<f32>(1.0, 1.0) * px, 0.0).rgb);
    let luma_m = luma(rgb_m);
    let luma_min = min(luma_m, min(min(luma_nw, luma_ne), min(luma_sw, luma_se)));
    let luma_max = max(luma_m, max(max(luma_nw, luma_ne), max(luma_sw, luma_se)));

    // La direction PERPENDICULAIRE au changement de luminosité : celle du bord.
    var dir = vec2<f32>(
        -((luma_nw + luma_ne) - (luma_sw + luma_se)),
        (luma_nw + luma_sw) - (luma_ne + luma_se),
    );
    let reduce = max((luma_nw + luma_ne + luma_sw + luma_se) * (0.25 / 8.0), 1.0 / 128.0);
    let scale = 1.0 / (min(abs(dir.x), abs(dir.y)) + reduce);
    // Au plus 8 pixels de part et d'autre.
    dir = clamp(dir * scale, vec2<f32>(-8.0), vec2<f32>(8.0)) * px;

    // Deux mélanges le long du bord : proche (A) et plus large (B).
    let rgb_a = 0.5 * (textureSampleLevel(source, linear_sampler, in.uv + dir * (1.0 / 3.0 - 0.5), 0.0).rgb
        + textureSampleLevel(source, linear_sampler, in.uv + dir * (2.0 / 3.0 - 0.5), 0.0).rgb);
    let rgb_b = rgb_a * 0.5 + 0.25 * (textureSampleLevel(source, linear_sampler, in.uv - dir * 0.5, 0.0).rgb
        + textureSampleLevel(source, linear_sampler, in.uv + dir * 0.5, 0.0).rgb);
    // Si le mélange large sort de la plage de luminosité locale, il a
    // débordé sur autre chose que le bord : on garde le mélange proche.
    let luma_b = luma(rgb_b);
    if luma_b < luma_min || luma_b > luma_max {
        return vec4<f32>(rgb_a, 1.0);
    }
    return vec4<f32>(rgb_b, 1.0);
}
