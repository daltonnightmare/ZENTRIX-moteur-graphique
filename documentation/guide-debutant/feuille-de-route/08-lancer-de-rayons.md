# Feuille de route : Phase 8 — Lancer de rayons logiciel

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phase 7 — Mondes, gameplay, outils et plateformes](07-mondes-gameplay-outils.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 9 et 10 — Beau sur petit matériel, contenu et production ➡️](09-10-beau-sur-petit-materiel.md)

---

## Phase 8 — Lancer de rayons logiciel (adapté aux petits GPU)

**Le but** : des effets en lancer de rayons (ombres douces des lampes, occlusion, reflets,
rebonds) **sans cœurs RT**. Les cartes modestes (Intel Iris Xe, GPU intégrés, mobiles) n'en ont
pas, et l'accès matériel de wgpu (*ray queries*) est encore expérimental. Les rayons sont donc
lancés par des **compute shaders**, dans une structure construite par le moteur.

**Le BVH** (*Bounding Volume Hierarchy*) : les triangles sont rangés dans une hiérarchie de
boîtes englobantes. Un rayon qui rate une boîte ignore tout son contenu : chercher parmi un
million de triangles ne coûte qu'une quarantaine de tests de boîtes.
- **Deux niveaux** : un BVH par maillage (BLAS, construit une fois) et un petit BVH des
  instances (TLAS, reconstruit à chaque image), pour que les objets puissent bouger.
- **Compact** : nœuds de 32 octets, triangles réduits à leurs positions. La mémoire est le
  goulot d'un petit GPU.
- **Géométrie simplifiée** pour les rayons, grâce aux niveaux de détail de l'étape 5.4.

**Les règles d'or du low-end** :
1. **Rendu hybride** : ce que la caméra voit reste rastérisé ; seuls les effets secondaires
   sont tracés.
2. **Moins d'un rayon par pixel** : demi ou quart de résolution, puis agrandissement qui
   respecte la profondeur et les normales (*upsampling bilatéral*).
3. **Étaler dans le temps** : 1 rayon par pixel et par image, accumulé sur les images
   précédentes (étape 6.3).
4. **Débruiter** : filtre temporel, puis spatial (*à-trous*, une version simplifiée de SVGF).
5. **Rayons courts** : au-delà d'une distance, retour à l'environnement précalculé.
6. **Budget mesuré** : nombre de rayons et résolution réglés selon le temps GPU mesuré ;
   chaque effet a un niveau « éteint », avec la technique actuelle en solution de repli.

**Avant de commencer** :
- les compute shaders (étape 5.1) ;
- les mesures GPU (Phase 4) ;
- l'accumulation temporelle (étape 6.3) pour les étapes 8.4 à 8.8.

| Étape | Contenu | Gain visible | Difficulté |
|-------|---------|--------------|------------|
| **8.1 — BVH sur le CPU** | Construction par l'heuristique de surface (SAH) par paquets, aplatissement dans un tableau, intersections rayon-boîte et rayon-triangle ; entièrement testé sans GPU | Les fondations, vérifiables par des tests | ⭐⭐⭐ |
| **8.2 — Parcours sur le GPU** | Compute shader qui parcourt le BVH (petite pile par rayon) ; vue de debug : distance touchée, nombre de boîtes testées en carte de chaleur | On « voit » la scène par les rayons, et on mesure leur coût | ⭐⭐⭐ |
| **8.3 — Deux niveaux (BLAS + TLAS)** | Un BVH par maillage, et celui des instances reconstruit à chaque image | Les objets qui bougent (cubes qui tournent, physique) restent corrects | ⭐⭐⭐ |
| **8.4 — Ombres des lampes** | Voir le détail sous le tableau | Les lampes et projecteurs ont enfin des ombres, douces | ⭐⭐⭐ |
| **8.5 — Débruiteur** | Filtre temporel (historique, rejet des pixels qui ne correspondent plus) puis spatial (à-trous guidé par la profondeur et les normales), réutilisable par tous les effets | Des effets à 1 rayon par pixel, propres et stables | ⭐⭐⭐⭐ |
| **8.6 — Occlusion ambiante tracée** | Rayons très courts (1 à 2 m) dans l'hémisphère de chaque point | Plus juste que la SSAO (étape 6.2), qui ne voit que l'écran | ⭐⭐⭐ |
| **8.7 — Reflets tracés** | 1 rayon par pixel sur les surfaces lisses seulement (or, verre, sol mouillé) ; repli sur la SSR (étape 6.4) ou sur l'environnement | Les objets se reflètent, même hors de l'écran | ⭐⭐⭐⭐ |
| **8.8 — Éclairage global et matériel** | Les sondes DDGI (étape 6.6) tracent dans le BVH au lieu du champ de distance, ou en combinant les deux. Chemin matériel facultatif (*ray queries*), derrière la même interface « lancer un rayon », quand le GPU le permet | Rebonds plus précis ; les cartes récentes vont plus vite sans code en double | ⭐⭐⭐⭐⭐ |

Détail de l'étape 8.4, le meilleur premier effet pour ZENTRIX :
- aujourd'hui, `PointLight` et `SpotLight` n'ont aucune ombre, car une shadow map en cube
  coûterait 6 passes par lampe ;
- ici, chaque pixel lance un rayon vers la lampe : le point visé varie un peu sur la taille de
  la lampe, ce qui donne des **ombres douces** ;
- seules les lampes les plus proches sont traitées, à demi-résolution.

**Mesurer tôt** : aucune promesse de vitesse n'est possible à l'avance. Elle dépend de la
taille de la scène et de la qualité du BVH. Les mesures commencent dès l'étape 8.2.

**Liens avec les autres phases** : la Phase 5 fournit la géométrie simplifiée et les compute
shaders ; la Phase 6 fournit l'accumulation temporelle, la SSR et les sondes DDGI. Le BVH
devient la représentation précise près de la caméra, et le champ de distance (étape 6.5) la
représentation économique au loin.

**Références** :
- *How to build a BVH*, série d'articles de Jacco Bikker ;
- *Ray Tracing Gems* I et II (gratuits) ;
- *Spatiotemporal Variance-Guided Filtering*, Schied et al., 2017.

---

[⬅️ Phase 7 — Mondes, gameplay, outils et plateformes](07-mondes-gameplay-outils.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 9 et 10 — Beau sur petit matériel, contenu et production ➡️](09-10-beau-sur-petit-materiel.md)
