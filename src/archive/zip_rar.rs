use anyhow::Result;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

/// Extrait une archive ZIP vers un répertoire temporaire
pub fn extract_zip(archive_path: &Path, temp_dir: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = temp_dir.join(file.name());

        if let Some(p) = outpath.parent() {
            fs::create_dir_all(p)?;
        }

        if file.name().ends_with('/') {
            continue;
        }

        let mut outfile = File::create(&outpath)?;
        std::io::copy(&mut file, &mut outfile)?;
    }
    Ok(())
}

/// Extrait une archive RAR vers un répertoire temporaire
pub fn extract_rar(archive_path: &Path, temp_dir: &Path) -> Result<()> {
    let archive = unrar::Archive::new(archive_path)
        .open_for_processing()
        .map_err(|e| anyhow::anyhow!("Erreur ouverture RAR: {:?}", e))?;

    let mut current = archive;
    loop {
        match current.read_header() {
            Ok(Some(header)) => {
                current = header.extract_with_base(temp_dir)
                    .map_err(|e| anyhow::anyhow!("Erreur extraction RAR: {:?}", e))?;
            }
            Ok(None) => break,
            Err(e) => return Err(anyhow::anyhow!("Erreur lecture RAR: {:?}", e)),
        }
    }
    Ok(())
}

/// Crée une archive CBZ (ZIP) à partir d'un répertoire
pub fn create_cbz(source_dir: &Path, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            let name = path.strip_prefix(source_dir)?;

            zip.start_file(name.to_string_lossy(), options)?;
            let content = fs::read(path)?;
            zip.write_all(&content)?;
        }
    }
    zip.finish()?;
    Ok(())
}