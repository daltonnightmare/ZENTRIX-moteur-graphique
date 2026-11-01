# 2026-11-01 — Changement majeur : l'éditeur (sélection, gizmos, création, suppression, annuler)

## Résumé

Quand l'inspecteur est ouvert (`F4`), ZENTRIX devient un **éditeur** :

| Touche / geste | Action |
|----------------|--------|
| clic gauche | Sélectionner l'objet sous le curseur (dans le vide : désélectionner) |
| glisser une poignée | Déplacer / tourner / agrandir (`Ctrl` : crans de 25 cm, 15°, 10 %) |
| `1` `2` `3` | Gizmo de déplacement, de rotation, d'agrandissement |
| clic droit maintenu | Piloter la caméra (souris et `Z` `Q` `S` `D`), comme dans Unreal ou Unity |
| `Suppr`, ou le bouton en haut de l'inspecteur | Supprimer l'objet sélectionné, avec ses enfants |
| `Ctrl+Z` / `Ctrl+Y` (ou `Ctrl+Maj+Z`) | Annuler / refaire |
| « Ajouter : Cube, Sphère, Lampe » | Créer un objet devant la caméra, sélectionné |

Inspecteur fermé, on retrouve le mode jeu d'avant : un clic capture la souris. Le titre de
la fenêtre rappelle le mode et ses commandes.

## Comment

- Nouveau module [editor.rs](../crates/zentrix-app/src/editor.rs).
  - **Sélection** :
    - `screen_ray` « dé-projette » le pixel sous le curseur, sur le plan proche et le plan
      lointain ;
    - `pick` teste ce rayon contre la boîte englobante de chaque maillage, **dans le repère
      de l'objet** (une boîte tournée et agrandie reste exacte), par la méthode des tranches ;
    - les lampes sans maillage se sélectionnent dans une petite sphère.
  - **Gizmos** :
    - dessinés par egui sur le calque de fond, donc sous les fenêtres ;
    - axes du monde pour déplacer et tourner, axes de l'objet pour agrandir ;
    - un geste repart toujours du `Transform` du début (pas d'erreurs qui s'accumulent) ;
    - la matrice du parent vient du composant `Parent`, et non du `GlobalTransform` de
      l'objet, qui a une image de retard pendant un geste.
  - **Création** : `SpawnKind` (cube et sphère fixes et solides, lampe avec `PointLight`).
  - **Historique** : `History` et `Edit` (voir plus bas).
- `zentrix-render` : `Renderer::mesh_bounds`.
- `zentrix-scene` : `Scene::descendants`.
- `zentrix-platform` : `Input::was_character_pressed` donne la **lettre imprimée** sur la
  touche. `Ctrl+Z` fonctionne ainsi en AZERTY, où la touche Z n'est pas à la place du Z
  d'un clavier QWERTY.
- `zentrix-physics` : un corps « périmé » est recréé (`forget_stale_bodies`), ce qui arrive
  quand une entité est restaurée par « annuler ».
- [main.rs](../crates/zentrix-app/src/main.rs) :
  - souris et raccourcis de l'éditeur ;
  - en mode éditeur, les touches ne pilotent la caméra ou le personnage que pendant le clic
    droit (`Ctrl+Z` ne fait plus descendre la caméra) ;
  - `F9` vide l'historique.

## Deux problèmes signalés à l'essai, corrigés

1. **La performance de l'historique.**
   - La première version gardait une copie texte de toute la scène (le format de `F5`)
     avant chaque modification.
   - Mesure en mode debug :

     | Scène | Copie | Annulation (rechargement) |
     |-------|-------|---------------------------|
     | Démo, 64 entités | 11 ms | 16 ms |
     | Champ de stress, 4 160 entités | **346 ms** | **377 ms** |

     Avec le champ de stress, chaque geste figeait donc l'image, et l'historique pouvait
     occuper environ 110 Mo.
   - La nouvelle version garde des **opérations** :
     - un geste du gizmo : le `Transform` avant et après ;
     - une suppression : les entités, dont les composants partent dans une réserve
       (`World::take` vers un `EntityBuilder`) ;
     - une création : l'inverse.
   - Une entité restaurée reçoit un **nouvel identifiant**. `World::spawn_at` aurait détruit
     sans prévenir une entité créée entre-temps au même numéro, une balle par exemple. Les
     liens `Parent`, l'historique et la sélection suivent le nouvel identifiant.
   - Un test vérifie qu'une suppression, une annulation et un refaire au milieu de
     5 000 entités prennent moins de 20 ms. En pratique, c'est quasi instantané.
2. **La suppression ne marchait pas.**
   - Le bouton « Supprimer » était tout en bas de l'inspecteur, hors de l'écran.
   - Surtout, egui-winit « consomme » toutes les touches dès qu'un **widget quelconque** a
     le focus, et cliquer sur un bouton (« Cube ») lui donne le focus. Ensuite, `Suppr`,
     `Ctrl+Z` et même `F1` n'arrivaient plus au moteur.
   - Les corrections :
     - `UiLayer` ne garde une touche que si un **champ de texte** a le focus ;
     - un clic dans la scène retire le focus de l'interface (`clear_focus`), sans quoi
       `Espace` « recliquerait » le dernier bouton ;
     - le bouton « Supprimer », avec le nom de l'objet sélectionné, est maintenant en haut
       de l'inspecteur.

## Vérifications

- 185 tests passent, dont les nouveaux :
  - rayon au centre et au bord de l'écran ;
  - objet le plus proche ;
  - boîte tournée et agrandie exacte ;
  - lampe sans maillage ;
  - glisser le long d'un axe ;
  - crans et angles ;
  - création des trois objets ;
  - geste annulé et refait ;
  - suppression d'un parent et de son enfant annulée (identifiants suivis, enfant
    rattaché) ;
  - création annulée ;
  - coût dans une scène de 5 000 entités ;
  - entité restaurée qui retrouve un corps physique ;
  - lettre imprimée pour les raccourcis ;
  - de bout en bout dans l'application : geste annulé et refait (caméra et sélection
    conservées), geste sans effet ignoré, suppression annulée puis refaite.
- Clippy (`-D warnings`) et `cargo fmt --check` sont propres.
- Essayé par l'utilisateur dans l'application : sélection, gizmos, création et annulation
  fonctionnent. Les deux problèmes ci-dessus viennent de cet essai.
- Un incident pendant la vérification automatique : la fenêtre de ZENTRIX n'était pas au
  premier plan, et des clics et raccourcis du script de capture sont partis dans le
  navigateur de l'utilisateur. Les scripts de clics ne sont plus lancés pendant que
  l'utilisateur se sert de l'ordinateur.

## Limites et suites

- Les modifications faites dans les **champs de l'inspecteur** (position, matériau...) ne
  s'annulent pas encore.
- Suites possibles : dupliquer (`Ctrl+D`), sélection multiple, gizmos dans le repère de
  l'objet, aimantation aux surfaces.
