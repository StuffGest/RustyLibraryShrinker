//! Définitions des structures de données et des types fondamentaux.
//!
//! Ce module contient les types partagés à travers toute l'application :
//! - Les énumérations des formats supportés.
//! - La structure de configuration issue de la ligne de commande.
//! - Les modèles de suivi des statistiques de traitement.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::collections::HashMap;
use fluent_templates::{static_loader, fluent_bundle::FluentValue, Loader};

// Charge statiquement les fichiers de langue au moment de la compilation.
static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
    };
}

/// Traduit une clé Fluent avec des arguments optionnels et une langue dynamique
pub fn tr(key: &str, lang: &str, args: Option<&HashMap<String, FluentValue>>) -> String {
    let lang_id: fluent_templates::LanguageIdentifier = lang.parse().unwrap_or_else(|_| "en".parse().unwrap());

    match args {
        Some(fluent_args) => {
            LOCALES.lookup_with_args(&lang_id, key, fluent_args)
        }
        None => {
            LOCALES.lookup(&lang_id, key)
        }
    }
}

/// Détecte dynamiquement la langue du système ou retombe sur le français par défaut.
pub fn detect_system_lang() -> String {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_else(|_| "fr".to_string())
        .split('.')
        .next()
        .unwrap_or("fr")
        .to_string()
}

/// Fonctions d'aide pour injecter les traductions statiques dans Clap au runtime
fn help_about() -> String { tr("cli-about", &detect_system_lang(), None) }
fn help_input() -> String { tr("cli-help-input", &detect_system_lang(), None) }
fn help_lang() -> String { tr("cli-help-lang", &detect_system_lang(), None) }
fn help_quality() -> String { tr("cli-help-quality", &detect_system_lang(), None) }
fn help_height() -> String { tr("cli-help-height", &detect_system_lang(), None) }
fn help_dim() -> String { tr("cli-help-dim", &detect_system_lang(), None) }
fn help_mode() -> String { tr("cli-help-mode", &detect_system_lang(), None) }
fn help_threads() -> String { tr("cli-help-threads", &detect_system_lang(), None) }
fn help_glob() -> String { tr("cli-help-glob", &detect_system_lang(), None) }
fn help_savings() -> String { tr("cli-help-savings", &detect_system_lang(), None) }
fn help_log() -> String { tr("cli-help-log", &detect_system_lang(), None) }
fn help_verbose() -> String { tr("cli-help-verbose", &detect_system_lang(), None) }
fn help_skip() -> String { tr("cli-help-skip", &detect_system_lang(), None) }
fn help_force() -> String { tr("cli-help-force", &detect_system_lang(), None) }

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
#[command(author, version, about = help_about(), long_about = None)]
pub struct Args {
    /// Chemin vers le fichier unique ou le répertoire contenant les archives à traiter.
    #[arg(value_name = "INPUT", help = help_input())]
    pub input: Option<PathBuf>,

    /// Niveau de qualité pour l'encodage WebP (de 1 à 100). Une valeur de 90 offre un excellent rapport poids/qualité.
    #[arg(long, default_value = "fr", help = help_lang())]
    pub lang: String,

    /// Niveau de qualité pour l'encodage WebP (de 1 à 100). Une valeur de 90 offre un excellent rapport poids/qualité.
    #[arg(short, long, default_value = "90", help = help_quality())]
    pub quality: u8,

    /// Hauteur cible en pixels pour le redimensionnement des pages.
    #[arg(short = 'H', long, default_value = "1800", help = help_height())]
    pub target_height: u32,

    /// Dimension maximale de sécurité utilisée en cas d'impossibilité de déterminer le ratio.
    #[arg(short, long, default_value = "1200", help = help_dim())]
    pub max_dimension: u32,

    /// Mode de gestion des fichiers de sortie (suffix, rename, replace).
    #[arg(short = 'r', long, value_enum, default_value_t = FileMode::Suffix, help = help_mode())]
    pub file_mode: FileMode,

    /// Nombre de threads maximum (0 pour auto)
    #[arg(short, long, default_value_t = 0, help = help_threads())]
    pub threads: usize,

    /// Utilisation d'un motif de recherche Glob pour filtrer les fichiers (ex: "**/Batman*.cbr").
    #[arg(short, long, help = help_glob())]
    pub glob_pattern: Option<String>,

    /// Pourcentage minimal de gain de poids requis pour valider le remplacement du fichier original.
    #[arg(long, default_value = "5.0", help = help_savings())]
    pub min_savings: f64,

    /// Chemin vers le fichier de log
    #[arg(short = 'l', long = "log-file", help = help_log())]
    pub log_file: Option<PathBuf>,

    /// Affiche plus d'informations dans la console durant l'exécution.
    #[arg(short, long, help = help_verbose())]
    pub verbose: bool,

    /// Désactive la compression des images : convertit uniquement le conteneur vers le format CBZ.
    #[arg(short = 'S', long, help = help_skip())]
    pub skip_compression: bool,

    /// Force le ré-encodage et le redimensionnement des images même si elles sont déjà au format WebP.
    #[arg(long, help = help_force())]
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