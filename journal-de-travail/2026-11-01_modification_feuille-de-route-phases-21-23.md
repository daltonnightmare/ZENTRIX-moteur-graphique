# 2026-11-01 — Modification : feuille de route, Phases 21 à 23 (effets spéciaux, apprentissage automatique, états et destruction)

## Résumé

Trois phases ont été ajoutées au chapitre 7 du guide
([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md)), après la
Phase 20, au même format que les précédentes.

| Phase | Étapes |
|-------|--------|
| **21 — Effets spéciaux** | 21.1 système d'effets (fichiers, déclenchement, recyclage, budget), 21.2 particules avancées, 21.3 foudre, 21.4 explosions, 21.5 impacts de balles (selon le matériau physique), 21.6 traînées, rayons et effets d'écran, 21.7 feu, fumée et volumes, 21.8 effets pour petit matériel (demi-résolution, LOD) et éditeur |
| **22 — Apprentissage automatique pour les PNJ** | 22.1 observations et actions (`Brain`), 22.2 mode entraînement (sans fenêtre, temps accéléré, interface Gymnasium), 22.3 inférence ONNX (`tract`, lots, quantification), 22.4 imitation, 22.5 cas d'usage conseillés, 22.6 motion matching, 22.7 dialogues générés (facultatif), 22.8 garde-fous |
| **23 — États des objets et destruction** | 23.1 composant d'état, 23.2 points de vie et dégâts, 23.3 dégâts visibles progressifs, 23.4 fracture précalculée, 23.5 destruction structurelle, 23.6 fracture dynamique (facultative), 23.7 persistance et réseau, 23.8 destruction pour petit matériel et outils |

Dans la Phase 7, les lignes « Particules GPU » et « Decals » renvoient maintenant à la
Phase 21.

La fracture des objets figurait dans la sous-section « La vie » de la Phase 10, remplacée à
l'étape [Phases 11 à 14](2026-11-01_modification_feuille-de-route-phases-11-14.md) sans avoir
été reprise. Elle est maintenant détaillée dans la Phase 23.

## Pourquoi

C'est la réponse à la demande de gérer les effets spéciaux (foudre, explosions, impacts de
balles), l'apprentissage automatique pour des PNJ dans des cas précis, et l'état des objets,
dont leur destruction.

Les choix de fond :
- **Un effet est un assemblage de briques** (particules, decals, lumière, son, tremblement,
  impulsion) déclenché en une ligne. Sur petit matériel, le danger est le remplissage
  (*overdraw*), d'où les particules à demi-résolution.
- **L'apprentissage automatique est un outil, pas la règle** : entraînement hors du jeu en
  Python, inférence CPU légère (ONNX, `tract`), un nœud parmi d'autres dans l'arbre de
  comportement, toujours avec une solution de repli classique.
- **La destruction est un état comme un autre** : fracture précalculée par défaut, graphe de
  soutien pour les bâtiments, transmission en réseau des changements d'état plutôt que des
  débris.
