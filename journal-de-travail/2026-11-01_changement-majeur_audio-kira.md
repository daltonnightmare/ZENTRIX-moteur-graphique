# 2026-11-01 — Changement majeur : audio spatialisé (kira), crate `zentrix-audio`

## Résumé

Le moteur a maintenant du **son**, avec la bibliothèque [kira](https://docs.rs/kira) 0.12. Les
sons sont **placés dans la scène** : ils sont plus forts quand on s'approche de leur source, et
sortent du côté où elle se trouve.

Dans la démo :
- la caméra est l'**oreille** ;
- chaque **choc** de la physique produit un « toc » à l'endroit du choc, d'autant plus fort que
  le choc est violent ;
- lancer une balle (`B`) fait un **souffle** ;
- les deux caisses sci-fi **bourdonnent** en continu (leurs néons).

Les sons sont **synthétisés** par le code, donc aucun fichier audio n'est livré. `F8` coupe le
son ; l'inspecteur règle le volume.

## Pourquoi

C'était la piste « Audio » de la feuille de route. `kira` a été préférée à `rodio` pour trois
raisons :
- elle gère nativement le **son spatialisé** (auditeur et pistes spatiales) ;
- elle fournit un backend **factice** (`MockBackend`), qui permet de tester sans carte son ;
- elle utilise **la même version de glam (0.34)** que ZENTRIX : contrairement à rapier, aucune
  conversion n'est nécessaire.

## Détail

### Nouvelle crate `zentrix-audio`

| Fichier | Contenu |
|---------|---------|
| `components.rs` | `AudioListener` (composant marqueur, sur la caméra). `SoundEmitter { sound, volume }` : un son joué en boucle qui suit l'entité ; il est désigné par son **nom** pour pouvoir être sauvegardé. Les deux composants sont sérialisables |
| `synth.rs` | Échantillons mono à 44,1 kHz. `impact(hauteur)` : un claquement de bruit, puis une note qui s'éteint en exponentielle. `whoosh()` : un bruit filtré (passe-bas dont la coupure monte), avec une enveloppe en cloche, normalisé à 0,8. `hum()` : 110 Hz + harmoniques, chaque fréquence faisant un nombre entier de périodes en 2 s pour boucler sans clic. Le bruit vient d'un *xorshift* déterministe |
| `lib.rs` | `Audio<B: Backend = DefaultBackend>`, détaillé ci-dessous ; `to_decibels` (volume → dB, -60 dB = silence) ; `AudioError` |

**La structure `Audio`** :
- **`Audio::new()`** ouvre la sortie son par défaut. En cas d'échec, l'audio est **muet** (avec
  un avertissement) au lieu de planter.
- **`with_backend(réglages)`** ouvre une sortie précise ; **`muted()`** crée un audio sans
  sortie.
- **`add_sound(nom, échantillons)`** et **`load_sound(nom, fichier)`** (WAV/OGG) ajoutent un
  son ; **`sound(nom)`** le retrouve.
- **`play(son, volume, vitesse)`** joue un son sans position.
- **`play_at(son, position, volume, vitesse)`** joue un son ponctuel. Il crée une piste spatiale
  qui disparaît d'elle-même à la fin du son (`persist_until_sounds_finish`).
- **`set_master_volume`** règle le volume général.
- **`update(&Scene)`**, à appeler après `update_transforms` :
  - place l'auditeur sur la première entité `AudioListener` ;
  - démarre, déplace, règle et arrête les émetteurs. Un émetteur dont le son change est recréé ;
    une entité disparue coupe son son.
- **Distances d'écoute** : plein volume jusqu'à 1,5 m, inaudible au-delà de 40 m.

### Workspace

- Ajout de `kira = "0.12"`, sans ses options par défaut : seulement la sortie `cpal` et les
  formats WAV, OGG et Vorbis.
- Nouveau membre `zentrix-audio`. Ses dépendances : `zentrix-core` et `zentrix-scene`.
- L'audio ne dépend pas de la physique : c'est l'application qui transforme les chocs en sons.

### Physique

`Physics::impacts()` : voir l'[entrée dédiée](2026-11-01_amelioration_physique-detection-des-chocs.md).

### `zentrix-app`

| Fichier | Changement |
|---------|------------|
| `demo.rs` | `AudioListener` sur la caméra ; `SoundEmitter` sur les caisses sci-fi (volume 0,35) ; noms des sons `choc`, `lancer`, `bourdonnement` |
| `main.rs` | Ouverture de l'audio dans `on_start` (et non dans `new`) et synthèse des 3 sons. `play_impact_sounds` : les 6 chocs les plus forts de l'image, au-dessus de 1 m/s ; volume selon la vitesse ; hauteur légèrement variée. Souffle au lancer d'une balle. `audio.update` après `update_transforms`. `F8` (muet). Enregistrement de `AudioListener` et `SoundEmitter` pour la sauvegarde |
| `inspector.rs` | Curseur « volume » et case « Muet (F8) » |

## Vérification

- `cargo build`, `cargo clippy --workspace --all-targets`, `cargo fmt --check` : propres.
- `cargo test --workspace` : **112 tests** réussis.
  - **Audio** (7 tests, avec le backend factice de kira) :
    - conversion en décibels ;
    - sons nommés, lecture ponctuelle et sans position ;
    - les émetteurs suivent les entités : création, pas de doublon, arrêt à la suppression, son
      inconnu ignoré ;
    - un audio muet ne fait rien ;
    - les sons synthétisés restent entre -1 et 1, le choc s'éteint, le bourdonnement boucle
      sans saut.
  - **Démo** : la caméra écoute, 2 émetteurs.
- Un test a attrapé une **saturation** : le souffle montait jusqu'à 1,37 avant normalisation.
- Démo lancée (sortie son réelle de Windows) :
  - « Audio initialisé (kira) » ;
  - une rafale de 8 balles sans erreur ;
  - avec les logs de debug de l'audio, aucun son refusé par kira (pas de limite de pistes
    atteinte).
- **Non vérifié à l'oreille** : je n'ai pas pu écouter le résultat. La justesse de la
  spatialisation (gauche/droite selon l'orientation de la caméra), le niveau des sons et leur
  agrément restent à confirmer en jouant.

## Limites connues

- **Musique** : pas de musique ni de son long lu en continu depuis le disque (*streaming*). Tous
  les sons sont chargés en mémoire.
- **Effets** : pas de réverbération ni d'effet Doppler (le son d'un objet qui passe vite).
- **Position des chocs** : le son d'un choc est placé sur l'objet le plus rapide, pas au point
  de contact exact.
- **Un seul son de choc** : la matière (bois, métal, plastique) ne change pas le son.
- **Sauvegarde** : les sons ponctuels en cours ne sont pas sauvegardés, seulement les
  composants.
