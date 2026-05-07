use anyhow::Result;
use indicatif::ProgressBar;
use std::fs;
use std::path::{Path, PathBuf};
use crossbeam_channel::{bounded, Receiver, Sender};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::models::{Args, ComicFile, ProcessingStats};
use crate::archive::{self, zip_rar};
use crate::image_utils;

pub fn process_comic_file(comic: &ComicFile, args: &Args, progress: &ProgressBar) -> Result<ProcessingStats> {
    let original_size = fs::metadata(&comic.path)?.len();
    let temp_dir = tempfile::tempdir()?;

    progress.set_position(10);
    archive::extract_comic(comic, temp_dir.path(), progress)?;

    progress.set_position(30);
    let mut images = find_images(temp_dir.path())?;

    // TRI ALPHABÉTIQUE : Crucial pour respecter l'ordre d'extraction (0001_..., 0002_...)
    images.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    if images.is_empty() {
        return Err(anyhow::anyhow!("Aucune image trouvée"));
    }

    let (proc, skip) = process_images_parallel(&images, args, progress)?;

    progress.set_position(85);
    let stem = comic.path.file_stem().unwrap().to_string_lossy();
    let parent = comic.path.parent().unwrap_or(Path::new("."));
    let temp_out = parent.join(format!("{}.new_tmp", stem));

    if temp_out.exists() { fs::remove_file(&temp_out)?; }

    zip_rar::create_cbz(temp_dir.path(), &temp_out)?;

    let new_size = fs::metadata(&temp_out)?.len();
    let savings_ratio = (original_size as f64 - new_size as f64) / original_size as f64;

    if savings_ratio < (args.min_savings as f64 / 100.0) {
        fs::remove_file(&temp_out)?;
        return Ok(ProcessingStats {
            original_size,
            compressed_size: original_size,
            images_processed: proc,
            images_skipped: skip,
            compression_skipped: true,
            output_path: None,
            error_message: None,
            status_message: Some("Gain insuffisant".to_string()),
        });
    }

    let final_path = finalize_output_paths(comic, &temp_out, args)?;

    Ok(ProcessingStats {
        original_size,
        compressed_size: new_size,
        images_processed: proc,
        images_skipped: skip,
        compression_skipped: false,
        output_path: Some(final_path),
        error_message: None,
        status_message: None,
    })
}

fn find_images(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut list = Vec::new();
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "bmp" | "webp" | "jp2") {
                list.push(entry.path().to_path_buf());
            }
        }
    }
    Ok(list)
}

fn process_images_parallel(files: &[PathBuf], args: &Args, progress: &ProgressBar) -> Result<(usize, usize)> {
    let total = files.len();
    let (s, r): (Sender<bool>, Receiver<bool>) = bounded(total);
    let proc = Arc::new(Mutex::new(0));
    let skip = Arc::new(Mutex::new(0));

    let p_clone = progress.clone();
    let proc_c = Arc::clone(&proc);
    let skip_c = Arc::clone(&skip);

    thread::spawn(move || {
        for success in r {
            if success { *proc_c.lock().unwrap() += 1; }
            else { *skip_c.lock().unwrap() += 1; }
            let curr = *proc_c.lock().unwrap() + *skip_c.lock().unwrap();
            p_clone.set_position(30 + ((curr * 50) / total) as u64);
        }
    });

    files.par_iter().for_each(|path| {
        let res = image_utils::process_single_image(path, args);
        let _ = s.send(res.is_ok());
    });

    drop(s);
    while (*proc.lock().unwrap() + *skip.lock().unwrap()) < total {
        thread::yield_now();
    }
    Ok((*proc.lock().unwrap(), *skip.lock().unwrap()))
}

fn finalize_output_paths(comic: &ComicFile, temp_out: &Path, args: &Args) -> Result<PathBuf> {
    let parent = comic.path.parent().unwrap_or(Path::new("."));
    let stem = comic.path.file_stem().unwrap().to_string_lossy();
    let original_ext = comic.path.extension().and_then(|e| e.to_str()).unwrap_or("");

    if args.rename_original {
        // CORRECTION : Masque <nom>_original.<ext>
        let backup_name = format!("{}_original.{}", stem, original_ext);
        let backup_path = parent.join(backup_name);
        fs::rename(&comic.path, &backup_path)?;

        let dest = parent.join(format!("{}.cbz", stem));
        fs::rename(temp_out, &dest)?;
        Ok(dest)
    } else {
        let dest = parent.join(format!("{} (Optimized).cbz", stem));
        fs::rename(temp_out, &dest)?;
        Ok(dest)
    }
}