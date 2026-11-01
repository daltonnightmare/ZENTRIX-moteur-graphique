# 2026-11-01 — Amélioration : mesures GPU (horodatages)

## Résumé

Le panneau **Performance** de l'inspecteur (`F4`) montre maintenant le temps passé par la
**carte graphique** sur chaque étape de l'image, en plus du temps CPU :

```text
✓ Mesures GPU   total 5.34 ms
GPU · ombres              1.03 ms
GPU · passe principale    2.72 ms
GPU · post-traitement     ...
GPU · interface           ...
```

Les barres sont rapportées à la durée d'une image. Quand le total GPU s'en approche, c'est la
carte graphique qui limite les FPS.

## Comment

- [zentrix-rhi](../crates/zentrix-rhi/src/context.rs) : l'option `TIMESTAMP_QUERY` est
  demandée **seulement si le GPU la propose**. Le journal indique « Mesures GPU
  (horodatages) : disponibles » ou « non proposées par ce GPU ».
- Nouveau module [gpu_timer.rs](../crates/zentrix-render/src/gpu_timer.rs).
  - **5 repères**, placés entre les groupes de passes : avant les ombres, puis après les
    ombres, la passe principale, le post-traitement et l'interface. Chaque repère est une
    passe de calcul **vide** qui n'écrit qu'un horodatage. Les passes existantes, dont la
    quinzaine de passes du bloom, n'ont pas changé.
  - **Aucune attente du GPU** : les horodatages sont recopiés dans un anneau de 3 buffers
    lisibles par le CPU (`map_async`), et relus une ou deux images plus tard (`poll` sans
    blocage). Si les 3 buffers sont occupés, l'image n'est simplement pas mesurée.
  - **Moyennes glissantes** (poids 0,1). Un écart négatif entre deux repères (horloge qui
    repart à zéro, pilote qui écrit dans le désordre) compte pour zéro.
  - **Sans l'option** : `GpuTimer::new` renvoie `None`, et le moteur fonctionne comme avant.
- [renderer.rs](../crates/zentrix-render/src/renderer.rs) :
  - les repères sont posés pendant l'encodage, les horodatages résolus avant
    `encoder.finish()`, la lecture demandée après l'envoi ;
  - `Renderer::gpu_timings()` donne les mesures, et `gpu_timing_enabled` permet de les couper.
- [inspector.rs](../crates/zentrix-app/src/inspector.rs) : les lignes « GPU · » et la case
  « Mesures GPU ».

## Vérifications

- 3 nouveaux tests :
  - conversion des graduations en millisecondes ;
  - écarts négatifs et horodatages manquants ;
  - tailles conformes aux règles de wgpu.
- 169 tests passent ; clippy (`-D warnings`) et `cargo fmt --check` sont propres.
- Sous Vulkan (Iris Xe), « Mesures GPU (horodatages) : disponibles » :
  - environ **5,3 ms** par image : 1,0 ms d'ombres, 2,7 ms de passe principale ;
  - avec le champ de stress (`N`), la passe principale passe à 3,6 ms ;
  - à 60 images/s (16,7 ms par image), le GPU de l'Iris Xe est donc occupé environ un tiers
    du temps sur la démo.

## Pourquoi

C'est l'étape 3 de l'ordre décidé (commit, intégration continue, mesures GPU, éditeur). Le
profileur ne voyait que le CPU. Or les chantiers à venir (upscaling, SSAO, Phase 9) se
jugent au temps **GPU** sur petit matériel, comme le demande la feuille de route.
