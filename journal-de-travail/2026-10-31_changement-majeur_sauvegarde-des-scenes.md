# 2026-10-31 — Changement majeur : sauvegarde et chargement de scènes (RON)

## Résumé

Une scène entière (entités, composants du moteur et du jeu, hiérarchie, références aux
ressources) se sauvegarde dans un fichier texte RON lisible et se recharge à l'identique. Dans
la démo : `F5` sauvegarde dans `assets/scenes/demo.ron`, `F9` recharge. Étape M de la
[feuille de route](../documentation/guide-debutant/07-feuille-de-route.md).

## Pourquoi

C'est la base de tout éditeur (créer une scène, l'enregistrer, la rouvrir), des niveaux d'un
jeu stockés dans des fichiers plutôt que dans le code, et des sauvegardes de partie.

## Les trois difficultés et leurs solutions

| Difficulté | Solution |
|------------|----------|
| Les handles (`MeshHandle`, `TextureHandle`) sont des numéros internes au `Renderer`, différents à chaque exécution | Annuaire `AssetNames` (nom ↔ handle) : le fichier contient `"cube"`, `"caisse"`, `"arbre/Tronc"`… |
| Les `Entity` sont des numéros internes au monde | Chaque entité reçoit un `id` local au fichier ; `Parent` est écrit avec cet `id`, puis reconstruit au chargement (toutes les entités sont créées avant de poser les liens) |
| Les composants du jeu sont inconnus du moteur | `SceneSerializer::register::<T>("Nom")` pour tout type `Serialize + Deserialize` |

## Détail

### Dépendances

- Catalogue du workspace : `serde` 1 (avec `derive`), `ron` 0.12 ; `glam` gagne la
  fonctionnalité `serde`.
- `zentrix-core` : `Transform` et `Color` dérivent `Serialize` / `Deserialize`.

### `zentrix-scene`

| Élément | Rôle |
|---------|------|
| `src/serialization.rs` *(nouveau)* | `AssetNames`, `SceneSerializer` (`register`, `save`, `load`), `SceneError` |
| Composants | `Camera3d`, `DirectionalLight`, `Name` sérialisables |
| `Cargo.toml` | `serde`, `ron`, `thiserror` |

Comportement :

- `SceneSerializer::default()` connaît `Name`, `Transform`, `Camera3d`, `DirectionalLight`
  (+ `Parent` et `MeshRenderer`, traités à part) ;
- fichier **stable** : entités triées, composants rangés par nom (`BTreeMap`) — deux
  sauvegardes de la même scène sont identiques (bon pour git et les comparaisons) ;
- chargement **tout ou rien** : en cas d'erreur (ressource inconnue, composant invalide,
  parent manquant), les entités déjà créées sont supprimées ;
- composant inconnu dans le fichier : ignoré avec un avertissement (compatibilité) ;
- champ `version` (1) : un fichier d'une autre version est refusé ;
- `GlobalTransform` n'est pas sauvegardé (il est recalculé).

6 tests + 1 doc-test : aller-retour complet avec des handles renumérotés, stabilité, maillage
sans nom, tout ou rien, composant inconnu, version.

### `zentrix-app`

- `Spin` et `FlyCamera` sérialisables et déclarés dans `scene_serializer()`.
- `model::upload` inscrit les ressources du modèle dans l'annuaire (`arbre/Tronc`,
  `arbre/Feuillage`, `arbre/image0`) ; les ressources de la démo y sont inscrites dans
  `on_start` (`cube`, `cube_blanc`, `sol`, `soleil`, `planete`, `lune`, `caisse`, `damier`).
- `F5` → `save_scene`, `F9` → `load_scene` : le chargement se fait dans une scène neuve,
  qui ne remplace l'actuelle qu'en cas de succès.
- 1 test d'intégration : la scène complète de la démo (31 entités) est sauvegardée, rechargée,
  puis re-sauvegardée à l'identique.

### Dépôt

`.gitignore` : `/assets/scenes/` (les sauvegardes faites avec `F5` sont des données de
l'utilisateur).

## Vérification

- `cargo build`, `cargo clippy --workspace --all-targets` : aucun avertissement.
- `cargo test --workspace` : 75 tests réussis.
- Lancement sous Vulkan : `F5` → « Scène sauvegardée : … (52 entités) », fichier de 15,7 Ko ;
  3 s plus tard (cubes et planète ayant tourné), `F9` → « Scène rechargée » : les captures
  avant sauvegarde et après rechargement sont identiques (à quelques millisecondes
  d'animation près).
- Note : lors d'un premier essai automatisé, la touche `F5` injectée par le script n'a pas été
  reçue (appui et relâchement envoyés au même instant) ; avec un appui de 50 ms, tout
  fonctionne. Le problème venait du script de test, pas du moteur — à surveiller toutefois
  si un appui très bref semble ignoré en usage réel.
