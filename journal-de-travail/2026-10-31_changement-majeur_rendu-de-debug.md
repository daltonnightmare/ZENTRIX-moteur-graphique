# 2026-10-31 — Changement majeur : rendu de debug (lignes, grille, axes, boîtes)

## Résumé

Le moteur sait dessiner des **lignes de debug** par-dessus la scène : grille au sol, axes du
repère (X rouge, Y vert, Z bleu) et boîtes englobantes qui suivent les objets. C'est l'étape G
de la [feuille de route](../documentation/guide-debutant/07-feuille-de-route.md) : la
**Phase 2 (noyau de rendu) est terminée**.

## Pourquoi

Visualiser l'invisible (repère du monde, orientation, volumes) est le moyen le plus rapide de
diagnostiquer les bugs de position et de rotation, et ce sera indispensable pour la physique et
l'ECS.

## Détail

### `zentrix-render`

| Fichier | Changement |
|---------|------------|
| `src/debug.rs` *(nouveau)* | `DebugLines` (public) : `line`, `grid`, `axes`, `wire_box(Mat4, Color)`, `clear`, `line_count` ; `LineVertex` (24 octets) ; 5 tests + 1 doc-test |
| `src/shaders/debug.wgsl` *(nouveau)* | Shader des lignes : caméra seulement, couleur par sommet |
| `src/pipeline.rs` | `create_line_pipeline` : `LineList`, pas de culling, profondeur `LessEqual` **sans écriture** |
| `src/renderer.rs` | `render(gpu, camera, draws, debug)` : lignes dessinées après les maillages, dans la même passe |
| `src/lib.rs` | Exporte `DebugLines` |

### `zentrix-app`

- `F1` : grille 16×16 et axes (activés par défaut) ; `F2` : boîtes englobantes jaunes des cubes.
- La grille est placée 1 cm au-dessus du sol pour éviter le *z-fighting*.

## Décisions techniques

- **Mode immédiat** : l'application remplit un `DebugLines` à chaque image et le passe à
  `render`, sur le même principe que les `DrawCommand` (le rendu ne garde aucun état de scène).
- **Profondeur testée mais pas écrite** : les lignes derrière un objet sont cachées, mais une
  ligne ne cache jamais rien.
- **`wire_box` à partir de la matrice de l'objet** : la boîte tourne et grandit avec lui ; les
  12 arêtes sont trouvées en reliant les coins qui ne diffèrent que d'un bit.

## Vérification

- `cargo build` et `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 38 tests réussis.
- Lancement sous Vulkan : grille et axes visibles, boîtes jaunes alignées sur les cubes en
  rotation, arêtes cachées correctement masquées, aucune erreur de validation.

## Impact / suite

- Prochaine étape : **H — scène et ECS** (Phase 3), crate `zentrix-scene`.
