# Journal de travail — ZENTRIX

Trace datée de tout ce qui change dans le moteur : un fichier `.md` par changement.

## Convention de nommage

```text
AAAA-MM-JJ_<type>_<sujet-court>.md
```

| Type | Quand l'utiliser |
|------|------------------|
| `changement-majeur` | Nouvelle fonctionnalité, nouvelle architecture, changement d'API publique |
| `modification` | Changement de comportement ou de code existant, sans nouvelle fonctionnalité |
| `amelioration` | Performance, lisibilité, robustesse, tests, documentation enrichie |
| `suppression` | Code, fichier, fonctionnalité ou dépendance retiré(e) |

Si plusieurs fichiers ont la même date et le même type, on les distingue par le sujet.
L'ordre alphabétique des noms de fichiers donne donc l'ordre chronologique.

## Contenu d'une entrée

1. **Résumé** : une ou deux phrases.
2. **Pourquoi** : la motivation.
3. **Détail** : fichiers touchés et ce qui a changé.
4. **Vérification** : comment on a validé (tests, lancement, capture...).
5. **Impact / suite** : ce que ça casse ou débloque, prochaines étapes.

## Index

| Date | Type | Entrée |
|------|------|--------|
| 2026-07-13 | changement-majeur | [Création de la fenêtre et du projet](2026-07-13_changement-majeur_creation-fenetre-et-projet.md) |
| 2026-10-18 | changement-majeur | [Passage à un workspace multi-crates](2026-10-18_changement-majeur_workspace-multi-crates.md) |
| 2026-10-18 | suppression | [Ancien programme monolithique `src/`](2026-10-18_suppression_programme-monolithique-src.md) |
| 2026-10-31 | changement-majeur | [Rendu 3D : maillages, caméra, profondeur, instanciation](2026-10-31_changement-majeur_rendu-3d-maillages-camera-profondeur.md) |
| 2026-10-31 | suppression | [Triangle codé en dur et fond animé](2026-10-31_suppression_triangle-code-en-dur-et-fond-anime.md) |
| 2026-10-31 | modification | [Documentation mise à jour pour la Phase 2](2026-10-31_modification_documentation-phase-2.md) |
| 2026-10-31 | amelioration | [Création du journal de travail](2026-10-31_amelioration_journal-de-travail.md) |
| 2026-10-31 | changement-majeur | [Entrées clavier/souris et caméra libre](2026-10-31_changement-majeur_entrees-et-camera-libre.md) |
| 2026-10-31 | modification | [Commandes de la démo](2026-10-31_modification_commandes-de-la-demo.md) |
| 2026-10-31 | suppression | [Caméra en orbite automatique](2026-10-31_suppression_camera-en-orbite.md) |
| 2026-10-31 | changement-majeur | [Textures et crate `zentrix-assets`](2026-10-31_changement-majeur_textures-et-crate-assets.md) |
| 2026-10-31 | modification | [API de `Vertex` / `DrawCommand`, message des commandes](2026-10-31_modification_api-vertex-drawcommand-et-logs.md) |

> Les entrées du 2026-07-13 et du 2026-10-18 ont été reconstituées a posteriori à partir de
> l'historique git (`git log --stat`).
