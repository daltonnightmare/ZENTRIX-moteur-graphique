//! Chargement des modèles 3D au format **glTF 2.0** (`.gltf` / `.glb`).
//!
//! glTF est le « JPEG de la 3D » : un format ouvert, exporté par Blender et
//! la plupart des outils. Un fichier contient une **scène** : un arbre de
//! **nœuds** (chacun avec sa transformation), dont certains portent un
//! **maillage**, fait de **primitives** (un morceau de géométrie + un matériau).
//!
//! ```text
//!   Scène
//!    └── nœud « Arbre »            (transformation)
//!         ├── nœud « Tronc »       maillage ─ primitive (géométrie + matériau « Écorce »)
//!         └── nœud « Feuillage »   maillage ─ primitive (géométrie + matériau « Aiguilles »)
//! ```
//!
//! Le chargeur **aplatit** cet arbre : chaque primitive devient une
//! [`ModelPart`] avec sa transformation calculée par rapport à la racine du
//! modèle. Bonne nouvelle : glTF utilise les mêmes conventions que ZENTRIX
//! (main droite, +Y en haut, faces avant dans le sens anti-horaire, UV avec
//! l'origine en haut à gauche) : aucune conversion n'est nécessaire.
//!
//! Les matériaux sont lus entièrement (facteurs et textures : couleur,
//! normal map, métal/rugosité, occlusion, émission). Limites actuelles :
//! seuls les triangles sont pris en compte ; les animations, les
//! squelettes, la transparence et les extensions de matériaux
//! (`KHR_materials_*`) sont ignorés. Toutes les textures utilisent le
//! premier jeu d'UV (`TEXCOORD_0`).

use std::path::Path;

use zentrix_core::math::Mat4;
use zentrix_core::{AlphaMode, Color, Image, Material, MeshData, Vertex};

use crate::AssetError;

/// Un modèle 3D chargé : ses morceaux et les images qu'ils utilisent.
#[derive(Debug, Clone, Default)]
pub struct Model {
    pub parts: Vec<ModelPart>,
    /// Les images du fichier, dans l'ordre du glTF. `ModelPart::texture` est
    /// un indice dans ce tableau.
    pub images: Vec<Image>,
}

/// Un morceau de modèle : une géométrie, sa place dans le modèle et sa texture.
#[derive(Debug, Clone)]
pub struct ModelPart {
    /// Le nom du nœud glTF (s'il en a un) : pratique pour retrouver « Roue avant ».
    pub name: Option<String>,
    /// La géométrie.
    pub mesh: MeshData,
    /// La place du morceau par rapport à la racine du modèle (produit des
    /// transformations de tous ses nœuds parents).
    pub transform: Mat4,
    /// Les images des textures du matériau (indices dans `Model::images`).
    pub textures: PartTextures,
    /// Les facteurs du matériau glTF (couleur de base, métal, rugosité,
    /// émission...).
    pub material: Material,
}

/// Les textures d'un morceau : pour chacune, l'indice de son image dans
/// `Model::images`, si elle existe. Mêmes rôles que `MaterialTextures`
/// (zentrix-render), qui attend des textures déjà envoyées au GPU.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PartTextures {
    pub base_color: Option<usize>,
    pub normal: Option<usize>,
    pub metallic_roughness: Option<usize>,
    pub occlusion: Option<usize>,
    pub emissive: Option<usize>,
}

impl PartTextures {
    /// Cette image est-elle une **couleur** (sRGB), et non des données ?
    /// Seules la couleur de base et l'émission le sont.
    pub fn is_color_image(&self, image: usize) -> bool {
        self.base_color == Some(image) || self.emissive == Some(image)
    }
}

/// Charge un fichier `.gltf` (avec ses fichiers annexes) ou `.glb`.
pub fn load_gltf(path: impl AsRef<Path>) -> Result<Model, AssetError> {
    let path = path.as_ref();
    let error = |source| AssetError::Gltf {
        path: path.to_path_buf(),
        source,
    };
    // `gltf::import` lit le fichier, les buffers binaires et décode les images.
    let (document, buffers, images) = gltf::import(path).map_err(error)?;
    let model = convert(&document, &buffers, &images);
    log::info!(
        "Modèle chargé : {} ({} morceaux, {} images)",
        path.display(),
        model.parts.len(),
        model.images.len()
    );
    Ok(model)
}

/// Charge un modèle glTF déjà en mémoire (fichier `.glb`, ou `.gltf` dont les
/// buffers sont intégrés en base64).
pub fn decode_gltf(bytes: &[u8]) -> Result<Model, gltf::Error> {
    let (document, buffers, images) = gltf::import_slice(bytes)?;
    Ok(convert(&document, &buffers, &images))
}

/// Parcourt la scène du document et convertit chaque primitive.
fn convert(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
) -> Model {
    let mut model = Model {
        parts: Vec::new(),
        images: images.iter().map(convert_image).collect(),
    };
    // La scène par défaut, sinon la première ; un fichier sans scène ne
    // contient que des ressources (rien à afficher).
    let Some(scene) = document
        .default_scene()
        .or_else(|| document.scenes().next())
    else {
        log::warn!("Fichier glTF sans scène : aucun objet à afficher");
        return model;
    };
    for node in scene.nodes() {
        visit_node(&node, Mat4::IDENTITY, buffers, &mut model);
    }
    model
}

/// Convertit un nœud et, récursivement, ses enfants.
fn visit_node(node: &gltf::Node, parent: Mat4, buffers: &[gltf::buffer::Data], model: &mut Model) {
    // Comme dans zentrix-scene : global = parent × local.
    let transform = parent * Mat4::from_cols_array_2d(&node.transform().matrix());

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                log::warn!(
                    "Primitive {:?} ignorée dans « {} » : seuls les triangles sont pris en charge",
                    primitive.mode(),
                    node.name().unwrap_or("?")
                );
                continue;
            }
            let Some(mesh) = convert_primitive(&primitive, buffers) else {
                log::warn!(
                    "Primitive sans positions ignorée dans « {} »",
                    node.name().unwrap_or("?")
                );
                continue;
            };
            let (textures, material) = convert_material(&primitive.material());
            model.parts.push(ModelPart {
                name: node.name().map(str::to_owned),
                mesh,
                transform,
                textures,
                material,
            });
        }
    }

    for child in node.children() {
        visit_node(&child, transform, buffers, model);
    }
}

/// Les textures et les facteurs d'un matériau glTF. Les facteurs glTF sont
/// déjà dans la convention de ZENTRIX (couleurs linéaires, métal et
/// rugosité entre 0 et 1) : aucune conversion.
fn convert_material(material: &gltf::Material) -> (PartTextures, Material) {
    let pbr = material.pbr_metallic_roughness();
    let normal = material.normal_texture();
    let occlusion = material.occlusion_texture();
    // L'indice de l'IMAGE derrière une texture glTF (une texture glTF =
    // une image + un sampler ; on utilise toujours notre propre sampler).
    let image = |texture: gltf::Texture| texture.source().index();

    let textures = PartTextures {
        base_color: pbr.base_color_texture().map(|t| image(t.texture())),
        normal: normal.as_ref().map(|t| image(t.texture())),
        metallic_roughness: pbr.metallic_roughness_texture().map(|t| image(t.texture())),
        occlusion: occlusion.as_ref().map(|t| image(t.texture())),
        emissive: material.emissive_texture().map(|t| image(t.texture())),
    };
    let [r, g, b, a] = pbr.base_color_factor();
    let [er, eg, eb] = material.emissive_factor();
    let material = Material {
        base_color: Color::rgba(r, g, b, a),
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        normal_scale: normal.as_ref().map_or(1.0, |n| n.scale()),
        emissive: Color::rgb(er, eg, eb),
        occlusion_strength: occlusion.as_ref().map_or(1.0, |o| o.strength()),
        alpha_mode: match material.alpha_mode() {
            gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
            // 0,5 : la valeur par défaut de glTF.
            gltf::material::AlphaMode::Mask => AlphaMode::Mask {
                cutoff: material.alpha_cutoff().unwrap_or(0.5),
            },
            gltf::material::AlphaMode::Blend => AlphaMode::Blend,
        },
        double_sided: material.double_sided(),
    };
    (textures, material)
}

/// Lit les attributs d'une primitive. `None` si elle n'a pas de positions.
fn convert_primitive(
    primitive: &gltf::Primitive,
    buffers: &[gltf::buffer::Data],
) -> Option<MeshData> {
    // Le « reader » sait lire les accesseurs glTF (types, décalages, pas)
    // dans les buffers ; on lui dit où trouver chaque buffer.
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    let positions: Vec<[f32; 3]> = reader.read_positions()?.collect();
    let count = positions.len();

    // Les attributs facultatifs : une valeur par défaut s'ils manquent.
    let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(Iterator::collect);
    // Même convention que ZENTRIX (xyz + signe w) : lues telles quelles.
    let tangents: Option<Vec<[f32; 4]>> = reader.read_tangents().map(Iterator::collect);
    let uvs: Vec<[f32; 2]> = reader
        .read_tex_coords(0)
        .map(|uv| uv.into_f32().collect())
        .unwrap_or_else(|| vec![[0.0; 2]; count]);
    let colors: Vec<[f32; 3]> = reader
        .read_colors(0)
        .map(|c| c.into_rgb_f32().collect())
        .unwrap_or_else(|| vec![[1.0; 3]; count]);

    let vertices = (0..count)
        .map(|i| {
            Vertex::new(
                positions[i],
                colors[i],
                uvs[i],
                normals.as_ref().map_or([0.0; 3], |n| n[i]),
            )
        })
        .collect();

    // Sans indices, les sommets se lisent trois par trois : 0, 1, 2, 3...
    let indices = reader
        .read_indices()
        .map(|i| i.into_u32().collect())
        .unwrap_or_else(|| (0..count as u32).collect());

    let mut mesh = MeshData { vertices, indices };
    if normals.is_none() {
        // La norme glTF demande alors des normales « plates » : c'est ce que
        // donne `compute_normals` quand les triangles ne partagent pas de sommets.
        mesh.compute_normals();
    }
    match tangents {
        Some(tangents) => {
            for (vertex, tangent) in mesh.vertices.iter_mut().zip(tangents) {
                vertex.tangent = tangent;
            }
        }
        // Absentes du fichier : calculées à partir des UV (après les normales).
        None => mesh.compute_tangents(),
    }
    Some(mesh)
}

/// Convertit une image glTF (décodée par la crate `gltf`) en `Image` RGBA 8 bits.
fn convert_image(data: &gltf::image::Data) -> Image {
    use gltf::image::Format;

    let expand = |channels: usize, to_rgba: fn(&[u8]) -> [u8; 4]| -> Vec<u8> {
        data.pixels
            .chunks_exact(channels)
            .flat_map(to_rgba)
            .collect()
    };
    let pixels = match data.format {
        Format::R8G8B8A8 => data.pixels.clone(),
        Format::R8G8B8 => expand(3, |p| [p[0], p[1], p[2], 255]),
        Format::R8G8 => expand(2, |p| [p[0], p[0], p[0], p[1]]), // gris + opacité
        Format::R8 => expand(1, |p| [p[0], p[0], p[0], 255]),
        other => {
            // 16 bits ou flottants : rares pour une couleur de base.
            log::warn!(
                "Format d'image glTF non pris en charge ({other:?}) : remplacée par du blanc"
            );
            return Image::new(1, 1, vec![255; 4]);
        }
    };
    Image::new(data.width, data.height, pixels)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zentrix_core::math::Vec3;

    /// Un glTF minimal : un triangle rouge, sans normales, sans UV, sans
    /// indices, dans un nœud décalé de +2 en X. Le buffer (3 positions =
    /// 36 octets) est intégré en base64.
    const TRIANGLE_GLTF: &str = r#"{
        "asset": { "version": "2.0" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "name": "Triangle", "mesh": 0, "translation": [2, 0, 0] }],
        "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0 }, "material": 0 }] }],
        "materials": [{ "pbrMetallicRoughness": { "baseColorFactor": [1, 0, 0, 1] } }],
        "accessors": [{
            "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3",
            "min": [0, 0, 0], "max": [1, 1, 0]
        }],
        "bufferViews": [{ "buffer": 0, "byteLength": 36 }],
        "buffers": [{
            "byteLength": 36,
            "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAA"
        }]
    }"#;

    #[test]
    fn triangle_minimal_avec_valeurs_par_defaut() {
        let model = decode_gltf(TRIANGLE_GLTF.as_bytes()).expect("glTF valide");
        assert_eq!(model.parts.len(), 1);
        let part = &model.parts[0];
        assert_eq!(part.name.as_deref(), Some("Triangle"));

        // Indices générés, normale calculée (+Z : triangle dans le plan XY,
        // sens anti-horaire), sommets blancs.
        assert_eq!(part.mesh.indices, [0, 1, 2]);
        assert_eq!(part.mesh.vertices[0].normal, [0.0, 0.0, 1.0]);
        assert_eq!(part.mesh.vertices[1].color, [1.0, 1.0, 1.0]);
        assert_eq!(part.textures, PartTextures::default());
        // Le matériau : la couleur du fichier, et les valeurs par défaut de
        // glTF pour ce qui manque (métal 1, rugosité 1).
        assert_eq!(part.material.base_color, Color::RED);
        assert_eq!(part.material.metallic, 1.0);
        assert_eq!(part.material.roughness, 1.0);
        // La transformation du nœud est conservée.
        assert_eq!(
            part.transform.transform_point3(Vec3::ZERO),
            Vec3::new(2.0, 0.0, 0.0)
        );
    }

    /// Un matériau qui utilise toutes les sortes de textures. Les 5 images
    /// sont le même PNG de 1 pixel, rangé dans le buffer binaire après les
    /// positions (comme dans un fichier `.glb`).
    const MATERIAL_GLTF: &str = r#"{
        "asset": { "version": "2.0" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0 }],
        "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0 }, "material": 0 }] }],
        "materials": [{
            "pbrMetallicRoughness": {
                "baseColorTexture": { "index": 0 },
                "metallicRoughnessTexture": { "index": 1 },
                "metallicFactor": 0.5, "roughnessFactor": 0.25
            },
            "normalTexture": { "index": 2, "scale": 0.8 },
            "occlusionTexture": { "index": 3, "strength": 0.6 },
            "emissiveTexture": { "index": 4 },
            "emissiveFactor": [1.0, 0.5, 0.0]
        }],
        "textures": [
            { "source": 0 }, { "source": 1 }, { "source": 2 }, { "source": 3 }, { "source": 4 }
        ],
        "images": [
            { "bufferView": 1, "mimeType": "image/png" },
            { "bufferView": 1, "mimeType": "image/png" },
            { "bufferView": 1, "mimeType": "image/png" },
            { "bufferView": 1, "mimeType": "image/png" },
            { "bufferView": 1, "mimeType": "image/png" }
        ],
        "accessors": [{
            "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3",
            "min": [0, 0, 0], "max": [1, 1, 0]
        }],
        "bufferViews": [
            { "buffer": 0, "byteLength": 36 },
            { "buffer": 0, "byteOffset": 36, "byteLength": 70 }
        ],
        "buffers": [{
            "byteLength": 106,
            "uri": "data:application/octet-stream;base64,AAAAAAAAAAAAAAAAAACAPwAAAAAAAAAAAAAAAAAAgD8AAAAAiVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="
        }]
    }"#;

    #[test]
    fn toutes_les_textures_et_facteurs_du_materiau() {
        let model = decode_gltf(MATERIAL_GLTF.as_bytes()).expect("glTF valide");
        assert_eq!(model.images.len(), 5);
        let part = &model.parts[0];
        assert_eq!(
            part.textures,
            PartTextures {
                base_color: Some(0),
                metallic_roughness: Some(1),
                normal: Some(2),
                occlusion: Some(3),
                emissive: Some(4),
            }
        );
        // Couleur (sRGB) : couleur de base et émission ; le reste : données.
        assert!(part.textures.is_color_image(0) && part.textures.is_color_image(4));
        assert!(!part.textures.is_color_image(2));
        let m = part.material;
        assert_eq!((m.metallic, m.roughness), (0.5, 0.25));
        assert_eq!(m.normal_scale, 0.8);
        assert_eq!(m.occlusion_strength, 0.6);
        assert_eq!(m.emissive, Color::rgb(1.0, 0.5, 0.0));
        // Pas de TANGENT dans le fichier : calculées à partir des UV (ici
        // toutes nulles : UV dégénérées, tangente inconnue).
        assert_eq!(part.mesh.vertices[0].tangent, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn json_invalide() {
        assert!(decode_gltf(b"{ pas du glTF").is_err());
    }

    #[test]
    fn l_arbre_de_la_demo() {
        let model = load_gltf(crate::asset_dir().join("models/arbre.glb")).expect("arbre.glb");
        assert_eq!(model.parts.len(), 2, "tronc + feuillage");
        assert_eq!(model.images.len(), 1, "texture d'écorce");

        let tronc = model
            .parts
            .iter()
            .find(|p| p.name.as_deref() == Some("Tronc"))
            .unwrap();
        assert_eq!(tronc.textures.base_color, Some(0));
        assert_eq!(tronc.material.metallic, 0.0, "l'écorce n'est pas un métal");
        assert_eq!(tronc.mesh.indices.len(), 6 * 6, "6 faces × 2 triangles");

        let feuillage = model
            .parts
            .iter()
            .find(|p| p.name.as_deref() == Some("Feuillage"))
            .unwrap();
        // Feuillage décalé de 0,85 vers le haut par son nœud.
        assert!((feuillage.transform.w_axis.y - 0.85).abs() < 1e-6);
        // Normales calculées par le chargeur : toutes de longueur 1.
        for v in &feuillage.mesh.vertices {
            assert!((Vec3::from(v.normal).length() - 1.0).abs() < 1e-5);
        }
    }
}
