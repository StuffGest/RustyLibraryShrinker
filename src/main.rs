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

fn main() -> Result<()> {
    let args = Args::parse();

    // On détermine le dossier de base (Mangas/ ou le dossier courant ".")
    let input_path = args.input.clone().unwrap_or_else(|| PathBuf::from("."));

    // 1. Découverte des fichiers avec correction de la récursion Glob
    let files = if let Some(pattern) = &args.glob_pattern {
        let full_pattern = if input_path.is_dir() {
            // Si le pattern ne contient pas de marqueur de récursion, on l'ajoute
            // pour scanner les sous-dossiers comme le faisait WalkDir.
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
        println!("❌ Aucun fichier trouvé pour le chemin : {:?}", input_path);
        return Ok(());
    }

    println!("🚀 RustyLibraryShrinker : {} fichier(s) à traiter", files.len());
    println!("-----------------------------------------------------");

    // 2. UI - Barres de progression
    let multi = Arc::new(MultiProgress::new());
    let main_bar = multi.add(ProgressBar::new(files.len() as u64));
    main_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} {pos}/{len} fichiers [{bar:40.cyan/blue}] {elapsed}")?);

    let stats_map = Arc::new(Mutex::new(HashMap::new()));

    // 3. Boucle de traitement parallèle
    files.par_iter().for_each(|f| {
        let pb = multi.add(ProgressBar::new(100));
        pb.set_style(ProgressStyle::default_bar()
            .template("  {msg} [{bar:30.green}] {percent}%")
            .unwrap());

        let file_name = f.path.file_name().unwrap_or_default().to_string_lossy().to_string();
        pb.set_message(file_name);

        match processor::process_comic_file(f, &args, &pb) {
            Ok(s) => {
                let msg = if s.compression_skipped { "⏭️ Skippé" } else { "✅ Terminé" };
                pb.finish_with_message(msg);
                stats_map.lock().unwrap().insert(f.path.clone(), s);
            }
            Err(e) => {
                pb.finish_with_message(format!("❌ Erreur: {}", e));
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
        main_bar.inc(1);
    });

    main_bar.finish_with_message("Traitement terminé !");

    let final_stats = stats_map.lock().unwrap();
    print_final_summary(&final_stats);

    Ok(())
}

fn detect_type(p: &Path) -> Result<ComicFile> {
    let ext = p.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase());
    let t = match ext.as_deref() {
        Some("cbz") => ComicType::Cbz,
        Some("cbr") => ComicType::Cbr,
        Some("pdf") => ComicType::Pdf,
        Some("epub") => ComicType::Epub,
        _ => anyhow::bail!("Format non supporté : {:?}", p),
    };
    Ok(ComicFile { path: p.to_path_buf(), file_type: t })
}

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

fn find_by_glob(pat: &str) -> Result<Vec<ComicFile>> {
    let mut res = Vec::new();
    // On active explicitement les options pour que le glob se comporte comme un scan récursif
    let options = glob::MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };

    for entry in glob::glob_with(pat, options).context("Format du pattern glob invalide")? {
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

fn print_final_summary(stats: &HashMap<PathBuf, ProcessingStats>) {
    println!("\n📊 RÉSUMÉ FINAL :");
    let mut total_saved = 0i64;
    let mut files_ok = 0;
    let mut files_skipped = 0;
    let mut files_err = 0;

    for (p, s) in stats {
        let name = p.file_name().unwrap_or_default().to_string_lossy();
        if let Some(err) = &s.error_message {
            println!("  ❌ {} : {}", name, err);
            files_err += 1;
        } else if s.compression_skipped {
            println!("  ⏭️  {} : Pas de gain significatif", name);
            files_skipped += 1;
        } else {
            let saved = s.original_size as i64 - s.compressed_size as i64;
            total_saved += saved;
            files_ok += 1;
            println!("  ✅ {} : -{:.1} Mo", name, saved as f64 / 1_048_576.0);
        }
    }

    println!("-----------------------------------------------------");
    println!("✅ Succès : {} | ⏭️ Skippés : {} | ❌ Erreurs : {}", files_ok, files_skipped, files_err);
    println!("💰 Économie totale : {:.2} Mo", total_saved as f64 / 1_048_576.0);
    println!("-----------------------------------------------------");
}