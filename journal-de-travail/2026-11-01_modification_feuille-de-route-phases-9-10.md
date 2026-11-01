# 2026-11-01 — Modification : feuille de route, cap « beau sur petit matériel », Phases 9 et 10

## Résumé

Trois sections ont été ajoutées au chapitre 7 du guide
([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md)), après la
Phase 8.

**« Le cap : beau sur petit matériel »** fixe l'ambition de ZENTRIX. Il ne s'agit pas de
rattraper Unreal en largeur, mais de le battre sur un terrain précis : **des visuels
éblouissants sur du matériel modeste**. Le fil conducteur est une **scène vitrine** (forêt au
coucher du soleil, eau, lanterne, brouillard), qui doit tourner à 60 images/s sur une Intel
Iris Xe ; chaque fonctionnalité se justifie et se mesure sur elle.

**« Phase 9 — Beau sur petit matériel »** regroupe quatre chantiers :

| Chantier | Pistes |
|----------|--------|
| Calculer moins de pixels | Upscaling temporel, résolution dynamique, cache des pipelines et précompilation |
| Précalculer ce qui ne bouge pas | Lightmapper (path tracer hors ligne sur le BVH de la Phase 8), sondes et cube maps locales précalculées, éclairage hybride |
| Le rendu « caméra » | Exposition automatique, étalonnage par LUT, profondeur de champ, flou de mouvement, effets d'objectif |
| La scalabilité automatique | Détection du matériel et préréglages, gouverneur de budget, version minimale testée en continu |

Ordre conseillé :
1. upscaling temporel et résolution dynamique ;
2. caméra cinéma ;
3. lightmapper ;
4. scalabilité.

**« Phase 10 — Contenu et production »** regroupe quatre chantiers :

| Chantier | Pistes |
|----------|--------|
| Des matériaux plus riches | Éditeur de matériaux par nœuds (génère du WGSL), vernis, tissu, anisotropie, peau, parallax occlusion mapping, triplanaire, texturing virtuel |
| La vie | Animation squelettique complète (blend trees, IK, root motion), textures d'animation (VAT), tissu, cheveux en cartes, destruction |
| Les mondes ouverts | Monde découpé en cellules, HLOD, imposteurs octaédriques, placement procédural |
| La production | Éditeur complet, script visuel par nœuds, séquenceur cinématique, préparation et packaging, rapports de plantage, plugins |

## Pourquoi

C'est la réponse à la question « qu'est-ce qu'on pourrait ajouter après ça ? pour en faire un
moteur du niveau de Unreal mais adapté pour les low end produisant des visuels
époustouflants ».

Le choix de fond :
- miser sur les **leviers à fort rendement sur petit matériel** : upscaling, précalcul,
  post-traitement « cinéma » ;
- ajouter une **discipline de mesure**, avec la scène vitrine et la version minimale testée en
  continu ;
- ne pas multiplier les techniques lourdes.
