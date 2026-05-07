use clap::Parser;
use std::path::PathBuf;

/// Types de fichiers de bande dessinée supportés
#[derive(Debug, Clone)]
pub enum ComicType {
    Cbz,
    Cbr,
    Pdf,
    Epub,
}

/// Structure représentant un fichier à traiter
#[derive(Debug, Clone)]
pub struct ComicFile {
    pub path: PathBuf,
    pub file_type: ComicType,
}

/// Arguments de la ligne de commande
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Compresseur de BD, Mangas et Comics vers CBZ", long_about = None)]
pub struct Args {
    /// Fichier ou répertoire d'entrée
    #[arg(value_name = "INPUT")]
    pub input: Option<PathBuf>,

    /// Qualité WebP (1-100, défaut : 90)
    #[arg(short, long, default_value = "90")]
    pub quality: u8,

    /// Hauteur cible pour les images (défaut : 1800)
    #[arg(short = 'H', long, default_value = "1800")]
    pub target_height: u32,

    /// Dimension maximale de repli (défaut : 1200)
    #[arg(short, long, default_value = "1200")]
    pub max_dimension: u32,

    /// Renomme l'original en _original et donne le nom d'origine au compressé
    #[arg(short, long)]
    pub rename_original: bool,

    /// Motif Glob pour la sélection (ex: "**/Batman*.cbr")
    #[arg(short, long)]
    pub glob_pattern: Option<String>,

    /// Économie minimale requise pour conserver le fichier (défaut : 5%)
    #[arg(long, default_value = "5.0")]
    pub min_savings: f64,

    /// Active la sortie détaillée
    #[arg(short, long)]
    pub verbose: bool,

    /// Mode conversion seule : conserve les images originales sans compression
    #[arg(short = 'S', long)]
    pub skip_compression: bool,
}

/// Statistiques de traitement pour un fichier
#[allow(dead_code)]
#[derive(Debug)]
pub struct ProcessingStats {
    pub original_size: u64,
    pub compressed_size: u64,
    pub images_processed: usize,
    pub images_skipped: usize,
    pub compression_skipped: bool,
    pub output_path: Option<PathBuf>,
    pub error_message: Option<String>,
    pub status_message: Option<String>,
}