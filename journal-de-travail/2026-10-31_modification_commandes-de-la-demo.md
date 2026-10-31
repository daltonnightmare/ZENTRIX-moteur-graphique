# 2026-10-31 — Modification : commandes de la démo

## Résumé

Les commandes de la démo changent pour laisser la place aux déplacements de la caméra libre
(voir [entrées et caméra libre](2026-10-31_changement-majeur_entrees-et-camera-libre.md)).

## Détail

| Commande | Avant | Après |
|----------|-------|-------|
| Pause de l'animation | `Espace` | `P` |
| `Espace` | pause | monter |
| `Échap` | quitter | libérer la souris si elle est capturée, sinon quitter |
| Nouveau | — | clic gauche : capturer la souris ; `Z` `Q` `S` `D` : se déplacer ; `Ctrl` : descendre ; `Maj` : sprint ; molette : vitesse |

- Les commandes passent de `on_key` à `on_update` (lecture de `Input`) ; `ZentrixApp`
  n'implémente plus `on_key` (la méthode reste disponible dans le trait).
- Le titre de la fenêtre rappelle « clic : piloter, Échap : libérer » et la liste des commandes
  est écrite dans les logs au démarrage.

## Documentation mise à jour

`README.md` (tableau des commandes, état, tableau des crates), guide débutant chapitres 1, 2
et 7.
