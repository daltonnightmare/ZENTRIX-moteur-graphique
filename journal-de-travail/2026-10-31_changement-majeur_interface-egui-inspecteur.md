# 2026-10-31 — Changement majeur : interface de debug egui et inspecteur

## Résumé

Une interface graphique (bibliothèque **egui**) est dessinée par-dessus la scène. La démo
l'utilise pour un **inspecteur** (`F4`) : FPS, options du moteur, lumière, liste filtrable des
entités, et modification en direct de l'entité sélectionnée (position, rotation, taille,
vitesse de rotation, suppression). C'est un premier pas vers un éditeur.

## Pourquoi

Les touches F1 à F9 ne suffisent plus pour régler et observer le moteur. Une interface permet
de voir l'état de la scène et de la modifier sans recompiler.

## Détail

### Nouvelle crate `zentrix-ui`

`UiLayer` relie egui au moteur :

- `on_window_event` : transmet les événements winit à egui-winit, renvoie « consommé » ;
- `run(window, |ctx| ...)` : décrit l'interface de l'image (mode immédiat), puis la
  transforme en triangles ;
- `wants_pointer` / `wants_keyboard` ;
- implémente `RenderOverlay` (rendu par egui-wgpu par-dessus la scène) ;
- ré-exporte `egui`.

Dépendances : `egui` 0.37 (avec `default_fonts`), `egui-wgpu` 0.37, `egui-winit` 0.37 —
vérifiées au préalable dans un projet jetable : elles utilisent exactement wgpu 30.0.1 et
winit 0.30.13, comme ZENTRIX.

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `src/inspector.rs` *(nouveau)* | `Inspector` (visible, sélection, filtre), sections Moteur / Lumière / Entités / Sélection ; rotation affichée en degrés (angles d'Euler YXZ, réécrite seulement si modifiée) ; `Requests` (sauvegarde, rechargement) ; contour de la sélection |
| `src/main.rs` | `UiLayer` dans `GraphicsState`, `on_window_event`, interface exécutée au début de `on_update` (avant les systèmes), `F4`, rendu avec `render_with_overlay`, contour cyan de l'entité sélectionnée |

## Problèmes rencontrés et corrigés

- **Aucun texte affiché** : dans egui 0.37, les polices ne sont plus incluses par défaut →
  fonctionnalité `default_fonts` activée.
- **Couleurs de l'interface délavées** (avertissement « linear framebuffer » d'egui-wgpu) :
  l'image de la fenêtre est en sRGB, egui attend une cible non-sRGB → seconde vue non-sRGB
  de la même image (voir l'[entrée dédiée](2026-10-31_modification_evenements-overlay-vue-non-srgb.md)).
- **Un clic sur l'interface capturait la souris** : la décision de capture interrogeait egui
  *avant* qu'il ait traité l'image en cours (information périmée : le curseur venait
  d'arriver sur le panneau). Le relâchement n'arrivait alors plus à egui, et le clic était
  perdu. Corrections : décider la capture **après** `ui.run`, et toujours transmettre les
  relâchements de bouton à egui, même souris capturée.
- **Ombre des fenêtres egui crénelée** (en escalier au lieu de floue) sur Intel Iris Xe /
  Vulkan : cause non identifiée ; ombres des fenêtres et menus désactivées
  (`Shadow::NONE`). Point ouvert.

## Vérification

- `cargo build`, `cargo clippy --workspace --all-targets`, `cargo fmt --check` : propres.
- `cargo test --workspace` : 75 tests réussis.
- Tests réels sous Vulkan (script d'entrées simulées, rendu DPI-aware pour que clics et
  captures partagent les mêmes coordonnées) : fenêtre lisible ; clic sur « Arbre 1 » → la
  section Sélection affiche sa position (-6,5 ; -0,5 ; -6,5) et sa taille 1,3, contour cyan
  dans la scène ; frappe de « cube » puis `Z` maintenue dans le champ de filtre → le texte
  apparaît (« cubez »), la liste est filtrée et **la caméra ne bouge pas**.
- Une instance de l'application était ouverte par l'utilisateur pendant ces essais :
  l'exécutable de test a été compilé à part, dans `target/verif` (ignoré par git).

## Limites connues

- Le contour de sélection est un cube unité autour de l'entité : exact pour les cubes,
  approximatif pour les modèles (arbres).
- Pas encore de sélection en cliquant dans la scène (lancer de rayon), ni d'annulation.
