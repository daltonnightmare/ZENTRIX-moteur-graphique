# 2026-11-01 — Changement majeur : performance (culling, profileur, Tracy, plusieurs threads)

## Résumé

Dernière piste de la feuille de route. Le moteur sait maintenant :
- **ne pas dessiner ce qui est hors du champ de vision** (*frustum culling*) ;
- ne mettre dans la shadow map **que les objets de la zone d'ombre** ;
- **mesurer** le temps de chaque étape de l'image : profileur intégré affiché dans
  l'inspecteur, et zones pour le profileur externe Tracy ;
- **répartir le tri des objets entre les cœurs** du processeur (au-delà de 1 024 objets).

Pour mettre tout cela à l'épreuve, la touche `N` ajoute un **champ de stress** de 4 096 cubes
qui tournent.

## Détail

### `zentrix-core`

| Fichier | Contenu |
|---------|---------|
| `bounds.rs` (nouveau) | `Aabb` : boîte alignée sur les axes ; `from_points` ; `transformed(matrice)` (méthode d'Arvo, sans transformer les 8 coins). `Frustum` : 6 plans extraits d'une matrice vue-projection (Gribb-Hartmann, profondeur 0..1 de wgpu), qui marche aussi en orthographique ; `intersects(aabb)` (test du coin « le plus en avant », prudent) |
| `profiler.rs` (nouveau) | `FrameProfiler`, un chronomètre à tours : `begin_frame`, puis `lap("étape")` à la fin de chaque étape. Moyenne glissante à 10 %, historique de 120 images. `add` et `transfer` détaillent un tour avec une durée mesurée ailleurs. Le temps entre deux images est compté dans « autre » : la somme des étapes vaut toujours la durée totale |
| `mesh.rs` | `MeshData::bounds()` |

### `zentrix-render`

| Élément | Changement |
|---------|------------|
| `culling.rs` (nouveau) | `sort_draws` range chaque commande dans la liste de la passe principale (vue par la caméra) et/ou celle de la passe d'ombre (dans la boîte de la lumière), puis trie les deux par lot. Au-delà de `PARALLEL_THRESHOLD` (1 024), le travail est réparti avec `rayon` (`par_chunks`, `par_sort_by_key`, stable) : le résultat est identique à celui d'un seul thread |
| Passe d'ombre | Sa propre liste d'instances, rangée après celle de la passe principale dans le même buffer. Les textures ne coupent plus ses lots (clé `NO_TEXTURES`) : moins d'appels de dessin |
| `Renderer` | Boîte englobante de chaque maillage, calculée à `add_mesh`. Champs publics `culling_enabled`, `freeze_culling` (fige la pyramide de la caméra, pour voir le culling en reculant ; `frozen_view_projection()` pour la dessiner), `multithreading`. `stats()` → `RenderStats` |
| `RenderStats` (nouveau, public) | Objets reçus, visibles, projetant une ombre ; appels de dessin (principal, ombre) ; triangles ; temps de tri et d'envoi ; **temps d'attente de l'image** (la V-Sync) |
| Tracy | `profiling::scope!` sur le rendu, l'attente de l'image et le tri |

### `zentrix-rhi`

`GpuContext::vsync()` / `set_vsync(bool)` : `Fifo` ou `AutoNoVsync`. Avec la V-Sync, les FPS
plafonnent à 60 et la durée d'une image ne mesure plus le travail du moteur.

### Tracy (toutes les crates)

- Workspace : dépendances `rayon = "1"` et `profiling = "1"`.
- Zones `profiling::scope!` dans `Scene::update_transforms`, `Scene::extract_draws`,
  `Physics::step`, `Audio::update`, `UiLayer::run`, et `profiling::finish_frame!()` dans la
  boucle de `zentrix-platform`.
- Feature `tracy` de zentrix-app : `cargo run --release --features tracy`. Le client
  `tracy-client-sys` 0.29 correspond à l'application **Tracy 0.14.0**. Sans la feature, les
  macros ne font rien.

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `main.rs` | `FrameProfiler` : tours « interface », « systèmes », « physique », « transforms », « audio », « extraction », « lignes de debug », « rendu », et « attente de l'écran » retirée du rendu. Touche `N`. Pyramide figée dessinée en orange. Constante `NDC_BOX` (partagée avec la boîte de la shadow map). Case V-Sync appliquée au `GpuContext`. `StressCube` enregistré pour la sauvegarde |
| `demo.rs` | `StressCube`, `spawn_stress_field` (64 × 64 cubes en nappe ondulée sur 160 m, `spawn_batch`), `remove_stress_field` |
| `inspector.rs` | Panneau « Performance » : FPS et ms par image, graphique des 120 dernières images (ligne à 16,7 ms, barres orange au-delà), temps moyen et part de chaque étape, compteurs du rendu, cases Culling / Figer / Plusieurs threads / V-Sync, bouton du champ de stress |

## Vérification

- `cargo test --workspace` : **131 tests** réussis. Nouveaux tests :
  - boîtes : points, transformation ;
  - pyramides : devant, derrière, à côté, trop loin, à cheval sur le bord, orthographique ;
  - profileur : ordre des étapes, moyenne glissante, transfert, somme égale au total,
    historique borné ;
  - tri : sans culling, objet derrière la caméra mais dans l'ombre, ombres sans textures,
    **plusieurs threads = un thread** (3 072 commandes) ;
  - démo : champ de stress, et le champ ne déplace pas la caméra.
- `cargo clippy --workspace --all-targets`, `cargo fmt --check` : propres.
- `cargo check --features tracy` : compile.
- Démo lancée, en **release**, sans V-Sync, champ de stress actif (4 161 objets, Intel Iris Xe,
  Vulkan) :

  | Réglage | Dessinés / ombres | Tri et envoi | Image | FPS |
  |---------|-------------------|--------------|-------|-----|
  | Culling + plusieurs threads | 1 387 / 188 | 0,43 ms | 3,96 ms | 251 |
  | Culling, un seul thread | 1 385 / 196 | 0,46 ms | 4,35 ms | 282 |
  | Sans culling | 4 161 / 4 161 | 2,08 ms | 5,84 ms | 193 |

  Les FPS varient d'une capture à l'autre de ±10 %.
  - Le **culling** fait gagner environ 1,9 ms par image : le rendu passe de 3,4 ms à 1,5 ms,
    et la passe d'ombre ne reçoit plus que 188 objets au lieu de 4 161.
  - Les **plusieurs threads** n'apportent **rien de mesurable** à cette échelle (voir les
    limites).

## Limites connues

- **Plusieurs threads** : seul le tri des objets est parallèle. Il ne coûte que 0,45 ms pour
  4 000 objets, et le partage du travail coûte presque autant qu'il fait gagner. Le gain
  apparaîtrait avec des dizaines de milliers d'objets. La physique, les systèmes et l'ECS
  restent sur un seul thread.
- **Mesures** : elles sont faites sur le CPU uniquement. Le temps passé par le GPU lui-même
  n'est pas mesuré (il faudrait des *timestamp queries*).
- **Culling** : il est prudent. Les boîtes droites autour d'objets tournés sont un peu trop
  grandes, et un objet caché **derrière** un autre est quand même dessiné (pas d'*occlusion
  culling*).
- **Interface** : avec 4 000 entités, egui reste l'étape la plus coûteuse du CPU (environ
  1,4 ms en release).
- **Tracy** : la feature compile, mais je n'ai pas lancé l'application Tracy pour voir les zones.
