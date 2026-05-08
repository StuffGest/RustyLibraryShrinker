# Rust Library Shrinker

**Rust Library Shrinker** est une application Rust haute performance conçue pour compresser les fichiers de bandes dessinées (EPUB/CBR/CBZ/PDF) via un traitement parallèle. Elle convertit les images au format WebP pour une réduction optimale de la taille des fichiers tout en préservant une excellente qualité visuelle.

## ✨ Caractéristiques

- ✅ **Compatibilité multiplateforme** - Fonctionne sur Windows, macOS et Linux.
- ✅ **Parallélisme massif** - Traite simultanément plusieurs fichiers ET plusieurs images à l'intérieur de chaque fichier (parallélisme Rayon imbriqué).
- ✅ **Support multi-format** - Gère les fichiers EPUB, CBR (RAR), CBZ (ZIP) et PDF avec détection automatique du format.
- ✅ **Support PDF Avancé** - Extraction directe des images intégrées (JPEG, PNG, JP2, CMYK) avec gestion des **SMasks (transparence)** et des profils **ICC**.
- ✅ **Support EPUB** - Extraction intelligente respectant l'ordre interne du manifeste.
- ✅ **Traitement automatique de dossiers** - Traite par défaut tous les fichiers de BD dans un répertoire.
- ✅ **Support des patterns Glob** - Traitement sélectif via des motifs (ex: "**/ABC*.cbr").
- ✅ **Visualisation de la progression** - Affichage multicouche style Docker (via Indicatif).
- ✅ **Préservation intelligente** - Conserve les fichiers déjà bien compressés si le gain est inférieur au seuil défini (5% par défaut).
- ✅ **Force la compression WebP** - Capable de redimensionner et de re-compresser des fichiers même s'ils sont déjà au format WebP.
- ✅ **Gestion robuste des erreurs** - Continue le traitement même avec des images corrompues ou des flux PDF inattendus.
- ✅ **Format de sortie CBZ** - Génère systématiquement des fichiers .cbz standardisés, quel que soit le format d'entrée.
- ✅ **Binaire autonome** - Aucune dépendance externe requise pour l'exécution.

## 🚀 Installation

### Télécharger les binaires pré-compilés pour Linux (Recommandé)

Téléchargez la dernière version pour Linux depuis la page [GitHub Releases](https://github.com/StuffGest/RustyLibraryShrinker/releases) :

- **Linux (x86_64)** : `RustyLibraryShrinker-x86_64-unknown-linux-gnu.tar.gz`

#### Installation sur Linux :
```bash
# Télécharger et extraire (remplacez l'URL par la dernière version)
tar -xzf RustyLibraryShrinker-x86_64-unknown-linux-gnu.tar.gz
chmod +x RustyLibraryShrinker
sudo mv RustyLibraryShrinker /usr/local/bin/  # Optionnel : ajouter au PATH
```

### Depuis les sources
```bash
git clone [https://github.com/StuffGest/RustyLibraryShrinker.git](https://github.com/StuffGest/RustyLibraryShrinker.git)
cd RustyLibraryShrinker
cargo build --release
```

Le binaire compilé sera disponible dans `target/release/RustyLibraryShrinker`.

## 🛠 Utilisation

### Traiter un fichier unique
```bash
RustyLibraryShrinker bd.cbz --quality 85
RustyLibraryShrinker bd.cbr --quality 85
RustyLibraryShrinker bd.pdf --quality 85
```

### Traiter toutes les BD du répertoire courant (comportement par défaut)
```bash
RustyLibraryShrinker
```

### Traiter toutes les BD d'un répertoire spécifique
```bash
RustyLibraryShrinker /chemin/vers/mes/bd/
```

### Traiter des fichiers via des patterns glob
```bash
# Motifs simples (recherche récursive automatique)
RustyLibraryShrinker --glob-pattern "ABC*.cbr"        # Fichiers commençant par "ABC" n'importe où
RustyLibraryShrinker --glob-pattern "*Killer*.cbr"    # Fichiers contenant "Killer" n'importe où

# Recherche récursive explicite
RustyLibraryShrinker --glob-pattern "**/De Killer*.cbr"  # Recherche récursive pour "De Killer"
RustyLibraryShrinker --glob-pattern "**/*Volume*/*.cbz"  # Motifs imbriqués complexes

# Motifs avec chemin absolu
RustyLibraryShrinker --glob-pattern "/chemin/complet/**/Killer*.cbr"

# Répertoire courant uniquement
RustyLibraryShrinker --glob-pattern "*.pdf"              # Fichiers PDF dans le dossier courant

# Déboguer vos motifs
RustyLibraryShrinker --glob-pattern "motif" --verbose    # Affiche les fichiers trouvés avant traitement
```

### Paramètres personnalisés
```bash
RustyLibraryShrinker bd/ --quality 75 --target-height 1600
```

### Renommer les fichiers originaux (flux de travail pratique)
```bash
RustyLibraryShrinker bd/ --rename-original --quality 85
# Résultat : Les originaux deviennent *_original.ext, les fichiers compressés gardent le nom propre.
```

### Ignorer les fichiers déjà bien compressés
```bash
RustyLibraryShrinker bd/ --min-savings 10.0  # Compresse uniquement si le gain est > 10%
```

## ⚙️ Options

- `--quality` / `-q` : Qualité WebP (1-100, défaut : 90)
  - 85-95 : Haute qualité, compression modérée.
  - 65-80 : Équilibre entre qualité et taille.
  - 40-60 : Petits fichiers, qualité inférieure.

- `--target-height` / `-H` : Hauteur cible des images en pixels (défaut : 1800).
- `--max-dimension` / `-m` : Dimension maximale de secours (défaut : 1200).
- `--rename-original` / `-r` : Renomme l'original en `<nom>_original.<ext>` et donne au fichier compressé le nom d'origine.
- `--glob-pattern` / `-g` : Traite uniquement les fichiers correspondant au motif glob.
- `--min-savings` : Pourcentage d'économie minimal requis pour conserver le fichier (défaut : 5.0).
- `--verbose` / `-v` : Active la sortie détaillée pour le débogage (utile pour l'analyse des flux PDF).
- `--skip-compression` / `-S` : Mode conversion uniquement : préserve les images originales sans ré-encodage.

## 💡 Astuces pour les Patterns Glob

Les patterns glob utilisent des jokers pour correspondre aux chemins :
- `*` correspond à n'importe quel caractère dans un nom de dossier ou fichier.
- `**` correspond à n'importe quel nombre de répertoires (récursif).
- `?` correspond à un seul caractère.
- `[abc]` correspond à n'importe quel caractère entre les crochets.

### Scénarios courants

**Trouver une série par nom n'importe où dans l'arborescence :**
```bash
RustyLibraryShrinker --glob-pattern "**/De Killer*.cbr"
```

**Trouver des fichiers dans une structure imbriquée spécifique :**
```bash
RustyLibraryShrinker --glob-pattern "**/Archives*/**/De Killer*.cbr"
```

**Trouver des volumes spécifiques :**
```bash
RustyLibraryShrinker --glob-pattern "**/*Volume 1*.cbr"
RustyLibraryShrinker --glob-pattern "**/*S0[1-3]*.cbr"  # Saisons 1 à 3
```

**Utiliser le mode verbeux pour déboguer :**
```bash
RustyLibraryShrinker --glob-pattern "**/Killer*.cbr" --verbose
```

## 📦 Sortie (Output)

### Comportement par défaut
L'outil crée de nouveaux fichiers avec le suffixe ` optimized_webp_q{quality}.cbz` :
- Entrée : `MaBD.cbz` → Sortie : `MaBD optimized_webp_q90.cbz`
- Entrée : `MaBD.cbr` → Sortie : `MaBD optimized_webp_q90.cbz`
- Entrée : `MaBD.pdf` → Sortie : `MaBD optimized_webp_q90.cbz`

### Avec l'option `--rename-original`
Le fichier compressé prend le nom d'origine :
- `MaBD.cbz` → `MaBD_original.cbz` (sauvegarde) + `MaBD.cbz` (compressé)
- `MaBD.cbr` → `MaBD_original.cbr` (sauvegarde) + `MaBD.cbz` (compressé)
- `MaBD.pdf` → `MaBD_original.pdf` (sauvegarde) + `MaBD.cbz` (compressé)

## ⚡ Fonctionnalités de Performance

### Traitement Parallèle
- Les fichiers sont traités en parallèle via un pool de threads (work-stealing).
- Les images à l'intérieur de chaque fichier sont également traitées en parallèle.
- La progression est affichée simultanément pour chaque fichier sans scintillement du terminal.

### Compression Intelligente
- **Ré-compression forcée** : Ré-encode systématiquement même si la source est en WebP pour garantir les dimensions et la qualité cibles.
- **Vérification du gain** : Revient automatiquement à l'original si le nouveau fichier ne respecte pas le seuil `--min-savings`.

### Efficacité Mémoire
- Utilise des répertoires temporaires sécurisés (via `tempfile`).
- Nettoyage automatique des images extraites après chaque fichier.
- Flux efficace pour la création d'archives volumineuses.

## 📊 Affichage de la Progression

L'outil affiche la progression de manière similaire aux téléchargements d'images Docker :

```text
🚀 Found 3 comic file(s) to process
Settings: Quality=90, Target Height=1800px
-----------------------------------------------------
⠋ [████████████████████████████████████████] 2/3 fichiers (00:00:45)
📖 BD1.cbz [████████████████████████████████] 100%
📖 BD2.cbz [████████████████░░░░░░░░░░░░░░░░] 65%
📖 BD3.cbz [████░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 15%
```

## 📊 Rapport de Résumé

Après le traitement, l'outil fournit un résumé détaillé :

```text
📊 RÉSUMÉ FINAL :
✅ BD1.cbz : -45.2 Mo (42.1% d'économie)
✅ BD2.cbz : -32.1 Mo (38.7% d'économie)
⏭️  BD3.cbz : Pas de gain (Ignoré)
-----------------------------------------------------
✅ 2 | ⏭️ 1 | ❌ 0
💰 Économie totale : 77.3 Mo
```

## 📉 Résultats Réels

### Fichiers CBR/CBZ
```text
📖 Amber Blake - 01.cbr: 61.1% d'économie (77.9 Mo sauvés, 104 images traitées, 0 ignorées)
📖 Auschwitz - 01.cbr: 67.9% d'économie (74.9 Mo sauvés, 84 images traitées, 0 ignorées)

Réduction totale : 64.3% (152.8 Mo sauvés)
Taille originale : 237.8 Mo → Taille finale : 85.0 Mo
```

### Fichiers PDF
```text
📖 Brocéliande - Tome 06.pdf: 76.3% d'économie (91.1 Mo sauvés, 55 images traitées, 0 ignorées)

Réduction totale : 76.3% (91.1 Mo sauvés)
Taille originale : 119.4 Mo → Taille finale : 28.3 Mo
```

### Pourquoi ces résultats ?
- **Les fichiers PDF** ont souvent les taux de compression les plus élevés car ils contiennent généralement des flux CMYK non compressés ou des scans bruts.
- **Le format WebP** combiné au **rééchantillonnage Lanczos3** offre un excellent rapport qualité/taille.
- **--rename-original** rend le flux de travail transparent pour maintenir les noms de votre bibliothèque tout en réduisant drastiquement l'espace disque.

## 🛠 Détails Techniques

- **Langage** : Rust (binaire autonome, zéro dépendance au runtime).
- **Traitement d'image** : Rééchantillonnage Lanczos3 de haute qualité via la crate `image`.
- **Compression** : Encodage WebP via la crate `webp`.
- **Format d'archive** : Fichiers CBZ basés sur le format ZIP standard.
- **Moteurs d'extraction** :
  - **CBR** : Support natif via `unrar`.
  - **CBZ** : Crate `zip`.
  - **PDF** : Extraction avancée via `lopdf` avec gestion ICC et SMask.
  - **EPUB** : Extraction basée sur le manifeste de ressources.
- **Multi-threading** : Rayon pour un parallélisme à haute efficacité.

## 📄 Détails du Support PDF

### ✅ Formats d'images PDF supportés
- **JPEG (DCTDecode)** : Extraction directe ou ré-encodage.
- **PNG/Compressé (FlateDecode)** : Reconstruction complète.
- **JPEG 2000 (JPXDecode)** : Supporté avec conversion de couleur ICC.
- **SMasks** : Application automatique du canal alpha pour les images transparentes.
- **CMYK** : Conversion automatique vers l'espace sRGB.

### ⚠️ Formats PDF non supportés
- **Graphismes vectoriels complexes** : Seules les images matricielles (raster) sont traitées.
- **Compression CCITT Fax** : Généralement ignorée pour les BD (utilisée pour les documents texte).

## ⚠️ Limitations

- La sortie est strictement standardisée au format CBZ.
- Les pages PDF contenant uniquement du vecteur n'auront pas d'images extraites.
- La taille du binaire est optimisée via LTO et le "stripping" pour la distribution.

## 📦 Compilation pour Distribution

Pour compiler des binaires optimisés :

```bash
cargo build --release
strip target/release/RustyLibraryShrinker  # Réduit la taille du binaire
```