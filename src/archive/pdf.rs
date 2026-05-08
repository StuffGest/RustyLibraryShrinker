//! Extraction et traitement des images au sein des fichiers PDF.
//!
//! Ce module permet de parcourir un document PDF, d'en extraire les ressources
//! graphiques (XObjects) et de les convertir en images utilisables.
//!
//! Les points forts de ce module incluent :
//! - Support des flux compressés (DCT, JPX, Flate).
//! - Gestion avancée des masques de transparence (SMasks).
//! - Recomposition intelligente des calques pour les pages complexes.
//!
//! L'objectif est de produire une image nette par page de bande dessinée.

use anyhow::{Result, Context};
use lopdf::{Document, Object};
use std::fs;
use std::path::{Path, PathBuf};

/// Parcourt un document PDF pour extraire et reconstruire les images de chaque page.
///
/// Cette fonction analyse la structure de chaque page, identifie les objets images
/// et leurs masques de transparence associés, puis lance le processus de composition.
///
/// # Paramètres
/// - `pdf_path`: Le chemin d'accès vers le fichier PDF à traiter.
/// - `temp_dir`: Le répertoire temporaire où les images extraites seront sauvegardées.
///
/// # Erreurs
/// Retourne une erreur si le fichier est corrompu ou si l'accès au système de fichiers échoue.
pub fn extract_pdf(pdf_path: &Path, temp_dir: &Path) -> Result<()> {
    let doc = Document::load(pdf_path).context("Échec chargement PDF")?;
    let pages = doc.get_pages();

    for (page_num, (_, page_id)) in pages.iter().enumerate() {
        let mut layers = Vec::new();
        let mut smask_ids = std::collections::HashSet::new();

        if let Ok(Object::Dictionary(page_dict)) = doc.get_object(*page_id) {
            if let Ok(Object::Dictionary(res)) = page_dict.get(b"Resources") {
                if let Ok(Object::Dictionary(xobjects)) = res.get(b"XObject") {
                    // Premier passage pour identifier les IDs utilisés comme SMasks
                    for (_, obj_ref) in xobjects.iter() {
                        if let Object::Reference(id) = obj_ref {
                            if let Ok(Object::Stream(s)) = doc.get_object(*id) {
                                if let Ok(Object::Reference(sid)) = s.dict.get(b"SMask") {
                                    smask_ids.insert(*sid);
                                }
                            }
                        }
                    }
                    // Second passage pour collecter les images réelles
                    for (name, obj_ref) in xobjects.iter() {
                        if let Object::Reference(id) = obj_ref {
                            if smask_ids.contains(id) { continue; }
                            if let Ok(Object::Stream(s)) = doc.get_object(*id) {
                                if s.dict.get(b"Subtype").map_or(false, |o| o == &Object::Name(b"Image".to_vec())) {
                                    let smask = s.dict.get(b"SMask").ok()
                                        .and_then(|o| if let Object::Reference(sid) = o { Some(*sid) } else { None });
                                    layers.push((String::from_utf8_lossy(name).into_owned(), *id, smask));
                                }
                            }
                        }
                    }
                }
            }
        }

        if layers.is_empty() { continue; }
        // Tri alphabétique des calques pour respecter l'ordre d'empilement PDF
        layers.sort_by(|a, b| a.0.cmp(&b.0));

        process_pdf_page_layers(&doc, layers, page_num + 1, temp_dir)?;
    }
    Ok(())
}

/// Gère la superposition et la composition des différents calques d'une page.
///
/// Si une page contient plusieurs ressources images, elles sont fusionnées pour
/// créer une image unique représentant la page complète.
///
/// # Paramètres
/// - `doc`: Référence au document PDF chargé en mémoire.
/// - `layers`: Vecteur contenant les informations des calques (nom, ID, SMask).
/// - `page_idx`: L'index de la page actuelle (utilisé pour le nommage du fichier).
/// - `temp_dir`: Chemin vers le répertoire de travail.
fn process_pdf_page_layers(
    doc: &Document,
    layers: Vec<(String, (u32, u16), Option<(u32, u16)>)>,
    page_idx: usize,
    temp_dir: &Path
) -> Result<()> {
    let mut composite: Option<image::RgbImage> = None;

    for (_, ref_id, smask_ref) in layers {
        let stream = doc.get_object(ref_id)?.as_stream()?;
        let (img_path, _) = extract_raw_stream(stream, temp_dir, &format!("tmp_{}_{}", ref_id.0, ref_id.1))?;
        if img_path.as_os_str().is_empty() { continue; }

        let layer_rgb = decode_to_rgb(&img_path)?;
        let _ = fs::remove_file(&img_path);

        let (w, h) = (layer_rgb.width(), layer_rgb.height());
        let alpha = smask_ref.and_then(|sid| extract_smask(doc, sid, w, h, temp_dir).ok()).flatten();

        match (&mut composite, alpha) {
            (None, None) => composite = Some(layer_rgb),
            (None, Some(a)) => {
                let mut base = image::RgbImage::from_pixel(w, h, image::Rgb([255, 255, 255]));
                apply_alpha(&mut base, &layer_rgb, &a);
                composite = Some(base);
            },
            (Some(ref mut b), None) => *b = layer_rgb,
            (Some(ref mut b), Some(a)) => apply_alpha(b, &layer_rgb, &a),
        }
    }

    if let Some(img) = composite {
        img.save(temp_dir.join(format!("page_{:04}.png", page_idx)))?;
    }
    Ok(())
}

/// Extrait un masque de transparence (Soft Mask) depuis les objets PDF.
///
/// # Paramètres
/// - `doc`: Le document PDF.
/// - `sid`: L'identifiant de référence de l'objet SMask.
/// - `_w`, `_h`: Les dimensions de l'image parente (pour validation).
/// - `temp_dir`: Le répertoire temporaire de travail.
fn extract_smask(doc: &Document, sid: (u32, u16), _w: u32, _h: u32, temp_dir: &Path) -> Result<Option<image::GrayImage>> {
    let s = doc.get_object(sid)?.as_stream()?;
    // On ignore les masques encodés en JBIG2 qui ne sont généralement pas des masques de transparence classiques
    if s.dict.get(b"Filter").map_or(false, |o| o == &Object::Name(b"JBIG2Decode".to_vec())) {
        return Ok(None);
    }
    let (p, _) = extract_raw_stream(s, temp_dir, &format!("mask_{}", sid.0))?;
    let mask = decode_to_luma(&p).ok();
    let _ = fs::remove_file(p);
    Ok(mask)
}

/// Extrait le flux binaire brut d'une image PDF vers un fichier sur le disque.
///
/// Cette méthode gère la décompression des flux FlateDecode et l'identification
/// des formats natifs comme DCT (JPEG) ou JPX (JPEG2000).
///
/// # Paramètres
/// - `stream`: Le flux de données extrait du dictionnaire PDF.
/// - `temp_dir`: Le dossier où stocker le fichier temporaire.
/// - `base`: Le nom de base pour le fichier généré.
fn extract_raw_stream(stream: &lopdf::Stream, temp_dir: &Path, base: &str) -> Result<(PathBuf, usize)> {
    let filter = stream.dict.get(b"Filter").ok().and_then(|o| o.as_name().ok());
    let ext = match filter {
        Some(b"DCTDecode") => "jpg",
        Some(b"JPXDecode") => "jp2",
        _ => "png",
    };
    let path = temp_dir.join(format!("{}.{}", base, ext));

    if ext == "png" {
        let w = stream.dict.get(b"Width")?.as_i64()? as u32;
        let h = stream.dict.get(b"Height")?.as_i64()? as u32;

        // On décompresse le contenu Flate/LZW via la méthode intégrée de lopdf
        let data = stream.decompressed_content()
            .map_err(|e| anyhow::anyhow!("Erreur décompression PDF : {:?}", e))?;

        if let Some(img) = image::GrayImage::from_raw(w, h, data.to_vec()) {
            img.save(&path)?;
        }
    } else {
        fs::write(&path, &stream.content)?;
    }
    Ok((path, 0))
}

/// Charge une image depuis le disque et la convertit au format RGB 8 bits.
fn decode_to_rgb(p: &Path) -> Result<image::RgbImage> {
    Ok(image::open(p)?.into_rgb8())
}

/// Charge une image depuis le disque et la convertit au format niveaux de gris 8 bits.
fn decode_to_luma(p: &Path) -> Result<image::GrayImage> {
    Ok(image::open(p)?.into_luma8())
}

/// Applique une fusion alpha d'un calque sur une image de base.
///
/// # Paramètres
/// - `base`: L'image de fond qui sera modifiée.
/// - `layer`: Le calque d'image à appliquer par-dessus.
/// - `alpha`: Le masque de transparence dictant l'intensité de la fusion.
fn apply_alpha(base: &mut image::RgbImage, layer: &image::RgbImage, alpha: &image::GrayImage) {
    for (x, y, pixel) in base.enumerate_pixels_mut() {
        let a = alpha.get_pixel(x, y)[0] as f32 / 255.0;
        let l = layer.get_pixel(x, y);
        pixel[0] = (l[0] as f32 * a + pixel[0] as f32 * (1.0 - a)) as u8;
        pixel[1] = (l[1] as f32 * a + pixel[1] as f32 * (1.0 - a)) as u8;
        pixel[2] = (l[2] as f32 * a + pixel[2] as f32 * (1.0 - a)) as u8;
    }
}