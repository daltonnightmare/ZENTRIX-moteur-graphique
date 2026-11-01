//! Sauvegarde et chargement des scènes, au format **RON**.
//!
//! RON (*Rusty Object Notation*) ressemble à la syntaxe de Rust : un fichier
//! de scène se lit et se modifie à la main.
//!
//! ```text
//! (
//!     version: 1,
//!     ambient_light: (r: 0.16, g: 0.18, b: 0.24, a: 1.0),
//!     entities: [
//!         (
//!             id: 3,
//!             parent: Some(2),
//!             mesh: Some((
//!                 mesh: "arbre/Tronc",
//!                 texture: Some("arbre/image0"),
//!                 normal_map: Some("arbre/image1"),
//!                 // aussi : metallic_roughness, occlusion, emissive
//!                 material: (metallic: 0.0, roughness: 1.0, ...),
//!             )),
//!             components: {
//!                 "Name": ("Arbre 1 / Tronc"),
//!                 "Transform": (translation: (0.0, 0.0, 0.0), ...),
//!             },
//!         ),
//!     ],
//! )
//! ```
//!
//! # Trois difficultés
//!
//! 1. **Les handles** (`MeshHandle`, `TextureHandle`) sont des numéros
//!    internes au `Renderer`, différents à chaque exécution : on écrit à la
//!    place des **noms**, grâce à l'annuaire [`AssetNames`].
//! 2. **Les entités** (`Entity`) sont aussi des numéros internes : les liens
//!    `Parent` sont renumérotés (`id` local au fichier), puis reconstruits.
//! 3. **Les composants du jeu** (`Spin`...) sont inconnus du moteur : le jeu
//!    les déclare avec [`SceneSerializer::register`].
//!
//! Ce qui n'est pas déclaré n'est pas sauvegardé (par exemple
//! `GlobalTransform`, recalculé à chaque image).

use std::collections::{BTreeMap, HashMap};

use hecs::{Entity, World};
use ron::value::RawValue;
use serde::Serialize;
use serde::de::DeserializeOwned;
use zentrix_core::{Color, Material, Transform};
use zentrix_render::{MaterialTextures, MeshHandle, TextureHandle};

use crate::{Camera3d, DirectionalLight, MeshRenderer, Name, Parent, PointLight, Scene, SpotLight};

/// Version du format de fichier. À augmenter à chaque changement incompatible.
const SCENE_VERSION: u32 = 1;

/// Une erreur de sauvegarde ou de chargement de scène.
#[derive(Debug, thiserror::Error)]
pub enum SceneError {
    #[error("fichier de scène invalide : {0}")]
    Parse(#[from] ron::error::SpannedError),
    #[error("impossible d'écrire la scène : {0}")]
    Write(#[from] ron::Error),
    #[error("version de scène {0} non prise en charge (attendue : {SCENE_VERSION})")]
    Version(u32),
    #[error("entité {id}, composant « {component} » : {message}")]
    Component {
        id: u32,
        component: String,
        message: String,
    },
    #[error("entité {0} : son maillage n'a pas de nom dans l'annuaire AssetNames")]
    UnnamedMesh(u32),
    #[error("entité {0} : sa texture n'a pas de nom dans l'annuaire AssetNames")]
    UnnamedTexture(u32),
    #[error("maillage inconnu : « {0} »")]
    UnknownMesh(String),
    #[error("texture inconnue : « {0} »")]
    UnknownTexture(String),
    #[error("entité {id} : son parent {parent} n'existe pas dans le fichier")]
    UnknownParent { id: u32, parent: u32 },
}

// ---------------------------------------------------------------------------
// L'annuaire des ressources
// ---------------------------------------------------------------------------

/// L'annuaire « nom ↔ handle » des ressources chargées dans le `Renderer`.
///
/// L'application y inscrit chaque maillage et texture qu'elle charge
/// (`"cube"`, `"caisse"`, `"arbre/Tronc"`...). Une scène sauvegardée ne
/// contient que ces noms ; au chargement, ils redeviennent des handles,
/// même si les numéros ont changé entre deux exécutions.
#[derive(Debug, Default, Clone)]
pub struct AssetNames {
    meshes: HashMap<String, MeshHandle>,
    textures: HashMap<String, TextureHandle>,
}

impl AssetNames {
    pub fn add_mesh(&mut self, name: impl Into<String>, handle: MeshHandle) {
        self.meshes.insert(name.into(), handle);
    }

    pub fn add_texture(&mut self, name: impl Into<String>, handle: TextureHandle) {
        self.textures.insert(name.into(), handle);
    }

    pub fn mesh(&self, name: &str) -> Option<MeshHandle> {
        self.meshes.get(name).copied()
    }

    pub fn texture(&self, name: &str) -> Option<TextureHandle> {
        self.textures.get(name).copied()
    }

    /// Le nom d'un maillage (recherche inverse, en parcourant l'annuaire :
    /// suffisant pour quelques centaines de ressources).
    pub fn mesh_name(&self, handle: MeshHandle) -> Option<&str> {
        self.meshes
            .iter()
            .find(|(_, h)| **h == handle)
            .map(|(name, _)| name.as_str())
    }

    pub fn texture_name(&self, handle: TextureHandle) -> Option<&str> {
        self.textures
            .iter()
            .find(|(_, h)| **h == handle)
            .map(|(name, _)| name.as_str())
    }
}

// ---------------------------------------------------------------------------
// Le format du fichier
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
struct SceneFile {
    version: u32,
    ambient_light: Color,
    entities: Vec<EntityData>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EntityData {
    /// Numéro de l'entité DANS LE FICHIER (pas son `Entity` en mémoire).
    id: u32,
    /// `#[serde(default, skip_serializing_if = ...)]` : le champ est omis du
    /// fichier quand il est vide, et vaut `None` s'il est absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mesh: Option<MeshRef>,
    /// Les autres composants, sous forme de texte RON brut, rangés par nom.
    /// `BTreeMap` (et non `HashMap`) : les clés restent triées, le fichier
    /// est identique d'une sauvegarde à l'autre.
    #[serde(default)]
    components: BTreeMap<String, Box<RawValue>>,
}

/// Un `MeshRenderer` dans le fichier : des noms au lieu des handles.
#[derive(serde::Serialize, serde::Deserialize)]
struct MeshRef {
    mesh: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    texture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    normal_map: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metallic_roughness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    occlusion: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    emissive: Option<String>,
    /// Absent du fichier = matériau par défaut : les fichiers écrits avant
    /// l'arrivée des matériaux se chargent toujours.
    #[serde(default, skip_serializing_if = "is_default_material")]
    material: Material,
}

fn is_default_material(material: &Material) -> bool {
    *material == Material::DEFAULT
}

/// Le nom d'une texture facultative (erreur si elle n'est pas dans l'annuaire).
fn texture_name(
    names: &AssetNames,
    texture: Option<TextureHandle>,
    id: u32,
) -> Result<Option<String>, SceneError> {
    texture
        .map(|t| {
            names
                .texture_name(t)
                .map(str::to_owned)
                .ok_or(SceneError::UnnamedTexture(id))
        })
        // `transpose` : Option<Result<..>> → Result<Option<..>>, pour `?`.
        .transpose()
}

/// La texture d'un nom facultatif (erreur si le nom est inconnu).
fn texture_handle(
    names: &AssetNames,
    name: &Option<String>,
) -> Result<Option<TextureHandle>, SceneError> {
    name.as_ref()
        .map(|n| {
            names
                .texture(n)
                .ok_or_else(|| SceneError::UnknownTexture(n.clone()))
        })
        .transpose()
}

// ---------------------------------------------------------------------------
// Le sérialiseur
// ---------------------------------------------------------------------------

type SaveFn = Box<dyn Fn(&World, Entity) -> Option<Result<Box<RawValue>, ron::Error>>>;
type LoadFn = Box<dyn Fn(&mut World, Entity, &RawValue) -> Result<(), ron::error::SpannedError>>;

/// Comment sauvegarder et recharger UN type de composant.
struct ComponentCodec {
    save: SaveFn,
    load: LoadFn,
}

/// Sait sauvegarder et charger une [`Scene`].
///
/// ```
/// use zentrix_core::Transform;
/// use zentrix_scene::{AssetNames, Scene, SceneSerializer};
///
/// // Un composant du jeu, sérialisable.
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct Vie(u32);
///
/// let mut serializer = SceneSerializer::default();
/// serializer.register::<Vie>("Vie");
///
/// let mut scene = Scene::default();
/// scene.world.spawn((Transform::IDENTITY, Vie(100)));
/// let texte = serializer.save(&scene, &AssetNames::default()).unwrap();
///
/// let mut copie = Scene::default();
/// serializer.load(&mut copie, &texte, &AssetNames::default()).unwrap();
/// assert_eq!(copie.world.query::<&Vie>().iter().next().unwrap().0, 100);
/// ```
pub struct SceneSerializer {
    /// `BTreeMap` : les composants sont toujours traités dans le même ordre.
    codecs: BTreeMap<String, ComponentCodec>,
}

impl Default for SceneSerializer {
    /// Un sérialiseur qui connaît les composants du moteur. `Parent` et
    /// `MeshRenderer` sont traités à part (renumérotation et noms).
    fn default() -> Self {
        let mut serializer = Self {
            codecs: BTreeMap::new(),
        };
        serializer
            .register::<Name>("Name")
            .register::<Transform>("Transform")
            .register::<Camera3d>("Camera3d")
            .register::<DirectionalLight>("DirectionalLight")
            .register::<PointLight>("PointLight")
            .register::<SpotLight>("SpotLight");
        serializer
    }
}

impl SceneSerializer {
    /// Déclare un type de composant à sauvegarder, sous le nom `name`
    /// (le nom écrit dans le fichier ; ne pas le changer ensuite, sinon les
    /// anciens fichiers ne le retrouveront plus).
    ///
    /// Le type doit implémenter `Serialize` et `Deserialize` (avec
    /// `#[derive(serde::Serialize, serde::Deserialize)]`).
    pub fn register<T>(&mut self, name: &str) -> &mut Self
    where
        T: hecs::Component + Serialize + DeserializeOwned,
    {
        let codec = ComponentCodec {
            // `world.get::<&T>(e).ok()` : `None` si l'entité n'a pas ce composant.
            save: Box::new(|world, entity| {
                world
                    .get::<&T>(entity)
                    .ok()
                    .map(|component| to_raw(&*component))
            }),
            load: Box::new(|world, entity, raw| {
                let component: T = raw.into_rust()?;
                // L'entité vient d'être créée par `load` : elle existe forcément.
                world
                    .insert_one(entity, component)
                    .expect("entité créée par le chargement");
                Ok(())
            }),
        };
        self.codecs.insert(name.to_owned(), codec);
        self
    }

    /// Convertit toute la scène en texte RON.
    pub fn save(&self, scene: &Scene, names: &AssetNames) -> Result<String, SceneError> {
        let world = &scene.world;

        // Les entités, triées pour un fichier stable, et leur numéro dans le fichier.
        let mut entities: Vec<Entity> = world.iter().map(|e| e.entity()).collect();
        entities.sort_by_key(|e| e.id());
        let ids: HashMap<Entity, u32> = entities
            .iter()
            .enumerate()
            .map(|(i, &e)| (e, i as u32))
            .collect();

        let mut file = SceneFile {
            version: SCENE_VERSION,
            ambient_light: scene.ambient_light,
            entities: Vec::with_capacity(entities.len()),
        };

        for (&entity, id) in entities.iter().zip(0u32..) {
            let mut components = BTreeMap::new();
            for (name, codec) in &self.codecs {
                if let Some(raw) = (codec.save)(world, entity) {
                    components.insert(name.clone(), raw?);
                }
            }

            // Un parent qui n'existe plus est simplement oublié.
            let parent = world
                .get::<&Parent>(entity)
                .ok()
                .and_then(|p| ids.get(&p.0).copied());

            let mesh = match world.get::<&MeshRenderer>(entity) {
                Ok(renderer) => Some(MeshRef {
                    mesh: names
                        .mesh_name(renderer.mesh)
                        .ok_or(SceneError::UnnamedMesh(id))?
                        .to_owned(),
                    texture: texture_name(names, renderer.textures.base_color, id)?,
                    normal_map: texture_name(names, renderer.textures.normal, id)?,
                    metallic_roughness: texture_name(
                        names,
                        renderer.textures.metallic_roughness,
                        id,
                    )?,
                    occlusion: texture_name(names, renderer.textures.occlusion, id)?,
                    emissive: texture_name(names, renderer.textures.emissive, id)?,
                    material: renderer.material,
                }),
                Err(_) => None,
            };

            file.entities.push(EntityData {
                id,
                parent,
                mesh,
                components,
            });
        }

        // « Pretty » : indentation et retours à la ligne, pour un fichier lisible.
        let config = ron::ser::PrettyConfig::new().depth_limit(3);
        Ok(ron::ser::to_string_pretty(&file, config)?)
    }

    /// Ajoute à la scène les entités décrites par `text` (les entités déjà
    /// présentes sont conservées : appeler `scene.world.clear()` avant pour
    /// remplacer la scène). Renvoie les entités créées.
    ///
    /// Le chargement est **tout ou rien** : en cas d'erreur, les entités
    /// déjà créées sont supprimées et la scène reste telle qu'avant.
    pub fn load(
        &self,
        scene: &mut Scene,
        text: &str,
        names: &AssetNames,
    ) -> Result<Vec<Entity>, SceneError> {
        let file: SceneFile = ron::from_str(text)?;
        if file.version != SCENE_VERSION {
            return Err(SceneError::Version(file.version));
        }

        // 1. Créer toutes les entités, vides : un enfant peut apparaître dans
        //    le fichier AVANT son parent.
        let spawned: HashMap<u32, Entity> = file
            .entities
            .iter()
            .map(|data| (data.id, scene.world.spawn(())))
            .collect();

        // 2. Leur donner leurs composants. En cas d'erreur : tout annuler.
        let result = self.fill_entities(&mut scene.world, &file, &spawned, names);
        if let Err(error) = result {
            for &entity in spawned.values() {
                let _ = scene.world.despawn(entity);
            }
            return Err(error);
        }

        scene.ambient_light = file.ambient_light;
        Ok(file.entities.iter().map(|d| spawned[&d.id]).collect())
    }

    fn fill_entities(
        &self,
        world: &mut World,
        file: &SceneFile,
        spawned: &HashMap<u32, Entity>,
        names: &AssetNames,
    ) -> Result<(), SceneError> {
        for data in &file.entities {
            let entity = spawned[&data.id];

            for (name, raw) in &data.components {
                let Some(codec) = self.codecs.get(name) else {
                    // Compatibilité : un fichier plus récent, ou un composant
                    // retiré du jeu. On l'ignore au lieu de tout refuser.
                    log::warn!("Composant inconnu « {name} » ignoré (entité {})", data.id);
                    continue;
                };
                (codec.load)(world, entity, raw).map_err(|e| SceneError::Component {
                    id: data.id,
                    component: name.clone(),
                    message: e.to_string(),
                })?;
            }

            if let Some(parent) = data.parent {
                let &parent_entity = spawned.get(&parent).ok_or(SceneError::UnknownParent {
                    id: data.id,
                    parent,
                })?;
                world
                    .insert_one(entity, Parent(parent_entity))
                    .expect("entité créée par le chargement");
            }

            if let Some(mesh) = &data.mesh {
                let renderer = MeshRenderer {
                    mesh: names
                        .mesh(&mesh.mesh)
                        .ok_or_else(|| SceneError::UnknownMesh(mesh.mesh.clone()))?,
                    textures: MaterialTextures {
                        base_color: texture_handle(names, &mesh.texture)?,
                        normal: texture_handle(names, &mesh.normal_map)?,
                        metallic_roughness: texture_handle(names, &mesh.metallic_roughness)?,
                        occlusion: texture_handle(names, &mesh.occlusion)?,
                        emissive: texture_handle(names, &mesh.emissive)?,
                    },
                    material: mesh.material,
                };
                world
                    .insert_one(entity, renderer)
                    .expect("entité créée par le chargement");
            }
        }
        Ok(())
    }
}

/// Convertit une valeur en texte RON « brut », qui sera recopié tel quel
/// dans le fichier.
fn to_raw<T: Serialize>(value: &T) -> Result<Box<RawValue>, ron::Error> {
    let text = ron::to_string(value)?;
    RawValue::from_boxed_ron(text.into_boxed_str()).map_err(|e| e.code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zentrix_core::math::{Quat, Vec3};

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Vie(u32);

    fn serializer() -> SceneSerializer {
        let mut s = SceneSerializer::default();
        s.register::<Vie>("Vie");
        s
    }

    fn names(cube: usize, bois: usize) -> AssetNames {
        let mut names = AssetNames::default();
        names.add_mesh("cube", MeshHandle::from_index(cube));
        names.add_texture("bois", TextureHandle::from_index(bois));
        names
    }

    /// Une petite scène : une caméra, une racine et un enfant visible.
    fn scene_de_test() -> Scene {
        let mut scene = Scene {
            ambient_light: Color::rgb(0.1, 0.2, 0.3),
            ..Scene::default()
        };
        scene.world.spawn((
            Name::new("Caméra"),
            Transform::from_translation(Vec3::new(0.0, 2.0, 5.0)),
            Camera3d::default(),
        ));
        let racine = scene.world.spawn((
            Name::new("Racine"),
            Transform::IDENTITY.with_rotation(Quat::from_rotation_y(0.5)),
            Vie(42),
        ));
        scene.world.spawn((
            Name::new("Enfant"),
            Transform::from_translation(Vec3::X),
            MeshRenderer::new(MeshHandle::from_index(0))
                .with_textures(MaterialTextures {
                    base_color: Some(TextureHandle::from_index(1)),
                    normal: Some(TextureHandle::from_index(1)),
                    emissive: Some(TextureHandle::from_index(1)),
                    ..MaterialTextures::default()
                })
                .with_material(Material::metal(Color::rgb(1.0, 0.8, 0.3), 0.2)),
            Parent(racine),
        ));
        scene
    }

    fn entite(scene: &Scene, nom: &str) -> Entity {
        scene
            .world
            .query::<(Entity, &Name)>()
            .iter()
            .find(|(_, name)| name.0 == nom)
            .map(|(e, _)| e)
            .expect(nom)
    }

    #[test]
    fn aller_retour_complet() {
        let texte = serializer().save(&scene_de_test(), &names(0, 1)).unwrap();
        assert!(texte.contains("\"Caméra\""), "{texte}");

        // Au rechargement, les handles ont d'autres numéros : seuls les noms comptent.
        let mut copie = Scene::default();
        let creees = serializer().load(&mut copie, &texte, &names(7, 9)).unwrap();
        assert_eq!(creees.len(), 3);
        assert_eq!(copie.ambient_light, Color::rgb(0.1, 0.2, 0.3));

        let enfant = entite(&copie, "Enfant");
        let racine = entite(&copie, "Racine");
        // Le lien de parenté pointe vers la NOUVELLE entité racine.
        assert_eq!(copie.world.get::<&Parent>(enfant).unwrap().0, racine);
        let rendu = *copie.world.get::<&MeshRenderer>(enfant).unwrap();
        assert_eq!(rendu.mesh, MeshHandle::from_index(7));
        let bois = Some(TextureHandle::from_index(9));
        assert_eq!(rendu.textures.base_color, bois);
        assert_eq!(rendu.textures.normal, bois);
        assert_eq!(rendu.textures.emissive, bois);
        assert_eq!(rendu.textures.occlusion, None);
        assert_eq!(
            rendu.material,
            Material::metal(Color::rgb(1.0, 0.8, 0.3), 0.2)
        );
        // Les composants du moteur et du jeu sont restitués à l'identique.
        assert_eq!(*copie.world.get::<&Vie>(racine).unwrap(), Vie(42));
        assert_eq!(
            copie.world.get::<&Transform>(racine).unwrap().rotation,
            Quat::from_rotation_y(0.5)
        );
        assert!(copie.camera().is_some());
    }

    #[test]
    fn un_fichier_sans_materiau_donne_le_materiau_par_defaut() {
        let texte = r#"(
            version: 1,
            ambient_light: (r: 0.1, g: 0.1, b: 0.1, a: 1.0),
            entities: [(id: 0, mesh: Some((mesh: "cube")))],
        )"#;
        let mut scene = Scene::default();
        serializer().load(&mut scene, texte, &names(0, 1)).unwrap();
        let rendu = scene.world.query::<&MeshRenderer>().iter().next().copied();
        assert_eq!(rendu.unwrap().material, Material::DEFAULT);
    }

    #[test]
    fn deux_sauvegardes_donnent_le_meme_texte() {
        let scene = scene_de_test();
        let a = serializer().save(&scene, &names(0, 1)).unwrap();
        let b = serializer().save(&scene, &names(0, 1)).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn un_maillage_sans_nom_ne_peut_pas_etre_sauvegarde() {
        let erreur = serializer()
            .save(&scene_de_test(), &AssetNames::default())
            .unwrap_err();
        assert!(matches!(erreur, SceneError::UnnamedMesh(_)), "{erreur}");
    }

    #[test]
    fn chargement_tout_ou_rien() {
        let texte = serializer().save(&scene_de_test(), &names(0, 1)).unwrap();
        let mut copie = Scene::default();
        let existante = copie.world.spawn((Name::new("Déjà là"),));

        // L'annuaire ne connaît pas « cube » : erreur, et rien n'a été ajouté.
        let erreur = serializer()
            .load(&mut copie, &texte, &AssetNames::default())
            .unwrap_err();
        assert!(matches!(erreur, SceneError::UnknownMesh(_)), "{erreur}");
        assert_eq!(copie.entity_count(), 1);
        assert!(copie.world.contains(existante));
    }

    #[test]
    fn composant_inconnu_ignore() {
        // Un fichier qui contient « Vie », relu par un sérialiseur qui ne le connaît pas.
        let texte = serializer().save(&scene_de_test(), &names(0, 1)).unwrap();
        let mut copie = Scene::default();
        SceneSerializer::default()
            .load(&mut copie, &texte, &names(0, 1))
            .unwrap();
        assert_eq!(copie.entity_count(), 3);
    }

    #[test]
    fn version_inconnue_refusee() {
        let texte = "(version: 99, ambient_light: (r: 0, g: 0, b: 0, a: 1), entities: [])";
        let erreur = serializer()
            .load(&mut Scene::default(), texte, &AssetNames::default())
            .unwrap_err();
        assert!(matches!(erreur, SceneError::Version(99)), "{erreur}");
    }
}
