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
- ✅ **Support PDF Avancé** - Extraction directe des images intégrées (JPEG, PNG, JP2, CMYK, TIFF, GIF, FF, ICO) avec gestion des **SMasks (transparence)** et des profils **ICC**.
- ✅ **Support EPUB** - Extraction intelligente respectant l'ordre interne du manifeste.
- ✅ **Internationalisation (i18n)** - Interface utilisateur, barres de progression, logs et documentation d'aide de la ligne de commande entièrement localisés en Français et Anglais.
- ✅ **Traitement automatique de dossiers** - Traite par défaut tous les fichiers de BD dans un répertoire.
- ✅ **Support des patterns Glob** - Traitement sélectif via des motifs (ex: "**/ABC*.cbr").
- ✅ **Visualisation de la progression** - Affichage multicouche style Docker (via Indicatif) traduit à la volée selon la langue active.
- ✅ **Préservation intelligente** - Conserve les fichiers déjà bien compressés si le gain est inférieur au seuil défini (5% par défaut) et préserve l'intégralité des métadonnées comme `ComicInfo.xml`.
- ✅ **Gestion des logs** - Sortie détaillée en texte brut localisée pour le suivi des erreurs et des succès via `--log-file`.
- ✅ **Force la compression WebP** - Capable de redimensionner et de re-compresser des fichiers même s'ils sont déjà au format WebP grâce à l'option `--force-shrink`.
- ✅ **Gestion robuste des erreurs** - Continue le traitement même avec des images corrompues ou hors limites WebP (max 16383px).
- ✅ **Format de sortie CBZ** - Génère systématiquement des fichiers .cbz standardisés, quel que soit le format d'entrée.
- ✅ **Binaire autonome** - Aucune dépendance externe requise pour l'exécution.

## 🚀 Installation

### Télécharger les binaires pré-compilés pour Linux (Recommandé) et Windows

Téléchargez la dernière version pour Linux et Windows depuis la page [GitHub Releases](https://github.com/StuffGest/RustyLibraryShrinker/releases).

#### Installation sur Linux :
```
# Télécharger et extraire
tar -xzf RustyLibraryShrinker-x86_64-unknown-linux-gnu.tar.gz
chmod +x RustyLibraryShrinker
sudo mv RustyLibraryShrinker /usr/local/bin/  # Optionnel : ajouter au PATH
```

#### Installation sur Windows :
```powershell
# Télécharger et extraire le .exe du RustyLibraryShrinker-x86_64-windows.zip
```

### Depuis les sources
```
git clone https://github.com/StuffGest/RustyLibraryShrinker.git
cd RustyLibraryShrinker
cargo build --release
```

Le binaire compilé sera disponible dans `target/release/RustyLibraryShrinker`.

## 🛠 Utilisation

### Gestion de la langue (Interface & CLI Help)
L'application s'adapte automatiquement à la langue du terminal système actuel (`LANG` ou `LC_ALL`). Néanmoins, vous pouvez forcer la langue manuellement.

**Exécuter l'application en Anglais (même si le système est en Français) :**
```
RustyLibraryShrinker --lang en
```

**Forcer l'anglais au niveau de la session de terminal pour le `--help` et l'affichage global :**
* *Linux/macOS :* `LANG=en_US.UTF-8 cargo run -- --help`
* *Windows (PowerShell) :* `$env:LANG="en_US.UTF-8"; cargo run -- --help`

### Traiter un fichier unique
```
RustyLibraryShrinker bd.cbz --quality 85
RustyLibraryShrinker bd.cbr --quality 85
RustyLibraryShrinker bd.pdf --quality 85
```

### Traiter toutes les BD du répertoire courant (comportement par défaut)
```
RustyLibraryShrinker
```

### Limiter l'usage CPU (Threads)
```
# Utilise seulement 4 threads pour rester fluide sur d'autres tâches
RustyLibraryShrinker --threads 4
```

### Générer un rapport de log
```
RustyLibraryShrinker /chemin/vers/bd/ --log-file session_shinker.log
```

### Traiter des fichiers déjà en WebP (Forcer le traitement)
```
RustyLibraryShrinker bd_webp.cbz --force-shrink --quality 75
```

### Traiter des fichiers via des patterns glob
```
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
```
RustyLibraryShrinker bd/ --quality 75 --target-height 1600
```

### Renommer les fichiers originaux (flux de travail pratique)
```
RustyLibraryShrinker bd/ --file-mode rename --quality 85
# Résultat : Les originaux deviennent *.original.ext, les fichiers compressés gardent le nom propre.
```

### Ignorer les fichiers déjà bien compressés
```
RustyLibraryShrinker bd/ --min-savings 10.0  # Compresse uniquement si le gain est > 10%
```

## ⚙️ Options

- `--lang` : Langue de l'interface (ex: `fr`, `en`). Par défaut retombe sur `fr`.
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
- `--force-shrink` : Force le décodage et la ré-optimisation des images même si elles sont déjà au format WebP.
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
```
RustyLibraryShrinker --glob-pattern "**/De Killer*.cbr"
```

**Trouver des fichiers dans une structure imbriquée spécifique :**
```
RustyLibraryShrinker --glob-pattern "**/Archives*/**/De Killer*.cbr"
```

**Trouver des volumes spécifiques :**
```
RustyLibraryShrinker --glob-pattern "**/*Volume 1*.cbr"
RustyLibraryShrinker --glob-pattern "**/*S0[1-3]*.cbr"  # Saisons 1 à 3
```

**Utiliser le mode verbeux pour déboguer :**
```
RustyLibraryShrinker --glob-pattern "**/Killer*.cbr" --verbose
```

### Gestion des fichiers de sortie (`--file-mode`)

L'option `--file-mode` (ou `-r`) permet de choisir entre trois stratégies :

1. **`suffix` (Défaut)** : L'original reste inchangé, un nouveau fichier est créé.
  - `MaBD.cbz` → `MaBD.optimise.cbz`
2. **`rename` (Backup)** : L'original est conservé sous un nouveau nom et le fichier compressé prend la place du nom d'origine.
  - `MaBD.cbz` → `MaBD.original.cbz` (sauvegarde) + `MaBD.cbz` (compressé)
  - `MaBD.cbr` → `MaBD.original.cbr` (sauvegarde) + `MaBD.cbz` (compressé)
  - `MaBD.pdf` → `MaBD.original.pdf` (sauvegarde) + `MaBD.cbz` (compressé)
3. **`replace` (Remplacement)** : L'original est supprimé et remplacé par le fichier compressé (pas de sauvegarde).
  - `MaBD.cbr` → (Supprimé) + `MaBD.cbz` (compressé)

**Exemple d'usage :**
```
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

L'affichage de la barre globale s'adapte dynamiquement selon la langue choisie (ex: `global [███] 2/3 fichiers` ou `global [███] 2/3 files`).

```
🚀 RustyLibraryShrinker : 3 fichier(s) à traiter
-----------------------------------------------------
⠋ global [████████████████████████████████████████] 2/3 fichiers (66%) [Temps écoulé: 00:00:45, Restant: 00:00:22]
  BD1.cbz [████████████████████████████████] 100%
  BD2.cbz [████████████████░░░░░░░░░░░░░░░░] 65%
  BD3.cbz [████░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 15%
```

## 📊 Rapport de Résumé (Log / Terminal)

```
--- DÉTAIL PAR FICHIER ---
✅ BD1.cbz : 85.12 Mo -> 42.10 Mo (-50.5%)
⏭️  BD2.cbz : 12.05 Mo -> 12.05 Mo (pas de gain)
❌ BD3.pdf : Échec du décodage de l'image (fichier peut-être corrompu)

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
```
📖 Amber Blake - 01.cbr: 61.1% d'économie (77.9 Mo sauvés, 104 images traitées, 0 ignorées)
Réduction totale : 64.3% (152.8 Mo sauvés)
```

### Fichiers PDF
```
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
- **Localisation** : `fluent-templates` (système de ressources Mozilla Fluent).

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

```
cargo build --release
strip target/release/RustyLibraryShrinker # Linux uniquement
cargo doc --no-deps                      # Générer la documentation technique interne en français
cross build --target x86_64-unknown-linux-gnu --release # Cross-compilation depuis Windows
```
