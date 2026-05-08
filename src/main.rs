//! Point d'entrée principal de RustyLibraryShrinker.
//!
//! Ce module gère l'interface utilisateur en ligne de commande, la découverte
//! des fichiers via patterns glob ou parcours de répertoires, et orchestre
//! le traitement parallèle des archives.
//!
//! L'application utilise :
//! - `clap` pour le parsing des arguments.
//! - `rayon` pour le parallélisme massif sur les coeurs CPU.
//! - `indicatif` pour un affichage multi-barres de progression élégant.

mod models;
mod image_utils;
mod archive;
mod processor;

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

use crate::models::{Args, ComicFile, ComicType, ProcessingStats};

/// Fonction principale initialisant l'environnement et lançant le traitement.
fn main() -> Result<()> {
    let args = Args::parse();
    let input_path = args.input.clone().unwrap_or_else(|| PathBuf::from("."));

    // 1. Découverte des fichiers (Logique Glob Récursive)
    let files = if let Some(pattern) = &args.glob_pattern {
        let full_pattern = if input_path.is_dir() {
            if !pattern.contains("**") {
                let mut p = input_path.clone();
                p.push("**");
                p.push(pattern);
                p.to_string_lossy().into_owned()
            } else {
                let mut p = input_path.clone();
                p.push(pattern);
                p.to_string_lossy().into_owned()
            }
        } else {
            pattern.clone()
        };
        find_by_glob(&full_pattern)?
    } else if input_path.is_file() {
        vec![detect_type(&input_path)?]
    } else {
        find_in_dir(&input_path)?
    };

    if files.is_empty() {
        println!("❌ Aucun fichier trouvé.");
        return Ok(());
    }

    println!("🚀 RustyLibraryShrinker : {} fichier(s) à traiter", files.len());
    println!("-----------------------------------------------------");

    // 2. UI - Configuration du MultiProgress pour l'affichage des barres
    let multi = Arc::new(MultiProgress::new());

    // Barre globale (toujours visible en haut de la console)
    let main_bar = multi.add(ProgressBar::new(files.len() as u64));
    main_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} global [{bar:40.cyan/blue}] {pos}/{len} fichiers | {elapsed}")?);

    let stats_map = Arc::new(Mutex::new(HashMap::new()));

    // 3. Boucle de traitement parallèle avec Rayon
    // Le nombre de fichiers traités simultanément est limité par le nombre de coeurs CPU.
    files.par_iter().for_each(|f| {
        // Création d'une barre de progression secondaire pour le fichier en cours
        let pb = multi.add(ProgressBar::new(100));
        pb.set_style(ProgressStyle::default_bar()
            .template("  {msg} [{bar:30.green}] {percent}%")
            .unwrap());

        let file_name = f.path.file_name().unwrap_or_default().to_string_lossy();
        pb.set_message(file_name.to_string());

        // Délégation du travail au processeur
        match processor::process_comic_file(f, &args, &pb) {
            Ok(s) => {
                stats_map.lock().unwrap().insert(f.path.clone(), s);
            }
            Err(e) => {
                let mut m = stats_map.lock().unwrap();
                let size = fs::metadata(&f.path).map(|m| m.len()).unwrap_or(0);
                m.insert(f.path.clone(), ProcessingStats {
                    original_size: size,
                    compressed_size: size,
                    images_processed: 0,
                    images_skipped: 0,
                    compression_skipped: false,
                    output_path: None,
                    error_message: Some(e.to_string()),
                    status_message: None,
                });
            }
        }

        // Nettoyage de la barre individuelle après fin de traitement pour garder la console propre
        pb.finish_and_clear();
        main_bar.inc(1);
    });

    main_bar.finish_with_message("Traitement terminé !");

    // Nettoyage de l'interface MultiProgress avant l'affichage du bilan
    let _ = multi.clear();

    let final_stats = stats_map.lock().unwrap();
    print_final_summary(&final_stats);

    Ok(())
}

/// Parcourt récursivement un répertoire pour trouver des fichiers de bande dessinée supportés.
///
/// # Paramètres
/// - `dir`: Le chemin du répertoire à scanner.
fn find_in_dir(dir: &Path) -> Result<Vec<ComicFile>> {
    let mut res = Vec::new();
    for e in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if e.file_type().is_file() {
            if let Ok(f) = detect_type(e.path()) {
                res.push(f);
            }
        }
    }
    Ok(res)
}

/// Recherche des fichiers à l'aide d'un pattern glob (ex: "**/*.cbz").
///
/// # Paramètres
/// - `pat`: Le pattern textuel à matcher.
fn find_by_glob(pat: &str) -> Result<Vec<ComicFile>> {
    let mut res = Vec::new();
    let options = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    for entry in glob::glob_with(pat, options).context("Pattern glob invalide")? {
        if let Ok(p) = entry {
            if p.is_file() {
                if let Ok(f) = detect_type(&p) {
                    res.push(f);
                }
            }
        }
    }
    Ok(res)
}

/// Identifie le type de fichier (CBZ, CBR, etc.) à partir de son extension.
///
/// # Paramètres
/// - `p`: Chemin du fichier à analyser.
fn detect_type(p: &Path) -> Result<ComicFile> {
    let ext = p.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase());
    let t = match ext.as_deref() {
        Some("cbz") => ComicType::Cbz,
        Some("cbr") => ComicType::Cbr,
        Some("pdf") => ComicType::Pdf,
        Some("epub") => ComicType::Epub,
        _ => anyhow::bail!("Format non supporté"),
    };
    Ok(ComicFile { path: p.to_path_buf(), file_type: t })
}

/// Affiche un résumé détaillé de l'exécution en fin de processus.
///
/// Présente les statistiques de compression, les erreurs rencontrées et
/// le gain total d'espace disque.
///
/// # Paramètres
/// - `stats`: Une table de hachage associant le chemin des fichiers à leurs statistiques.
fn print_final_summary(stats: &HashMap<PathBuf, ProcessingStats>) {
    println!("\n📊 RÉSUMÉ FINAL :");
    let mut total_saved = 0i64;
    let (mut ok, mut skip, mut err) = (0, 0, 0);

    for (p, s) in stats {
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        if let Some(e) = &s.error_message {
            println!("  ❌ {} : {}", name, e);
            err += 1;
        } else if s.compression_skipped {
            println!("  ⏭️  {} : Pas de gain", name);
            skip += 1;
        } else {
            let saved = s.original_size as i64 - s.compressed_size as i64;
            total_saved += saved;
            ok += 1;
            println!("  ✅ {} : -{:.1} Mo", name, saved as f64 / 1_048_576.0);
        }
    }
    println!("-----------------------------------------------------");
    println!("✅ {} | ⏭️ {} | ❌ {}", ok, skip, err);
    println!("💰 Économie totale : {:.2} Mo", total_saved as f64 / 1_048_576.0);
}