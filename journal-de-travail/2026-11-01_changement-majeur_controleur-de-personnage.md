# 2026-11-01 — Changement majeur : contrôleur de personnage et mode marche

## Résumé

On peut maintenant **marcher** dans la démo, au lieu de voler. La touche **`C`** passe d'un mode
à l'autre.

| Touche (mode marche) | Action |
|----------------------|--------|
| `Z` `Q` `S` `D` | Marcher (4 m/s), dans la direction du regard mais toujours à plat |
| `Maj` | Courir (7 m/s) |
| `Espace` (maintenu) | Sauter (environ 1,3 m), dès que le personnage touche le sol |
| souris | Tourner la tête |

Le personnage :
- tombe, se pose sur le sol, s'arrête contre les murs ou glisse le long d'eux ;
- monte les marches de 35 cm et les pentes de 45° ;
- **pousse les caisses** selon sa masse (70 kg).

La chasse aux cristaux se joue aussi à pied : il faut alors sauter pour attraper les cristaux
les plus hauts.

## Le moteur : `zentrix-physics`

- **`Shape::Capsule`** et `Collider::capsule(demi_hauteur, rayon)` : la forme d'un personnage,
  dessinée en debug (`F2`) par deux sphères reliées par quatre traits.
- **`CharacterController`**, un nouveau composant :
  - le jeu écrit `desired_velocity` et `jump` ;
  - la physique met à jour `grounded` et `vertical_speed`.
  - Réglages : `jump_speed`, `step_height`, `max_slope`, `mass`.
- **`Physics::step`** déplace les personnages avant de transmettre les corps cinématiques, une
  fois par **image** (et non par pas fixe) pour répondre aux touches sans délai. Il utilise le
  `KinematicCharacterController` de rapier :
  - la forme est « sondée » dans la direction du mouvement (*shape cast*) ;
  - le personnage glisse le long des murs, franchit les marches automatiquement et reste collé
    au sol en descendant ;
  - les objets dynamiques heurtés reçoivent une impulsion (`solve_character_collision_impulses`).
- **`Physics::cast_ray_excluding`** : un rayon qui ignore un corps. Depuis les yeux, le rayon
  du clic droit toucherait sinon la capsule du joueur.

## Deux pièges rencontrés

1. **La première image.**
   - Ce qui se passait : les recherches de rapier passent par sa *broad phase*, qui n'apprend
     l'existence des nouveaux corps qu'en simulant. Un personnage créé en même temps que le sol
     s'y enfonçait de quelques millimètres pendant la première image, puis restait coincé.
   - Ce qui l'a montré : 4 tests sur 5 échouaient (il avançait de 13 cm en 0,5 s au lieu de 2 m).
   - Les corrections :
     - tant qu'aucun pas n'a été simulé depuis la création de nouveaux corps, les personnages
       attendent, une image ou deux ;
     - au sol, le personnage ne subit plus la gravité. Immobile, il ne demande alors aucun
       déplacement, et rapier en profite pour le sortir d'un objet qui se serait glissé en lui.
   - **Une première version de la correction a fait planter l'application.**
     - Elle appelait `PhysicsWorld::detect_collisions` (une détection des collisions sans
       simulation) quand des corps venaient d'être créés.
     - Au premier lancement par l'utilisateur, `cargo run` (mode debug) s'est arrêté sur une
       vérification interne de rapier : « touching pair not linked in the persistent islands ».
     - La cause : cette détection crée des paires de contact sans mettre à jour les « îlots »
       de rapier (ses groupes d'objets en contact, qu'il endort ensemble). Elle survenait quand
       le joueur apparaissait **posé sur des objets dynamiques endormis**, par exemple en
       passant en marche au-dessus de la pile de caisses.
     - Le cas est reproduit par le test `creer_un_personnage_au_milieu_d_objets_poses` : un
       personnage posé sur une pile de caisses endormies. Il échouait avec l'ancienne version
       et passe avec la nouvelle.
     - Un second test, `marcher_dans_la_scene_de_demo`, passe en marche dans la vraie scène de
       démo, avec des durées d'image irrégulières comme dans l'application.
2. **Le point de départ hors du sol.**
   - Ce qui se passait : la caméra de départ (z = 11) est au-delà du sol (de −8 à 8). Le rayon
     vertical ne trouvait rien, et le point de secours (z = 9) était lui aussi dans le vide.
   - Ce qui l'a montré : les captures d'écran.
   - La correction : le point de départ est en (0,9 ; 7,5), dans l'allée entre deux colonnes
     de sphères, et un test vérifie qu'il est sur le sol.

## L'application

- **Nouveau** : [player.rs](../crates/zentrix-app/src/player.rs).
  - Le joueur est une entité invisible (`Player`) dont le `Transform` est la position des
    **pieds**. C'est une capsule de 1,8 m, et les yeux sont à 1,6 m.
  - `toggle` le pose sous la caméra, sur ce qu'un rayon vertical trouve. Au-dessus du vide, il
    va au point de départ.
  - `desired_velocity` est une fonction pure, testée sans clavier.
  - `input_system` gère la souris et les touches, et ramène au départ un joueur tombé du monde.
  - `camera_follow_system` place la caméra aux yeux, **après** la physique.
- [fly_camera.rs](../crates/zentrix-app/src/fly_camera.rs) : le regard (`look`) est séparé du
  vol, pour être partagé avec la marche. Ajout de `forward_flat`.
- [main.rs](../crates/zentrix-app/src/main.rs) :
  - touche `C` ;
  - choix du système (marche ou vol) selon la présence du joueur ;
  - caméra qui suit le joueur après la physique ;
  - clic droit qui ignore le joueur ;
  - `Player` et `CharacterController` enregistrés dans le sérialiseur.
- Le HUD d'accueil de la partie rappelle la touche `C`. Elle figure aussi dans le README et le
  chapitre 1 du guide, et la piste « Contrôleur de personnage » de la
  [Phase 4](../documentation/guide-debutant/feuille-de-route/04-nouvelles-pistes.md) est
  marquée ✅.

## Vérifications

- 14 nouveaux tests, tous sans GPU, dont les deux tests de régression du plantage ci-dessus.
  - Dans la physique (7, plus la régression) :
    - chute puis station debout ;
    - marche et arrêt contre un mur (x ≈ 2,7 pour un mur en x = 3 et un rayon de 0,3) ;
    - saut à environ 1,27 m (v²/2g), sans relance en l'air ;
    - marche de 20 cm montée ;
    - caisse poussée ;
    - rayon qui ignore un corps ;
    - dessin d'une capsule.
  - Dans le joueur (5, plus la régression) :
    - pose sous la caméra puis retrait ;
    - retour au départ au-dessus du vide ;
    - départ sur le sol de la démo ;
    - caméra à hauteur des yeux après 1 m de marche ;
    - vitesse horizontale, diagonale normalisée, touches opposées qui s'annulent.
- 166 tests passent dans le workspace ; clippy (`-D warnings`) et `cargo fmt --check` sont
  propres.
- Vérification visuelle sous Vulkan (Iris Xe) :
  - apparition dans l'allée, à hauteur des yeux ;
  - marche jusqu'à la grille de cubes, où le personnage s'arrête ;
  - saut au-dessus de la vitrine, puis retour à la hauteur normale.
