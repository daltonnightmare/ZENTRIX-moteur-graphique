# 2026-10-31 — Modification : documentation mise à jour pour la Phase 2

## Résumé

Mise en cohérence de la documentation avec le nouveau rendu 3D.

## Détail

| Fichier | Changement |
|---------|------------|
| `README.md` | État actuel (scène 3D) ; tableau des crates (nouveaux fichiers de `zentrix-core` et `zentrix-render`) |
| `Cargo.toml` (racine) | Commentaire de `glam` : ré-exporté par `zentrix_core::math` |
| `documentation/guide-debutant/05-visite-du-code.md` | Tableau « où trouver quoi » mis à jour + note sur les renommages |
| `documentation/guide-debutant/07-feuille-de-route.md` | Phase 2 : étapes A à D marquées ✅, choix réalisés expliqués, étape E indiquée comme prochaine |

## Reste à faire

Les chapitres 4 à 6 du guide débutant décrivent encore le code v1 (`const TRIANGLE`,
`triangle.wgsl`, exercices 4 et 7). Ils restent valables comme exercices, mais pourraient être
réécrits pour suivre le code actuel.
