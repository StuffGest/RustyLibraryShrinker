//! Gestion des archives compressées ZIP et RAR pour les bandes dessinées.
//!
//! Ce module fournit les fonctionnalités nécessaires pour manipuler les formats
//! d'archives les plus courants dans le monde des comics (CBZ et CBR).
//!
//! Les services inclus sont :
//! - L'extraction complète d'archives ZIP (CBZ).
//! - L'extraction complète d'archives RAR (CBR).
//! - La création d'archives CBZ optimisées à partir d'images traitées, en
//!   garantissant le respect de l'ordre alphabétique des fichiers.

use anyhow::Result;
use std::fs::{self, File};
use std::io::{BufReader};
use std::path::Path;
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

/// Extrait le contenu d'une archive ZIP vers un répertoire de destination.
///
/// Cette méthode parcourt l'index de l'archive et recrée l'arborescence des fichiers
/// dans le dossier temporaire spécifié.
///
/// # Paramètres
/// - `archive_path`: Le chemin d'accès vers le fichier `.zip` ou `.cbz`.
/// - `temp_dir`: Le répertoire cible où les fichiers doivent être extraits.
///
/// # Erreurs
/// Retourne une erreur si le fichier est inaccessible ou si le format ZIP est invalide.
pub fn extract_zip(archive_path: &Path, temp_dir: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = temp_dir.join(file.name());

        if let Some(p) = outpath.parent() {
            fs::create_dir_all(p)?;
        }

        if file.name().ends_with('/') {
            continue;
        }

        let mut outfile = File::create(&outpath)?;
        std::io::copy(&mut file, &mut outfile)?;
    }
    Ok(())
}

/// Extrait le contenu d'une archive RAR vers un répertoire de destination.
///
/// Utilise la bibliothèque `unrar` pour traiter les archives `.rar` ou `.cbr`.
///
/// # Paramètres
/// - `archive_path`: Le chemin d'accès vers le fichier `.rar` ou `.cbr`.
/// - `temp_dir`: Le répertoire cible pour l'extraction.
///
/// # Erreurs
/// Retourne une erreur si l'archive est protégée par mot de passe, corrompue
/// ou si l'extraction échoue.
pub fn extract_rar(archive_path: &Path, temp_dir: &Path) -> Result<()> {
    let archive = unrar::Archive::new(archive_path)
        .open_for_processing()
        .map_err(|e| anyhow::anyhow!("Erreur ouverture RAR: {:?}", e))?;

    let mut current = archive;
    loop {
        match current.read_header() {
            Ok(Some(header)) => {
                current = header.extract_with_base(temp_dir)
                    .map_err(|e| anyhow::anyhow!("Erreur extraction RAR: {:?}", e))?;
            }
            Ok(None) => break,
            Err(e) => return Err(anyhow::anyhow!("Erreur lecture RAR: {:?}", e)),
        }
    }
    Ok(())
}

/// Crée une nouvelle archive CBZ (format ZIP) à partir des images traitées.
///
/// Cette fonction est cruciale pour l'ordre de lecture : elle trie les fichiers
/// par nom de manière alphabétique avant de les intégrer dans l'archive.
/// Elle ne sélectionne que les fichiers au format `.webp`.
///
/// # Paramètres
/// - `source_dir`: Le répertoire contenant les images optimisées.
/// - `output_path`: Le chemin du fichier `.cbz` final à générer.
///
/// # Erreurs
/// Retourne une erreur si la création du fichier ou l'écriture des données échoue.
pub fn create_cbz(source_dir: &Path, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);

    // On utilise "Stored" (pas de compression supplémentaire) car les WebP sont déjà compressés.
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Stored);

    let mut entries: Vec<_> = WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    // TRI ALPHABÉTIQUE IMPÉRATIF pour garantir l'ordre de lecture des pages.
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for entry in entries {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        let name_lower = name.to_lowercase();

        // On n'ajoute que les fichiers convertis en .webp
        if name_lower.ends_with(".webp")
            || name_lower == "comicinfo.xml"
            || matches!(path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "bmp" | "jp2")
        {
            zip.start_file(name, options)?;
            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        }
    }

    zip.finish()?;
    Ok(())
}