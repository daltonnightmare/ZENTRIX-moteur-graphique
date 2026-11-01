# 2026-11-01 — Amélioration : petit gameplay de test, la chasse aux cristaux

## Résumé

La démo devient jouable. La touche **`G`** lance une partie de 60 secondes :

| Élément | Règle |
|---------|-------|
| **8 cristaux** | Cubes cyan lumineux posés sur la pointe, qui tournent, avec une petite lampe. On les ramasse en passant à moins de 1,2 m avec la caméra : **+10 points** |
| **5 cibles** | Plaques rouges lumineuses sur un arc (côtés et fond de l'arène), tournées vers le centre. Touchées par une balle (`B`) : **+25 points**, et elles tombent |
| **Bonus** | Tout ramasser et tout toucher avant la fin : **2 points par seconde restante** |
| **Fin** | Temps écoulé ou tout fait : score final, « sans faute », record de la session |

Un **HUD** egui affiche en haut de l'écran le score, le temps (en rouge dans les 10 dernières
secondes) et les compteurs. Les messages « +10 cristal » s'effacent en fondu, et un **viseur**
apparaît au centre quand la souris est capturée. Les sons réutilisent les sons synthétisés : un
choc aigu pour un cristal, un choc grave pour une cible, un souffle au début et à la fin.

## Fichiers

- **Nouveau** : [game.rs](../crates/zentrix-app/src/game.rs).
  - Composants `GameItem` (tout ce que la partie crée), `Crystal` et `Target`.
  - `Game::start`, `Game::update` et `Game::reset`, qui renvoient des `GameEvent`.
  - `Game::hud` pour l'affichage.
  - La place des cristaux est tirée par un petit générateur *xorshift* : elle change à chaque
    partie, mais reste reproductible.
- [main.rs](../crates/zentrix-app/src/main.rs) :
  - touche `G` ;
  - mise à jour de la partie **après la physique**, pour lire les chocs du pas qui vient d'être
    simulé (une nouvelle étape « jeu » dans le profileur) ;
  - `play_game_events` pour les sons et le journal ;
  - HUD dessiné après l'inspecteur ;
  - le rechargement de scène (`F9`) abandonne la partie ;
  - `GameItem` enregistré dans le sérialiseur, pour que les objets d'une partie sauvegardée
    soient retrouvés et retirés au rechargement.
- [demo.rs](../crates/zentrix-app/src/demo.rs) : le module de tests et ses ressources factices
  (`assets()`) sont partagés avec les tests de game.rs.
- Documentation : touche `G` dans le README et le chapitre 1 du guide ; piste « Petit
  gameplay » marquée ✅ dans la [Phase 4](../documentation/guide-debutant/feuille-de-route/04-nouvelles-pistes.md).

## Vérifications

- 8 nouveaux tests, sans GPU :
  - placement dans l'arène ;
  - cibles tournées vers le centre ;
  - ramassage ;
  - cible touchée, une seule fois même si le choc est signalé deux fois, et seulement par une
    balle ;
  - fin au bout du temps ;
  - bonus de temps ;
  - record ;
  - abandon au rechargement.
- 152 tests passent dans le workspace ; clippy (`-D warnings`) et `cargo fmt --check` sont
  propres.
- Vérification visuelle sous Vulkan (Iris Xe) : HUD d'accueil, partie en cours (chrono,
  compteurs), viseur après capture de la souris. Une première version plaçait deux cibles au
  premier plan, dont l'une traversait une caisse sci-fi : elles sont maintenant sur un arc
  dont les angles évitent les arbres.

## Pourquoi

C'est la piste « petit gameplay » de la Phase 4, choisie comme premier objectif concret
après l'extension de la feuille de route. Elle teste l'API du moteur du point de vue d'un jeu.
Ce qu'elle a montré, et ce qui manque encore :
- on **vole** au lieu de marcher : il faut le contrôleur de personnage ;
- les touches sont **en dur** : il faut la carte des entrées (Phase 15) ;
- le HUD est en **egui** : il faut l'interface de jeu (Phase 16).
