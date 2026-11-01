# 2026-10-31 — Modification : rendu en deux passes, `FrameUniform` étendu

## Résumé

Conséquences des [ombres portées](2026-10-31_changement-majeur_ombres-portees.md) sur
l'organisation du rendu.

## Détail

| Élément | Avant | Après |
|---------|-------|-------|
| Passes par image | 1 (principale) | 2 : ombre (si activée), puis principale |
| `FrameUniform` | 128 octets | 208 octets : + `light_view_proj` (mat4) et `shadow_params` (vec4) ; déclaré à l'identique dans les trois shaders |
| Bind groups du pipeline des maillages | 0 (image), 1 (texture) | + 2 (shadow map) |
| Boucle de dessin par lots | dans `render` | méthode `draw_batches(pass, bind_textures)`, partagée par les deux passes |
| `Renderer` | — | champs publics `shadows_enabled: bool` (vrai par défaut) et `shadow_area: ShadowArea` |
| Étapes numérotées de `render` | 1 à 7 | 1 à 8 (la passe d'ombre est l'étape 5) |

## Impact

Une application qui ne règle rien obtient des ombres sur une zone de rayon 12 autour de
l'origine (`ShadowArea::default()`).
