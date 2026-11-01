//! [`GpuContext`] : la connexion au GPU et à la fenêtre.

use std::sync::Arc;

use winit::window::Window;

use crate::error::RhiError;

/// Tout ce qu'il faut pour parler au GPU et afficher dans la fenêtre.
///
/// Créé une seule fois au démarrage, puis prêté (`&GpuContext`) au moteur de
/// rendu à chaque image.
pub struct GpuContext {
    // `Surface<'static>` : la surface pourrait vivre aussi longtemps que le
    // programme. C'est possible parce qu'on lui donne un `Arc<Window>` : elle
    // devient copropriétaire de la fenêtre, qui ne peut donc pas être détruite
    // avant elle. (La v1 du projet utilisait `Box::leak` pour obtenir ce
    // `'static`, ce qui « fuyait » volontairement la mémoire de la fenêtre.)
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    adapter_info: wgpu::AdapterInfo,
}

/// Une image prête à recevoir des dessins pour la frame en cours.
///
/// On dessine dans `view`, puis on appelle [`GpuContext::present`] pour l'afficher.
pub struct Frame {
    texture: wgpu::SurfaceTexture,
    /// La « vue » de la texture, c'est-à-dire la façon dont le GPU y accède.
    /// C'est elle qu'on donne aux passes de rendu comme cible.
    pub view: wgpu::TextureView,
    /// La MÊME image, vue au format non-sRGB ([`GpuContext::overlay_format`]).
    /// Pour les interfaces (egui) qui font elles-mêmes la conversion de
    /// couleurs : sans cela, le GPU la ferait une seconde fois et
    /// l'interface serait délavée.
    pub overlay_view: wgpu::TextureView,
}

impl GpuContext {
    /// Initialise le GPU pour la fenêtre donnée.
    ///
    /// # Pourquoi `async` ?
    ///
    /// Certaines étapes (trouver un GPU, ouvrir le device) peuvent prendre du
    /// temps, surtout sur le web. wgpu les expose donc comme des fonctions
    /// **asynchrones** : elles renvoient un `Future` (« une valeur qui sera
    /// disponible plus tard »), et `.await` attend ce résultat.
    ///
    /// Sur PC, l'appelant utilise simplement `pollster::block_on(...)` pour
    /// attendre le résultat sans installer de runtime async complet (comme tokio).
    pub async fn new(window: Arc<Window>) -> Result<Self, RhiError> {
        let size = window.inner_size();

        // --- 1. L'instance : le point d'entrée de wgpu -----------------------
        // `from_env` permet de choisir l'API avec une variable d'environnement,
        // par ex. `WGPU_BACKEND=vulkan cargo run` (ou dx12, metal, gl).
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());

        // --- 2. La surface : relie le GPU à notre fenêtre ---------------------
        // On donne un clone de l'`Arc` : la surface garde la fenêtre en vie.
        // Le `?` convertit l'éventuelle erreur en `RhiError::Surface` grâce au `#[from]`.
        let surface = instance.create_surface(Arc::clone(&window))?;

        // --- 3. L'adapter : choisir une carte graphique -----------------------
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // Préférer le GPU dédié (plus puissant) au GPU intégré.
                power_preference: wgpu::PowerPreference::HighPerformance,
                // Le GPU doit savoir afficher dans NOTRE surface.
                compatible_surface: Some(&surface),
                // `..Default::default()` : « tous les autres champs à leur valeur par défaut ».
                ..Default::default()
            })
            .await?;

        let adapter_info = adapter.get_info();
        log::info!(
            "GPU sélectionné : {} ({:?}, API {:?})",
            adapter_info.name,
            adapter_info.device_type,
            adapter_info.backend
        );

        // --- 4. Device + Queue : la connexion au GPU --------------------------
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("ZENTRIX Device"),
                // Une seule fonctionnalité optionnelle, et seulement si le GPU
                // la propose : les horodatages, pour mesurer le temps GPU de
                // chaque étape de l'image (voir gpu_timer.rs dans zentrix-render).
                // Les limites restent celles par défaut (compatibles avec
                // quasiment tous les GPU).
                required_features: adapter.features() & wgpu::Features::TIMESTAMP_QUERY,
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await?;
        log::info!(
            "Mesures GPU (horodatages) : {}",
            if device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
                "disponibles"
            } else {
                "non proposées par ce GPU"
            }
        );

        // --- 5. Configuration de la surface -----------------------------------
        let capabilities = surface.get_capabilities(&adapter);

        // On préfère un format **sRGB** : le GPU convertira automatiquement nos
        // couleurs « linéaires » vers l'espace de couleur de l'écran.
        // `.iter()` parcourt la liste, `.find(...)` renvoie le premier élément
        // qui satisfait la condition (dans une `Option`), `.copied()` copie la valeur.
        // `.or_else(...)` : sinon, on prend le premier format disponible.
        // `.ok_or(...)` : transforme l'`Option` en `Result` (None devient une erreur).
        let format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .or_else(|| capabilities.formats.first().copied())
            .ok_or(RhiError::NoSurfaceFormat)?;

        let config = wgpu::SurfaceConfiguration {
            // Les images de la surface serviront de cible de rendu.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            // `.max(1)` : une surface de taille 0 est interdite (fenêtre minimisée).
            width: size.width.max(1),
            height: size.height.max(1),
            // `Fifo` = V-Sync : on attend le rafraîchissement de l'écran.
            // Toujours supporté, pas de déchirement d'image, et le GPU ne chauffe pas inutilement.
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            // `Auto` : wgpu choisit l'espace colorimétrique standard (sRGB).
            // Les autres valeurs servent à l'affichage HDR, pour plus tard.
            color_space: wgpu::SurfaceColorSpace::Auto,
            // On autorise une seconde « vue » de l'image, au format non-sRGB
            // (même pixels, sans conversion de couleurs) : voir `Frame::overlay_view`.
            view_formats: vec![format.remove_srgb_suffix()],
            // Nombre d'images que le CPU peut préparer en avance sur le GPU.
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        log::info!(
            "Surface configurée : {}x{} {:?}",
            config.width,
            config.height,
            config.format
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            adapter_info,
        })
    }

    /// La synchronisation verticale (V-Sync) est-elle active ?
    pub fn vsync(&self) -> bool {
        self.config.present_mode == wgpu::PresentMode::Fifo
    }

    /// Active ou coupe la synchronisation verticale.
    ///
    /// Avec la V-Sync, une image attend le rafraîchissement de l'écran : le
    /// nombre d'images par seconde plafonne à la fréquence de l'écran (60 Hz
    /// en général), et le temps d'une image ne dit plus rien du travail du
    /// moteur. Sans elle, le moteur va aussi vite qu'il peut : utile pour
    /// **mesurer**, au prix d'un éventuel déchirement de l'image.
    ///
    /// `AutoNoVsync` choisit le meilleur mode sans attente disponible
    /// (`Immediate` ou `Mailbox`), et revient à `Fifo` s'il n'y en a aucun.
    pub fn set_vsync(&mut self, on: bool) {
        self.config.present_mode = if on {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        self.surface.configure(&self.device, &self.config);
        log::info!("V-Sync {}", if on { "activée" } else { "désactivée" });
    }

    /// Adapte la surface à la nouvelle taille de la fenêtre.
    ///
    /// À appeler à chaque `on_resize`. Une taille de 0 (fenêtre minimisée)
    /// est ignorée car interdite par wgpu.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Récupère la prochaine image de la surface, sur laquelle on va dessiner.
    ///
    /// Renvoie :
    /// - `Ok(Some(frame))` : on peut dessiner ;
    /// - `Ok(None)` : pas d'image cette fois (fenêtre cachée, délai dépassé...),
    ///   on saute simplement cette frame ;
    /// - `Err(...)` : problème grave, l'application doit s'arrêter.
    pub fn acquire_frame(&mut self) -> Result<Option<Frame>, RhiError> {
        // `get_current_texture` renvoie un enum : chaque variante décrit une
        // situation. Seules `Success` et `Suboptimal` contiennent une texture.
        // Notez que `match` est une EXPRESSION : sa valeur est rangée dans `texture`.
        let texture = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                // L'image est utilisable mais ne correspond plus parfaitement à
                // la fenêtre : on la reconfigure pour la frame suivante.
                self.surface.configure(&self.device, &self.config);
                texture
            }
            // La fenêtre est cachée ou le GPU met trop de temps : on saute la frame.
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(None);
            }
            // La surface est obsolète (ex. la fenêtre vient de changer de taille) :
            // on la reconfigure et on réessaiera à la frame suivante.
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(None);
            }
            // Tout autre cas (surface perdue, erreur de validation) est fatal.
            other => {
                log::error!("Échec d'acquisition de la surface : {other:?}");
                return Err(RhiError::SurfaceLost);
            }
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let overlay_view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.overlay_format()),
            ..Default::default()
        });
        Ok(Some(Frame {
            texture,
            view,
            overlay_view,
        }))
    }

    /// Affiche à l'écran une image obtenue avec [`GpuContext::acquire_frame`].
    ///
    /// `frame: Frame` (sans `&`) : la fonction **consomme** la frame. Après
    /// l'appel, la variable n'est plus utilisable — le compilateur empêche ainsi
    /// de dessiner dans une image déjà affichée. C'est le système de propriété
    /// de Rust qui garantit la logique de l'API !
    pub fn present(&self, frame: Frame) {
        // On détruit d'abord la vue (elle n'est plus utile), puis on remet la
        // texture à la queue, qui l'affichera une fois le GPU prêt.
        drop(frame.view);
        drop(frame.overlay_view);
        self.queue.present(frame.texture);
    }

    // -------------------------------------------------------------------------
    // Accesseurs (« getters »)
    // -------------------------------------------------------------------------
    // Les champs sont privés ; ces méthodes les PRÊTENT en lecture seule (`&`).
    // Ainsi personne ne peut, par exemple, remplacer le device par erreur.

    /// Le device, pour créer des ressources GPU (buffers, pipelines...).
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// La queue, pour envoyer du travail au GPU.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Le format des pixels de la surface (nécessaire pour créer les pipelines).
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Le format de [`Frame::overlay_view`] : celui de la surface, sans sRGB
    /// (par exemple `Bgra8Unorm` au lieu de `Bgra8UnormSrgb`).
    pub fn overlay_format(&self) -> wgpu::TextureFormat {
        self.config.format.remove_srgb_suffix()
    }

    /// La taille actuelle de la surface en pixels : `(largeur, hauteur)`.
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Informations sur le GPU utilisé (nom, type, API).
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }
}
