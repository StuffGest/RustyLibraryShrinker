use anyhow::{Result, Context};
use lopdf::{Document, Object};
use std::fs;
use std::path::{Path, PathBuf};

/// Extrait les images d'un PDF en gérant les calques et SMasks
pub fn extract_pdf(pdf_path: &Path, temp_dir: &Path) -> Result<()> {
    let doc = Document::load(pdf_path).context("Échec chargement PDF")?;
    let pages = doc.get_pages();

    for (page_num, (_, page_id)) in pages.iter().enumerate() {
        let mut layers = Vec::new();
        let mut smask_ids = std::collections::HashSet::new();

        if let Ok(Object::Dictionary(page_dict)) = doc.get_object(*page_id) {
            if let Ok(Object::Dictionary(res)) = page_dict.get(b"Resources") {
                if let Ok(Object::Dictionary(xobjects)) = res.get(b"XObject") {
                    for (_, obj_ref) in xobjects.iter() {
                        if let Object::Reference(id) = obj_ref {
                            if let Ok(Object::Stream(s)) = doc.get_object(*id) {
                                if let Ok(Object::Reference(sid)) = s.dict.get(b"SMask") {
                                    smask_ids.insert(*sid);
                                }
                            }
                        }
                    }
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
        layers.sort_by(|a, b| a.0.cmp(&b.0));

        process_pdf_page_layers(&doc, layers, page_num + 1, temp_dir)?;
    }
    Ok(())
}

fn process_pdf_page_layers(doc: &Document, layers: Vec<(String, (u32, u16), Option<(u32, u16)>)>, page_idx: usize, temp_dir: &Path) -> Result<()> {
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

fn extract_smask(doc: &Document, sid: (u32, u16), _w: u32, _h: u32, temp_dir: &Path) -> Result<Option<image::GrayImage>> {
    let s = doc.get_object(sid)?.as_stream()?;
    if s.dict.get(b"Filter").map_or(false, |o| o == &Object::Name(b"JBIG2Decode".to_vec())) {
        return Ok(None);
    }
    let (p, _) = extract_raw_stream(s, temp_dir, &format!("mask_{}", sid.0))?;
    let mask = decode_to_luma(&p).ok();
    let _ = fs::remove_file(p);
    Ok(mask)
}

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

        // CORRECTION : On utilise decompressed_content() qui gère les filtres FlateDecode, etc.
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

fn decode_to_rgb(p: &Path) -> Result<image::RgbImage> {
    Ok(image::open(p)?.into_rgb8())
}

fn decode_to_luma(p: &Path) -> Result<image::GrayImage> {
    Ok(image::open(p)?.into_luma8())
}

fn apply_alpha(base: &mut image::RgbImage, layer: &image::RgbImage, alpha: &image::GrayImage) {
    for (x, y, pixel) in base.enumerate_pixels_mut() {
        let a = alpha.get_pixel(x, y)[0] as f32 / 255.0;
        let l = layer.get_pixel(x, y);
        pixel[0] = (l[0] as f32 * a + pixel[0] as f32 * (1.0 - a)) as u8;
        pixel[1] = (l[1] as f32 * a + pixel[1] as f32 * (1.0 - a)) as u8;
        pixel[2] = (l[2] as f32 * a + pixel[2] as f32 * (1.0 - a)) as u8;
    }
}