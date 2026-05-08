//! Gestion de l'extraction des ressources d'images au sein des fichiers EPUB.
//!
//! Ce module permet de parcourir la structure d'un livre numérique EPUB pour en
//! extraire les images tout en préservant scrupuleusement l'ordre alphabétique
//! et les noms de fichiers originaux.
//!
//! Les fonctionnalités incluses :
//! - Analyse du manifest de l'EPUB.
//! - Extraction brute des ressources binaires.
//! - Nettoyage des chemins pour une extraction à plat dans un dossier temporaire.

use anyhow::{Result, anyhow};
use epub::doc::EpubDoc;
use std::fs;
use std::path::Path;
use std::collections::HashSet;

/// Extrait toutes les images d'un fichier EPUB vers un répertoire cible.
///
/// Cette méthode parcourt le manifest des ressources du document pour identifier les
/// fichiers dont le type MIME correspond à une image, puis les sauvegarde.
///
/// # Paramètres
/// - `epub_path`: Chemin vers le fichier source .epub.
/// - `temp_dir`: Chemin vers le répertoire où les images seront déposées.
///
/// # Erreurs
/// Retourne une erreur si le fichier est illisible ou si aucune image n'est trouvée.
pub fn extract_epub(epub_path: &Path, temp_dir: &Path) -> Result<()> {
    let mut doc = EpubDoc::new(epub_path).map_err(|e| anyhow!("Erreur ouverture EPUB: {:?}", e))?;
    let mut image_count = 0;
    let mut seen_resources = HashSet::new();

    // On récupère les ressources d'images directement depuis le manifest.
    // C'est la méthode la plus fiable pour garder les noms de fichiers originaux
    // et permettre un tri alphabétique cohérent ultérieurement.
    let resources = doc.resources.clone();

    for (path, item) in resources {
        if item.mime.starts_with("image/") {
            if let Some(data) = doc.get_resource_by_path(&item.path) {
                save_epub_resource(&path, &data, &item.mime, temp_dir, &mut image_count, &mut seen_resources)?;
            }
        }
    }

    if image_count == 0 {
        return Err(anyhow!("Aucune image trouvée dans l'EPUB"));
    }

    Ok(())
}

/// Sauvegarde une ressource binaire d'image sur le disque.
///
/// Nettoie le nom de fichier original pour supprimer les structures de dossiers internes
/// à l'EPUB (ex: OEBPS/Images/cover.jpg devient cover.jpg) et normalise l'extension.
///
/// # Paramètres
/// - `original_path`: Chemin interne de la ressource dans l'EPUB.
/// - `data`: Contenu binaire de l'image.
/// - `mime`: Type MIME de l'image pour validation de l'extension.
/// - `temp_dir`: Dossier de destination.
/// - `count`: Compteur mutable d'images traitées.
/// - `seen`: Set pour garantir l'unicité des fichiers extraits.
fn save_epub_resource(
    original_path: &str,
    data: &[u8],
    mime: &str,
    temp_dir: &Path,
    count: &mut usize,
    seen: &mut HashSet<String>
) -> Result<()> {
    // On extrait uniquement le nom du fichier pour une extraction à plat
    let file_name = original_path.split('/').last().unwrap_or(original_path).to_string();

    // Protection contre les doublons de ressources
    if !seen.insert(file_name.clone()) {
        return Ok(());
    }

    *count += 1;

    // Détermination de l'extension correcte basée sur le type MIME
    let ext = match mime {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => file_name.split('.').last().unwrap_or("jpg"),
    };

    // On garde le NOM ORIGINAL pour respecter le tri alphabétique du comic.
    // On s'assure juste que l'extension est cohérente avec la donnée.
    let base_name = if let Some(pos) = file_name.rfind('.') {
        &file_name[..pos]
    } else {
        &file_name
    };

    let dest_name = format!("{}.{}", base_name, ext);
    fs::write(temp_dir.join(dest_name), data)?;
    Ok(())
}