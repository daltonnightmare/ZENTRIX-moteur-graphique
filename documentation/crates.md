Crate	            Rôle	                Dépendances
zentrix-core	Types fondamentaux (maths, couleurs, gestion d'erreurs). Indépendant du GPU et de la fenêtre.	nalgebra ou glam pour les maths


zentrix-platform	Abstrait la fenêtre, les entrées clavier/souris et la boucle d’événements (wrapper de winit).	winit, zentrix-core


zentrix-rhi	Encapsule wgpu : device, surface, buffers, textures, pipelines. C’est votre couche bas niveau GPU.	wgpu, pollster, bytemuck


zentrix-render	Gère le rendu haut niveau : les passes de rendu, les matériaux, les shaders, les caméras et les lumières. Utilise le RHI pour soumettre des commandes.	zentrix-core, zentrix-rhi


zentrix-scene	Implémente l’ECS (via bevy_ecs ou votre propre système). Contient les entités, composants et systèmes logiques.	zentrix-core, zentrix-render, zentrix-assets  

    
zentrix-assets	Chargement des fichiers (modèles, textures, shaders) depuis le disque, avec mise en cache et hot-reload.	

zentrix-core, tokio ou std::fs


zentrix-app	Point d’entrée. Assemble les crates, initialise la fenêtre, le renderer, la scène, et lance la boucle de jeu.	toutes les autres crates