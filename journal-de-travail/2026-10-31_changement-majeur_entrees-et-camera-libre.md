# 2026-10-31 — Changement majeur : entrées clavier/souris et caméra libre

## Résumé

Ajout d'un système d'entrées (`Input`) dans `zentrix-platform` et d'une caméra libre (`FlyCamera`)
dans la démo : on se déplace en ZQSD et on regarde autour de soi à la souris. Étape E de la
[feuille de route](../documentation/guide-debutant/07-feuille-de-route.md).

## Pourquoi

Jusqu'ici, on ne pouvait que regarder la scène. Pouvoir s'y déplacer est indispensable pour
vérifier le rendu sous tous les angles, et ce sera la base des contrôles de jeu.

## Détail

### `zentrix-platform`

| Fichier | Changement |
|---------|------------|
| `src/input.rs` *(nouveau)* | `Input` : touches et boutons enfoncés / appuyés / relâchés pendant l'image, déplacement brut de la souris, molette, position du curseur, focus ; `set_cursor_captured()` (essaie `Locked` puis `Confined`) ; 3 tests |
| `src/runner.rs` | Alimente `Input` : clavier, boutons, molette (lignes ou pixels ÷ 40), curseur, focus, et `device_event` → `DeviceEvent::MouseMotion` (ignoré sans focus) ; `input.end_frame()` après chaque rendu |
| `src/application.rs` | **API modifiée** : `on_update(&mut self, dt: f32, input: &Input)` |
| `src/lib.rs` | Exporte `Input`, `set_cursor_captured`, `MouseButton` |

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `src/fly_camera.rs` *(nouveau)* | `FlyCamera` : position, lacet/tangage (tangage limité à ±89°), vitesse, sensibilité ; `looking_at()`, `forward()`, `right()`, `update(input, dt, look)`, `apply_to(camera)` ; 3 tests |
| `src/main.rs` | Gestion des commandes dans `on_update` via `Input`, capture de la souris au clic, libération sur `Échap` ou perte du focus |

## Décisions techniques

- **Touches physiques** (`KeyCode::KeyW`…) : « WASD » devient automatiquement « ZQSD » sur un
  clavier AZERTY.
- **Mouvement brut de la souris** (`DeviceEvent::MouseMotion`) plutôt que la position du
  curseur : il continue d'arriver quand le curseur est bloqué au bord.
- **Perte du focus** : toutes les touches sont relâchées, sinon une touche relâchée pendant un
  Alt+Tab resterait « enfoncée ».
- Direction de déplacement **normalisée** : pas de bonus de vitesse en diagonale.
- `FlyCamera` est dans `zentrix-app`, seule crate qui dépend à la fois de la plateforme et du
  rendu : les couches du moteur restent indépendantes.

## Vérification

- `cargo build` et `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 22 tests réussis (6 nouveaux).
- Test réel sous Vulkan, avec entrées simulées (script PowerShell : clic, touche W maintenue
  1,5 s, mouvement relatif de la souris, deux `Échap`) : la caméra avance et tourne à droite ;
  le premier `Échap` libère la souris, le second ferme l'application proprement.

## Impact / suite

- Toute implémentation d'`Application` doit adopter la nouvelle signature d'`on_update`.
- Prochaine étape : **F — textures** (crate `zentrix-assets`, crate `image`, UV, sampler).
