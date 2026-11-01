//! Le [`Renderer`] : point d'entrée du rendu pour l'application.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use zentrix_core::math::{Mat4, Vec3};
use zentrix_core::{Aabb, Color, Frustum, Image, Material, MeshData};
use zentrix_rhi::wgpu::util::DeviceExt; // apporte la méthode `create_buffer_init`
use zentrix_rhi::{GpuContext, RhiError, wgpu};

use crate::buffer::GrowableBuffer;
use crate::camera::Camera;
use crate::cascades::{CASCADE_COUNT, Cascade, ShadowSettings, compute_cascades};
use crate::culling::{self, Culling, DrawLists, Item, SHADOW_MASKED};
use crate::debug::{DebugLines, LineVertex};
use crate::depth::DepthTexture;
use crate::environment::{self, Environment, Sky};
use crate::gpu_timer::{GpuTimer, GpuTimings};
use crate::lighting::{LightRaw, Lighting, LocalLight};
use crate::mesh::{Mesh, MeshHandle};
use crate::pipeline::{
    MeshVariant, create_line_pipeline, create_mesh_pipeline, create_shadow_pipeline,
    create_sky_pipeline,
};
use crate::postprocess::{HDR_FORMAT, PostProcess, PostSettings};
use crate::shadow::{self, SHADOW_MAP_SIZE, ShadowMap};
use crate::texture::{self, MaterialTextures, Texture, TextureHandle, TextureKind};
use crate::vertex::InstanceRaw;

/// La texture utilisée quand une commande n'en précise pas : 1 pixel blanc,
/// qui laisse voir la couleur des sommets telle quelle (blanc × couleur = couleur).
/// Elle est créée en premier, elle a donc toujours le numéro 0.
const WHITE_TEXTURE: TextureHandle = TextureHandle(0);

/// La normal map par défaut : un pixel (128, 128, 255), soit la normale
/// (0, 0, 1) qui ne change rien. Créée en second : numéro 1.
const FLAT_NORMAL_MAP: TextureHandle = TextureHandle(1);

/// Les 5 textures d'un matériau, une fois les absentes remplacées par les
/// textures neutres (dans l'ordre de `MaterialTextures`).
pub(crate) type TextureSet = [TextureHandle; 5];

/// Les textures « neutres » : la clé de lot de la passe d'ombre, qui n'en
/// utilise aucune.
pub(crate) const NO_TEXTURES: TextureSet = [
    WHITE_TEXTURE,
    FLAT_NORMAL_MAP,
    WHITE_TEXTURE,
    WHITE_TEXTURE,
    WHITE_TEXTURE,
];

/// Ce qui oblige à changer d'état entre deux appels de dessin : le pipeline
/// (son numéro : voir `MeshVariant::index`), les textures du matériau et le
/// maillage. Les instances de même clé sont dessinées en un seul appel.
pub(crate) type BatchKey = (u8, TextureSet, MeshHandle);

/// Remplace les textures absentes par les textures neutres.
pub(crate) fn resolve(textures: &MaterialTextures) -> TextureSet {
    [
        textures.base_color.unwrap_or(WHITE_TEXTURE),
        textures.normal.unwrap_or(FLAT_NORMAL_MAP),
        textures.metallic_roughness.unwrap_or(WHITE_TEXTURE),
        textures.occlusion.unwrap_or(WHITE_TEXTURE),
        textures.emissive.unwrap_or(WHITE_TEXTURE),
    ]
}

/// « Dessine ce maillage, avec cette texture, à cet endroit. »
///
/// C'est **tout** ce que le rendu sait de la scène. Le jeu (ou plus tard
/// l'ECS) construit une liste de `DrawCommand` à chaque image et la donne à
/// [`Renderer::render`]. Ainsi le rendu ne dépend jamais de la logique de jeu.
#[derive(Debug, Clone, Copy)]
pub struct DrawCommand {
    pub mesh: MeshHandle,
    /// Les textures du matériau (couleur, relief, métal/rugosité, occlusion,
    /// émission), toutes facultatives. La normal map exige des tangentes
    /// dans le maillage.
    pub textures: MaterialTextures,
    /// La matrice modèle, généralement obtenue avec `Transform::matrix()`.
    pub transform: Mat4,
    /// De quoi l'objet est fait (couleur, métal, rugosité). Il ne coupe pas
    /// les lots : deux cubes de matériaux différents sont dessinés ensemble.
    pub material: Material,
}

/// Quelque chose à dessiner **par-dessus** l'image finale : une interface
/// graphique (`zentrix-ui`), un affichage de statistiques...
///
/// Le rendu ne sait pas ce que c'est : il lui donne simplement accès au GPU,
/// puis à une passe dont la cible est l'image de la fenêtre (sans depth
/// buffer : un overlay est toujours devant la scène), vue au format
/// **non-sRGB** `GpuContext::overlay_format()`.
pub trait RenderOverlay {
    /// Avant la passe : envoyer textures et buffers au GPU. Les command
    /// buffers renvoyés sont soumis AVANT ceux de l'image.
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        size_in_pixels: (u32, u32),
    ) -> Vec<wgpu::CommandBuffer>;

    /// Dessine dans la passe d'overlay.
    ///
    /// `RenderPass<'static>` : la passe ne garde plus d'emprunt vers
    /// l'encodeur (`forget_lifetime`), ce que demandent certaines
    /// bibliothèques comme egui-wgpu.
    fn draw(&mut self, pass: &mut wgpu::RenderPass<'static>);
}

/// Les données communes à tous les objets de l'image, telles que le shader
/// les lit (`FrameUniform` dans mesh.wgsl) : la caméra et la lumière.
///
/// # L'alignement
///
/// WGSL impose qu'un `vec3<f32>` d'un uniform commence à un multiple de 16
/// octets. Plutôt que d'ajouter des octets de remplissage à la main, on
/// n'utilise que des `mat4` et des `vec4` (16 octets) : le 4e nombre sert
/// parfois ou vaut simplement 0. Total : 5 × 64 + 9 × 16 = 464 octets.
///
/// Les champs portent leur nom côté Rust ; les shaders mesh, debug et sky
/// déclarent la même struct.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FrameUniform {
    view_proj: [[f32; 4]; 4],
    /// L'inverse de `view_proj` : de l'écran vers le monde (pour le ciel).
    inv_view_proj: [[f32; 4]; 4],
    /// La scène vue depuis la lumière, une matrice par tranche d'ombre.
    cascades: [[[f32; 4]; 4]; CASCADE_COUNT],
    /// Position de l'œil (pour le reflet spéculaire). w inutilisé.
    camera_position: [f32; 4],
    /// Direction du regard (pour la profondeur d'un pixel). w inutilisé.
    camera_forward: [f32; 4],
    /// Direction des rayons, normalisée. w inutilisé.
    light_direction: [f32; 4],
    /// Couleur × intensité de la lumière. w inutilisé.
    light_color: [f32; 4],
    /// Lumière ambiante. w inutilisé.
    ambient: [f32; 4],
    /// x : ombres activées (1.0) ou non (0.0) ; y : taille d'un texel de la
    /// shadow map en coordonnées de texture (pour le filtrage PCF) ;
    /// z : tranches d'ombre colorées (debug) ; w : nombre de lumières
    /// ponctuelles et projecteurs.
    params: [f32; 4],
    /// x : environnement activé (1.0) ou non (0.0) ; y : son intensité ;
    /// z : dernier niveau de mipmap de la cube map des reflets.
    environment: [f32; 4],
    /// x, y, z : la profondeur où se termine chaque tranche.
    cascade_far: [f32; 4],
    /// x, y, z : la taille d'un texel de chaque tranche, en mètres.
    cascade_texel: [f32; 4],
}

impl FrameUniform {
    fn new(
        view_proj: Mat4,
        camera: &Camera,
        lighting: &Lighting,
        cascades: &[Cascade; CASCADE_COUNT],
        params: [f32; 4],
        environment: [f32; 4],
    ) -> Self {
        let light = lighting.color;
        let k = lighting.intensity;
        let ambient = lighting.ambient;
        let forward = (camera.target - camera.position).normalize_or(Vec3::NEG_Z);
        let per_cascade = |f: fn(&Cascade) -> f32| {
            let mut v = [0.0; 4];
            for (slot, cascade) in v.iter_mut().zip(cascades) {
                *slot = f(cascade);
            }
            v
        };
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
            cascades: cascades.map(|c| c.view_projection.to_cols_array_2d()),
            camera_position: camera.position.extend(1.0).to_array(),
            camera_forward: forward.extend(0.0).to_array(),
            // `normalize_or` : une direction nulle (erreur de l'appelant)
            // retombe sur « droit vers le bas » au lieu de produire des NaN.
            light_direction: lighting
                .direction
                .normalize_or(Vec3::NEG_Y)
                .extend(0.0)
                .to_array(),
            light_color: [light.r * k, light.g * k, light.b * k, 0.0],
            ambient: [ambient.r, ambient.g, ambient.b, 0.0],
            params,
            environment,
            cascade_far: per_cascade(|c| c.far),
            cascade_texel: per_cascade(|c| c.texel_size),
        }
    }
}

/// Le nombre maximal de lumières locales par image (après culling). Le
/// shader les parcourt TOUTES pour chaque pixel : au-delà de quelques
/// dizaines, il faudrait un tri par zone de l'écran (*clustered shading*).
pub const MAX_LOCAL_LIGHTS: usize = 256;

/// Nombre d'instances que le buffer peut contenir au départ. Il grandit
/// automatiquement si la scène en demande plus.
const INITIAL_INSTANCE_CAPACITY: usize = 256;
/// Nombre de sommets de lignes de debug au départ (2 par ligne).
const INITIAL_LINE_VERTEX_CAPACITY: usize = 1024;

/// Ce que le rendu a fait pendant la dernière image : de quoi comprendre
/// où part le temps (affiché par l'inspecteur).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RenderStats {
    /// Les commandes de dessin reçues.
    pub objects: usize,
    /// Celles qui sont dans le champ de la caméra (les autres sont écartées
    /// par le culling).
    pub visible: usize,
    /// Parmi les visibles, celles qui sont transparentes (dessinées après
    /// les autres, triées par distance).
    pub transparent: usize,
    /// Celles qui projettent une ombre dans la zone couverte par la shadow map.
    pub shadow_casters: usize,
    /// Les lumières locales envoyées au shader (celles qui touchent la vue).
    pub lights: usize,
    /// Les appels de dessin (`draw_indexed`) de la passe principale et de
    /// la passe d'ombre.
    pub draw_calls: u32,
    pub shadow_draw_calls: u32,
    /// Les triangles envoyés par la passe principale.
    pub triangles: u64,
    /// Le temps passé à trier les objets (culling, lots) et à envoyer leurs
    /// données au GPU.
    pub prepare_time: Duration,
    /// Le temps passé à **attendre** une image libre de la fenêtre. Avec la
    /// synchronisation verticale, c'est le CPU qui attend l'écran : ce
    /// temps n'est pas du travail, et ne se réduit pas en optimisant.
    pub wait_time: Duration,
}

/// Le moteur de rendu. Il **possède** les ressources GPU (pipeline, buffers,
/// maillages) et dessine une image à chaque appel de [`Renderer::render`].
///
/// Quand le `Renderer` est détruit (sort de portée), Rust appelle
/// automatiquement le « destructeur » (`Drop`) de chaque champ : les buffers
/// et le pipeline sont libérés sur le GPU. Pas de `free` ou de `delete` à
/// écrire, et impossible de l'oublier.
pub struct Renderer {
    /// Couleur de fond utilisée pour effacer l'écran à chaque image.
    pub clear_color: Color,
    /// L'éclairage de la scène, envoyé au GPU à chaque image.
    pub lighting: Lighting,
    /// Les lampes et projecteurs de la scène (au plus `MAX_LOCAL_LIGHTS`
    /// dans le champ de la caméra).
    pub lights: Vec<LocalLight>,
    /// Les ombres portées sont-elles calculées ?
    pub shadows_enabled: bool,
    /// Jusqu'où vont les ombres, et comment les tranches sont réparties
    /// (ombres en cascade : voir cascades.rs).
    pub shadow_settings: ShadowSettings,
    /// Debug : chaque tranche d'ombre est teintée (rouge, vert, bleu).
    pub show_cascades: bool,
    /// Les tranches de la dernière image (pour les dessiner en debug).
    cascades: [Cascade; CASCADE_COUNT],
    /// Le **tone mapping** est-il appliqué ? Il ramène doucement les
    /// couleurs trop lumineuses (reflets) dans ce que l'écran peut afficher,
    /// au lieu de les écrêter brutalement à 1 (voir mesh.wgsl).
    pub tone_mapping: bool,
    /// Le **bloom** : les zones très lumineuses (néons, reflets du soleil)
    /// « bavent » autour d'elles, comme dans un œil ou un objectif.
    pub bloom_enabled: bool,
    /// La force du halo (0,5 à 1 : discret ; au-delà : rêveur).
    pub bloom_intensity: f32,
    /// La luminosité HDR à partir de laquelle un pixel produit un halo
    /// (1 = le blanc d'une surface bien éclairée).
    pub bloom_threshold: f32,
    /// L'anticrénelage **FXAA** : les bords en escalier sont adoucis.
    pub fxaa_enabled: bool,
    /// La lumière et les reflets de l'**environnement** (le ciel) sont-ils
    /// utilisés ? Le ciel est alors aussi dessiné en arrière-plan (à la
    /// place de `clear_color`). Sinon : une lumière ambiante uniforme
    /// (`Lighting::ambient`).
    pub environment_enabled: bool,
    /// Multiplie la lumière de l'environnement (et le ciel affiché).
    pub environment_intensity: f32,
    /// Le **culling** est-il actif ? Les objets hors du champ de la caméra
    /// (et, pour les ombres, hors de la zone d'ombre) ne sont pas dessinés.
    pub culling_enabled: bool,
    /// Le culling garde-t-il la pyramide de vision de la caméra **figée** ?
    /// Pour voir le culling à l'œuvre : on fige, puis on recule. Les objets
    /// hors de la pyramide figée disparaissent.
    pub freeze_culling: bool,
    /// Le tri des objets peut-il utiliser plusieurs threads ? (Pour
    /// comparer : voir `culling.rs`.)
    pub multithreading: bool,
    /// La matrice de la caméra au moment où le culling a été figé.
    frozen_view_projection: Option<Mat4>,
    stats: RenderStats,
    /// Les 4 variantes du pipeline des maillages (indice : `MeshVariant::index`).
    mesh_pipelines: Vec<wgpu::RenderPipeline>,
    sky_pipeline: wgpu::RenderPipeline,
    environment_layout: wgpu::BindGroupLayout,
    environment: Environment,
    line_pipeline: wgpu::RenderPipeline,
    /// Les pipelines d'ombre : n°0 pour les objets pleins, n°1
    /// (`SHADOW_MASKED`) pour ceux à découpe.
    shadow_pipelines: Vec<wgpu::RenderPipeline>,
    shadow_map: ShadowMap,

    /// Tous les maillages chargés. Un `MeshHandle` est un indice dans ce `Vec`.
    meshes: Vec<Mesh>,
    /// La boîte englobante de chaque maillage (même indice), pour le culling.
    mesh_bounds: Vec<Aabb>,

    /// Toutes les textures chargées (la n°0 est la texture blanche, la n°1
    /// la normal map plate).
    textures: Vec<Texture>,
    material_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// Un bind group par combinaison de textures déjà rencontrée, créé à la
    /// première utilisation puis réutilisé à chaque image.
    material_bind_groups: HashMap<TextureSet, wgpu::BindGroup>,

    frame_buffer: wgpu::Buffer,
    /// Les lumières locales de l'image (`MAX_LOCAL_LIGHTS` places), lues
    /// par le shader comme un tableau (storage buffer).
    light_buffer: wgpu::Buffer,
    light_data: Vec<LightRaw>,
    frame_bind_group: wgpu::BindGroup,

    /// Les matrices de toutes les instances de l'image, à la suite.
    instance_buffer: GrowableBuffer,
    /// Les sommets des lignes de debug de l'image.
    line_buffer: GrowableBuffer,
    /// Tableaux de travail réutilisés d'une image à l'autre : `clear()` vide
    /// un `Vec` sans libérer sa mémoire, on évite donc une allocation par image.
    /// `lists` sert au tri (opaques, transparents, ombres : voir
    /// culling.rs) ; `instance_data` est le format attendu par le GPU : les
    /// trois listes, l'une après l'autre, dans le même buffer.
    lists: DrawLists,
    instance_data: Vec<InstanceRaw>,

    depth: DepthTexture,
    /// Le post-traitement : l'image HDR où la scène est dessinée, le bloom,
    /// le tone mapping et le FXAA (voir postprocess.rs).
    post: PostProcess,
    /// Les mesures GPU (voir gpu_timer.rs) ; `None` si le GPU ne propose
    /// pas les horodatages.
    gpu_timer: Option<GpuTimer>,
    /// Les mesures GPU sont-elles prises ? (Elles ajoutent quelques passes
    /// vides : on peut les couper pour comparer.)
    pub gpu_timing_enabled: bool,
}

impl Renderer {
    /// Crée toutes les ressources GPU nécessaires au rendu.
    ///
    /// `gpu: &GpuContext` : on **emprunte** le contexte GPU le temps de la
    /// fonction ; on n'en devient pas propriétaire.
    pub fn new(gpu: &GpuContext) -> Self {
        let device = gpu.device();

        // --- Caméra et lumière : un uniform buffer + son bind group -----------
        // Le LAYOUT décrit la forme (« un uniform buffer, lu par les shaders
        // de sommets et de pixels ») ; le BIND GROUP relie cette forme à un
        // buffer précis.
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Frame Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, // `@binding(0)` dans le shader
                    // Le fragment shader en a besoin pour la lumière.
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Les lumières locales : un STORAGE buffer. Contrairement à
                // un uniform, sa taille peut être grande et le shader le lit
                // comme un tableau de longueur quelconque.
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // COPY_DST : on pourra y écrire depuis le CPU à chaque image (`write_buffer`).
        let frame_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Frame Buffer"),
            // Des zéros : le contenu est écrit avant chaque image.
            contents: bytemuck::bytes_of(&<FrameUniform as bytemuck::Zeroable>::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size: (MAX_LOCAL_LIGHTS * std::mem::size_of::<LightRaw>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Frame Bind Group"),
            layout: &frame_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
        });

        // --- Les textures : un layout et un sampler communs à tous les matériaux
        let material_layout = texture::create_material_layout(device);
        let sampler = texture::create_sampler(device);

        // --- L'environnement : le ciel, précalculé ----------------------------
        let environment_layout = environment::create_bind_group_layout(device);
        let environment =
            Environment::new(device, gpu.queue(), &environment_layout, &Sky::default());

        // --- Les ombres : la shadow map et son pipeline ------------------------
        let shadow_layout = shadow::create_bind_group_layout(device);
        let cascade_layout = shadow::create_cascade_layout(device);
        let shadow_map = ShadowMap::new(device, &shadow_layout, &cascade_layout);
        let shadow_pipelines = [false, true]
            .map(|masked| create_shadow_pipeline(device, &cascade_layout, &material_layout, masked))
            .into();

        let mesh_pipelines = MeshVariant::ALL
            .map(|variant| {
                create_mesh_pipeline(
                    device,
                    HDR_FORMAT,
                    &frame_layout,
                    &material_layout,
                    &shadow_layout,
                    &environment_layout,
                    variant,
                )
            })
            .into();
        let line_pipeline = create_line_pipeline(device, HDR_FORMAT, &frame_layout);
        let sky_pipeline =
            create_sky_pipeline(device, HDR_FORMAT, &frame_layout, &environment_layout);

        let mut renderer = Self {
            clear_color: Color::default(),
            lighting: Lighting::default(),
            lights: Vec::new(),
            shadows_enabled: true,
            shadow_settings: ShadowSettings::default(),
            show_cascades: false,
            cascades: [Cascade {
                view_projection: Mat4::IDENTITY,
                far: 0.0,
                texel_size: 0.0,
            }; CASCADE_COUNT],
            tone_mapping: true,
            bloom_enabled: true,
            bloom_intensity: 0.8,
            bloom_threshold: 1.0,
            fxaa_enabled: true,
            post: PostProcess::new(device, gpu.surface_format(), gpu.size()),
            gpu_timer: GpuTimer::new(device, gpu.queue()),
            gpu_timing_enabled: true,
            environment_enabled: true,
            environment_intensity: 1.0,
            culling_enabled: true,
            freeze_culling: false,
            multithreading: true,
            frozen_view_projection: None,
            stats: RenderStats::default(),
            mesh_pipelines,
            sky_pipeline,
            environment_layout,
            environment,
            line_pipeline,
            shadow_pipelines,
            shadow_map,
            meshes: Vec::new(),
            mesh_bounds: Vec::new(),
            textures: Vec::new(),
            material_layout,
            sampler,
            material_bind_groups: HashMap::new(),
            frame_buffer,
            light_buffer,
            light_data: Vec::new(),
            frame_bind_group,
            instance_buffer: GrowableBuffer::new(
                device,
                "Instance Buffer",
                wgpu::BufferUsages::VERTEX,
                (INITIAL_INSTANCE_CAPACITY * std::mem::size_of::<InstanceRaw>()) as u64,
            ),
            line_buffer: GrowableBuffer::new(
                device,
                "Debug Line Buffer",
                wgpu::BufferUsages::VERTEX,
                (INITIAL_LINE_VERTEX_CAPACITY * std::mem::size_of::<LineVertex>()) as u64,
            ),
            lists: DrawLists::default(),
            instance_data: Vec::new(),
            depth: DepthTexture::new(device, gpu.size()),
        };

        let white = renderer.add_texture(gpu, &Image::solid(1, 1, Color::WHITE));
        debug_assert_eq!(white, WHITE_TEXTURE);
        let flat = renderer.add_normal_map(gpu, &Image::new(1, 1, vec![128, 128, 255, 255]));
        debug_assert_eq!(flat, FLAT_NORMAL_MAP);

        log::info!("Renderer initialisé");
        renderer
    }

    /// Envoie un maillage sur le GPU et renvoie le handle qui permet de le dessiner.
    pub fn add_mesh(&mut self, gpu: &GpuContext, data: &MeshData) -> MeshHandle {
        self.meshes.push(Mesh::upload(gpu.device(), data));
        self.mesh_bounds.push(data.bounds());
        MeshHandle(self.meshes.len() - 1)
    }

    /// Les tranches d'ombre de la dernière image (pour les dessiner en debug).
    pub fn cascades(&self) -> &[Cascade; CASCADE_COUNT] {
        &self.cascades
    }

    /// Ce que le rendu a fait pendant la dernière image.
    pub fn stats(&self) -> RenderStats {
        self.stats
    }

    /// Le temps passé par le GPU sur chaque étape de l'image (moyennes
    /// glissantes). `None` si le GPU ne permet pas de le mesurer.
    pub fn gpu_timings(&self) -> Option<GpuTimings> {
        self.gpu_timer.as_ref().map(|t| *t.timings())
    }

    /// La matrice de la caméra figée par `freeze_culling` (pour dessiner la
    /// pyramide figée), ou `None`.
    pub fn frozen_view_projection(&self) -> Option<Mat4> {
        self.frozen_view_projection
    }

    /// Envoie une image sur le GPU (avec ses mipmaps) et renvoie le handle
    /// qui permet de la plaquer sur des maillages.
    pub fn add_texture(&mut self, gpu: &GpuContext, image: &Image) -> TextureHandle {
        self.upload_texture(gpu, image, TextureKind::Color)
    }

    /// Envoie une **normal map** sur le GPU. Contrairement à `add_texture`,
    /// ses octets ne subissent aucune conversion sRGB : ce sont des
    /// directions, pas des couleurs (voir `Image::normal_map_from_heights`).
    pub fn add_normal_map(&mut self, gpu: &GpuContext, image: &Image) -> TextureHandle {
        self.add_data_texture(gpu, image)
    }

    /// Envoie une texture de **données** (métal/rugosité, occlusion, normal
    /// map...) : lue telle quelle, sans conversion sRGB.
    pub fn add_data_texture(&mut self, gpu: &GpuContext, image: &Image) -> TextureHandle {
        self.upload_texture(gpu, image, TextureKind::Data)
    }

    /// Remplace le ciel de l'environnement. Le précalcul prend quelques
    /// dizaines de millisecondes : à faire au chargement, pas à chaque image.
    pub fn set_sky(&mut self, gpu: &GpuContext, sky: &Sky) {
        self.environment =
            Environment::new(gpu.device(), gpu.queue(), &self.environment_layout, sky);
    }

    fn upload_texture(
        &mut self,
        gpu: &GpuContext,
        image: &Image,
        kind: TextureKind,
    ) -> TextureHandle {
        let label = format!("Texture #{} ({kind:?})", self.textures.len());
        self.textures.push(Texture::upload(
            gpu.device(),
            gpu.queue(),
            image,
            kind,
            &label,
        ));
        TextureHandle(self.textures.len() - 1)
    }

    /// Dessine une image complète, vue par `camera`, et l'affiche :
    ///
    /// 1. la **passe d'ombre** : les maillages vus depuis la lumière, dans la
    ///    shadow map (si `shadows_enabled`) ;
    /// 2. la **passe principale** : les maillages de `draws`, éclairés et
    ///    ombrés, puis les lignes de `debug` par-dessus (mais cachées par les
    ///    objets placés devant elles), dans une image HDR ;
    /// 3. le **post-traitement** : bloom, tone mapping et FXAA, de l'image
    ///    HDR vers la fenêtre.
    ///
    /// `&mut GpuContext` car acquérir une image peut nécessiter de
    /// reconfigurer la surface (donc de modifier le contexte).
    pub fn render(
        &mut self,
        gpu: &mut GpuContext,
        camera: &Camera,
        draws: &[DrawCommand],
        debug: &DebugLines,
    ) -> Result<(), RhiError> {
        self.render_with_overlay(gpu, camera, draws, debug, None)
    }

    /// Comme [`Renderer::render`], avec un [`RenderOverlay`] dessiné en
    /// dernier, par-dessus tout le reste (par exemple l'interface graphique).
    pub fn render_with_overlay(
        &mut self,
        gpu: &mut GpuContext,
        camera: &Camera,
        draws: &[DrawCommand],
        debug: &DebugLines,
        overlay: Option<&mut dyn RenderOverlay>,
    ) -> Result<(), RhiError> {
        profiling::scope!("Renderer::render");
        // 1. Obtenir l'image de la fenêtre. `let ... else` : si le motif ne
        //    correspond pas (ici `None`), on exécute le bloc `else`, qui doit
        //    obligatoirement sortir de la fonction.
        let wait_start = Instant::now();
        let frame = {
            profiling::scope!("attente de l'image");
            gpu.acquire_frame()?
        };
        self.stats.wait_time = wait_start.elapsed();
        let Some(frame) = frame else {
            return Ok(()); // pas d'image disponible : on saute cette frame
        };

        // 2. Le depth buffer doit avoir la taille de la fenêtre : on le recrée
        //    si elle a changé (redimensionnement).
        if self.depth.size != gpu.size() {
            self.depth = DepthTexture::new(gpu.device(), gpu.size());
        }
        self.post.ensure_size(gpu.device(), gpu.size());

        // 3. Mettre à jour les données de l'image sur le GPU.
        let (width, height) = gpu.size();
        let aspect = width as f32 / height as f32;
        // Les tranches d'ombre suivent la caméra : recalculées à chaque image.
        self.cascades = compute_cascades(
            camera,
            aspect,
            self.lighting.direction,
            &self.shadow_settings,
            SHADOW_MAP_SIZE,
        );
        self.shadow_map
            .write_cascades(gpu.queue(), &self.cascades.map(|c| c.view_projection));
        let flag = |on: bool| if on { 1.0 } else { 0.0 };
        let view_projection = camera.view_projection(aspect);
        let light_count = self.upload_lights(gpu, &view_projection);
        let frame_uniform = FrameUniform::new(
            view_projection,
            camera,
            &self.lighting,
            &self.cascades,
            [
                flag(self.shadows_enabled),
                1.0 / SHADOW_MAP_SIZE as f32,
                flag(self.show_cascades),
                light_count as f32,
            ],
            [
                flag(self.environment_enabled),
                self.environment_intensity,
                self.environment.max_lod,
                0.0,
            ],
        );
        gpu.queue()
            .write_buffer(&self.frame_buffer, 0, bytemuck::bytes_of(&frame_uniform));
        let prepare_start = Instant::now();
        self.upload_instances(
            gpu,
            draws,
            camera.position,
            view_projection,
            // La plus grande tranche contient les autres : un objet hors
            // d'elle ne fait d'ombre dans aucune.
            self.cascades[CASCADE_COUNT - 1].view_projection,
        );
        self.stats.prepare_time = prepare_start.elapsed();
        self.line_buffer.write(
            gpu.device(),
            gpu.queue(),
            bytemuck::cast_slice(debug.vertices()),
        );

        // 4. Le « command encoder » : un carnet où l'on enregistre des commandes.
        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });

        // Les mesures GPU : un repère entre chaque groupe de passes (voir
        // gpu_timer.rs). `timer` vaut `None` si l'image n'est pas mesurée.
        let measured = self.gpu_timing_enabled
            && self
                .gpu_timer
                .as_mut()
                .is_some_and(|timer| timer.begin_frame(gpu.device()));
        let timer = self.gpu_timer.as_ref().filter(|_| measured);
        let mark = |encoder: &mut wgpu::CommandEncoder, index| {
            if let Some(timer) = timer {
                timer.mark(encoder, index);
            }
        };
        mark(&mut encoder, 0);

        // 5. Les passes d'ombre : les mêmes objets, vus depuis la lumière,
        //    une fois par tranche, chacune dans sa couche de la shadow map.
        //    Elles doivent être terminées AVANT la passe principale, qui lit
        //    leur résultat.
        self.stats.shadow_draw_calls = 0;
        for cascade in 0..CASCADE_COUNT {
            if !self.shadows_enabled {
                break;
            }
            let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow Pass"),
                // Aucune image de couleur : seule la profondeur nous intéresse.
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_map.layer_views[cascade],
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store, // la passe principale va la lire
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            // La matrice de CETTE tranche : un décalage dans le buffer des tranches.
            shadow_pass.set_bind_group(
                0,
                &self.shadow_map.cascade_bind_group,
                &[self.shadow_map.cascade_offset(cascade)],
            );
            shadow_pass.set_vertex_buffer(1, self.instance_buffer.buffer().slice(..));
            // Les textures ne servent qu'aux objets à découpe (leurs trous) :
            // pour les autres, seule la forme projette une ombre. Les
            // instances d'ombre sont rangées après celles de la passe
            // principale dans le buffer.
            let first = (self.lists.opaque.len() + self.lists.transparent.len()) as u32;
            let (calls, _) = self.draw_batches(
                &mut shadow_pass,
                &self.lists.shadow,
                first,
                &self.shadow_pipelines,
                |pipeline| pipeline == SHADOW_MASKED,
            );
            self.stats.shadow_draw_calls += calls;
        }
        mark(&mut encoder, 1);

        // 6. La passe principale : une séquence de dessins vers l'image HDR.
        //
        // Les accolades `{ ... }` créent une PORTÉE. La `render_pass` emprunte
        // `encoder` de façon exclusive ; elle doit donc être détruite (à la `}`)
        // avant qu'on puisse appeler `encoder.finish()`. Le borrow checker de
        // Rust vérifie cela pour nous.
        {
            let c = self.clear_color;
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    // La scène est dessinée dans l'image HDR, pas dans la
                    // fenêtre : le post-traitement l'y recopiera.
                    view: self.post.hdr_view(),
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Au début de la passe : remplir l'image avec la couleur de fond.
                        // (wgpu attend des f64, d'où les conversions `as f64`.)
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: c.r as f64,
                            g: c.g as f64,
                            b: c.b as f64,
                            a: c.a as f64,
                        }),
                        // À la fin : conserver le résultat (on veut l'afficher !).
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        // 1.0 = « infiniment loin » : le premier objet dessiné
                        // sur un pixel passe toujours le test de profondeur.
                        load: wgpu::LoadOp::Clear(1.0),
                        // Conservée pour de futures passes (post-traitement...).
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None, // notre format n'a pas de stencil
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Les bind groups restent en place quand on change de pipeline :
            // les 4 variantes déclarent les mêmes layouts.
            render_pass.set_bind_group(0, &self.frame_bind_group, &[]);
            render_pass.set_bind_group(2, &self.shadow_map.bind_group, &[]);
            render_pass.set_bind_group(3, &self.environment.bind_group, &[]);
            render_pass.set_vertex_buffer(1, self.instance_buffer.buffer().slice(..));
            let (calls, triangles) = self.draw_batches(
                &mut render_pass,
                &self.lists.opaque,
                0,
                &self.mesh_pipelines,
                |_| true,
            );
            self.stats.draw_calls = calls;
            self.stats.triangles = triangles;

            // Le ciel, APRÈS les objets : il ne remplit que les pixels restés
            // vides (voir sky.wgsl). Le même bind group d'environnement, placé
            // en n°1 pour ce pipeline.
            if self.environment_enabled {
                render_pass.set_pipeline(&self.sky_pipeline);
                render_pass.set_bind_group(1, &self.environment.bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }

            // Les objets TRANSPARENTS, après tout ce qui est opaque (le ciel
            // compris) : ils se mélangent avec ce qui est déjà dessiné
            // derrière eux. Ils arrivent triés du plus loin au plus proche.
            if !self.lists.transparent.is_empty() {
                let (calls, triangles) = self.draw_batches(
                    &mut render_pass,
                    &self.lists.transparent,
                    self.lists.opaque.len() as u32,
                    &self.mesh_pipelines,
                    |_| true,
                );
                self.stats.draw_calls += calls;
                self.stats.triangles += triangles;
            }

            // Les lignes de debug, APRÈS les maillages : le depth buffer est
            // alors rempli, et les lignes situées derrière un objet sont cachées.
            let line_vertices = debug.vertices().len() as u32;
            if line_vertices > 0 {
                render_pass.set_pipeline(&self.line_pipeline);
                // Le bind group n°0 (caméra) reste en place : les deux
                // pipelines déclarent le même layout pour ce groupe.
                render_pass.set_vertex_buffer(0, self.line_buffer.buffer().slice(..));
                render_pass.draw(0..line_vertices, 0..1);
            }
        } // <- `render_pass` est détruite ici, l'emprunt de `encoder` prend fin.
        mark(&mut encoder, 2);

        // 7. Le post-traitement : bloom, tone mapping et FXAA, de l'image
        //    HDR vers l'image de la fenêtre (voir postprocess.rs).
        {
            profiling::scope!("post-traitement");
            let settings = PostSettings {
                bloom_intensity: if self.bloom_enabled {
                    self.bloom_intensity
                } else {
                    0.0
                },
                bloom_threshold: self.bloom_threshold,
                tone_mapping: self.tone_mapping,
                fxaa: self.fxaa_enabled,
            };
            self.post
                .run(&mut encoder, gpu.queue(), &settings, &frame.view);
        }
        mark(&mut encoder, 3);

        // 8. L'overlay (interface...), par-dessus l'image terminée.
        let mut extra_commands = Vec::new();
        if let Some(overlay) = overlay {
            extra_commands = overlay.prepare(gpu.device(), gpu.queue(), &mut encoder, gpu.size());
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Overlay Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &frame.overlay_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            // `Load` : on GARDE l'image déjà dessinée.
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();
            overlay.draw(&mut pass);
        }
        mark(&mut encoder, 4);
        if let Some(timer) = timer {
            timer.resolve(&mut encoder);
        }

        // 9. Fermer le carnet et l'envoyer au GPU. C'est seulement maintenant
        //    que le GPU commence réellement à travailler. Les commandes de
        //    préparation de l'overlay passent en premier.
        gpu.queue().submit(
            extra_commands
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        // La lecture des mesures de cette image, quand le GPU aura fini.
        if measured && let Some(timer) = &mut self.gpu_timer {
            timer.after_submit();
        }

        // 10. Afficher l'image. `present` consomme `frame` : on ne peut plus l'utiliser.
        gpu.present(frame);
        Ok(())
    }

    /// Dessine les instances `items`, un appel de dessin par groupe
    /// (pipeline, textures, maillage). Utilisé par la passe d'ombre et la
    /// passe principale. Renvoie le nombre d'appels de dessin et de triangles.
    ///
    /// Les instances de même clé qui se suivent sont dessinées ensemble :
    /// `chunk_by` découpe le tableau en groupes consécutifs de même clé.
    /// Pour les opaques, triés par clé, cela fait un appel par combinaison.
    /// Pour les transparents, triés par distance, deux objets du même lot ne
    /// sont regroupés que s'ils se suivent : l'ordre de dessin est respecté.
    ///
    /// `first` : la place de `items[0]` dans le buffer d'instances.
    /// `pipelines` : les pipelines désignés par le 1er élément de la clé.
    /// `bind_textures(pipeline)` : ce pipeline lit-il les textures (bind
    /// group n°1) ? Non pour la passe d'ombre des objets pleins.
    fn draw_batches(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        items: &[Item],
        mut first: u32,
        pipelines: &[wgpu::RenderPipeline],
        bind_textures: impl Fn(u8) -> bool,
    ) -> (u32, u64) {
        let mut current_pipeline = None;
        let mut bound = None;
        let (mut calls, mut triangles) = (0, 0);
        for group in items.chunk_by(|a, b| a.0 == b.0) {
            let (pipeline, set, mesh) = group[0].0;
            if current_pipeline != Some(pipeline) {
                pass.set_pipeline(&pipelines[usize::from(pipeline)]);
                current_pipeline = Some(pipeline);
                // Un autre pipeline peut avoir un autre layout : on
                // redonnera les textures.
                bound = None;
            }
            if bind_textures(pipeline) && bound != Some(set) {
                // Le bind group existe : `upload_instances` l'a créé.
                pass.set_bind_group(1, &self.material_bind_groups[&set], &[]);
                bound = Some(set);
            }
            let mesh = &self.meshes[mesh.0];
            let count = group.len() as u32;
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // Tous les indices du maillage, pour les instances first..first+count.
            pass.draw_indexed(0..mesh.index_count, 0, first..first + count);
            first += count;
            calls += 1;
            triangles += u64::from(mesh.index_count / 3) * u64::from(count);
        }
        (calls, triangles)
    }

    /// Envoie au GPU les lumières locales qui touchent le champ de la
    /// caméra : celles dont la sphère de portée est entièrement hors de la
    /// vue n'éclairent aucun pixel visible. Renvoie leur nombre.
    fn upload_lights(&mut self, gpu: &GpuContext, view_projection: &Mat4) -> usize {
        let frustum = Frustum::from_view_projection(view_projection);
        self.light_data.clear();
        self.light_data.extend(
            self.lights
                .iter()
                .filter(|light| {
                    let r = Vec3::splat(light.range);
                    frustum.intersects(&Aabb::new(light.position - r, light.position + r))
                })
                .take(MAX_LOCAL_LIGHTS)
                .map(LightRaw::from),
        );
        if !self.light_data.is_empty() {
            gpu.queue().write_buffer(
                &self.light_buffer,
                0,
                bytemuck::cast_slice(&self.light_data),
            );
        }
        self.stats.lights = self.light_data.len();
        self.light_data.len()
    }

    /// Trie les commandes (culling, lots : voir `culling.rs`), copie leurs
    /// données dans `instance_buffer` (en l'agrandissant si nécessaire), et
    /// crée les bind groups des combinaisons de textures encore inconnues.
    fn upload_instances(
        &mut self,
        gpu: &GpuContext,
        draws: &[DrawCommand],
        camera_position: Vec3,
        view_projection: Mat4,
        light_view_projection: Mat4,
    ) {
        profiling::scope!("tri des objets");
        // Figer : on garde la matrice du moment où l'option a été cochée.
        if !self.freeze_culling {
            self.frozen_view_projection = None;
        } else if self.frozen_view_projection.is_none() {
            self.frozen_view_projection = Some(view_projection);
        }
        let camera = self.frozen_view_projection.unwrap_or(view_projection);
        let options = culling::Options {
            bounds: &self.mesh_bounds,
            culling: self.culling_enabled.then(|| Culling {
                camera: Frustum::from_view_projection(&camera),
                light: Frustum::from_view_projection(&light_view_projection),
            }),
            shadows: self.shadows_enabled,
            parallel: self.multithreading,
            camera_position,
        };
        culling::sort_draws(draws, &options, &mut self.lists);
        let lists = &self.lists;
        self.stats.objects = draws.len();
        self.stats.visible = lists.opaque.len() + lists.transparent.len();
        self.stats.transparent = lists.transparent.len();
        self.stats.shadow_casters = lists.shadow.len();

        // `chunk_by` : une fois par suite d'instances de mêmes textures.
        let all = [&lists.opaque, &lists.transparent, &lists.shadow];
        for group in all
            .into_iter()
            .flat_map(|l| l.chunk_by(|a, b| a.0.1 == b.0.1))
        {
            let set = group[0].0.1;
            if !self.material_bind_groups.contains_key(&set) {
                let views = set.map(|handle| &self.textures[handle.0].view);
                let bind_group = texture::create_material_bind_group(
                    gpu.device(),
                    &self.material_layout,
                    &self.sampler,
                    views,
                );
                self.material_bind_groups.insert(set, bind_group);
            }
        }

        // On n'envoie que les données des instances (pas les handles) : les
        // opaques, les transparents, puis la passe d'ombre.
        self.instance_data.clear();
        self.instance_data.extend(
            lists
                .opaque
                .iter()
                .chain(&lists.transparent)
                .chain(&lists.shadow)
                .map(|(_, raw)| *raw),
        );
        self.instance_buffer.write(
            gpu.device(),
            gpu.queue(),
            bytemuck::cast_slice(&self.instance_data),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taille_du_frame_uniform() {
        // Doit correspondre à `FrameUniform` dans les trois shaders :
        // 5 mat4 (5 × 64 octets) + 9 vec4 (9 × 16 octets).
        assert_eq!(std::mem::size_of::<FrameUniform>(), 464);
    }

    #[test]
    fn une_direction_de_lumiere_nulle_ne_produit_pas_de_nan() {
        let lighting = Lighting {
            direction: Vec3::ZERO,
            ..Lighting::default()
        };
        let uniform = FrameUniform::new(
            Mat4::IDENTITY,
            &Camera::default(),
            &lighting,
            &[Cascade {
                view_projection: Mat4::IDENTITY,
                far: 1.0,
                texel_size: 0.1,
            }; CASCADE_COUNT],
            [0.0; 4],
            [0.0; 4],
        );
        assert_eq!(uniform.light_direction, [0.0, -1.0, 0.0, 0.0]);
    }
}
