# 2026-11-01 — Modification : feuille de route, Phase 8 (lancer de rayons logiciel)

## Résumé

Une section **« Phase 8 — Lancer de rayons logiciel (adapté aux petits GPU) »** a été ajoutée
au chapitre 7 du guide ([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md)),
après la Phase 7.

Elle explique :
- pourquoi les rayons sont lancés **en logiciel** (compute shaders) : pas de cœurs RT sur les
  petits GPU, et des *ray queries* encore expérimentales dans wgpu ;
- le **BVH** à deux niveaux (BLAS par maillage, TLAS des instances), compact, sur une
  géométrie simplifiée ;
- six **règles d'or** : rendu hybride, moins d'un rayon par pixel, accumulation temporelle,
  débruitage, rayons courts, budget mesuré avec solution de repli.

Le plan compte 8 étapes :

| Étape | Contenu |
|-------|---------|
| 8.1 | BVH sur le CPU (SAH par paquets), testé sans GPU |
| 8.2 | Parcours dans un compute shader, vues de debug |
| 8.3 | Deux niveaux (BLAS + TLAS) pour les objets qui bougent |
| 8.4 | Ombres douces des lampes et projecteurs : le premier effet conseillé |
| 8.5 | Débruiteur temporel puis spatial (à-trous) |
| 8.6 | Occlusion ambiante tracée |
| 8.7 | Reflets tracés sur les surfaces lisses |
| 8.8 | Sondes DDGI tracées dans le BVH ; chemin matériel facultatif |

La section précise aussi :
- les prérequis : compute shaders (5.1), mesures GPU, accumulation temporelle (6.3) ;
- les liens avec les Phases 5 et 6 : le BVH sert près de la caméra, le champ de distance au
  loin ;
- les références : Bikker, *Ray Tracing Gems*, SVGF.

## Pourquoi

C'est la réponse à la question « comment implémenter le raytracing adapté au low end
device ? ».

Le premier effet proposé, les ombres des lampes, comble une limite notée à l'étape
[lampes et projecteurs](2026-11-01_changement-majeur_lampes-et-projecteurs.md) : `PointLight`
et `SpotLight` n'ont pas d'ombres.
