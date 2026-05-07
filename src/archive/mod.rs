pub mod zip_rar;
pub mod pdf;
pub mod epub;

use anyhow::Result;
use std::path::Path;
use crate::models::{ComicFile, ComicType};
use indicatif::ProgressBar;

/// Point d'entrée unique pour extraire n'importe quel format supporté
pub fn extract_comic(comic: &ComicFile, temp_dir: &Path, _progress: &ProgressBar) -> Result<()> {
    match comic.file_type {
        ComicType::Cbz => zip_rar::extract_zip(&comic.path, temp_dir),
        ComicType::Cbr => {
            // Tente RAR, si échec tente ZIP (beaucoup de CBR sont des ZIP renommés)
            if zip_rar::extract_rar(&comic.path, temp_dir).is_err() {
                zip_rar::extract_zip(&comic.path, temp_dir)
            } else {
                Ok(())
            }
        },
        ComicType::Pdf => pdf::extract_pdf(&comic.path, temp_dir),
        ComicType::Epub => epub::extract_epub(&comic.path, temp_dir),
    }
}