![logo](./src/images/logo_transparant_256.png "logo")

![releaseRustyLibraryShrinker](./src/images/releaseRustyLibraryShrinker.svg "releaseRustyLibraryShrinker")
![license](./src/images/license.svg "license")
# Rust Library Shrinker

**Rust Library Shrinker** est une application Rust haute performance conçue pour compresser les fichiers de bandes dessinées (EPUB/CBR/CBZ/PDF) via un traitement parallèle. Elle convertit les images au format WebP pour une réduction optimale de la taille des fichiers tout en préservant une excellente qualité visuelle.

## ✨ Caractéristiques

- ✅ **Compatibilité multiplateforme** - Fonctionne sur Windows, macOS et Linux.
- ✅ **Parallélisme massif** - Traite simultanément plusieurs fichiers ET plusieurs images à l'intérieur de chaque fichier (parallélisme Rayon imbriqué).
- ✅ **Contrôle des ressources** - Possibilité de limiter le nombre de threads maximum via l'option `--threads`.
- ✅ **Support multi-format** - Gère les fichiers EPUB, CBR (RAR), CBZ (ZIP) et PDF avec détection automatique du format.
- ✅ **Support PDF Avancé** - Extraction directe des images intégrées (JPEG, PNG, JP2, CMYK) avec gestion des **SMasks (transparence)** et des profils **ICC**.
- ✅ **Support EPUB** - Extraction intelligente respectant l'ordre interne du manifeste.
- ✅ **Traitement automatique de dossiers** - Traite par défaut tous les fichiers de BD dans un répertoire.
- ✅ **Support des patterns Glob** - Traitement sélectif via des motifs (ex: "**/ABC*.cbr").
- ✅ **Visualisation de la progression** - Affichage multicouche style Docker (via Indicatif).
- ✅ **Préservation intelligente** - Conserve les fichiers déjà bien compressés si le gain est inférieur au seuil défini (5% par défaut).
- ✅ **Gestion des logs** - Sortie détaillée en texte brut pour le suivi des erreurs et des succès via `--log-file`.
- ✅ **Force la compression WebP** - Capable de redimensionner et de re-compresser des fichiers même s'ils sont déjà au format WebP.
- ✅ **Gestion robuste des erreurs** - Continue le traitement même avec des images corrompues ou hors limites WebP (max 16383px).
- ✅ **Format de sortie CBZ** - Génère systématiquement des fichiers .cbz standardisés, quel que soit le format d'entrée.
- ✅ **Binaire autonome** - Aucune dépendance externe requise pour l'exécution.

## 🚀 Installation

### Télécharger les binaires pré-compilés pour Linux (Recommandé) et Windows

Téléchargez la dernière version pour Linux et Windows depuis la page [GitHub Releases](https://github.com/StuffGest/RustyLibraryShrinker/releases).

#### Installation sur Linux :
```bash
# Télécharger et extraire
tar -xzf RustyLibraryShrinker-x86_64-unknown-linux-gnu.tar.gz
chmod +x RustyLibraryShrinker
sudo mv RustyLibraryShrinker /usr/local/bin/  # Optionnel : ajouter au PATH
```

#### Installation sur Windows :
```bash
# Télécharger et extraire le .exe
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

### Limiter l'usage CPU (Threads)
```bash
# Utilise seulement 4 threads pour rester fluide sur d'autres tâches
RustyLibraryShrinker --threads 4
```

### Générer un rapport de log
```bash
RustyLibraryShrinker /chemin/vers/bd/ --log-file session_shinker.log
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
RustyLibraryShrinker bd/ --file-mode rename --quality 85
# Résultat : Les originaux deviennent *.original.ext, les fichiers compressés gardent le nom propre.
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
- `--threads` / `-t` : Nombre de threads maximum (défaut : 0 pour auto).
- `--log-file` / `-l` : Chemin vers un fichier texte brut pour enregistrer le déroulement.
- `--max-dimension` / `-m` : Dimension maximale de secours (défaut : 1200).
- `--file-mode` / `-r` : Mode de gestion des fichiers (`suffix`, `rename`, `replace`).
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

### Gestion des fichiers de sortie (`--file-mode`)

L'option `--file-mode` (ou `-m`) permet de choisir entre trois stratégies :

1. **`suffix` (Défaut)** : L'original reste inchangé, un nouveau fichier est créé.
  - `MaBD.cbz` → `MaBD.optimise.cbz`
2. **`rename` (Backup)** : L'original est conservé sous un nouveau nom et le fichier compressé prend la place du nom d'origine.
  - `MaBD.cbz` → `MaBD.original.cbz` (sauvegarde) + `MaBD.cbz` (compressé)
  - `MaBD.cbr` → `MaBD.original.cbr` (sauvegarde) + `MaBD.cbz` (compressé)
  - `MaBD.pdf` → `MaBD.original.pdf` (sauvegarde) + `MaBD.cbz` (compressé)
3. **`replace` (Remplacement)** : L'original est supprimé et remplacé par le fichier compressé (pas de sauvegarde).
  - `MaBD.cbr` → (Supprimé) + `MaBD.cbz` (compressé)

**Exemple d'usage :**
```bash
# Remplacement direct sans créer de backup
RustyLibraryShrinker -r replace /chemin/bd/

# Mode avec sauvegarde (comportement classique)
RustyLibraryShrinker -r rename /chemin/bd/
```

## ⚡ Fonctionnalités de Performance

### Traitement Parallèle
- Les fichiers sont traités en parallèle via un pool de threads (work-stealing).
- Les images à l'intérieur de chaque fichier sont également traitées en parallèle.
- La progression est affichée simultanément pour chaque fichier sans scintillement du terminal.

### Compression Intelligente
- **Vérification du gain** : Revient automatiquement à l'original si le nouveau fichier ne respecte pas le seuil `--min-savings`.

### Efficacité Mémoire
- Utilise des répertoires temporaires sécurisés (via `tempfile`).
- Nettoyage automatique des images extraites après chaque fichier.
- Flux efficace pour la création d'archives volumineuses.

## 📊 Affichage de la Progression

```text
🚀 Found 3 comic file(s) to process
Settings: Quality=90, Target Height=1800px
-----------------------------------------------------
⠋ [████████████████████████████████████████] 2/3 fichiers (00:00:45)
📖 BD1.cbz [████████████████████████████████] 100%
📖 BD2.cbz [████████████████░░░░░░░░░░░░░░░░] 65%
📖 BD3.cbz [████░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 15%
```

## 📊 Rapport de Résumé (Log / Terminal)

```text
--- DÉTAIL PAR FICHIER ---
✅ BD1.cbz : 85.12 Mo -> 42.10 Mo (-50.5%)
⏭️  BD2.cbz : 12.05 Mo -> 12.05 Mo (pas de gain)
❌ BD3.pdf : Image corrompue ou dimension invalide

=====================================================
📊 RÉSUMÉ GLOBAL
-----------------------------------------------------
Fichiers total     : 3
Optimisés          : 2 ✅
Non optimisés      : 1 ⏭️
Échecs             : 0 ❌
-----------------------------------------------------
Images optimisées  : 154
Images ignorées    : 1
-----------------------------------------------------
Taille originale   : 97.17 Mo
Taille finale      : 54.15 Mo
Gain total         : 43.02 Mo (44.3%) 📉
=====================================================
```

## 📉 Résultats Réels

### Fichiers CBR/CBZ
```text
📖 Amber Blake - 01.cbr: 61.1% d'économie (77.9 Mo sauvés, 104 images traitées, 0 ignorées)
Réduction totale : 64.3% (152.8 Mo sauvés)
```

### Fichiers PDF
```text
📖 Brocéliande - Tome 06.pdf: 76.3% d'économie (91.1 Mo sauvés, 55 images traitées, 0 ignorées)
Taille originale : 119.4 Mo → Taille finale : 28.3 Mo
```

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

```bash
cargo build --release
strip target/release/RustyLibraryShrinker #Linux only
cargo doc --no-deps #generate doc
```