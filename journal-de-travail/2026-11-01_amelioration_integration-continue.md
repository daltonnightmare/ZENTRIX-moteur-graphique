# 2026-11-01 — Amélioration : intégration continue (GitHub Actions)

## Résumé

Nouveau fichier : [.github/workflows/ci.yml](../.github/workflows/ci.yml). À chaque push et à
chaque pull request, GitHub vérifie le projet sur ses propres machines :

| Tâche | Système | Commande |
|-------|---------|----------|
| Formatage | Linux | `cargo fmt --all -- --check` |
| Clippy et tests | Windows **et** Linux | `cargo clippy --workspace --all-targets -- -D warnings`, puis `cargo test --workspace` |

Détails :
- **Avertissements bloquants** : `RUSTFLAGS=-D warnings` fait des avertissements du compilateur
  des erreurs, comme `-D warnings` le fait pour clippy.
- **Les deux systèmes vont jusqu'au bout** (`fail-fast: false`) : un échec sous Linux
  n'empêche pas de voir le résultat sous Windows.
- **Un push annule la vérification précédente** de la même branche, devenue inutile
  (`concurrency`).
- **Cache des dépendances** (`Swatinem/rust-cache`) : la première exécution compile tout,
  les suivantes vont beaucoup plus vite.
- **Linux** : installation des bibliothèques nécessaires à la compilation du son (ALSA, pour
  kira) et de la fenêtre (Wayland, xkbcommon, udev).

Les tests n'ont besoin ni de GPU ni de carte son (handles factices, `Audio::muted`). Ils
tournent donc sur ces machines sans écran.

La piste « Intégration continue » de la
[Phase 7](../documentation/guide-debutant/feuille-de-route/07-mondes-gameplay-outils.md)
est marquée ✅.

## Première exécution

Réussie du premier coup : formatage en 7 s, Linux en 6 min 52 s, Windows en 11 min 23 s
(compilation complète, avant que le cache serve). ZENTRIX compile et passe ses tests sous
Linux.

GitHub signalait que `actions/checkout@v4` repose sur Node.js 20, en fin de vie : le
workflow utilise maintenant `actions/checkout@v5`.

## Pourquoi

C'est l'étape 2 de l'ordre décidé (commit, intégration continue, mesures GPU, éditeur) : la
plus petite des pistes, et celle qui protège toutes les autres. Elle vérifie aussi, pour la
première fois, que ZENTRIX **compile sous Linux**.
