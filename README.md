![logo](./src/images/logo_transparant_256.png "logo")

![releaseRustyLibraryShrinker](./src/images/releaseRustyLibraryShrinker.svg "releaseRustyLibraryShrinker")
![license](./src/images/license.svg "license")
# Rust Library Shrinker

**Rust Library Shrinker** is a high-performance Rust application designed to compress comic book files (EPUB/CBR/CBZ/PDF) using parallel processing. It converts images to WebP format for optimal file size reduction while preserving excellent visual quality.

## ✨ Features

- ✅ **Cross-platform compatibility** - Works on Windows, macOS, and Linux.
- ✅ **Massive parallelism** - Concurrently processes multiple files AND multiple images within each file (nested Rayon parallelism).
- ✅ **Resource control** - Ability to limit the maximum number of threads using the `--threads` option.
- ✅ **Multi-format support** - Handles EPUB, CBR (RAR), CBZ (ZIP), and PDF files with automatic format detection.
- ✅ **Advanced PDF Support** - Direct extraction of embedded images (JPEG, PNG, JP2, CMYK, TIFF, GIF, FF, ICO) with full management of **SMasks (transparency)** and **ICC** profiles.
- ✅ **EPUB Support** - Intelligent extraction respecting the internal order of the resource manifest.
- ✅ **Internationalization (i18n)** - User interface, progress bars, logs, and command-line help documentation fully localized in French and English.
- ✅ **Automatic folder processing** - Processes all comic book files in a directory by default.
- ✅ **Glob pattern support** - Selective processing using patterns (e.g., "**/ABC*.cbr").
- ✅ **Progress visualization** - Docker-like multi-layered progress display (via Indicatif) translated on the fly according to the active language.
- ✅ **Intelligent preservation** - Keeps files that are already well-compressed if the size savings are below the defined threshold (5% by default) and preserves all metadata like `ComicInfo.xml`.
- ✅ **Log management** - Detailed localized plain-text output for tracking errors and successes via `--log-file`.
- ✅ **Force WebP compression** - Capable of resizing and re-compressing files even if they are already in WebP format thanks to the `--force-shrink` option.
- ✅ **Robust error handling** - Continues processing even with corrupted images or images exceeding WebP limits (max 16383px).
- ✅ **CBZ output format** - Systematically generates standardized .cbz files, regardless of the input format.
- ✅ **Standalone binary** - No external dependencies required for execution.

## 🚀 Installation

### Download pre-compiled binaries for Linux (Recommended) and Windows

Download the latest version for Linux and Windows from the [GitHub Releases](https://github.com/StuffGest/RustyLibraryShrinker/releases) page.

#### Installation on Linux:
```
# Download and extract
tar -xzf RustyLibraryShrinker-x86_64-unknown-linux-gnu.tar.gz
chmod +x RustyLibraryShrinker
sudo mv RustyLibraryShrinker /usr/local/bin/  # Optional: add to PATH
```

#### Installation on Windows:
```
# Download and extract the .exe from RustyLibraryShrinker-x86_64-windows.zip
```

### From Source
```
git clone https://github.com/StuffGest/RustyLibraryShrinker.git
cd RustyLibraryShrinker
cargo build --release
```

The compiled binary will be available in `target/release/RustyLibraryShrinker`.

## 🛠 Usage

### Language Management (Interface & CLI Help)
The application automatically adapts to the current system terminal language (`LANG` or `LC_ALL`). However, you can force the language manually.

**Run the application in English (even if the system is in French):**
```
RustyLibraryShrinker --lang en
```

**Force English at the terminal session level for `--help` and global display:**
* *Linux/macOS:* ``` LANG=en_US.UTF-8 cargo run -- --help ```
* *Windows (PowerShell):* ``` $env:LANG="en_US.UTF-8"; cargo run -- --help ```

### Process a Single File
```
RustyLibraryShrinker comic.cbz --quality 85
RustyLibraryShrinker comic.cbr --quality 85
RustyLibraryShrinker comic.pdf --quality 85
```

### Process All Comics in Current Directory (Default Behavior)
```
RustyLibraryShrinker
```

### Limit CPU Usage (Threads)
```
# Uses only 4 threads to remain smooth for other system tasks
RustyLibraryShrinker --threads 4
```

### Generate a Log Report
```
RustyLibraryShrinker /path/to/comics/ --log-file session_shrink.log
```

### Process Files Already in WebP (Force Processing)
```
RustyLibraryShrinker comic_webp.cbz --force-shrink --quality 75
```

### Process Files Using Glob Patterns
```
# Simple patterns (automatic recursive search)
RustyLibraryShrinker --glob-pattern "ABC*.cbr"        # Files starting with "ABC" anywhere
RustyLibraryShrinker --glob-pattern "*Killer*.cbr"    # Files containing "Killer" anywhere

# Explicit recursive search
RustyLibraryShrinker --glob-pattern "**/The Killer*.cbr"  # Recursive search for "The Killer"
RustyLibraryShrinker --glob-pattern "**/*Volume*/*.cbz"  # Complex nested patterns

# Patterns with absolute path
RustyLibraryShrinker --glob-pattern "/absolute/path/**/Killer*.cbr"

# Current directory only
RustyLibraryShrinker --glob-pattern "*.pdf"              # PDF files in the current folder Only

# Debug your patterns
RustyLibraryShrinker --glob-pattern "pattern" --verbose    # Displays found files before processing
```

### Custom Parameters
```
RustyLibraryShrinker comics/ --quality 75 --target-height 1600
```

### Rename Original Files (Practical Workflow)
```
RustyLibraryShrinker comics/ --file-mode rename --quality 85
# Result: Originals become *.original.ext, optimized files keep the clean original name.
```

### Skip Files with Insufficient Savings
```
RustyLibraryShrinker comics/ --min-savings 10.0  # Compresses only if savings are > 10%
```

## ⚙️ Options

- `--lang` : Interface language (e.g., `fr`, `en`). Defaults to `fr`.
- `--quality` / `-q` : WebP quality (1-100, default: 90)
  - 85-95: High quality, moderate compression.
  - 65-80: Balance between quality and size.
  - 40-60: Small files, lower quality.
- `--target-height` / `-H` : Target image height in pixels (default: 1800).
- `--threads` / `-t` : Maximum number of threads (default: 0 for auto).
- `--log-file` / `-l` : Path to a plain text file to save the execution logs.
- `--max-dimension` / `-m` : Maximum fallback safety dimension (default: 1200).
- `--file-mode` / `-r` : Output file management mode (`suffix`, `rename`, `replace`).
- `--glob-pattern` / `-g` : Processes only files matching the glob pattern.
- `--min-savings` : Minimum savings percentage required to keep the optimized file (default: 5.0).
- `--force-shrink` : Forces image decoding and re-optimization even if they are already in WebP format.
- `--verbose` / `-v` : Enables detailed output for debugging (useful for PDF stream analysis).
- `--skip-compression` / `-S` : Conversion mode only: preserves original images without re-encoding.

## 💡 Glob Pattern Tips

Glob patterns use wildcards to match paths:
- `*` matches any character within a folder or file name.
- `**` matches any number of directories (recursive).
- `?` matches a single character.
- `[abc]` matches any character enclosed inside the brackets.

### Common Scenarios

**Find a series by name anywhere in the directory tree:**
```
RustyLibraryShrinker --glob-pattern "**/The Killer*.cbr"
```

**Find files in a specific nested structure:**
```
RustyLibraryShrinker --glob-pattern "**/Archives*/**/The Killer*.cbr"
```

**Find specific volumes:**
```
RustyLibraryShrinker --glob-pattern "**/*Volume 1*.cbr"
RustyLibraryShrinker --glob-pattern "**/*S0[1-3]*.cbr"  # Seasons 1 to 3
```

**Use verbose mode to debug:**
```
RustyLibraryShrinker --glob-pattern "**/Killer*.cbr" --verbose
```

### Output File Management (`--file-mode`)

The `--file-mode` (or `-r`) option allows choosing between three strategies:

1. **`suffix` (Default)**: The original remains unchanged, a new file is created.
- `MyComic.cbz` → `MyComic.optimise.cbz`
2. **`rename` (Backup)**: The original is kept under a new name and the compressed file takes the original filename.
- `MyComic.cbz` → `MyComic.original.cbz` (backup) + `MyComic.cbz` (compressed)
- `MyComic.cbr` → `MyComic.original.cbr` (backup) + `MyComic.cbz` (compressed)
- `MyComic.pdf` → `MyComic.original.pdf` (backup) + `MyComic.cbz` (compressed)
3. **`replace` (Replacement)**: The original is deleted and replaced by the compressed file (no backup).
- `MyComic.cbr` → (Deleted) + `MyComic.cbz` (compressed)

**Usage Example:**
```
# Direct replacement without creating a backup
RustyLibraryShrinker -r replace /path/to/comics/

# Mode with backup (classic behavior)
RustyLibraryShrinker -r rename /path/to/comics/
```

## ⚡ Performance Features

### Parallel Processing
- Files are processed in parallel via a thread pool (work-stealing).
- Images inside each file are also processed in parallel.
- Progress is displayed simultaneously for each file without terminal flickering.

### Intelligent Compression
- **Savings Verification**: Automatically rolls back to the original file if the new file does not meet the `--min-savings` threshold.

### Memory Efficiency
- Uses secure temporary directories (via `tempfile`).
- Automatic cleanup of extracted images after each file.
- Efficient streaming layout for creating large archives.

## 📊 Progress Display

The global bar display adapts dynamically depending on the chosen language (e.g., `global [███] 2/3 fichiers` or `global [███] 2/3 files`).

```
🚀 RustyLibraryShrinker: 3 file(s) to process
-----------------------------------------------------
⠋ global [████████████████████████████████████████] 2/3 files (66%) [Elapsed time: 00:00:45, Remaining: 00:00:22]
Comic1.cbz [████████████████████████████████] 100%
Comic2.cbz [████████████████░░░░░░░░░░░░░░░░] 65%
Comic3.cbz [████░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 15%
```

## 📊 Summary Report (Log / Terminal)

```
--- DETAILED RESULTS PER FILE ---
✅ Comic1.cbz : 85.12 MB -> 42.10 MB (-50.5%)
⏭️  Comic2.cbz : 12.05 MB -> 12.05 MB (no gain)
❌ Comic3.pdf : Failed to decode image (file might be corrupted)

=====================================================
📊 GLOBAL SUMMARY
-----------------------------------------------------
Total files        : 3
Optimized          : 2 ✅
Not optimized      : 1 ⏭️
Failed             : 0 ❌
-----------------------------------------------------
Images optimized   : 154
Images skipped     : 1
-----------------------------------------------------
Original size      : 97.17 MB
Final size         : 54.15 MB
Total gain         : 43.02 MB (44.3%) 📉
=====================================================
```

## 📉 Real-World Results

### CBR/CBZ Files
```
📖 Amber Blake - 01.cbr: 61.1% savings (77.9 MB saved, 104 images processed, 0 skipped)
Total reduction: 64.3% (152.8 MB saved)
```

### PDF Files
```
📖 Brocéliande - Tome 06.pdf: 76.3% savings (91.1 MB saved, 55 images processed, 0 skipped)
Original size: 119.4 MB → Final size: 28.3 MB
```

## 🛠 Technical Details

- **Language**: Rust (standalone binary, zero runtime dependencies).
- **Image Processing**: High-quality Lanczos3 resampling via the `image` crate.
- **Compression**: WebP encoding via the `webp` crate.
- **Archive Format**: CBZ files based on the standard ZIP format.
- **Extraction Engines**:
  - **CBR**: Native support via `unrar`.
  - **CBZ**: `zip` crate.
  - **PDF**: Advanced extraction via `lopdf` with ICC and SMask management.
  - **EPUB**: Extraction based on the resource manifest.
- **Multi-threading**: Rayon for high-efficiency parallelism.
- **Localization**: `fluent-templates` (Mozilla Fluent resource system).

## 📄 PDF Support Details

### ✅ Supported PDF Image Formats
- **JPEG (DCTDecode)**: Direct extraction or re-encoding.
- **PNG/Compressed (FlateDecode)**: Full reconstruction.
- **JPEG 2000 (JPXDecode)**: Supported with ICC color conversion.
- **SMasks**: Automatic alpha channel application for transparent images.
- **CMYK**: Automatic conversion to sRGB color space.

### ⚠️ Unsupported PDF Formats
- **Complex Vector Graphics**: Only raster images are processed.
- **CCITT Fax Compression**: Generally ignored for comic books (used for text documents).

## ⚠️ Limitations

- Output is strictly standardized to the CBZ format.
- PDF pages containing only vectors will not have images extracted.
- Binary size is optimized via LTO and stripping for distribution.

## 📦 Compiling for Distribution

```
cargo build --release
strip target/release/RustyLibraryShrinker # Linux only
cargo doc --no-deps                      # Generate internal technical documentation
cross build --target x86_64-unknown-linux-gnu --release # Cross-compilation from Windows
```