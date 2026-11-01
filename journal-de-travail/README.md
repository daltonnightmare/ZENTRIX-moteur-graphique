# Journal de travail — ZENTRIX

Trace datée de tout ce qui change dans le moteur : un fichier `.md` par changement.

## Convention de nommage

```text
AAAA-MM-JJ_<type>_<sujet-court>.md
```

| Type | Quand l'utiliser |
|------|------------------|
| `changement-majeur` | Nouvelle fonctionnalité, nouvelle architecture, changement d'API publique |
| `modification` | Changement de comportement ou de code existant, sans nouvelle fonctionnalité |
| `amelioration` | Performance, lisibilité, robustesse, tests, documentation enrichie |
| `suppression` | Code, fichier, fonctionnalité ou dépendance retiré(e) |

Si plusieurs fichiers ont la même date et le même type, on les distingue par le sujet.
L'ordre alphabétique des noms de fichiers donne donc l'ordre chronologique.

## Contenu d'une entrée

1. **Résumé** : une ou deux phrases.
2. **Pourquoi** : la motivation.
3. **Détail** : fichiers touchés et ce qui a changé.
4. **Vérification** : comment on a validé (tests, lancement, capture...).
5. **Impact / suite** : ce que ça casse ou débloque, prochaines étapes.

## Index

| Date | Type | Entrée |
|------|------|--------|
| 2026-07-13 | changement-majeur | [Création de la fenêtre et du projet](2026-07-13_changement-majeur_creation-fenetre-et-projet.md) |
| 2026-10-18 | changement-majeur | [Passage à un workspace multi-crates](2026-10-18_changement-majeur_workspace-multi-crates.md) |
| 2026-10-18 | suppression | [Ancien programme monolithique `src/`](2026-10-18_suppression_programme-monolithique-src.md) |
| 2026-10-31 | changement-majeur | [Rendu 3D : maillages, caméra, profondeur, instanciation](2026-10-31_changement-majeur_rendu-3d-maillages-camera-profondeur.md) |
| 2026-10-31 | suppression | [Triangle codé en dur et fond animé](2026-10-31_suppression_triangle-code-en-dur-et-fond-anime.md) |
| 2026-10-31 | modification | [Documentation mise à jour pour la Phase 2](2026-10-31_modification_documentation-phase-2.md) |
| 2026-10-31 | amelioration | [Création du journal de travail](2026-10-31_amelioration_journal-de-travail.md) |
| 2026-10-31 | changement-majeur | [Entrées clavier/souris et caméra libre](2026-10-31_changement-majeur_entrees-et-camera-libre.md) |
| 2026-10-31 | modification | [Commandes de la démo](2026-10-31_modification_commandes-de-la-demo.md) |
| 2026-10-31 | suppression | [Caméra en orbite automatique](2026-10-31_suppression_camera-en-orbite.md) |
| 2026-10-31 | changement-majeur | [Textures et crate `zentrix-assets`](2026-10-31_changement-majeur_textures-et-crate-assets.md) |
| 2026-10-31 | modification | [API de `Vertex` / `DrawCommand`, message des commandes](2026-10-31_modification_api-vertex-drawcommand-et-logs.md) |
| 2026-10-31 | changement-majeur | [Rendu de debug : lignes, grille, axes, boîtes](2026-10-31_changement-majeur_rendu-de-debug.md) |
| 2026-10-31 | amelioration | [Buffer GPU extensible commun, FPS dans le titre](2026-10-31_amelioration_buffer-extensible-et-fps.md) |
| 2026-10-31 | changement-majeur | [Scène et ECS (`zentrix-scene`)](2026-10-31_changement-majeur_scene-et-ecs.md) |
| 2026-10-31 | suppression | [État « manuel » de la démo](2026-10-31_suppression_etat-manuel-de-la-demo.md) |
| 2026-10-31 | changement-majeur | [Hiérarchie de transformations](2026-10-31_changement-majeur_hierarchie-de-transformations.md) |
| 2026-10-31 | modification | [Extraction en position monde, boîtes de debug généralisées](2026-10-31_modification_extraction-position-monde.md) |
| 2026-10-31 | changement-majeur | [Éclairage : normales, lumière directionnelle, Lambert + Blinn-Phong](2026-10-31_changement-majeur_eclairage.md) |
| 2026-10-31 | amelioration | [Validation des shaders WGSL dans les tests](2026-10-31_amelioration_validation-des-shaders.md) |
| 2026-10-31 | modification | [API : `Vertex` avec normale, `FrameUniform`, lumière de la scène](2026-10-31_modification_api-vertex-normale-et-frame-uniform.md) |
| 2026-10-31 | changement-majeur | [Modèles 3D glTF](2026-10-31_changement-majeur_modeles-gltf.md) |
| 2026-10-31 | modification | [`Vertex` et `MeshData` déplacés dans `zentrix-core`](2026-10-31_modification_geometrie-dans-le-noyau.md) |
| 2026-10-31 | changement-majeur | [Ombres portées (shadow mapping)](2026-10-31_changement-majeur_ombres-portees.md) |
| 2026-10-31 | modification | [Rendu en deux passes, `FrameUniform` étendu](2026-10-31_modification_rendu-en-deux-passes.md) |
| 2026-10-31 | changement-majeur | [Sauvegarde et chargement de scènes (RON)](2026-10-31_changement-majeur_sauvegarde-des-scenes.md) |
| 2026-10-31 | changement-majeur | [Interface de debug egui et inspecteur](2026-10-31_changement-majeur_interface-egui-inspecteur.md) |
| 2026-10-31 | modification | [Événements bruts, overlay de rendu, vue non-sRGB](2026-10-31_modification_evenements-overlay-vue-non-srgb.md) |
| 2026-10-31 | changement-majeur | [Matériaux PBR (métal / rugosité)](2026-10-31_changement-majeur_materiaux-pbr.md) |
| 2026-10-31 | changement-majeur | [Normal maps et tone mapping](2026-10-31_changement-majeur_normal-maps-et-tone-mapping.md) |
| 2026-10-31 | modification | [`Vertex` à 60 octets, `params` dans `FrameUniform`](2026-10-31_modification_vertex-tangente-et-params.md) |
| 2026-10-31 | changement-majeur | [Textures de matériau complètes et environnement (IBL)](2026-10-31_changement-majeur_textures-materiau-et-environnement.md) |
| 2026-10-31 | modification | [`MaterialTextures`, bind groups réorganisés](2026-10-31_modification_material-textures-et-bind-groups.md) |
| 2026-10-31 | changement-majeur | [Physique (rapier3d) : crate zentrix-physics](2026-10-31_changement-majeur_physique-rapier.md) |
| 2026-11-01 | changement-majeur | [Audio spatialisé (kira) : crate zentrix-audio](2026-11-01_changement-majeur_audio-kira.md) |
| 2026-11-01 | amelioration | [Physique : détection des chocs (`Physics::impacts`)](2026-11-01_amelioration_physique-detection-des-chocs.md) |
| 2026-11-01 | changement-majeur | [Performance : culling, profileur, Tracy, plusieurs threads](2026-11-01_changement-majeur_performance-culling-profileur.md) |
| 2026-11-01 | amelioration | [Propagation des transforms et liste des entités : 10 et 5 fois plus rapides](2026-11-01_amelioration_transforms-et-liste-des-entites.md) |
| 2026-11-01 | modification | [Feuille de route : Phase 4, nouvelles pistes](2026-11-01_modification_feuille-de-route-phase-4.md) |
| 2026-11-01 | changement-majeur | [Transparence : découpe, mélange, double face](2026-11-01_changement-majeur_transparence.md) |
| 2026-11-01 | changement-majeur | [Post-traitement : image HDR, bloom, tone mapping, FXAA](2026-11-01_changement-majeur_post-traitement.md) |
| 2026-11-01 | changement-majeur | [Ombres en cascade (CSM)](2026-11-01_changement-majeur_ombres-en-cascade.md) |
| 2026-11-01 | changement-majeur | [Lampes et projecteurs (`PointLight`, `SpotLight`)](2026-11-01_changement-majeur_lampes-et-projecteurs.md) |
| 2026-11-01 | modification | [Feuille de route : Phase 5, géométrie virtualisée](2026-11-01_modification_feuille-de-route-phase-5-nanite.md) |
| 2026-11-01 | modification | [Feuille de route : Phase 6, éclairage global](2026-11-01_modification_feuille-de-route-phase-6-lumen.md) |
| 2026-11-01 | modification | [Feuille de route : Phase 7, mondes, gameplay, outils et plateformes](2026-11-01_modification_feuille-de-route-phase-7.md) |
| 2026-11-01 | modification | [Feuille de route : Phase 8, lancer de rayons logiciel](2026-11-01_modification_feuille-de-route-phase-8-raytracing.md) |
| 2026-11-01 | modification | [Feuille de route : cap « beau sur petit matériel », Phases 9 et 10](2026-11-01_modification_feuille-de-route-phases-9-10.md) |
| 2026-11-01 | modification | [Feuille de route : Phases 11 à 14, animation, cinématiques, types de mondes, 2D et 3D](2026-11-01_modification_feuille-de-route-phases-11-14.md) |
| 2026-11-01 | modification | [Feuille de route : Phases 15 à 17, entrées, fenêtres et interface, icônes, Blender et rigs](2026-11-01_modification_feuille-de-route-phases-15-17.md) |
| 2026-11-01 | modification | [Feuille de route : Phases 18 à 20, masse et gravité, saisons, PNJ](2026-11-01_modification_feuille-de-route-phases-18-20.md) |
| 2026-11-01 | modification | [Feuille de route : Phases 21 à 23, effets spéciaux, apprentissage automatique, états et destruction](2026-11-01_modification_feuille-de-route-phases-21-23.md) |
| 2026-11-01 | modification | [Découpage de la feuille de route en petits fichiers](2026-11-01_modification_decoupage-feuille-de-route.md) |
| 2026-11-01 | modification | [Feuille de route : Phases 24 à 27 et petites pistes](2026-11-01_modification_feuille-de-route-phases-24-27.md) |
| 2026-11-01 | amelioration | [Petit gameplay de test : la chasse aux cristaux](2026-11-01_amelioration_petit-gameplay-chasse-aux-cristaux.md) |
| 2026-11-01 | changement-majeur | [Contrôleur de personnage et mode marche](2026-11-01_changement-majeur_controleur-de-personnage.md) |
| 2026-11-01 | amelioration | [Intégration continue (GitHub Actions)](2026-11-01_amelioration_integration-continue.md) |
| 2026-11-01 | amelioration | [Mesures GPU (horodatages)](2026-11-01_amelioration_mesures-gpu.md) |

> Les entrées du 2026-07-13 et du 2026-10-18 ont été reconstituées a posteriori à partir de
> l'historique git (`git log --stat`).
