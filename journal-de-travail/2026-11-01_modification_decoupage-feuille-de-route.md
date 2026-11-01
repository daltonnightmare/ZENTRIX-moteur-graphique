# 2026-11-01 — Modification : découpage de la feuille de route en petits fichiers

## Résumé

Le chapitre 7 du guide ([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md))
dépassait 1 500 lignes. Les phases 4 à 23 sont maintenant réparties en dix fichiers, dans le
dossier [feuille-de-route/](../documentation/guide-debutant/feuille-de-route/) :

| Fichier | Phases |
|---------|--------|
| `04-nouvelles-pistes.md` | 4 |
| `05-06-nanite-et-lumen.md` | 5, 6 |
| `07-mondes-gameplay-outils.md` | 7 |
| `08-lancer-de-rayons.md` | 8 |
| `09-10-beau-sur-petit-materiel.md` | le cap « beau sur petit matériel », 9, 10 |
| `11-12-animation-et-cinematiques.md` | 11, 12 |
| `13-14-mondes-et-2d-3d.md` | 13, 14 |
| `15-17-entrees-interface-blender.md` | 15, 16, 17 |
| `18-20-physique-saisons-pnj.md` | 18, 19, 20 |
| `21-23-effets-ia-destruction.md` | 21, 22, 23 |

Le chapitre 7 garde :
- l'état d'avancement ;
- les étapes A à M et le tableau « Et ensuite ? » ;
- une nouvelle section **« Les phases suivantes »**, qui sert de sommaire ;
- l'architecture cible et le paragraphe « Et Nanite / Lumen ? », qui renvoie maintenant au
  fichier des Phases 5 et 6.

Dans chaque fichier :
- un en-tête et un pied de page de navigation (groupe précédent, sommaire, groupe suivant) ;
- les liens relatifs corrigés d'un niveau (`../`), tous vérifiés par un script : aucun lien
  cassé.

Le contenu des phases n'a pas changé. Seules les phrases qui renvoyaient « ci-dessus » ou
« ci-dessous » pointent maintenant vers le bon fichier. Les entrées plus anciennes du journal
renvoient toujours au chapitre 7, qui existe encore et mène à chaque phase.

## Pourquoi

C'est la réponse à la demande de découper la feuille de route en fichiers de petits groupes.
Un fichier par thème (de 40 à 240 lignes) se lit et se modifie plus facilement, et les
modifications d'une phase ne touchent plus un fichier géant.
