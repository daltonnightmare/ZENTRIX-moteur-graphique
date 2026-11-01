# 2026-11-01 — Modification : feuille de route, Phases 11 à 14 (animation, cinématiques, types de mondes, 2D et 3D)

## Résumé

Quatre phases ont été ajoutées au chapitre 7 du guide
([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md)), après la
Phase 10. Chacune a un but, un principe, des prérequis, des étapes numérotées et des
références.

| Phase | Étapes |
|-------|--------|
| **11 — Animation** | 11.1 animations rigides glTF, 11.2 squelettes et skinning, 11.3 morph targets, 11.4 mélanges et couches, 11.5 machines à états et blend spaces, 11.6 root motion et IK, 11.7 animation physique (ragdolls, ressorts, tissu), 11.8 animation pour petit matériel (LOD d'animation, skinning en compute, VAT) |
| **12 — Cinématiques et cutscenes** | 12.1 séquences et pistes, 12.2 caméras de cinéma (`CineCamera`, coupes, rails), 12.3 liaison avec le jeu (possessables / spawnables, déclencheurs, retour en douceur), 12.4 dialogues et sous-titres, 12.5 éditeur de séquences, 12.6 qualité cinéma et préchargement, 12.7 rendu vidéo hors ligne et vidéos précalculées |
| **13 — Mondes fermés, semi-ouverts et ouverts** | 13.1 niveaux et transitions, 13.2 pièces et portails, 13.3 zones et volumes de chargement, 13.4 grille de cellules, 13.5 grandes distances (rendu relatif à la caméra, recentrage de l'origine, `f64`), 13.6 le lointain (HLOD, imposteurs), 13.7 contenu et état du monde, 13.8 outils du monde |
| **14 — 2D et 3D** | 14.1 caméra orthographique, 14.2 sprites, 14.3 animation 2D, 14.4 tilemaps, 14.5 physique 2D (`rapier2d`), 14.6 éclairage 2D, 14.7 mélanges 2D et 3D (HD-2D, 2.5D, parallaxe), 14.8 texte et formes vectorielles |

Dans la Phase 10 :
- les sous-sections « La vie : animation et effets » et « Les mondes ouverts » sont remplacées
  par un renvoi vers les Phases 11 à 14 (leur contenu y a été repris et détaillé) ;
- la ligne « Séquenceur cinématique » renvoie à la Phase 12.

## Pourquoi

C'est la réponse à la demande d'établir l'implémentation des cutscenes, des cinématiques et de
l'animation, l'adaptabilité aux mondes ouverts, semi-ouverts et fermés, et la gestion de la 2D
et de la 3D.

Les choix de fond :
- **Un seul système de chargement, trois politiques** (niveau entier, volumes, grille de
  cellules) plutôt que trois systèmes : un jeu peut les combiner, par exemple un monde ouvert
  avec des intérieurs fermés.
- **Une séquence évaluée au temps *t*** sert à la fois aux cinématiques, aux événements
  scénarisés et aux bandes-annonces rendues hors ligne.
- **La 2D n'est pas un autre moteur** : même ECS, même renderer, même post-traitement, dans une
  crate `zentrix-2d` qui respecte le sens des dépendances. Le style HD-2D est mis en avant comme
  une voie très économique vers des visuels éblouissants sur petit matériel.
- L'animation peut commencer tôt : elle s'appuie sur la hiérarchie de transformations et le
  chargeur glTF déjà présents.
