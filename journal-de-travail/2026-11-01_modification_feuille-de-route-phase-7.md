# 2026-11-01 — Modification : feuille de route, Phase 7 (mondes, gameplay, outils, plateformes)

## Résumé

Une section **« Phase 7 — Mondes, gameplay, outils et plateformes »** a été ajoutée au
chapitre 7 du guide ([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md)),
après la Phase 6. Elle rassemble 22 pistes en quatre groupes, chacune avec son contenu et sa
difficulté :

| Groupe | Pistes |
|--------|--------|
| Rendu : des mondes plus grands | Terrain, végétation en masse, eau, ciel physique et cycle jour/nuit, brouillard volumétrique, particules GPU, decals |
| Moteur et gameplay | Scripting, prefabs, entrées avancées (manettes, actions), physique avancée (articulations, véhicules, déclencheurs), navigation et IA, audio avancé, réseau |
| Outils et pipeline | Pipeline d'assets (chargement asynchrone, compression, cache), interface de jeu, render graph |
| Plateformes et qualité | Intégration continue, tests d'images, version web (WASM + WebGPU), Linux et macOS, benchmarks |

La section propose un ordre de départ :
1. l'intégration continue ;
2. le render graph, avant les Phases 5 et 6 ;
3. les particules et le ciel physique ;
4. la version web.

## Pourquoi

C'est la réponse à la question « qu'est-ce qu'on pourrait ajouter à la feuille de route ? ».
Les Phases 4 à 6 couvrent surtout le rendu ; la Phase 7 couvre ce qui manque pour faire de
vrais jeux avec ZENTRIX. Les pistes d'audio notées dans le journal de l'audio (streaming,
réverbération, Doppler) y sont reprises.
