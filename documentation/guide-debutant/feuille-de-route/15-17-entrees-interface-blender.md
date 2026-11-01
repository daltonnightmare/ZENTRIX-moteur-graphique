# Feuille de route : Phases 15 à 17 — Entrées, interface et Blender

> Partie de la [feuille de route](../07-feuille-de-route.md) (chapitre 7 du guide).
> [⬅️ Phases 13 et 14 — Types de mondes, 2D et 3D](13-14-mondes-et-2d-3d.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 18 à 20 — Masse et gravité, saisons, PNJ ➡️](18-20-physique-saisons-pnj.md)

---

## Phase 15 — Entrées

**Le but** : que le joueur contrôle le jeu **comme il le veut** (clavier, souris, manette,
écran tactile), et que le code du jeu ne dépende jamais d'une touche précise.

**Où on en est** : `zentrix-platform` fournit `Input` (touches et boutons enfoncés, appuyés,
relâchés, mouvement et molette de la souris, capture du curseur). C'est la bonne fondation,
mais le jeu y lit des touches en dur (`KeyCode::KeyW`).

**Le principe** : le jeu ne demande plus « la touche W est-elle enfoncée ? » mais « l'action
**Avancer** vaut combien ? ». Une **carte des entrées** relie chaque action à des touches,
des boutons ou des axes, et le joueur peut la modifier.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **15.1 — Actions et contextes** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **15.2 — Manettes** | Crate `gilrs` : boutons, sticks, gâchettes analogiques, **zones mortes** et courbes de réponse, vibrations, branchement et débranchement à chaud, plusieurs manettes (jeu à plusieurs sur un écran) | ⭐⭐ |
| **15.3 — Touches reconfigurables** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **15.4 — Déclencheurs et modificateurs** | Appui long, appui bref, double appui, combinaisons (Ctrl + clic), **mémoire tampon** des entrées (un saut appuyé juste avant d'atterrir compte quand même), inversion d'axe, sensibilité | ⭐⭐⭐ |
| **15.5 — Pointeurs et tactile** | Mouvement brut de la souris (sans l'accélération du système, pour viser), écran tactile et gestes (glisser, pincer) pour le web et le mobile, joysticks virtuels à l'écran | ⭐⭐⭐ |
| **15.6 — Saisie de texte** | Les champs de texte de l'interface de jeu : saisie par l'**IME** de winit (accents, langues asiatiques), presse-papiers, clavier virtuel sur mobile | ⭐⭐ |
| **15.7 — Enregistrement et rejeu** | Enregistrer les actions avec leur instant, puis les rejouer : **tests automatiques** de gameplay, démos qui tournent seules, rapports de bug reproductibles, base de la prédiction réseau (Phase 7) | ⭐⭐⭐ |
| **15.8 — Accessibilité** | Toutes les actions reconfigurables, appui long remplaçable par une bascule, modes à une main, aide à la visée réglable, indications visuelles pour les sons importants | ⭐⭐ |

Détail de l'étape 15.1 :
- une **action** est un bouton (Sauter), un axe (Accélérer, de 0 à 1) ou un vecteur 2D
  (Se déplacer, depuis ZQSD, les flèches ou le stick gauche) ;
- des **contextes** empilés : à pied, en véhicule, dans un menu, pendant une cinématique
  (Phase 12). Seul le contexte du dessus reçoit les entrées : Échap ferme le menu au lieu
  d'ouvrir la pause ;
- la carte des entrées est un fichier, rechargeable sans recompiler ;
- la caméra libre actuelle devient le premier utilisateur de ce système.

Détail de l'étape 15.3 :
- un écran d'options où l'on clique sur une action puis on appuie sur la nouvelle touche, avec
  la détection des **conflits** (deux actions sur la même touche) ;
- la configuration est sauvegardée dans le dossier de l'utilisateur ;
- **claviers AZERTY et QWERTY** : winit distingue la touche **physique** (`KeyCode`, sa place
  sur le clavier) de la touche **logique** (la lettre imprimée). On relie les actions aux
  touches physiques, et ZQSD fonctionne sans réglage en AZERTY. L'interface affiche en
  revanche la lettre **logique**, celle que le joueur voit sur son clavier ;
- les **glyphes** : l'interface montre l'icône du bon bouton selon l'appareil utilisé en
  dernier (touche clavier, bouton Xbox ou PlayStation), voir la Phase 16.

**Références** :
- l'*Enhanced Input* d'Unreal et l'*Input System* d'Unity (actions, contextes,
  modificateurs) ;
- `leafwing-input-manager` (Rust, Bevy) ;
- les *Game Accessibility Guidelines* (gameaccessibilityguidelines.com).

---

## Phase 16 — Fenêtres, interface et icônes

**Le but** : un jeu qui **s'ouvre proprement** sur tous les écrans, avec une interface belle,
cohérente et lisible, et une identité visuelle (icônes, curseurs, écran de démarrage).

**Où on en est** : une seule fenêtre winit, configurée par `WindowConfig` (titre, taille),
et une interface egui qui sert d'inspecteur de debug. egui est parfait pour les outils, mais
pas pour l'interface d'un jeu (style, animations, navigation à la manette).

**Le principe** : deux interfaces séparées.
- **egui reste l'outil** des développeurs : inspecteur et futur éditeur.
- Une **interface de jeu** dessinée par le moteur, avec son propre **design system** : des
  thèmes, des styles et des composants réutilisables, décrits dans des fichiers que l'on
  modifie sans recompiler.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **16.1 — La fenêtre système** | Voir le détail sous le tableau | ⭐⭐ |
| **16.2 — Plusieurs fenêtres** | Pour l'éditeur (Phase 10) : vue de la scène, inspecteur et ligne de temps détachables sur un second écran. Une surface wgpu par fenêtre, un seul `Device` partagé | ⭐⭐⭐ |
| **16.3 — Interface de jeu** | Voir le détail sous le tableau | ⭐⭐⭐⭐ |
| **16.4 — Design system** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **16.5 — Adaptation aux écrans** | Ancrages (coin, bord, centre), **zones sûres** (marges des téléviseurs, encoches des téléphones), formats 16:9, 21:9, 4:3 et portrait, échelle d'interface réglable par le joueur, textes traduits plus longs, langues écrites de droite à gauche | ⭐⭐⭐ |
| **16.6 — Icônes** | Voir le détail sous le tableau | ⭐⭐ |
| **16.7 — Curseurs et identité** | Curseurs personnalisés (`CustomCursor` de winit) ou dessinés par le jeu, **écran de démarrage** (logo), écran de chargement (Phase 13) : la première impression du jeu | ⭐⭐ |
| **16.8 — Écrans standards** | Menu principal, pause, options (graphismes reliés aux préréglages de la Phase 9, touches de la Phase 15, audio, accessibilité), sauvegardes, crédits : des modèles prêts à l'emploi, à habiller avec le thème du jeu | ⭐⭐⭐ |

Détail de l'étape 16.1 :
- les **modes** : fenêtré, plein écran sans bordure, plein écran exclusif ;
- le choix de l'écran, de la résolution et de la fréquence de rafraîchissement ;
- la **mise à l'échelle** des écrans haute densité (`scale_factor` de winit) : l'interface
  garde la même taille physique sur un écran 4K ;
- la position, la taille et le mode mémorisés d'une session à l'autre ;
- le rendu en **pause** quand la fenêtre est réduite, et des économies (cadence réduite)
  quand elle perd le focus : important sur un portable sur batterie.

Détail de l'étape 16.3 :
- une **mise en page** de type flexbox (crate `taffy`, celle de Bevy et de Dioxus) : lignes,
  colonnes, marges, alignements, tailles en pixels ou en pourcentage ;
- des **widgets** : bouton, case à cocher, curseur, liste, onglets, champ de texte, barre de
  vie, info-bulle ;
- la **navigation à la manette** et au clavier : le focus passe d'un bouton à l'autre avec le
  stick, sans souris ;
- des **transitions** animées (apparition, survol, appui) avec les courbes de la Phase 12 ;
- le dessin réutilise les sprites (14.2), le texte (14.8) et le post-traitement : une
  interface peut briller avec le bloom.

Détail de l'étape 16.4 :
- un **thème** : palette de couleurs, typographies et tailles, espacements, arrondis, ombres
  portées, fichiers de sons d'interface ;
- des **styles** réutilisables, à la manière de CSS : changer la couleur primaire change tout
  le jeu ;
- des formes dessinées par **champ de distance** : des coins arrondis et des contours nets à
  toutes les tailles, sans image ;
- des **panneaux en 9 tranches** (*9-slice*) pour les cadres dessinés par un artiste ;
- un **verre dépoli** économique (flou de l'arrière-plan) qui réutilise la chaîne de mips du
  bloom ;
- le **rechargement à chaud** du thème, et un aperçu à plusieurs résolutions côte à côte.

Détail de l'étape 16.6 :
- **l'icône de l'application** : celle de la fenêtre (`Window::set_window_icon` de winit), de
  l'exécutable Windows (un `.ico` intégré au moment de la compilation, crate `embed-resource`),
  le `.icns` de macOS, le fichier `.desktop` de Linux, et le *favicon* de la version web. Une
  seule image source, et toutes les tailles générées automatiquement ;
- **les icônes d'interface** : un jeu d'icônes en **SVG**, transformées en images à la taille
  exacte demandée (crate `resvg`) et rangées dans un atlas : nettes sur tous les écrans ;
- **les glyphes de manette** : la bonne icône de touche ou de bouton selon l'appareil
  (Phase 15) ;
- **les vignettes de l'éditeur** : chaque modèle, matériau ou texture du projet a son aperçu,
  rendu hors écran (un matériau sur une sphère, un modèle sous trois quarts) et mis en cache.

**Références** :
- `taffy` (mise en page), `resvg` (SVG), `embed-resource` (ressources Windows) ;
- les interfaces de jeu de Godot (thèmes) et le *Common UI* d'Unreal (navigation à la
  manette) ;
- *Game UI Database* (gameuidatabase.com) pour l'inspiration.

---

## Phase 17 — Blender, rigs et pipeline de contenu

**Le but** : qu'un artiste crée **dans Blender** un modèle, un personnage riggé, ses
animations, voire un niveau entier, et le voie dans le moteur **en un clic**, sans retouche
manuelle.

**Où on en est** : `zentrix-assets` charge déjà le glTF (maillages, matériaux PBR, textures,
modes de transparence). Blender exporte le glTF nativement : c'est le bon **format pivot**.

**Le principe** : des **conventions** claires (unités, axes, noms, propriétés) que l'artiste
suit dans Blender, et un moteur qui les lit, les **vérifie** et explique précisément ce qui ne
va pas.

| Étape | Contenu | Difficulté |
|-------|---------|------------|
| **17.1 — Conventions d'export** | Voir le détail sous le tableau | ⭐⭐ |
| **17.2 — Propriétés Blender vers composants** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **17.3 — Collections, instances et niveaux** | Une collection Blender devient un **prefab** (Phase 7) ; les duplicatas liés deviennent des instances (extension `EXT_mesh_gpu_instancing`) ; un **niveau entier** peut être construit dans Blender et importé comme une scène | ⭐⭐⭐ |
| **17.4 — Rigs et squelettes** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **17.5 — Actions et animations** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **17.6 — Retargeting** | Un **squelette humanoïde de référence** ; chaque rig (Rigify, Mixamo, rig maison) y est relié os par os, une fois. Les animations passent ensuite d'un personnage à l'autre, même de proportions différentes : une bibliothèque d'animations partagée | ⭐⭐⭐⭐ |
| **17.7 — Lien direct Blender ↔ ZENTRIX** | Voir le détail sous le tableau | ⭐⭐⭐ |
| **17.8 — Autres formats et outils** | FBX (bibliothèque `ufbx`, ou conversion), OBJ ; textures de Substance Painter ou Material Maker, avec **empaquetage ORM** (occlusion, rugosité, métal dans une seule texture) ; Aseprite, Tiled et LDtk (Phase 14) ; le tout converti au format du moteur à la préparation (*cooking*, Phase 10) | ⭐⭐⭐ |

Détail de l'étape 17.1 : une page de documentation et une vérification à l'import.
- **unités** : 1 unité Blender = 1 mètre ; échelle appliquée (`Ctrl + A`) ;
- **axes** : Blender a Z vers le haut, glTF et ZENTRIX Y vers le haut ; l'exportateur
  convertit, à condition de garder l'option « +Y Up » ;
- **origine** au pied des objets posés au sol, modificateurs appliqués à l'export, UV
  présentes, au plus 4 os par sommet ;
- l'import **signale** clairement chaque écart : « caisse.glb : échelle non uniforme sur
  l'objet Couvercle », plutôt qu'un modèle invisible ou géant.

Détail de l'étape 17.2 :
- les **propriétés personnalisées** de Blender sont exportées dans les `extras` glTF. Le
  moteur les transforme en composants : `rigidbody = "dynamic"`, `mass = 20`, `sound =
  "bois.ogg"`, `script = "porte.rhai"` ;
- des **suffixes de noms** : `_col` (maillage de collision invisible), `_trigger` (zone de
  déclenchement), `_lod1`, `_lod2` (niveaux de détail faits à la main) ;
- les **lampes** Blender (extension `KHR_lights_punctual`) deviennent des `PointLight` et des
  `SpotLight` ; les **caméras** deviennent des `CineCamera` (Phase 12) ;
- un petit **add-on Blender** (étape 17.7) ajoute un panneau pour régler ces propriétés sans
  rien taper.

Détail de l'étape 17.4 :
- seuls les **os de déformation** sont exportés ; les os de contrôle (cibles d'IK, poignées
  des animateurs) servent dans Blender et sont « cuits » dans les animations ;
- un rig **Rigify** fonctionne, à condition d'exporter son squelette de déformation ;
- les noms d'os suivent une convention gauche / droite (`.L`, `.R`), pour la symétrie et le
  retargeting ;
- l'import vérifie la pose de repos, les poids normalisés et le nombre d'os.

Détail de l'étape 17.5 :
- chaque **action** Blender (ou piste NLA) devient un clip d'animation (Phase 11) ;
- les contraintes et l'IK de Blender sont **cuites** en clés à l'export ;
- les **shape keys** deviennent des morph targets (11.3) : visages, lèvres ;
- les **marqueurs** de la ligne de temps de Blender deviennent des **événements
  d'animation** (11.5) : un marqueur « pas_gauche » joue un bruit de pas ;
- un os racine nommé par convention porte le **root motion** (11.6).

Détail de l'étape 17.7 :
- un **add-on Blender** en Python exporte le modèle ou la collection dans le dossier `assets/`
  du projet, en un clic ou à chaque sauvegarde ;
- le moteur **surveille** ce dossier (crate `notify`) et **recharge à chaud** le modèle
  modifié dans la scène qui tourne, sans la relancer ;
- l'artiste voit son travail avec l'éclairage, les ombres et le post-traitement du jeu, en
  quelques secondes.

**Références** :
- la documentation de l'exportateur glTF de Blender (options, extras, actions, shape keys) ;
- les add-ons « Blender to Bevy » et `bevy_components`, qui font déjà ce pont pour Rust ;
- Rigify (Blender), Mixamo (animations gratuites), et la bibliothèque `ufbx`.

---

[⬅️ Phases 13 et 14 — Types de mondes, 2D et 3D](13-14-mondes-et-2d-3d.md) · [Sommaire de la feuille de route](../07-feuille-de-route.md#les-phases-suivantes) · [Phases 18 à 20 — Masse et gravité, saisons, PNJ ➡️](18-20-physique-saisons-pnj.md)
