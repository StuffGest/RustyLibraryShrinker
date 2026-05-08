//! Gestion de l'extraction des archives de bandes dessinées.
//!
//! Ce module centralise les mécanismes d'extraction pour les différents formats
//! de fichiers supportés par l'application :
//! - Archives compressées (CBZ, CBR) via `zip_rar`
//! - Documents PDF via `pdf`
//! - Livres numériques EPUB via `epub`
//!
//! Il fournit une interface unifiée pour décompresser ces fichiers dans des
//! répertoires temporaires avant le traitement des images.

pub mod zip_rar;
pub mod pdf;
pub mod epub;

use anyhow::Result;
use std::path::Path;
use crate::models::{ComicFile, ComicType};
use indicatif::ProgressBar;

/// Point d'entrée unique pour l'extraction de fichiers de bande dessinée.
///
/// Cette fonction identifie le format du fichier via son `ComicType` et délègue
/// l'extraction au sous-module approprié.
///
/// # Paramètres
/// - `comic`: Une référence vers la structure `ComicFile` contenant le chemin et le type du fichier.
/// - `temp_dir`: Le chemin du répertoire temporaire où les images doivent être extraites.
/// - `_progress`: Une instance de `ProgressBar` pour le suivi visuel de l'opération.
///
/// # Erreurs
/// Retourne une erreur si le format est corrompu ou si l'accès au système de fichiers échoue.
pub fn extract_comic(comic: &ComicFile, temp_dir: &Path, _progress: &ProgressBar) -> Result<()> {
    match comic.file_type {
        ComicType::Cbz => zip_rar::extract_zip(&comic.path, temp_dir),
        ComicType::Cbr => {
            // Tente l'extraction RAR. Si elle échoue, tente l'extraction ZIP car
            // de nombreux fichiers .cbr sont en réalité des archives ZIP renommées.
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