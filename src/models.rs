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

// Liste des langues réellement supportées par l'application
pub const SUPPORTED_LANGS: &[&str] = &["fr", "en"];
pub const DEFAULT_LANG: &str = "en";

// Charge statiquement les fichiers de langue au moment de la compilation.
static_loader! {
    static LOCALES = {
        locales: "./locales",
        fallback_language: "en",
    };
}

/// Traduit une clé Fluent avec des arguments optionnels et une langue dynamique
pub fn tr(key: &str, lang: &str, args: Option<&HashMap<String, FluentValue>>) -> String {
    // Vérification de sécurité : si la langue n'est pas supportée, on force le fallback
    let safe_lang = if SUPPORTED_LANGS.contains(&lang) { lang } else { DEFAULT_LANG };
    let lang_id: fluent_templates::LanguageIdentifier = safe_lang.parse().unwrap_or_else(|_| DEFAULT_LANG.parse().unwrap());

    match args {
        Some(fluent_args) => {
            LOCALES.lookup_with_args(&lang_id, key, fluent_args)
        }
        None => {
            LOCALES.lookup(&lang_id, key)
        }
    }
}

/// Détecte dynamiquement la langue du système ou retombe sur l'anglais par défaut.
pub fn detect_system_lang() -> String {
    let lang_var = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .unwrap_or_else(|_| DEFAULT_LANG.to_string());

    lang_var.split('.')
        .next()
        .map(|s| s.split('_').next().unwrap_or(DEFAULT_LANG))
        .unwrap_or(DEFAULT_LANG)
        .to_string()
}

/// Fonctions d'aide pour injecter les traductions statiques dans Clap au runtime.
/// Note : On passe la langue détectée ou configurée.
fn help_about(lang: &str) -> String { tr("cli-about", lang, None) }
fn help_input(lang: &str) -> String { tr("cli-help-input", lang, None) }
fn help_lang(lang: &str) -> String { tr("cli-help-lang", lang, None) }
fn help_quality(lang: &str) -> String { tr("cli-help-quality", lang, None) }
fn help_height(lang: &str) -> String { tr("cli-help-height", lang, None) }
fn help_dim(lang: &str) -> String { tr("cli-help-dim", lang, None) }
fn help_mode(lang: &str) -> String { tr("cli-help-mode", lang, None) }
fn help_threads(lang: &str) -> String { tr("cli-help-threads", lang, None) }
fn help_glob(lang: &str) -> String { tr("cli-help-glob", lang, None) }
fn help_savings(lang: &str) -> String { tr("cli-help-savings", lang, None) }
fn help_log(lang: &str) -> String { tr("cli-help-log", lang, None) }
fn help_verbose(lang: &str) -> String { tr("cli-help-verbose", lang, None) }
fn help_skip(lang: &str) -> String { tr("cli-help-skip", lang, None) }
fn help_force(lang: &str) -> String { tr("cli-help-force", lang, None) }

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
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = help_about(DEFAULT_LANG))]
pub struct Args {
    /// Chemin vers le fichier unique ou le répertoire contenant les archives à traiter.
    #[arg(value_name = "INPUT", help = help_input(DEFAULT_LANG))]
    pub input: Option<PathBuf>,

    /// Langue de l'interface (fr, en).
    #[arg(long, default_value = "", help = help_lang(DEFAULT_LANG))]
    pub lang: String,

    /// Niveau de qualité pour l'encodage WebP (de 1 à 100).
    #[arg(short, long, default_value = "90", help = help_quality(DEFAULT_LANG))]
    pub quality: u8,

    /// Hauteur cible en pixels pour le redimensionnement des pages.
    #[arg(short = 'H', long, default_value = "1800", help = help_height(DEFAULT_LANG))]
    pub target_height: u32,

    /// Dimension maximale de sécurité utilisée en cas d'impossibilité de déterminer le ratio.
    #[arg(short, long, default_value = "1200", help = help_dim(DEFAULT_LANG))]
    pub max_dimension: u32,

    /// Mode de gestion des fichiers de sortie (suffix, rename, replace).
    #[arg(short = 'r', long, value_enum, default_value_t = FileMode::Suffix, help = help_mode(DEFAULT_LANG))]
    pub file_mode: FileMode,

    /// Nombre de threads maximum (0 pour auto)
    #[arg(short, long, default_value_t = 0, help = help_threads(DEFAULT_LANG))]
    pub threads: usize,

    /// Utilisation d'un motif de recherche Glob pour filtrer les fichiers.
    #[arg(short, long, help = help_glob(DEFAULT_LANG))]
    pub glob_pattern: Option<String>,

    /// Pourcentage minimal de gain de poids requis.
    #[arg(long, default_value = "5.0", help = help_savings(DEFAULT_LANG))]
    pub min_savings: f64,

    /// Chemin vers le fichier de log
    #[arg(short = 'l', long = "log-file", help = help_log(DEFAULT_LANG))]
    pub log_file: Option<PathBuf>,

    /// Affiche plus d'informations dans la console durant l'exécution.
    #[arg(short, long, help = help_verbose(DEFAULT_LANG))]
    pub verbose: bool,

    /// Désactive la compression des images.
    #[arg(short = 'S', long, help = help_skip(DEFAULT_LANG))]
    pub skip_compression: bool,

    /// Force le ré-encodage des images même si elles sont déjà en WebP.
    #[arg(long, help = help_force(DEFAULT_LANG))]
    pub force_shrink: bool,
}

/// Résultat détaillé du traitement d'un fichier.
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
    pub skipped_details: Vec<(String, String)>,
}