use anyhow::{Context, Result};
use image::ImageReader;
use std::fs;
use std::path::Path;
use crate::models::Args;

/// Encode une image au format WebP avec la qualité spécifiée
pub fn encode_webp(img: &image::DynamicImage, quality: u8) -> Result<Vec<u8>> {
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();

    let encoder = webp::Encoder::from_rgb(&rgb_img, width, height);
    let encoded = encoder.encode(quality as f32);

    Ok(encoded.to_vec())
}

/// Traite une image unique : décodage, redimensionnement et sauvegarde en WebP
pub fn process_single_image(image_path: &Path, args: &Args) -> Result<()> {
    if args.skip_compression {
        return Ok(());
    }

    // Gestion spécifique du format JPEG 2000 (souvent trouvé dans les PDF)
    if image_path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase() == "jp2").unwrap_or(false) {
        return process_jp2_image(image_path, args);
    }

    let img = ImageReader::open(image_path)
        .context("Impossible d'ouvrir l'image")?
        .decode()
        .context("Échec du décodage de l'image")?;

    let (width, height) = (img.width(), img.height());
    let aspect_ratio = width as f32 / height as f32;

    let new_height = args.target_height;
    let new_width = (new_height as f32 * aspect_ratio) as u32;

    // Redimensionnement haute qualité (Lanczos3)
    let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

    let webp_path = image_path.with_extension("webp");
    let webp_bytes = encode_webp(&resized, args.quality)?;

    fs::write(&webp_path, webp_bytes).context("Échec de l'écriture du fichier WebP")?;
    fs::remove_file(image_path).context("Impossible de supprimer l'image originale")?;

    Ok(())
}

/// Traitement spécifique pour les images JP2 avec conversion d'espace colorimétrique ICC
fn process_jp2_image(image_path: &Path, args: &Args) -> Result<()> {
    let jp2_img = jpeg2k::Image::from_file(image_path)
        .map_err(|e| anyhow::anyhow!("Erreur JP2: {:?}", e))?;
    let pixels = jp2_img.get_pixels(None)
        .map_err(|e| anyhow::anyhow!("Erreur pixels JP2: {:?}", e))?;

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
        _ => return Ok(()),
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

fn finalize_image_save(img: &image::DynamicImage, original_path: &Path, quality: u8) -> Result<()> {
    let webp_bytes = encode_webp(img, quality)?;
    let webp_path = original_path.with_extension("webp");
    fs::write(&webp_path, webp_bytes)?;
    fs::remove_file(original_path)?;
    Ok(())
}