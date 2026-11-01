# Feuille de route : Phases 5 et 6 — Nanite et Lumen

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phase 4 — Nouvelles pistes](04-nouvelles-pistes.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phase 7 — Mondes, gameplay, outils et plateformes ➡️](07-mondes-gameplay-outils.md)

---

## Phase 5 — Géométrie virtualisée (notre « Nanite »)

**Le but** : dessiner à peu près **un triangle par pixel**, quel que soit le nombre de
triangles des modèles. Un rocher de 2 millions de triangles vu de loin ne doit en coûter que
quelques centaines.

**Le principe** :
- les maillages sont découpés en petits paquets de triangles (*clusters*), organisés en
  niveaux de détail ;
- le GPU choisit, cluster par cluster, le niveau suffisant pour l'écran ;
- il écarte lui-même tout ce qui est invisible ;
- il écrit sa propre liste de dessins.

Le CPU ne voit plus les triangles.

**Avant de commencer** : les *mesures GPU* (Phase 4) sont fortement conseillées. Chaque étape
doit prouver qu'elle est **plus rapide**, pas seulement plus complexe. Pour tester, il faut un
modèle très détaillé : un scan 3D gratuit, ou une sphère d'un million de triangles.

| Étape | Contenu | Gain visible | Difficulté |
|-------|---------|--------------|------------|
| **5.1 — Culling sur GPU** | Premiers **compute shaders** : le culling et le tri actuels (culling.rs) passent sur le GPU, qui écrit lui-même ses commandes de dessin (*indirect draw*, `draw_indexed_indirect`) | Le CPU ne touche plus aux objets : des dizaines de milliers d'objets sans ralentir la boucle | ⭐⭐⭐ |
| **5.2 — Meshlets** | À l'import, les maillages sont découpés en clusters d'environ 128 triangles (crate `meshopt`). Chaque cluster a sa sphère englobante et son **cône de normales**, ce qui permet d'écarter un cluster entièrement tourné vers l'arrière. Culling par cluster | Les gros modèles ne dessinent que leurs parties visibles | ⭐⭐⭐ |
| **5.3 — Occlusion culling (Hi-Z)** | La profondeur de l'image précédente est réduite en pyramide (*Hi-Z*), et les clusters cachés derrière d'autres objets sont écartés. Deux passes : d'abord ce qui était visible à l'image précédente, puis un nouveau test du reste contre la nouvelle profondeur | Ce qui est caché (derrière un mur, une colline) ne coûte plus rien | ⭐⭐⭐⭐ |
| **5.4 — Hiérarchie de LOD sans fissures** | Voir le détail sous le tableau | Le cœur de Nanite : des millions de triangles, fluides, sans changement de niveau visible | ⭐⭐⭐⭐⭐ |
| **5.5 — Visibility buffer** | Chaque pixel ne stocke que « tel cluster, tel triangle » (un entier). Une passe plein écran calcule ensuite les matériaux et l'éclairage, une seule fois par pixel | Plus aucun pixel n'est éclairé pour rien ; prépare l'étape suivante | ⭐⭐⭐⭐ |
| **5.6 — Rastérisation logicielle** | Les tout petits triangles (environ 1 pixel), mal traités par le matériel, sont dessinés par un compute shader. Il faut les **atomiques 64 bits** (option de wgpu, à vérifier sur chaque GPU), et garder le rendu de l'étape 5.5 en solution de repli | Le dernier gain de vitesse sur les scènes très détaillées | ⭐⭐⭐⭐⭐ |
| **5.7 — Streaming** | Ne garder en mémoire GPU que les clusters utiles, chargés depuis le disque à la demande, selon la distance | Des scènes plus grandes que la mémoire de la carte graphique | ⭐⭐⭐⭐⭐ |

Détail de l'étape 5.4, calculée une fois à l'import :
- on groupe des clusters voisins, on simplifie chaque groupe en divisant ses triangles par
  deux, en **verrouillant ses bords** (`meshopt::simplify`), puis on le redécoupe ;
- on recommence jusqu'à quelques triangles ;
- chaque cluster garde son **erreur de simplification**.

À l'affichage, un compute shader choisit, par cluster, le niveau dont l'erreur projetée à
l'écran fait moins d'un pixel. Les bords verrouillés permettent de mélanger des niveaux
différents dans le même objet sans fissures.

**Les étapes 5.1 à 5.3** apportent déjà un gros gain et sont accessibles. **L'étape 5.4** est
la vraie difficulté algorithmique. **Les étapes 5.5 à 5.7** relèvent du projet de recherche :
elles sont à faire seulement si les mesures montrent qu'on en a besoin.

**Références** :
- *Nanite: A Deep Dive*, Brian Karis, SIGGRAPH 2021 ;
- l'implémentation « meshlet » de Bevy, open source, en Rust et sur wgpu : la plus proche de
  ZENTRIX ;
- la bibliothèque meshoptimizer (Arseny Kapoulkine) et ses articles sur les clusters ;
- [nanites_et_lumen.md](../../nanites_et_lumen.md) : les adaptations pour les petits GPU et les
  solutions de repli.

---

## Phase 6 — Éclairage global (notre « Lumen »)

**Le but** : la lumière qui **rebondit**, sans précalcul. Le sol rouge teinte le mur voisin,
une lampe éclaire aussi le plafond par réflexion, et un intérieur devient sombre. Si un objet
bouge ou si le soleil tourne, l'éclairage suit.

Aujourd'hui, ZENTRIX ne calcule que la lumière **directe** (soleil, lampes), plus un ciel
précalculé identique partout. Il ne sait donc pas qu'un intérieur doit être sombre ; seules
les ombres du soleil s'en approchent.

**Comment Lumen fonctionne** :
- une scène simplifiée pour les rayons : des **champs de distance signée** (SDF) par objet,
  fusionnés en un champ global autour de la caméra ;
- un **cache de surfaces** (la couleur et l'éclairage des objets, mis à jour peu à peu, et qui
  accumule les rebonds) ;
- peu de rayons, lancés depuis des **sondes** espacées, avec une **accumulation
  temporelle** ;
- l'image déjà calculée est testée d'abord (*screen space*), le champ de distance ensuite.

**Notre version** : un « Lumen allégé » qui donne l'essentiel de l'effet avec des techniques
plus simples, qui s'empilent. À la place du cache de surfaces de Lumen (très complexe), on
utilise des **sondes DDGI**, plus accessibles et faciles à mettre à l'échelle.

**Avant de commencer** :
- les compute shaders et les mesures GPU (Phases 4 et 5) ;
- l'étape 6.4 réutilise la pyramide Hi-Z de l'étape 5.3 ;
- garder le ciel précalculé actuel en **solution de repli**, avec des réglages de qualité,
  comme le recommande [nanites_et_lumen.md](../../nanites_et_lumen.md).

| Étape | Contenu | Gain visible | Difficulté |
|-------|---------|--------------|------------|
| **6.1 — Prépasse profondeur et normales** | Une passe qui écrit la profondeur et la normale de chaque pixel dans des textures lisibles par les passes suivantes | Prépare tout le reste | ⭐⭐ |
| **6.2 — SSAO** | Occlusion ambiante à l'écran : quelques échantillons autour de chaque pixel dans le depth buffer, puis un flou | Les coins, creux et contacts s'assombrissent ; les objets « se posent » sur le sol | ⭐⭐⭐ |
| **6.3 — Accumulation temporelle** | **Vecteurs de mouvement** (où était ce pixel à l'image précédente ?), images d'historique, rejet des pixels qui ne correspondent plus | Indispensable pour lisser le bruit des effets à peu de rayons ; permet aussi un vrai TAA | ⭐⭐⭐⭐ |
| **6.4 — Reflets à l'écran (SSR)** | Un rayon par pixel réfléchissant, suivi dans le depth buffer grâce à la pyramide Hi-Z ; repli sur le ciel quand le rayon sort de l'écran | Les métaux et le sol mouillé reflètent la scène, plus seulement le ciel | ⭐⭐⭐⭐ |
| **6.5 — Champs de distance (SDF)** | À l'import, un SDF par maillage (texture 3D) ; chaque image, un **champ global** autour de la caméra, en niveaux de plus en plus grossiers (*clipmap*) ; le *sphere tracing* y avance par grands pas | Une scène traçable par rayons sur n'importe quel GPU, sans lancer de rayons matériel | ⭐⭐⭐⭐⭐ |
| **6.6 — Sondes d'éclairage dynamiques (DDGI)** | Voir le détail sous le tableau | **Le vrai éclairage global** : couleurs qui bavent, coins sombres ; de nuit, les lampes éclairent indirectement toute la zone | ⭐⭐⭐⭐⭐ |
| **6.7 — Rebonds multiples et reflets lointains** | Les sondes relisent leur propre résultat de l'image précédente : chaque image ajoute un rebond. Les reflets qui sortent de l'écran (SSR) continuent dans le champ de distance | Intérieurs crédibles, reflets complets | ⭐⭐⭐⭐ |
| **6.8 — Lancer de rayons matériel (facultatif)** | Si le GPU le permet (*ray queries*, expérimentales dans wgpu), les rayons touchent les vrais triangles au lieu du champ de distance | Plus précis, sur les cartes récentes seulement | ⭐⭐⭐⭐ |

Détail de l'étape 6.6 :
- une **grille 3D de sondes** couvre la scène ;
- à chaque image, une partie des sondes lance quelques dizaines de rayons dans le champ de
  distance, et mémorise la lumière reçue et la distance des obstacles (dans des textures
  « octaédriques ») ;
- chaque pixel interpole ses 8 sondes voisines. La distance des obstacles empêche la lumière
  de « fuir » à travers les murs.

**Les difficultés** :
- **le bruit** : avec peu de rayons, l'image grouille, et le débruitage temporel est plus
  subtil que le reste ;
- **le budget** : sur un petit GPU, l'éclairage global doit être mis à jour sur plusieurs
  images.

**Les étapes 6.1 à 6.4** sont accessibles et améliorent déjà nettement l'image. **Les étapes
6.5 et 6.6** forment le cœur de l'éclairage global.

**Références** :
- *Lumen: Real-time Global Illumination in Unreal Engine 5*, Daniel Wright, SIGGRAPH 2022 ;
- *Dynamic Diffuse Global Illumination with Ray-Traced Irradiance Fields*, Majercik et al.,
  JCGT 2019 ;
- SDFGI de Godot, open source ;
- *GTAO* (Jimenez et al., 2016) pour une SSAO de qualité.

---

[⬅️ Phase 4 — Nouvelles pistes](04-nouvelles-pistes.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phase 7 — Mondes, gameplay, outils et plateformes ➡️](07-mondes-gameplay-outils.md)
