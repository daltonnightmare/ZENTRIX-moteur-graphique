# 2026-11-01 — Modification : feuille de route, Phases 15 à 17 (entrées, fenêtres et interface, icônes, Blender et rigs)

## Résumé

Trois phases ont été ajoutées au chapitre 7 du guide
([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md)), après la
Phase 14. Chacune part de l'existant (« Où on en est ») et donne un principe, des étapes
numérotées et des références.

| Phase | Étapes |
|-------|--------|
| **15 — Entrées** | 15.1 actions et contextes empilés, 15.2 manettes (`gilrs`), 15.3 touches reconfigurables (touches physiques et logiques, AZERTY / QWERTY, glyphes), 15.4 déclencheurs et modificateurs (appui long, mémoire tampon), 15.5 pointeurs et tactile, 15.6 saisie de texte (IME), 15.7 enregistrement et rejeu, 15.8 accessibilité |
| **16 — Fenêtres, interface et icônes** | 16.1 fenêtre système (modes, écrans, haute densité, mémorisation, pause), 16.2 plusieurs fenêtres pour l'éditeur, 16.3 interface de jeu (`taffy`, widgets, navigation à la manette, transitions), 16.4 design system (thèmes, styles, formes par champ de distance, 9-slice, verre dépoli), 16.5 adaptation aux écrans, 16.6 icônes (application, interface en SVG avec `resvg`, glyphes, vignettes de l'éditeur), 16.7 curseurs et identité, 16.8 écrans standards |
| **17 — Blender, rigs et pipeline de contenu** | 17.1 conventions d'export vérifiées à l'import, 17.2 propriétés Blender vers composants (`extras`, suffixes `_col`, `_trigger`, `_lod`, lampes et caméras), 17.3 collections, instances et niveaux, 17.4 rigs et squelettes (os de déformation, Rigify), 17.5 actions, shape keys et marqueurs, 17.6 retargeting, 17.7 lien direct Blender ↔ ZENTRIX (add-on, rechargement à chaud avec `notify`), 17.8 autres formats et outils (FBX, ORM, cooking) |

Dans la Phase 7, les lignes « Entrées avancées » et « Interface de jeu » renvoient maintenant
aux Phases 15 et 16.

## Pourquoi

C'est la réponse à la demande d'ajouter la gestion des entrées, les fenêtres et leur design,
les icônes, et la gestion des modèles Blender, des rigs et des autres contenus.

Les choix de fond :
- **Des actions plutôt que des touches** : le jeu ne lit plus de `KeyCode` en dur. Les actions
  sont reliées aux touches **physiques**, pour que ZQSD fonctionne en AZERTY, et l'interface
  affiche les touches **logiques**.
- **Deux interfaces séparées** : egui reste l'outil des développeurs, et le jeu a sa propre
  interface, pilotée par un design system rechargeable à chaud.
- **glTF comme format pivot** avec Blender, des conventions documentées et une vérification
  qui explique chaque erreur, plutôt que des corrections manuelles.
