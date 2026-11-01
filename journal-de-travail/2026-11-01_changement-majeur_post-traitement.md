# 2026-11-01 — Changement majeur : post-traitement (image HDR, bloom, tone mapping, FXAA)

## Résumé

La scène n'est plus dessinée directement dans la fenêtre. Elle est d'abord dessinée dans une
**image HDR**, où les couleurs peuvent dépasser 1 (un néon vaut 4, un reflet du soleil bien
plus). Des passes « plein écran » produisent ensuite l'image affichée :

1. le **bloom** : ce qui dépasse un seuil de luminosité déborde en halo (néons, flamme de la
   lanterne, soleil, reflets) ;
2. la **composition** : l'image HDR et le bloom sont additionnés, puis le **tone mapping**
   ramène l'image entre 0 et 1. Le tone mapping était jusqu'ici fait dans chaque shader ;
3. le **FXAA** : il adoucit les bords en escalier.

C'était la piste « Post-traitement » de la Phase 4.

## Détail

### `zentrix-render`

| Élément | Changement |
|---------|------------|
| `shaders/post.wgsl` (nouveau) | Triangle plein écran (`vs_main`, sans vertex buffer). `fs_prefilter` : réduction 13 lectures (Jimenez) et seuil à genou doux, valeur bornée à 64 contre les pixels isolés qui clignotent. `fs_downsample`. `fs_upsample` : filtre tente 3 × 3. `fs_composite` : HDR + moyenne du bloom × intensité, puis tone mapping Khronos PBR Neutral, déplacé ici. `fs_fxaa` : FXAA de Lottes, sur la luminosité perçue |
| `postprocess.rs` (nouveau) | `HDR_FORMAT = Rgba16Float`. `PostProcess` : 5 pipelines plein écran, un bind group commun (2 textures, sampler linéaire, uniform), les images qui dépendent de la taille (HDR, LDR pour le FXAA, bloom à mi-taille avec jusqu'à 6 niveaux de mipmap, une vue par niveau). Elles sont recréées par `ensure_size` quand la fenêtre change. L'agrandissement **ajoute** chaque niveau au précédent (mélange `One + One`). `run` enchaîne les passes. Sans FXAA, la composition écrit directement dans la fenêtre |
| `renderer.rs` | Voir le détail sous le tableau |
| `mesh.wgsl`, `sky.wgsl` | Plus de tone mapping : ils produisent des couleurs HDR. `FrameUniform.params.z` est inutilisé |
| Tests | Validation naga de `post.wgsl`, nombre de niveaux du bloom, taille de l'uniform |

Changements de `renderer.rs` :
- les pipelines des maillages, du ciel et des lignes écrivent au format HDR ;
- la passe principale dessine dans l'image HDR, puis le post-traitement est appelé avant
  l'overlay (egui reste net, sans bloom ni FXAA) ;
- nouveaux champs publics : `bloom_enabled`, `bloom_intensity` (0,8), `bloom_threshold` (1,0),
  `fxaa_enabled` ;
- `tone_mapping` agit maintenant sur la passe de composition.

### Conséquence pour la transparence

Le verre se mélange désormais **dans l'image HDR**, sur de vraies intensités lumineuses, et non
plus sur l'image déjà ramenée entre 0 et 1. C'était une limite notée dans l'entrée sur la
[transparence](2026-11-01_changement-majeur_transparence.md).

### `zentrix-app`

`inspector.rs` : cases « Bloom » et « FXAA », curseurs « halo » (intensité) et « seuil ».

## Vérification

- `cargo test --workspace` : **139 tests** réussis.
- `cargo clippy --workspace --all-targets` et `cargo fmt --check` : propres.
- Démo lancée (Vulkan, Intel Iris Xe), aucune erreur de validation. Captures comparées, avec
  et sans les effets :
  - avec bloom : halo autour du soleil, des néons des caisses sci-fi et de la flamme de la
    lanterne ;
  - avec FXAA : les bords du sapin et des caisses sont lissés ;
  - sans les deux : bords nets en escalier, aucun halo.

## Limites connues

- **Coût GPU non mesuré** : le profileur ne mesure que le CPU. Le post-traitement ajoute une
  dizaine de passes plein écran et une image HDR de 8 octets par pixel. C'est raisonnable,
  mais à confirmer avec les *timestamp queries* (piste « Mesures GPU »).
- **FXAA** : il floute un peu les détails fins (textures, grillage). Il ne remplace pas le
  MSAA sur les arêtes très fines, et ne résout pas la disparition du grillage au loin.
- **Exposition fixe** : pas d'adaptation automatique à la luminosité de la scène, comme l'œil
  qui s'habitue à l'obscurité.
- **Lignes de debug** : elles passent aussi par le bloom et le FXAA. Une ligne très claire
  peut donc briller un peu.
