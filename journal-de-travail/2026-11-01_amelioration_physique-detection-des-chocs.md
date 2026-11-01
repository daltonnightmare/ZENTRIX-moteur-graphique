# 2026-11-01 — Amélioration : physique, détection des chocs (`Physics::impacts`)

## Résumé

`zentrix-physics` signale désormais les **chocs** : deux objets qui viennent de se toucher, avec
la vitesse à laquelle ils se sont rencontrés et l'endroit du choc. Il le fait pour les sons
d'impact de l'[audio](2026-11-01_changement-majeur_audio-kira.md), et pour tout ce qui doit
réagir à une collision (dégâts, effets).

## Détail

| Élément | Changement |
|---------|------------|
| `Impact { entities, speed, position }` | Nouveau type public. `speed` : vitesse **relative** juste avant le choc. `position` : celle de l'objet le plus rapide des deux |
| `Physics::impacts()` | Les chocs du dernier `step`, sur tous ses pas de simulation |
| Formes de collision | `ActiveEvents::COLLISION_EVENTS` est activé : rapier signale le **début** de chaque contact |
| `ContactCollector` (interne) | Implémente `rapier::EventHandler` et garde les contacts qui commencent. La liste est protégée par un `Mutex`, parce que rapier peut appeler le collecteur depuis plusieurs threads |
| `simulate_one_step` (interne) | Relève les vitesses des corps **avant** le pas (après, le choc les a déjà changées), appelle `world.step_with_events`, puis transforme chaque contact commencé en `Impact` |

Un objet posé ne produit qu'**un** choc, au moment où il touche. Un contact qui dure ne produit
pas de nouveaux chocs.

## Vérification

Un nouveau test vérifie le cas d'une balle lâchée de 3 m :
- elle produit un choc à environ 7 m/s, la valeur théorique √(2 × 9,81 × 2,5) ;
- le choc est situé près du sol ;
- une fois la balle posée, il n'y a plus de chocs.

La physique compte maintenant 11 tests.

## Limites

- La fin d'un contact n'est pas signalée.
- La position est approximative : celle de l'objet le plus rapide, pas le point de contact
  exact.
