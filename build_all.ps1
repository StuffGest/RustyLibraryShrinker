# Script d'automatisation de build pour RustyLibraryShrinker
# S'execute automatiquement depuis le dossier racine du script

$ErrorActionPreference = "Stop"

# Définition dynamique des dossiers (utilise le dossier du script)
$BASE_DIR = $PSScriptRoot
$BUILD_DIR = Join-Path $BASE_DIR "build"
$DOC_DIR = Join-Path $BASE_DIR "doc"

Write-Host "=== Demarrage du script d'automatisation ===" -ForegroundColor Cyan

# --- ÉTAPE O : PURGE ET PRÉPARATION DES DOSSIERS ---
Write-Host "`n=== Nettoyage et preparation des dossiers ===" -ForegroundColor Magenta

# Purge du dossier build
if (Test-Path $BUILD_DIR) {
    Write-Host "Purge du dossier build existant..." -ForegroundColor Gray
    Remove-Item $BUILD_DIR -Recurse -Force
}
New-Item -ItemType Directory -Path $BUILD_DIR | Out-Null
Write-Host "Dossier cree : $BUILD_DIR" -ForegroundColor Gray

# Purge du dossier doc
if (Test-Path $DOC_DIR) {
    Write-Host "Purge du dossier doc existant..." -ForegroundColor Gray
    Remove-Item $DOC_DIR -Recurse -Force
}
New-Item -ItemType Directory -Path $DOC_DIR | Out-Null
Write-Host "Dossier cree : $DOC_DIR" -ForegroundColor Gray


# --- ÉTAPE 1 : BUILD WINDOWS ---
Write-Host "`n=== Compilation et archivage pour Windows ===" -ForegroundColor Magenta
Write-Host "Compilation native..." -ForegroundColor Gray
cargo build --release

$WIN_EXE = Join-Path $BASE_DIR "target\release\RustyLibraryShrinker.exe"
$WIN_ZIP = Join-Path $BUILD_DIR "RustyLibraryShrinker-x86_64-windows.zip"

Write-Host "Creation de l'archive ZIP..." -ForegroundColor Gray
Compress-Archive -Path $WIN_EXE -DestinationPath $WIN_ZIP
Write-Host "Archive Windows creee avec succes dans : $WIN_ZIP" -ForegroundColor Green


# --- ÉTAPE 2 : BUILD LINUX ---
Write-Host "`n=== Compilation et archivage pour Linux ===" -ForegroundColor Magenta
Write-Host "Compilation croisee via Cross..." -ForegroundColor Gray
cross build --target x86_64-unknown-linux-gnu --release

$LIN_TAR = Join-Path $BUILD_DIR "RustyLibraryShrinker-x86_64-unknown-linux-gnu.tar.gz"

Write-Host "Creation de l'archive TAR.GZ propre..." -ForegroundColor Gray
tar -czf $LIN_TAR -C target/x86_64-unknown-linux-gnu/release RustyLibraryShrinker
Write-Host "Archive Linux creee avec succes dans : $LIN_TAR" -ForegroundColor Green


# --- ÉTAPE 3 : GÉNÉRATION DE LA DOCUMENTATION ---
Write-Host "`n=== Generation de la documentation ===" -ForegroundColor Magenta
Write-Host "Generation via cargo doc..." -ForegroundColor Gray
cargo doc --no-deps

$SRC_DOC_DIR = Join-Path $BASE_DIR "target\doc"

Write-Host "Transfert de la doc vers : $DOC_DIR" -ForegroundColor Gray
# Copie du contenu complet de target/doc vers /doc
Copy-Item -Path "$SRC_DOC_DIR\*" -Destination $DOC_DIR -Recurse -Force
Write-Host "Documentation transferee avec succes !" -ForegroundColor Green

Write-Host "`n=== Tout est pret ! ===" -ForegroundColor Cyan
Write-Host "Les archives sont dans : $BUILD_DIR" -ForegroundColor Yellow
Write-Host "La doc est dans       : $DOC_DIR" -ForegroundColor Yellow