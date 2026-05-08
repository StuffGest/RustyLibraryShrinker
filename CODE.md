# RustyLibraryShrinker

**RustyLibraryShrinker** est un outil en ligne de commande (CLI) ultra-rapide écrit en Rust, conçu pour optimiser et compresser les bibliothèques de bandes dessinées numériques (BD, Mangas, Comics). Il traite les fichiers en parallèle pour convertir les images internes au format WebP tout en réduisant leurs dimensions.

## 🚀 Commandes de Développement

### Build
```bash
cargo build --release     # Build de production optimisé
cargo check               # Vérification rapide de la syntaxe
```

### Exécution
```bash
# Exemple de compression agressive (test)
cargo run -- -H 250 --quality 10 ./ma_bd.cbz

# Utilisation courante
./target/release/RustyLibraryShrinker -H 2650 --quality 90 --rename-original /chemin/bd/
```

### Documentation
```bash
cargo doc --open          # Génère et ouvre la documentation technique (rustdoc)
```

---

## 🛠 Dépendances Clés

- **rayon** : Parallélisme massif (work-stealing) pour les fichiers et les images.
- **image** : Décodage et redimensionnement haute qualité (filtre Lanczos3).
- **webp** : Encodage WebP performant.
- **zip & unrar** : Gestion des archives CBZ (lecture/écriture) et CBR (lecture seule).
- **lopdf** : Extraction avancée d'images PDF avec gestion des **SMasks** (transparence) et des calques.
- **epub** : Parsing et extraction des ressources d'images dans les livres numériques.
- **indicatif** : Interface riche avec barres de progression multi-fichiers.
- **moxcms** : Gestion des profils colorimétriques **ICC** pour les extractions PDF/JP2.

---

## 🔄 Flux de Traitement (Pipeline)

1.  **Découverte** : Scan récursif ou via pattern **Glob** des fichiers (CBZ, CBR, PDF, EPUB).
2.  **Extraction** : Décompression dans un dossier temporaire sécurisé.
    * *PDF* : Recomposition des calques et application des masques alpha.
    * *EPUB* : Extraction via le manifest pour garantir l'ordre original.
3.  **Traitement d'Image** :
    * Redimensionnement proportionnel selon la hauteur cible (`--target-height`).
    * **Force Compression** : Ré-encodage systématique même si la source est déjà en WebP (si les dimensions ou la qualité diffèrent).
4.  **Reconstruction** : Création d'une nouvelle archive **systématiquement au format CBZ** (plus standard et ouvert que le CBR).
    * Tri alphabétique strict des fichiers pour respecter l'ordre de lecture.
5.  **Finalisation** : Vérification du gain de poids (seuil par défaut de 5%). Si le gain est insuffisant, l'original est conservé.

---

## ✨ Fonctionnalités Spéciales

-   **Sortie Universelle CBZ** : Quel que soit le format d'entrée (PDF, EPUB, CBR), le résultat est toujours un CBZ optimisé.
-   **Gestion Intelligente du WebP** : Capable de traiter et de réduire des fichiers déjà encodés en WebP en utilisant un mécanisme de fichier temporaire pour forcer l'écrasement.
-   **Extraction PDF Haute Fidélité** : Gère les images découpées en plusieurs flux et les espaces colorimétriques complexes (conversion sRGB via profil ICC).
-   **Optimisation du Gain** : Utilise l'argument `--min-savings` pour éviter de remplacer un fichier si l'économie d'espace est négligeable.
-   **Interface Docker-like** : Affichage clair de l'avancement global et individuel de chaque thread.

---

## 📈 Optimisations de Performance

-   **Double Parallélisme** : Les fichiers sont traités en parallèle, et à l'intérieur de chaque fichier, les images sont compressées simultanément.
-   **Zero-Copy (partiel)** : Utilisation de canaux `crossbeam` pour la communication entre les threads de traitement et l'UI.
-   **Profil Release** : Optimisations agressives (LTO, codegen-units = 1) pour réduire la taille du binaire et maximiser la vitesse d'exécution.
-   **Nettoyage Automatique** : Gestion rigoureuse des dossiers temporaires, même en cas de crash (via `tempfile`).

---

## 📖 Documentation Interne
Le code source est entièrement documenté avec les standards `rustdoc`. Utilisez `cargo doc` pour consulter les détails des modules `archive`, `image_utils`, et `processor`.