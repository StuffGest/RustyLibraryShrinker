//! Traitement et optimisation des images pour les archives de bandes dessinées.
//!
//! Ce module fournit les outils nécessaires pour transformer les images extraites :
//! - Décodage multi-format.
//! - Redimensionnement avec filtrage de haute qualité (Lanczos3).
//! - Encodage performant vers le format WebP.
//! - Gestion avancée des fichiers JPEG 2000 (JP2) incluant la conversion des profils ICC.
//!
//! L'objectif est de réduire l'espace disque tout en préservant une excellente
//! qualité visuelle pour la lecture de comics.

use anyhow::{Context, Result};
use image::{GenericImageView, ImageReader};
use std::fs;
use std::path::Path;
use crate::models::Args;

/// Encode une image en mémoire au format WebP avec la qualité spécifiée.
///
/// Transforme une image dynamique en tampon d'octets encodés en WebP.
///
/// # Paramètres
/// - `img`: La référence vers l'image `DynamicImage` à encoder.
/// - `quality`: Facteur de qualité entre 1 et 100.
///
/// # Retour
/// Retourne un `Result` contenant le vecteur d'octets (`Vec<u8>`) de l'image compressée.
pub fn encode_webp(img: &image::DynamicImage, quality: u8) -> Result<Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();

    // SÉCURITÉ : WebP (VP8) ne supporte pas de dimensions supérieures à 16383px.
    // Une tentative d'encodage au-delà de cette limite fait paniquer la bibliothèque C.
    if width > 16383 || height > 16383 || width == 0 || height == 0 {
        return Err(anyhow::anyhow!(
            "Dimensions invalides pour WebP : {}x{} (max 16383px)",
            width, height
        ));
    }

    let encoder = webp::Encoder::from_rgb(&rgb_img, width, height);

    // On utilise l'encodage et on s'assure de ne pas avoir de panic ici.
    // Note : Selon la version de la crate 'webp', .encode() peut renvoyer WebPMemory.
    let encoded = encoder.encode(quality as f32);

    Ok(encoded.to_vec())
}

/// Traite une image individuelle sur le disque : décodage, redimensionnement et conversion.
///
/// Cette fonction est le point d'entrée pour l'optimisation de chaque page. Elle remplace
/// l'image originale par sa version WebP optimisée.
///
/// # Paramètres
/// - `image_path`: Chemin d'accès au fichier image original.
/// - `args`: Paramètres globaux de l'application (qualité, hauteur cible).
///
/// # Erreurs
/// Retourne une erreur si l'image est corrompue, si le redimensionnement échoue
/// ou si les permissions d'écriture sont insuffisantes.
pub fn process_single_image(image_path: &Path, args: &Args) -> Result<()> {
    if args.skip_compression {
        return Ok(());
    }

    // Gestion spécifique du format JPEG 2000 (fréquent dans les PDF de qualité)
    if image_path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase() == "jp2").unwrap_or(false) {
        return process_jp2_image(image_path, args);
    }

    let img = ImageReader::open(image_path)
        .context("Impossible d'ouvrir l'image")?
        .decode()
        .context("Échec du décodage de l'image (fichier peut-être corrompu)")?;

    let (width, height) = img.dimensions();

    // Si l'image est déjà corrompue au décodage (0px), on sort proprement
    if width == 0 || height == 0 {
        return Err(anyhow::anyhow!("Image vide détectée : {}x{}", width, height));
    }

    let aspect_ratio = width as f32 / height as f32;
    let new_height = args.target_height;
    let new_width = (new_height as f32 * aspect_ratio) as u32;

    // Si après redimensionnement la largeur dépasse la limite WebP, on capte l'erreur
    if new_width > 16383 {
        return Err(anyhow::anyhow!("Largeur calculée trop grande pour WebP : {}px", new_width));
    }

    // Redimensionnement haute qualité utilisant l'algorithme Lanczos3
    let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

    let webp_path = image_path.with_extension("webp");
    let webp_bytes = encode_webp(&resized, args.quality)?;

    fs::write(&webp_path, webp_bytes).context("Échec de l'écriture du fichier WebP")?;
    fs::remove_file(image_path).context("Impossible de supprimer l'image originale")?;

    Ok(())
}

/// Gère le traitement spécifique des images JPEG 2000 (JP2).
///
/// Inclut la gestion de la conversion d'espace colorimétrique ICC si un profil
/// est associé à l'image extraite.
///
/// # Paramètres
/// - `image_path`: Chemin vers le fichier .jp2.
/// - `args`: Configuration de traitement.
fn process_jp2_image(image_path: &Path, args: &Args) -> Result<()> {
    let jp2_img = jpeg2k::Image::from_file(image_path)
        .map_err(|e| anyhow::anyhow!("Erreur JP2 (corrompu ?) : {:?}", e))?;
    let pixels = jp2_img.get_pixels(None)
        .map_err(|e| anyhow::anyhow!("Erreur pixels JP2 : {:?}", e))?;

    let (rgb_data, width, height) = match pixels.data {
        jpeg2k::ImagePixelData::Rgb8(data) => (data, pixels.width, pixels.height),
        jpeg2k::ImagePixelData::Rgba8(data) => {
            let mut rgb = Vec::with_capacity(pixels.width as usize * pixels.height as usize * 3);
            for chunk in data.chunks_exact(4) {
                rgb.extend_from_slice(&[chunk[0], chunk[1], chunk[2]]);
            }
            (rgb, pixels.width, pixels.height)
        }
        jpeg2k::ImagePixelData::L8(data) => {
            let img = image::DynamicImage::ImageLuma8(
                image::GrayImage::from_raw(pixels.width, pixels.height, data)
                    .ok_or_else(|| anyhow::anyhow!("Erreur raw luma JP2"))?
            );
            return finalize_image_save(&img, image_path, args.quality);
        }
        _ => return Err(anyhow::anyhow!("Format de pixels JP2 non supporté")),
    };

    // Gestion du profil ICC si présent
    let icc_path = image_path.with_extension("icc");
    let final_rgb = if icc_path.exists() {
        let icc_data = fs::read(&icc_path)?;
        let source_profile = moxcms::ColorProfile::new_from_slice(&icc_data)
            .map_err(|e| anyhow::anyhow!("Erreur profil ICC: {:?}", e))?;
        let dest_profile = moxcms::ColorProfile::new_srgb();

        let transform = source_profile.create_transform_8bit(
            moxcms::Layout::Rgb,
            &dest_profile,
            moxcms::Layout::Rgb,
            moxcms::TransformOptions::default(),
        ).map_err(|e| anyhow::anyhow!("Erreur transform ICC: {:?}", e))?;

        let mut transformed = vec![0u8; rgb_data.len()];
        transform.transform(&rgb_data, &mut transformed)
            .map_err(|e| anyhow::anyhow!("Échec conversion couleur: {:?}", e))?;
        transformed
    } else {
        rgb_data
    };

    let _ = fs::remove_file(&icc_path);

    let img = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(width, height, final_rgb)
            .ok_or_else(|| anyhow::anyhow!("Erreur reconstruction RGB"))?
    );

    finalize_image_save(&img, image_path, args.quality)
}

/// Finalise l'enregistrement de l'image après transformation.
///
/// Enregistre l'image au format WebP et supprime le fichier original pour libérer
/// de l'espace dans le répertoire temporaire.
///
/// # Paramètres
/// - `img`: L'image traitée prête à être sauvegardée.
/// - `original_path`: Chemin de l'image source à remplacer.
/// - `quality`: Qualité de compression WebP.
fn finalize_image_save(img: &image::DynamicImage, original_path: &Path, quality: u8) -> Result<()> {
    let webp_bytes = encode_webp(img, quality)?;
    let webp_path = original_path.with_extension("webp");
    fs::write(&webp_path, webp_bytes).context("Erreur écriture finale")?;
    fs::remove_file(original_path).context("Erreur suppression temporaire")?;
    Ok(())
}