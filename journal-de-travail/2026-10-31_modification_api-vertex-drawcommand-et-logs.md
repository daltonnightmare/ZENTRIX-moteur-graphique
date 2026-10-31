# 2026-10-31 — Modification : API de `Vertex` et `DrawCommand`, message des commandes

## Résumé

Changements d'API liés aux textures
(voir [textures et zentrix-assets](2026-10-31_changement-majeur_textures-et-crate-assets.md)),
et correction d'un message de log.

## Détail

| Élément | Avant | Après |
|---------|-------|-------|
| `Vertex::new` | `new(position, color)` | `new(position, color, uv)` |
| Taille de `Vertex` | 24 octets | 32 octets |
| Locations des instances (shader) | 2 à 5 | 3 à 6 |
| `DrawCommand` | `{ mesh, transform }` | `{ mesh, texture, transform }` (`texture: Option<TextureHandle>`) |
| `create_mesh_pipeline` | layout caméra | layouts caméra **et** texture |
| Couleur des sommets | couleur finale | **teinte** multipliée par la texture |

## Correction

Le message « Commandes : … » écrit au démarrage contenait une longue suite d'espaces : la
continuation de ligne `\` avait été perdue lors de l'étape précédente. Il est maintenant découpé
en deux lignes de log propres.

## Impact

Tout code qui construit des `Vertex` ou des `DrawCommand` doit ajouter le nouveau champ
(`uv`, `texture: None` pour garder le comportement précédent).
