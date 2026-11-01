# 2026-11-01 — Modification : feuille de route, Phase 5 (géométrie virtualisée)

## Résumé

Une section **« Phase 5 — Géométrie virtualisée (notre « Nanite ») »** a été ajoutée au
chapitre 7 du guide ([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md)),
après la Phase 4.

Elle découpe le chemin vers un équivalent de Nanite en 7 étapes, chacune utile seule :

| Étape | Contenu |
|-------|---------|
| 5.1 | Culling sur GPU : compute shaders et *indirect draw* |
| 5.2 | Meshlets : clusters d'environ 128 triangles (crate `meshopt`), culling par cluster et par cône de normales |
| 5.3 | Occlusion culling avec une pyramide de profondeur (Hi-Z), en deux passes |
| 5.4 | Hiérarchie de LOD sans fissures (groupes simplifiés à bords verrouillés), niveau choisi par cluster selon l'erreur projetée à l'écran |
| 5.5 | Visibility buffer et passe des matériaux |
| 5.6 | Rastérisation logicielle des petits triangles (atomiques 64 bits, avec une solution de repli) |
| 5.7 | Streaming des clusters |

La section précise aussi :
- les **prérequis** : mesures GPU conseillées, et un modèle très détaillé pour tester ;
- les **références** : Karis (SIGGRAPH 2021), les meshlets de Bevy, meshoptimizer.

Le paragraphe « Et Nanite / Lumen ? » renvoie maintenant à cette Phase 5.

## Pourquoi

C'est la réponse à la question « comment créer notre propre version de Nanite ? ». Le plan
part de ce que ZENTRIX sait déjà faire : instanciation, culling sur CPU (culling.rs), storage
buffers, passes plein écran. Il n'ajoute la complexité qu'étape par étape, chacune devant
être justifiée par une mesure.
