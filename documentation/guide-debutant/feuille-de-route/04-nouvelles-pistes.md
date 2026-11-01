# Feuille de route : Phase 4 — Nouvelles pistes

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Chapitre 7](../07-feuille-de-route.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 5 et 6 — Nanite et Lumen ➡️](05-06-nanite-et-lumen.md)

---

## Phase 4 — Nouvelles pistes

Toutes les pistes du [chapitre 7](../07-feuille-de-route.md#et-ensuite--pistes-pour-la-suite) sont réalisées. Les suivantes sont regroupées par ce qu'elles
apportent, dans l'ordre conseillé à l'intérieur de chaque groupe.

### Ce qui se voit à l'écran

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Transparence** ✅ | Verre, eau, feuillage découpé : `alpha` du matériau, textures avec canal alpha, *alpha test* (découpe) et *alpha blending* (mélange). Les objets transparents sont dessinés **après** les opaques, triés de l'arrière vers l'avant, sans écrire dans le depth buffer — réalisée : `AlphaMode` (opaque, découpe, mélange) et `double_sided` dans `Material` (lus depuis glTF), alpha prémultiplié (le verre garde ses reflets), 4 variantes du pipeline des maillages, tri par distance ([culling.rs](../../../crates/zentrix-render/src/culling.rs)), ombres trouées pour la découpe ; démo : vitre, cubes de verre, lanterne grillagée. Pistes suivantes : transparence indépendante de l'ordre (OIT), réfraction, ombres colorées du verre | ⭐⭐ |
| **Post-traitement** ✅ | Dessiner d'abord dans une texture HDR, puis des passes plein écran : **bloom** (les zones très lumineuses « bavent »), **anti-aliasing** (FXAA ou MSAA : bords sans escaliers), tone mapping déplacé dans cette passe — réalisé : image HDR `Rgba16Float`, bloom à la Call of Duty (seuil doux, 6 niveaux de réduction puis remontée additive), tone mapping et FXAA en passes finales ([postprocess.rs](../../../crates/zentrix-render/src/postprocess.rs), [post.wgsl](../../../crates/zentrix-render/src/shaders/post.wgsl)) ; réglages dans l'inspecteur. Pistes suivantes : MSAA, exposition automatique, profondeur de champ, flou de mouvement, SSAO | ⭐⭐⭐ |
| **Ombres en cascade (CSM)** ✅ | Plusieurs shadow maps de plus en plus grandes selon la distance : ombres nettes près de la caméra, présentes jusqu'au loin (aujourd'hui, une seule zone de 23 m) — réalisées : 3 tranches jusqu'à 50 m ([cascades.rs](../../../crates/zentrix-render/src/cascades.rs)), sphères englobantes et calage sur les texels contre le scintillement, fondu entre tranches, décalage selon la normale, debug « Tranches ». Pistes suivantes : culling par tranche, ombres douces (PCSS), mise à jour moins fréquente des tranches lointaines | ⭐⭐⭐ |
| **Lumières ponctuelles et projecteurs** ✅ | Lampes et spots colorés, avec atténuation selon la distance ; plusieurs lumières par image (storage buffer) ; les néons éclairent vraiment la scène — réalisées : composants `PointLight` et `SpotLight` (sauvegardés), `LocalLight` côté rendu, jusqu'à 256 lumières par image dans un storage buffer, atténuation en 1/d² avec fenêtre, cône à bord doux, culling des lumières hors de la vue ; démo : flamme de la lanterne, néons, manège de lampes colorées, projecteur, touche `L` (nuit). Pistes suivantes : ombres des lampes (cube maps), *clustered shading* pour des centaines de lampes, lumières surfaciques | ⭐⭐⭐ |

### Ce qui rend le moteur « jouable »

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Contrôleur de personnage** ✅ | Marcher sur le sol, sauter, monter des marches, glisser le long des murs (`KinematicCharacterController` de rapier), caméra à la première ou à la troisième personne — réalisé : composant `CharacterController` et forme `Capsule` dans [zentrix-physics](../../../crates/zentrix-physics/src/lib.rs) (déplacement à chaque image dans `Physics::step` : gravité, saut, marches de 35 cm, pentes de 45°, glissement le long des murs, caisses poussées selon la masse), `cast_ray_excluding` pour viser depuis les yeux ; mode marche à la première personne dans [player.rs](../../../crates/zentrix-app/src/player.rs) (touche `C`, `Espace` pour sauter, `Maj` pour courir, retour au départ en cas de chute). Pistes suivantes : caméra à la troisième personne (Phase 24.1), s'accroupir, plateformes mobiles qui emportent le personnage, mémoire tampon du saut (Phase 15.4) | ⭐⭐ |
| **Petit gameplay** ✅ | Un mini-jeu dans le moteur (ramasser des objets, toucher des cibles avec les balles, score à l'écran) : le meilleur test de l'API du moteur — réalisé : la « chasse aux cristaux » ([game.rs](../../../crates/zentrix-app/src/game.rs), touche `G`) : 8 cristaux lumineux à ramasser en passant dessus, 5 cibles à toucher avec les balles (les chocs de `Physics::impacts`), 60 secondes, bonus de temps, record, HUD egui avec viseur ; logique testée sans GPU (événements `GameEvent`). Pistes suivantes : contrôleur de personnage (marcher au lieu de voler), carte des entrées (Phase 15), interface de jeu (Phase 16) | ⭐⭐ |
| **Animation de modèles** | Animations glTF : images clés (position, rotation), puis squelette et *skinning* pour des personnages | ⭐⭐⭐⭐ |

### Ce qui en fait un vrai outil

| Piste | Contenu | Difficulté |
|-------|---------|------------|
| **Éditeur** | Sélection par clic dans la scène (lancer de rayon), *gizmos* pour déplacer / tourner / agrandir à la souris, création et suppression d'entités, annuler / refaire (Ctrl+Z) | ⭐⭐⭐ |
| **Rechargement à chaud** | Modifier un shader `.wgsl`, une texture ou une scène et voir le résultat sans relancer (surveillance des fichiers avec la crate `notify`) | ⭐⭐ |
| **Mesures GPU** ✅ | *Timestamp queries* : le temps de chaque passe sur la carte graphique, affiché à côté du profileur CPU — réalisées : [gpu_timer.rs](../../../crates/zentrix-render/src/gpu_timer.rs) (option `TIMESTAMP_QUERY` demandée quand le GPU la propose, 5 repères entre les groupes de passes : ombres, passe principale, post-traitement, interface ; lecture sans attente grâce à un anneau de 3 buffers ; moyennes glissantes), `Renderer::gpu_timings`, lignes « GPU · » dans le panneau Performance (désactivables). Sur l'Iris Xe : environ 5,3 ms par image, dont 1 ms d'ombres et 2,7 ms de passe principale. Pistes suivantes : détail des passes du post-traitement, historique graphique du temps GPU | ⭐⭐ |

---

[⬅️ Chapitre 7](../07-feuille-de-route.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 5 et 6 — Nanite et Lumen ➡️](05-06-nanite-et-lumen.md)
