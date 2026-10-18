Voici une documentation complète pour vous guider dans la création de votre propre système de contrôle de version (VCS) en Rust, inspiré de Git et GitHub.

---

# Créer un VCS (Git/GitHub-like) en Rust : Documentation complète

## 1. Introduction

Construire un système de contrôle de version est un projet ambitieux et formateur. En Rust, c'est particulièrement intéressant car le langage vous pousse à réfléchir à la propriété, à la gestion de la mémoire et à la sécurité, des aspects cruciaux pour un outil manipulant des données sensibles.

> *"J'ai construit Veridian, mon propre système de contrôle de version en Rust. Non pas parce que le monde a besoin d'un autre Git, mais parce que j'avais besoin de comprendre celui que nous avons déjà."*

Ce guide vous aidera à structurer votre réflexion et votre projet.

---

## 2. Concepts Fondamentaux : Ce qu'est un VCS

Avant d'écrire du code, il est essentiel de comprendre le modèle de données. Un VCS comme Git repose sur un principe simple : un **système de stockage adressable par le contenu**.

### 2.1. Le Modèle de Données (Inspiré de Git)

Au cœur du système, on trouve trois types d'objets stockés dans une base de données (le dossier `.git/objects`) :

*   **Blob (Binary Large Object)** : C'est le contenu d'un fichier. Git ne se soucie pas du nom du fichier, seulement de son contenu. Il ajoute un en-tête (`blob <taille>\0`), calcule un hash (SHA-1 ou BLAKE3), compresse le tout (avec zlib) et le stocke.
*   **Tree (Arbre)** : C'est un répertoire. Il liste les fichiers et sous-dossiers avec leurs noms et leurs hashs. Un tree pointe vers des blobs et d'autres trees.
*   **Commit** : C'est un instantané du projet. Chaque commit pointe vers un tree (l'état du projet), vers un ou plusieurs commits parents (l'historique), et contient des métadonnées (auteur, date, message).

### 2.2. La Magie du Système de Hash

Ce système est puissant pour plusieurs raisons :
*   **Dé-duplication** : Un même fichier dans plusieurs commits n'est stocké qu'une seule fois.
*   **Efficacité** : Si vous modifiez une ligne d'un gros fichier, seule la nouvelle version est stockée.
*   **Intégrité** : Le hash permet de vérifier instantanément l'intégrité des données.

### 2.3. Les Branches

Une branche est extrêmement simple : c'est un fichier (par exemple, `.git/refs/heads/main`) qui contient le hash du dernier commit de cette branche. C'est pourquoi créer une branche est une opération si légère.

---

## 3. Projets Existants en Rust (Pour s'inspirer)

Avant de commencer, étudier des projets existants est une excellente façon d'apprendre.

| Projet | Description | Lien |
| :--- | :--- | :--- |
| **gitoxide** | Une implémentation **pure Rust** de Git, très performante et modulaire. La bibliothèque `gix` est un excellent point de départ pour réutiliser des composants. | [GitoxideLabs/gitoxide](https://github.com/GitoxideLabs/gitoxide) |
| **Veridian** | Un VCS construit de zéro en Rust, avec une philosophie éducative. Parfait pour comprendre les bases. | [Kayleexx/veridian](https://github.com/Kayleexx/veridian) |
| **MUG** | Un VCS hybride avec des fonctionnalités Git-like, mais aussi des concepts modernes comme le hachage cryptographique (Ed25519) et le stockage centralisé de gros fichiers. | [rudywasfound/mug](https://github.com/rudywasfound/mug) |
| **RevTool** | Un VCS moderne mettant l'accent sur la simplicité. Il utilise BLAKE3 pour le hachage et propose une résolution de conflit interactive. | [SamuelSchlesinger/revtool](https://github.com/SamuelSchlesinger/revtool) |
| **Rustory** | Un outil de snapshot local léger, sans dépendances externes. Idéal pour une version simplifiée. | [uselibrary/rustory](https://github.com/uselibrary/rustory) |
| **version-control-rs** | Un moteur de contrôle de version compatible SVN, avec une base de données SQLite. | [ARyaskov/version-control-rs](https://github.com/ARyaskov/version-control-rs) |

---

## 4. Choix Techniques et Bibliothèques (Crates)

Voici les crates Rust que vous pourriez utiliser, organisées par fonctionnalité.

### 4.1. Cœur du VCS (Manipulation des objets)
*   **`gix`** : La bibliothèque de `gitoxide`. C'est la solution la plus complète pour interagir avec des dépôts Git. Elle gère la lecture/écriture d'objets, de références, l'index, etc.
*   **`flate2`** : Pour la compression/décompression zlib, utilisée par Git pour stocker les objets.
*   **`sha1`** ou **`blake3`** : Pour le hachage. SHA-1 est le standard de Git, BLAKE3 est plus moderne et rapide.
*   **`serde`** : Pour la sérialisation/désérialisation des métadonnées (commits, trees) en formats comme JSON, YAML ou TOML.

### 4.2. Interface en Ligne de Commande (CLI)
*   **`clap`** : La bibliothèque de référence pour parser les arguments en ligne de commande. Elle permet de créer facilement des sous-commandes (`init`, `add`, `commit`...).
*   **`anyhow`** / **`thiserror`** : Pour une gestion d'erreur simple et ergonomique.

### 4.3. Persistance et Base de Données
*   **`rusqlite`** : Pour une base de données SQLite. Utile pour stocker l'état de l'espace de travail, les fichiers ignorés, ou un index plus performant.
*   **Fichiers système** : Le plus simple est d'écrire et de lire des fichiers compressés dans le dossier `.monvcs/objects/`, comme le fait Git.

### 4.4. Réseau et Serveur (Pour la partie "GitHub")
*   **`axum`** ou **`actix-web`** : Pour créer le serveur HTTP qui hébergera les dépôts distants.
*   **`reqwest`** : Pour le client HTTP, afin de communiquer avec le serveur (`push`, `pull`, `clone`).
*   **`tokio`** : Le runtime asynchrone, indispensable pour un serveur performant.

### 4.5. Autres
*   **`ignore`** : Pour gérer les fichiers d'ignorance (`.gitignore`).
*   **`notify`** : Pour surveiller les changements dans le système de fichiers (utile pour un mode "capture continue").

---

## 5. Structure de Projet Proposée

Une architecture en workspace Cargo est idéale pour ce type de projet.

```text
my_vcs/
├── Cargo.toml                 # Workspace principal
├── .cargo/
│   └── config.toml
├── crates/
│   ├── core/                  # Le cœur du VCS (objets, repo, index)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── object/        # Blob, Tree, Commit
│   │       ├── storage/       # Lecture/Écriture sur le disque
│   │       └── repo.rs        # Structure représentant un dépôt
│   ├── cli/                   # L'interface en ligne de commande
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs        # Point d'entrée, parsing avec clap
│   │       └── commands/      # Implémentation des sous-commandes
│   ├── server/                # Le serveur (partie "GitHub")
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── routes.rs      # Endpoints HTTP (push, pull, etc.)
│   └── client/                # Le client pour communiquer avec le serveur
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── tests/                     # Tests d'intégration
└── examples/                  # Exemples d'utilisation
```

---

## 6. Guide de Développement par Étapes

### Étape 1 : Le Stockage de Base (Object Store)

**Objectif** : Savoir écrire et lire des objets (blobs) sur le disque.

*   Créer une fonction `write_object(content: &[u8]) -> String` qui :
    1.  Calcule le hash du contenu.
    2.  Compresse le contenu avec `flate2`.
    3.  Écrit le contenu compressé dans un fichier dont le chemin est basé sur le hash (ex: `.vcs/objects/ab/cdef1234...`).
*   Créer une fonction `read_object(hash: &str) -> Vec<u8>` qui effectue l'opération inverse.

### Étape 2 : Les Objets (Blob, Tree, Commit)

**Objectif** : Implémenter les structures de données.

*   Définir une énumération `ObjectType { Blob, Tree, Commit }`.
*   La structure `Tree` contient une liste de `(mode, name, hash)`.
*   La structure `Commit` contient `(tree_hash, parent_hash, author, timestamp, message)`.
*   Implémenter la sérialisation/désérialisation (par exemple, vers un format texte comme Git).

### Étape 3 : Le Dépôt (Repository)

**Objectif** : Créer la structure qui gère le dépôt.

*   `Repository::init(path)` : Crée le dossier `.vcs/` et les sous-dossiers (`objects/`, `refs/heads/`).
*   `Repository::add(path)` : Crée un blob à partir d'un fichier et l'ajoute à l'index (un fichier `.vcs/index` qui liste les fichiers suivis).
*   `Repository::commit(message)` : Crée un objet `Tree` à partir de l'index, puis un objet `Commit`, et met à jour la référence de la branche actuelle (`HEAD`).

### Étape 4 : L'Interface en Ligne de Commande (CLI)

**Objectif** : Rendre le moteur utilisable.

*   Utiliser `clap` pour définir la structure des commandes.
    ```rust
    // Exemple avec clap
    #[derive(Subcommand)]
    enum Commands {
        Init { path: Option<String> },
        Add { files: Vec<String> },
        Commit { message: String },
        // ... etc
    }
    ```
*   Connecter chaque commande à la logique du crate `core`.

### Étape 5 : Les Branches et les Références

**Objectif** : Gérer les branches.

*   `refs/heads/<branch_name>` : Un fichier contenant le hash du dernier commit.
*   `HEAD` : Un fichier contenant `ref: refs/heads/<current_branch>` ou un hash (détaché).
*   `Repository::branch(name)` : Crée un nouveau fichier de référence.
*   `Repository::checkout(name)` : Met à jour `HEAD` et restaure les fichiers de l'espace de travail à partir du commit pointé.

### Étape 6 : Le Serveur (Partie "GitHub")

**Objectif** : Permettre le travail à distance.

*   Créer un serveur HTTP avec `axum`.
*   **Endpoint `GET /repo/<name>/info/refs`** : Pour découvrir les branches disponibles (smart HTTP).
*   **Endpoint `POST /repo/<name>/git-upload-pack`** : Pour recevoir les objets (`git fetch`/`git clone`).
*   **Endpoint `POST /repo/<name>/git-receive-pack`** : Pour envoyer les objets (`git push`).
*   **Sécurité** : Ajouter une couche d'authentification (tokens, clés SSH).

### Étape 7 : Le Client Réseau

**Objectif** : Connecter le CLI au serveur.

*   Ajouter des commandes `clone`, `push`, `pull` au CLI.
*   Utiliser `reqwest` pour parler au serveur.
*   Pour `clone` : Récupérer les objets du serveur et les écrire dans le dépôt local.
*   Pour `push` : Envoyer les nouveaux objets et mettre à jour les références distantes.

---

## 7. Exemples de Commandes à Implémenter

| Commande | Description |
| :--- | :--- |
| `myvcs init` | Initialise un nouveau dépôt dans le dossier courant. |
| `myvcs add <fichier>` | Ajoute un fichier à l'index pour le prochain commit. |
| `myvcs commit -m "msg"` | Crée un commit avec les fichiers de l'index. |
| `myvcs status` | Affiche l'état de l'espace de travail par rapport au dernier commit. |
| `myvcs log` | Affiche l'historique des commits. |
| `myvcs branch` | Liste, crée ou supprime des branches. |
| `myvcs checkout <branche>` | Bascule sur une autre branche. |
| `myvcs remote add <origin> <url>` | Ajoute un dépôt distant. |
| `myvcs push` | Envoie les commits vers le dépôt distant. |
| `myvcs pull` | Récupère et fusionne les commits du dépôt distant. |
| `myvcs clone <url>` | Clone un dépôt distant en local. |

---

## 8. Concepts Avancés à Explorer

Une fois les bases en place, vous pouvez ajouter des fonctionnalités plus complexes :

*   **Merge (Fusion)** : Implémenter un algorithme de fusion à 3 points (base commune + deux branches).
*   **Gestion des conflits** : Proposer une interface interactive pour résoudre les conflits, comme RevTool.
*   **Rebase** : Rejouer des commits sur une autre base.
*   **Stash** : Sauvegarder temporairement des modifications non commitées.
*   **Hooks** : Permettre l'exécution de scripts personnalisés avant/après certaines actions (pre-commit, post-push).
*   **Packfiles** : Optimiser le stockage en groupant et compressant plusieurs objets ensemble.
*   **Serveur Web et Interface Graphique** : Créer une interface web pour visualiser l'historique et les branches.

---

## 9. Pour Aller Plus Loin

*   **Lisez le code de `gitoxide`** : C'est une référence en termes de code Rust idiomatique et performant pour un VCS.
*   **Suivez le tutoriel "Build your own Git"** : Il existe de nombreuses ressources en ligne qui décomposent l'implémentation de Git étape par étape.
*   **Implémentez un protocole de transport** : Le protocole "Smart HTTP" est utilisé par GitHub et GitLab. Le supporterait rendra votre outil compatible avec l'écosystème existant.

Bonne programmation !