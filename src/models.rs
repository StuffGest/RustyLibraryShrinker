//! Définitions des structures de données et des types fondamentaux.
//!
//! Ce module contient les types partagés à travers toute l'application :
//! - Les énumérations des formats supportés.
//! - La structure de configuration issue de la ligne de commande.
//! - Les modèles de suivi des statistiques de traitement.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Modes de gestion des fichiers de sortie après traitement.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum FileMode {
    /// Crée un nouveau fichier avec un suffixe ".optimise.cbz" (Défaut).
    Suffix,
    /// Renomme l'original en ".original.cbz" et donne le nom d'origine au compressé.
    Rename,
    /// Remplace directement l'original (pas de sauvegarde/backup).
    Replace,
}

/// Formats de fichiers de bande dessinée pris en charge par l'outil.
#[derive(Debug, Clone)]
pub enum ComicType {
    /// Comic Book Zip : Archives au format ZIP.
    Cbz,
    /// Comic Book Rar : Archives au format RAR.
    Cbr,
    /// Portable Document Format.
    Pdf,
    /// Electronic Publication : Livres numériques.
    Epub,
}

/// Représente un fichier de bande dessinée identifié pour le traitement.
#[derive(Debug, Clone)]
pub struct ComicFile {
    /// Chemin absolu ou relatif vers le fichier sur le disque.
    pub path: PathBuf,
    /// Type de fichier détecté par son extension.
    pub file_type: ComicType,
}

/// Configuration globale de l'application extraite des arguments CLI.
///
/// Gère les paramètres de qualité d'image, les dimensions de sortie
/// et les options de manipulation de fichiers.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Compresseur de BD, Mangas et Comics vers CBZ", long_about = None)]
pub struct Args {
    /// Chemin vers le fichier unique ou le répertoire contenant les archives à traiter.
    #[arg(value_name = "INPUT")]
    pub input: Option<PathBuf>,

    /// Niveau de qualité pour l'encodage WebP (de 1 à 100). Une valeur de 90 offre un excellent rapport poids/qualité.
    #[arg(short, long, default_value = "90")]
    pub quality: u8,

    /// Hauteur cible en pixels pour le redimensionnement des pages.
    #[arg(short = 'H', long, default_value = "1800")]
    pub target_height: u32,

    /// Dimension maximale de sécurité utilisée en cas d'impossibilité de déterminer le ratio.
    #[arg(short, long, default_value = "1200")]
    pub max_dimension: u32,

    /// Mode de gestion des fichiers de sortie (suffix, rename, replace).
    #[arg(short = 'r', long, value_enum, default_value_t = FileMode::Suffix)]
    pub file_mode: FileMode,

    /// Nombre de threads maximum (0 pour auto)
    #[arg(short, long, default_value_t = 0)]
    pub threads: usize,

    /// Utilisation d'un motif de recherche Glob pour filtrer les fichiers (ex: "**/Batman*.cbr").
    #[arg(short, long)]
    pub glob_pattern: Option<String>,

    /// Pourcentage minimal de gain de poids requis pour valider le remplacement du fichier original.
    #[arg(long, default_value = "5.0")]
    pub min_savings: f64,

    /// Chemin vers le fichier de log
    #[arg(short = 'l', long = "log-file")]
    pub log_file: Option<PathBuf>,

    /// Affiche plus d'informations dans la console durant l'exécution.
    #[arg(short, long)]
    pub verbose: bool,

    /// Désactive la compression des images : convertit uniquement le conteneur vers le format CBZ.
    #[arg(short = 'S', long)]
    pub skip_compression: bool,

    /// Force le ré-encodage et le redimensionnement des images même si elles sont déjà au format WebP.
    #[arg(long)]
    pub force_shrink: bool,
}

/// Résultat détaillé du traitement d'un fichier.
///
/// Utilisé pour générer le rapport final à l'utilisateur.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ProcessingStats {
    /// Taille du fichier original en octets.
    pub original_size: u64,
    /// Taille du fichier généré après optimisation.
    pub compressed_size: u64,
    /// Nombre d'images traitées avec succès.
    pub images_processed: usize,
    /// Nombre d'images ignorées (erreurs ou formats non supportés).
    pub images_skipped: usize,
    /// Indique si la compression a été annulée faute de gain suffisant.
    pub compression_skipped: bool,
    /// Chemin final du fichier généré.
    pub output_path: Option<PathBuf>,
    /// Message détaillé en cas d'erreur lors du pipeline.
    pub error_message: Option<String>,
    /// Message de statut informatif (ex: "Gain insuffisant").
    pub status_message: Option<String>,
    /// Nom image ignoree
    pub skipped_details: Vec<(String, String)>,
}