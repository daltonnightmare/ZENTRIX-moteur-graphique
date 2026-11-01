# 2026-11-01 — Modification : feuille de route, Phase 6 (éclairage global)

## Résumé

Une section **« Phase 6 — Éclairage global (notre « Lumen ») »** a été ajoutée au chapitre 7
du guide ([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md)),
après la Phase 5.

Elle propose un **« Lumen allégé »** en 8 étapes qui s'empilent :

| Étape | Contenu |
|-------|---------|
| 6.1 | Prépasse profondeur et normales |
| 6.2 | SSAO (occlusion ambiante à l'écran) |
| 6.3 | Accumulation temporelle (vecteurs de mouvement, historique ; ouvre aussi la voie au TAA) |
| 6.4 | Reflets à l'écran (SSR), avec la pyramide Hi-Z de l'étape 5.3 |
| 6.5 | Champs de distance signée : par maillage à l'import, et un champ global en *clipmap* |
| 6.6 | Sondes d'éclairage dynamiques (DDGI) tracées dans le champ de distance |
| 6.7 | Rebonds multiples, reflets hors de l'écran |
| 6.8 | Lancer de rayons matériel, facultatif |

La section précise aussi :
- le fonctionnement de Lumen (SDF, cache de surfaces, sondes, accumulation temporelle) ;
- les prérequis et les difficultés (bruit, budget GPU) ;
- les références : Wright (SIGGRAPH 2022), DDGI (JCGT 2019), SDFGI de Godot, GTAO.

Le paragraphe « Et Nanite / Lumen ? » renvoie maintenant aux Phases 5 et 6.

## Pourquoi

C'est la réponse à la question « comment faire notre propre version de Lumen ? ».
- **DDGI plutôt que le cache de surfaces** : le cache de surfaces de Lumen est trop complexe
  pour une première version. Les sondes DDGI donnent un éclairage global dynamique crédible
  et se mettent facilement à l'échelle sur un petit GPU.
- **L'environnement précalculé actuel** reste la solution de repli.
