//! Orchestration du traitement des fichiers de bande dessinée.
//!
//! Ce module constitue le cœur de l'application. Il définit le flux de travail
//! pour l'optimisation d'un fichier :
//! 1. Extraction de l'archive (CBZ, CBR, PDF, EPUB).
//! 2. Identification et tri alphabétique des images extraites.
//! 3. Compression et redimensionnement des images en parallèle.
//! 4. Reconstruction d'une nouvelle archive CBZ.
//! 5. Renommage et archivage des fichiers originaux.

use anyhow::Result;
use indicatif::ProgressBar;
use std::fs;
use std::path::{Path, PathBuf};
use crossbeam_channel::{bounded, Receiver, Sender};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::{Args, ComicFile, ProcessingStats, FileMode};
use crate::archive::{self, zip_rar};
use crate::image_utils;

/// Orchestre le processus complet de traitement pour un fichier donné.
///
/// Cette fonction coordonne l'extraction, le tri, le traitement d'image et la
/// finalisation du fichier sur le disque.
///
/// # Paramètres
/// - `comic`: Référence vers les métadonnées du fichier de bande dessinée à traiter.
/// - `args`: Arguments de configuration (qualité, dimensions, etc.).
/// - `progress`: Barre de progression pour notifier l'avancement à l'utilisateur.
///
/// # Erreurs
/// Retourne une erreur si une étape du pipeline échoue (extraction, lecture, écriture).
pub fn process_comic_file(comic: &ComicFile, args: &Args, progress: &ProgressBar) -> Result<ProcessingStats> {
    let original_size = fs::metadata(&comic.path)?.len();
    let temp_dir = tempfile::tempdir()?;

    // 1. Extraction des images
    progress.set_position(10);
    archive::extract_comic(comic, temp_dir.path(), progress)?;

    // 2. Recherche et tri des fichiers extraits
    progress.set_position(30);
    let mut files = find_all_files(temp_dir.path())?;

    // Le tri alphabétique est essentiel pour garantir l'ordre de lecture
    files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    if files.is_empty() {
        return Err(anyhow::anyhow!("Aucun fichier trouvé après extraction"));
    }

    // 3. Traitement parallèle (compression/redimensionnement des images uniquement)
    let (proc, skip, skip_details) = process_images_parallel(&files, args, progress)?;

    // 4. Reconstruction de l'archive optimisée
    progress.set_position(85);
    let stem = comic.path.file_stem().unwrap().to_string_lossy();
    let parent = comic.path.parent().unwrap_or(Path::new("."));
    let temp_out = parent.join(format!("{}.new_tmp", stem));

    if temp_out.exists() { fs::remove_file(&temp_out)?; }

    zip_rar::create_cbz(temp_dir.path(), &temp_out)?;

    let new_size = fs::metadata(&temp_out)?.len();
    let savings_ratio = (original_size as f64 - new_size as f64) / original_size as f64;

    // 5. Vérification du gain de compression
    if savings_ratio < (args.min_savings as f64 / 100.0) {
        fs::remove_file(&temp_out)?;
        return Ok(ProcessingStats {
            original_size,
            compressed_size: original_size,
            images_processed: proc, // Type usize
            images_skipped: skip,   // Type usize
            skipped_details: skip_details,
            compression_skipped: true,
            output_path: None,
            error_message: None,
            status_message: Some("Gain insuffisant".to_string()),
        });
    }

    // 6. Déplacement final et renommage (gestion des 3 cas)
    let final_path = finalize_output_paths(comic, &temp_out, args)?;

    Ok(ProcessingStats {
        original_size,
        compressed_size: new_size,
        images_processed: proc, // Type usize
        images_skipped: skip,   // Type usize
        skipped_details: skip_details,
        compression_skipped: false,
        output_path: Some(final_path),
        error_message: None,
        status_message: None,
    })
}

/// Recherche récursivement tous les fichiers dans un répertoire.
///
/// Collecte l'ensemble des éléments afin de préserver la structure et les métadonnées.
///
/// # Paramètres
/// - `dir`: Chemin du répertoire à scanner.
fn find_all_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut list = Vec::new();
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            list.push(entry.path().to_path_buf());
        }
    }
    Ok(list)
}

/// Gère le traitement des images en parallèle à l'aide d'un pool de threads.
///
/// Utilise Rayon pour la distribution des tâches et un canal de communication
/// pour mettre à jour la progression via un thread dédié. Les fichiers qui ne sont
/// pas des images (comme ComicInfo.xml) sont ignorés volontairement sans erreur.
///
/// # Paramètres
/// - `files`: Liste des chemins vers les fichiers images à traiter.
/// - `args`: Paramètres de compression.
/// - `progress`: Barre de progression pour le suivi.
fn process_images_parallel(files: &[PathBuf], args: &Args, progress: &ProgressBar) -> Result<(usize, usize, Vec<(String, String)>)> {
    let total = files.len();
    // Le canal transmet désormais Result<(), (String, String)> pour capturer l'erreur
    let (s, r): (Sender<Result<(), (String, String)>>, Receiver<Result<(), (String, String)>>) = bounded(total);

    let proc = Arc::new(Mutex::new(0));
    let skip = Arc::new(Mutex::new(0));
    let skip_details = Arc::new(Mutex::new(Vec::new()));

    let p_clone = progress.clone();
    let proc_c = Arc::clone(&proc);
    let skip_c = Arc::clone(&skip);
    let details_c = Arc::clone(&skip_details);

    // Thread de gestion de la barre de progression et collecte des erreurs
    thread::spawn(move || {
        for res in r {
            match res {
                Ok(_) => { *proc_c.lock().unwrap() += 1; }
                Err(detail) => {
                    *skip_c.lock().unwrap() += 1;
                    details_c.lock().unwrap().push(detail);
                }
            }
            let curr = *proc_c.lock().unwrap() + *skip_c.lock().unwrap();
            p_clone.set_position(30 + ((curr * 50) / total) as u64);
        }
    });

    // Traitement parallèle des fichiers
    files.par_iter().for_each(|path| {
        let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        // Si ce n'est pas une extension d'image supportée, on n'y touche pas, on le garde intact (ex: ComicInfo.xml)
        if !matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "bmp" | "webp" | "jp2") {
            let _ = s.send(Ok(()));
            return;
        }

        let res = image_utils::process_single_image(path, args);

        match res {
            Ok(_) => { let _ = s.send(Ok(())); }
            Err(e) => { let _ = s.send(Err((file_name, e.to_string()))); }
        }
    });

    drop(s);

    // Attente de la fin du traitement de tous les fichiers
    while (*proc.lock().unwrap() + *skip.lock().unwrap()) < total {
        thread::yield_now();
    }

    let final_proc = *proc.lock().unwrap();
    let final_skip = *skip.lock().unwrap();
    let final_details = skip_details.lock().unwrap().clone();

    Ok((final_proc, final_skip, final_details))
}

/// Gère les opérations de renommage final et de sauvegarde de l'original.
///
/// Selon les options, renomme le fichier original en `nom_original.ext` et
/// place le nouveau fichier optimisé au nom d'origine.
///
/// # Paramètres
/// - `comic`: Informations sur le fichier original.
/// - `temp_out`: Chemin vers l'archive temporaire générée.
/// - `args`: Configuration contenant l'option de renommage.
fn finalize_output_paths(comic: &ComicFile, temp_out: &Path, args: &Args) -> Result<PathBuf> {
    let parent = comic.path.parent().unwrap_or(Path::new("."));
    let stem = comic.path.file_stem().unwrap().to_string_lossy();
    let original_ext = comic.path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match args.file_mode {
        FileMode::Suffix => {
            let dest = parent.join(format!("{}.optimise.cbz", stem));
            fs::rename(temp_out, &dest)?;
            Ok(dest)
        },
        FileMode::Rename => {
            let backup_name = format!("{}.original.{}", stem, original_ext);
            let backup_path = parent.join(backup_name);
            fs::rename(&comic.path, &backup_path)?;

            let dest = parent.join(format!("{}.cbz", stem));
            fs::rename(temp_out, &dest)?;
            Ok(dest)
        },
        FileMode::Replace => {
            let dest = parent.join(format!("{}.cbz", stem));
            if comic.path != dest {
                fs::remove_file(&comic.path)?;
            } else if dest.exists() {
                fs::remove_file(&dest)?;
            }
            fs::rename(temp_out, &dest)?;
            Ok(dest)
        }
    }
}