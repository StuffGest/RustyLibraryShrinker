use anyhow::{Result, anyhow};
use epub::doc::EpubDoc;
use std::fs;
use std::path::Path;
use std::collections::HashSet;

pub fn extract_epub(epub_path: &Path, temp_dir: &Path) -> Result<()> {
    let mut doc = EpubDoc::new(epub_path).map_err(|e| anyhow!("Erreur ouverture EPUB: {:?}", e))?;
    let mut image_count = 0;
    let mut seen_resources = HashSet::new();

    // 1. On récupère les ressources d'images directement depuis le manifest
    // C'est la méthode la plus fiable pour garder les noms de fichiers originaux
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

fn save_epub_resource(original_path: &str, data: &[u8], mime: &str, temp_dir: &Path, count: &mut usize, seen: &mut HashSet<String>) -> Result<()> {
    // On extrait uniquement le nom du fichier (ex: OEBPS/images/cover.jpg -> cover.jpg)
    let file_name = original_path.split('/').last().unwrap_or(original_path).to_string();

    if !seen.insert(file_name.clone()) {
        return Ok(());
    }

    *count += 1;
    let ext = match mime {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => file_name.split('.').last().unwrap_or("jpg"),
    };

    // On garde le NOM ORIGINAL. On s'assure juste que l'extension est cohérente.
    let base_name = if let Some(pos) = file_name.rfind('.') {
        &file_name[..pos]
    } else {
        &file_name
    };

    let dest_name = format!("{}.{}", base_name, ext);
    fs::write(temp_dir.join(dest_name), data)?;
    Ok(())
}