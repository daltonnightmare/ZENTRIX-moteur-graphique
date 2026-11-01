//! Des modèles 3D chargés aux entités de la scène.
//!
//! ```text
//!   arbre.glb ──load_gltf──▶ Model ──upload──▶ ModelHandles ──spawn──▶ entités
//!               (assets)     (CPU)   (GPU)     (handles)      (scène)  racine + 1 enfant par morceau
//! ```
//!
//! On sépare `upload` (une fois par fichier) de `spawn` (une fois par
//! exemplaire) : dix arbres partagent les mêmes maillages et textures sur le
//! GPU, seules leurs entités diffèrent.
//!
//! Ce code relie trois crates (assets, render, scene) : il vit pour l'instant
//! dans l'application. Un futur « gestionnaire de ressources » du moteur
//! pourra l'accueillir.

use zentrix_assets::Model;
use zentrix_core::{Material, Transform};
use zentrix_render::{MaterialTextures, MeshHandle, Renderer, TextureHandle};
use zentrix_rhi::GpuContext;
use zentrix_scene::{AssetNames, Entity, MeshRenderer, Name, Parent, hecs};

/// Un modèle dont les ressources sont sur le GPU, prêt à être instancié.
#[derive(Debug, Clone, Default)]
pub struct ModelHandles {
    parts: Vec<PartHandles>,
}

#[derive(Debug, Clone)]
struct PartHandles {
    name: Option<String>,
    mesh: MeshHandle,
    textures: MaterialTextures,
    material: Material,
    /// Place du morceau dans le modèle (relative à la racine).
    transform: Transform,
}

/// Envoie les maillages et les images d'un modèle sur le GPU, et les
/// inscrit dans l'annuaire `names` sous le préfixe `prefix` :
/// `"arbre/Tronc"`, `"arbre/image0"`... (noms utilisés par les scènes sauvegardées).
pub fn upload(
    renderer: &mut Renderer,
    gpu: &GpuContext,
    model: &Model,
    prefix: &str,
    names: &mut AssetNames,
) -> ModelHandles {
    let textures: Vec<TextureHandle> = model
        .images
        .iter()
        .enumerate()
        .map(|(i, image)| {
            // Seules la couleur de base et l'émission sont des couleurs
            // (sRGB). Les autres images (normal map, métal/rugosité,
            // occlusion) contiennent des données : pas de conversion sRGB.
            let is_color = model.parts.iter().any(|p| p.textures.is_color_image(i));
            let handle = if is_color {
                renderer.add_texture(gpu, image)
            } else {
                renderer.add_data_texture(gpu, image)
            };
            names.add_texture(format!("{prefix}/image{i}"), handle);
            handle
        })
        .collect();
    // `.and_then(|i| textures.get(i))` : un indice invalide (fichier abîmé)
    // donne simplement « pas de texture » au lieu d'un panic.
    let handle = |index: Option<usize>| index.and_then(|i| textures.get(i).copied());
    let parts = model
        .parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            let mesh = renderer.add_mesh(gpu, &part.mesh);
            // Un morceau sans nom dans le fichier est désigné par son numéro.
            let part_name = part.name.clone().unwrap_or_else(|| format!("morceau{i}"));
            names.add_mesh(format!("{prefix}/{part_name}"), mesh);
            PartHandles {
                name: part.name.clone(),
                mesh,
                textures: MaterialTextures {
                    base_color: handle(part.textures.base_color),
                    normal: handle(part.textures.normal),
                    metallic_roughness: handle(part.textures.metallic_roughness),
                    occlusion: handle(part.textures.occlusion),
                    emissive: handle(part.textures.emissive),
                },
                material: part.material,
                transform: Transform::from_matrix(part.transform),
            }
        })
        .collect();
    ModelHandles { parts }
}

/// Crée un exemplaire du modèle dans le monde : une entité racine placée en
/// `transform`, et un enfant par morceau. Renvoie la racine.
///
/// Déplacer, tourner ou supprimer (`Scene::despawn_recursive`) la racine agit
/// sur le modèle entier, grâce à la hiérarchie.
pub fn spawn(
    world: &mut hecs::World,
    model: &ModelHandles,
    name: &str,
    transform: Transform,
) -> Entity {
    let root = world.spawn((Name::new(name), transform));
    for part in &model.parts {
        let renderer = MeshRenderer {
            mesh: part.mesh,
            textures: part.textures,
            material: part.material,
        };
        let part_name = part.name.as_deref().unwrap_or("morceau");
        world.spawn((
            Name::new(format!("{name} / {part_name}")),
            part.transform,
            renderer,
            Parent(root),
        ));
    }
    root
}

#[cfg(test)]
mod tests {
    use super::*;
    use zentrix_core::math::Vec3;
    use zentrix_scene::{GlobalTransform, Scene};

    #[test]
    fn un_exemplaire_par_appel_avec_ses_morceaux() {
        let model = ModelHandles {
            parts: vec![
                PartHandles {
                    name: Some("Tronc".into()),
                    mesh: MeshHandle::from_index(0),
                    textures: MaterialTextures::color(TextureHandle::from_index(1)),
                    material: Material::DEFAULT,
                    transform: Transform::IDENTITY,
                },
                PartHandles {
                    name: Some("Feuillage".into()),
                    mesh: MeshHandle::from_index(1),
                    textures: MaterialTextures::default(),
                    material: Material::DEFAULT,
                    transform: Transform::from_translation(Vec3::Y),
                },
            ],
        };
        let mut scene = Scene::default();
        spawn(
            &mut scene.world,
            &model,
            "Arbre A",
            Transform::from_translation(Vec3::X * 5.0),
        );
        spawn(&mut scene.world, &model, "Arbre B", Transform::IDENTITY);
        assert_eq!(scene.entity_count(), 6, "2 racines + 2 × 2 morceaux");

        // Le feuillage du premier arbre est à 5 unités en X et 1 en Y.
        scene.update_transforms();
        let feuillage_a = scene
            .world
            .query::<(&Name, &GlobalTransform)>()
            .iter()
            .find(|(name, _)| name.0 == "Arbre A / Feuillage")
            .map(|(_, global)| global.translation())
            .unwrap();
        assert_eq!(feuillage_a, Vec3::new(5.0, 1.0, 0.0));
    }
}
