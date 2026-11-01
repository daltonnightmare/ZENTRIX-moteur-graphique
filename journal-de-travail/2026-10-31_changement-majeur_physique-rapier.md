# 2026-10-31 — Changement majeur : physique (rapier3d), crate `zentrix-physics`

## Résumé

Les objets ont maintenant une **physique**, calculée par
[rapier](https://rapier.rs) 0.36 : gravité, collisions, rebonds, frottements.

- **Composants.** On donne un `RigidBody` (dynamique, fixe ou cinématique) et un `Collider` (une
  boîte ou une sphère) à une entité. La nouvelle crate `zentrix-physics` se charge du reste et
  met à jour les `Transform`.
- **Démo.**
  - Une **pyramide de 10 caisses** dynamiques se dresse derrière la grille.
  - `B` **lance une balle** depuis la caméra.
  - Le **clic droit** (souris capturée) pousse l'objet visé, à l'aide d'un lancer de rayon.
  - `F2` dessine les formes de collision.

## Pourquoi

C'était la piste « Physique » de la feuille de route. Écrire un moteur physique stable
(empilements, rotations, frottements) est un travail énorme. rapier est écrit en Rust, rapide et
bien maintenu : on l'intègre au lieu de le réécrire, comme egui pour l'interface.

## Vérification préalable (projet jetable)

Dans un projet jetable, compilé dans `target/essai`, j'ai vérifié :
- **Version de glam** : rapier 0.36 → parry 0.31 → glamx 0.3 → **glam 0.33**, alors que ZENTRIX
  utilise glam 0.34. Les deux versions coexistent, mais leurs types sont distincts :
  `zentrix-physics` convertit à la frontière avec `to_array` / `from_array`, sans coût.
- **API** : rapier 0.36 fournit `PhysicsWorld`, qui regroupe tous les ensembles (corps, formes,
  phases de détection...). Son `step` prend aussi un ensemble de corps mous (`SoftBodySet`).
- **Dossier de compilation** : le scratchpad (dossier temporaire) refusait l'écriture des
  exécutables de build par l'éditeur de liens (`LNK1104`).

## Détail

### Nouvelle crate `zentrix-physics`

**`components.rs`**

| Élément | Rôle |
|---------|------|
| `BodyKind` | `Dynamic` (déplacé par la physique), `Fixed` (immobile), `Kinematic` (déplacé par le jeu, il pousse les autres) |
| `RigidBody { kind, initial_velocity }` | Composant sérialisable (serde, `#[serde(default)]`) ; `dynamic()`, `fixed()`, `kinematic()`, `with_velocity` |
| `Shape` | `Cuboid { half_extents }` ou `Ball { radius }` |
| `Collider { shape, offset, friction, restitution, density }` | Composant sérialisable ; `cuboid`, `ball`, `with_offset`, `with_restitution`, `with_density` |

**`lib.rs`** : la structure `Physics`.

- **`step(&mut Scene, dt)`**, en quatre temps :
  1. création des corps des nouvelles entités ; suppression de ceux dont l'entité a disparu ou a
     été remplacée ;
  2. scène → rapier : les corps cinématiques suivent leur `Transform`
     (`set_next_kinematic_position`) ; un corps déplacé à la main est téléporté et sa vitesse
     remise à zéro ;
  3. pas **fixes** de 1/60 s, avec au plus 5 pas par image pour éviter la « spirale de la
     mort » ;
  4. rapier → scène : les corps dynamiques écrivent leur position et leur rotation.
- **Composant interne `PhysicsBody`** : le handle rapier, le type de corps et le dernier
  `Transform` échangé (pour détecter un déplacement à la main).
- **Retrouver l'entité** : elle est rangée dans le `user_data` du corps (`Entity::to_bits`).
- **Forme et échelle** : la forme est agrandie par le `scale` du `Transform` à la création. Une
  sphère prend la plus grande des trois échelles.
- **CCD** activée pour les corps dynamiques : une balle rapide ne traverse pas un objet fin.
- **`cast_ray(origine, direction, distance) -> Option<RayHit>`** : renvoie l'entité, la
  distance, le point et la normale.
- **`apply_impulse(entité, impulsion, point)`**, **`clear()`** (avant de charger une scène),
  **`body_count()`**, `gravity` / `set_gravity`.
- **`debug_draw(&mut DebugLines)`** :
  - boîtes et sphères (3 cercles) grossies de 3 %, sinon le test de profondeur cache leurs
    lignes ;
  - couleurs : gris = fixe, orange = cinématique, vert = dynamique, vert foncé = endormi.

### Workspace

- `rapier3d = "0.36"` est ajouté au catalogue des dépendances.
- `zentrix-physics` est ajouté aux membres du workspace.
- Graphe de dépendances : `zentrix-physics` → `zentrix-scene`, `zentrix-render` (pour
  `DebugLines`), `zentrix-core`. La scène et le rendu ne dépendent pas de la physique.

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `demo.rs` | **Sol** : échelle (16, 1, 16) au lieu de 16 partout, et dalle fixe de 20 cm dont le dessus est le plan du sol. **Cubes** de la grille et **caisses sci-fi** : cinématiques (ils tournent et repoussent les balles). **Sphères** de la vitrine : fixes. `spawn_crate_pile` : pyramide de 10 caisses dynamiques de densité 0,6. `launch_ball` : balle orange de 25 cm, lancée à 16 m/s, densité 3, rebond 0,5. Composant `Ball { serial }`. `cleanup_system` : supprime ce qui tombe sous y = -20 et les balles au-delà de 40 (les plus anciennes) |
| `main.rs` | Champs `physics`, `assets` (pour créer des balles en cours de jeu) et `next_ball`. Ordre dans `on_update` : systèmes → `cleanup_system` → `physics.step` (sauf en pause) → `update_transforms`. Touche `B` ; clic droit ; `F2` ajoute les formes ; `load_scene` vide la physique et renumérote les balles ; enregistrement de `Ball`, `RigidBody` et `Collider` pour la sauvegarde |

## Vérification

- `cargo build`, `cargo clippy --workspace --all-targets`, `cargo fmt --check` : propres.
- `cargo test --workspace` : **103 tests** réussis.
  - **Physique** (10 tests) :
    - une balle tombe et se pose à un rayon au-dessus du sol ;
    - le pas est fixe quelle que soit la durée des images (et plafonné à 5) ;
    - le corps d'une entité supprimée disparaît ;
    - déplacer un `Transform` téléporte le corps ;
    - un corps cinématique suit son `Transform` ;
    - le rayon trouve l'entité, le point et la normale ;
    - la forme suit l'échelle ;
    - une impulsion met en mouvement ;
    - le dessin des formes ;
    - les composants se relisent avec leurs valeurs par défaut.
  - **Démo** :
    - nombres de corps par type (10 dynamiques, 27 cinématiques, 11 fixes) ;
    - la pyramide tient debout après 2 s de simulation ;
    - les balles en trop et les objets tombés disparaissent ;
    - la sauvegarde complète (avec les composants physiques) se relit à l'identique.
- La première compilation de rapier (et de nalgebra, simba...) prend plusieurs minutes. Ensuite,
  les tests de la physique durent moins d'une seconde.
- Démo lancée sous Vulkan, avec des captures d'écran :
  - une rafale de 8 balles (`B`) : elles volent, rebondissent sur les cubes qui tournent et
    roulent au sol ;
  - avec `F2`, les formes de collision s'affichent : vertes sur la pile, orange sur les cubes,
    cercles gris sur les sphères.

  La démo tourne à 60 FPS.

## Limites connues

- **Entités avec parent** : seules les entités **sans parent** ont un corps. Les arbres (modèle
  glTF : racine + morceaux) n'ont donc pas de collision.
- **Échelle** : changer l'échelle d'un objet après la création de son corps ne change pas sa
  forme.
- **Formes** : pas encore de formes « maillage » (collision exacte avec un modèle), ni de
  capsules.
- **Pas d'interpolation** entre deux pas de simulation. À 60 FPS, avec un pas de 1/60 s, c'est
  invisible ; un écran à 144 Hz verrait de petites saccades.
- **Sauvegarde** : les vitesses ne sont pas sauvegardées. Une balle en vol repart à l'arrêt
  après F9.
- **Fonctionnalités absentes** : pas d'événements de collision (sons d'impact, dégâts...), ni
  d'articulations.
- **Non essayés à la main** : le clic droit (pousser) et le déplacement d'un corps dynamique
  depuis l'inspecteur. Ces deux comportements sont couverts par des tests unitaires
  (`apply_impulse`, téléportation).
