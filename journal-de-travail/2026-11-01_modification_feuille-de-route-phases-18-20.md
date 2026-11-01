# 2026-11-01 — Modification : feuille de route, Phases 18 à 20 (masse et gravité, saisons, PNJ)

## Résumé

Trois phases ont été ajoutées au chapitre 7 du guide
([07-feuille-de-route.md](../documentation/guide-debutant/07-feuille-de-route.md)), après la
Phase 17, au même format que les précédentes.

| Phase | Étapes |
|-------|--------|
| **18 — Masse et gravité** | 18.1 masse explicite, 18.2 centre de masse et inertie, 18.3 matériaux physiques (densité réelle, friction, rebond, son d'impact, garde-fou du rapport de masses), 18.4 effets de jeu (pousser, explosions, flottaison), 18.5 gravité par objet et par zone, 18.6 gravité ponctuelle, 18.7 le « haut » local, 18.8 gravité animable |
| **19 — Temps du monde, météo et saisons** | 19.1 `WorldTime`, 19.2 météo, 19.3 surfaces saisonnières (neige, feuillage, sol mouillé), 19.4 lumière et ambiance, 19.5 la saison change le jeu (glace, neige profonde, routines), 19.6 adaptation aux mondes fermés, semi-ouverts et ouverts, 19.7 sauvegarde et outils |
| **20 — PNJ : comportements et animations** | 20.1 perception, 20.2 décision (machine à états, arbres de comportement, IA utilitaire), 20.3 navigation avancée, 20.4 routines et vie sociale, 20.5 objets intelligents, 20.6 animation des PNJ, 20.7 LOD d'IA et foules, 20.8 outils |

Dans la Phase 7, la ligne « Navigation et IA » renvoie maintenant à la Phase 20.

## Pourquoi

C'est la réponse à la demande d'ajouter la gestion de la masse, la gravité modifiable, les
saisons selon le type de monde, et les PNJ avec leurs comportements et leurs animations.

Les choix de fond :
- **La Phase 18 part de l'existant** : `Collider.density` et `Physics::set_gravity`. Les
  étapes 18.1 à 18.3 peuvent démarrer tout de suite. Le vrai travail est le « haut » local
  (18.7) : le moteur suppose aujourd'hui `Vec3::Y` à plusieurs endroits.
- **Une saison est un jeu de paramètres, pas un jeu de modèles** : les shaders changent
  l'aspect des mêmes modèles, ce qui ne coûte presque rien sur petit matériel. Le précalcul de
  l'éclairage par saison est réservé aux mondes fermés.
- **Des PNJ en quatre couches** (perception, décision, navigation, action), avec un LOD d'IA
  indispensable pour les mondes ouverts et le petit matériel.
