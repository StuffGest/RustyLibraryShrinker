//! Extraction et traitement des images au sein des fichiers PDF.
//!
//! Ce module permet de parcourir un document PDF, de charger dynamiquement la bibliothèque
//! native Pdfium embarquée dans l'exécutable, et de convertir chaque page en image PNG.
//!
//! L'accès à la bibliothèque FFI est entièrement sécurisé par un Mutex global afin
//! de garantir la stabilité de l'application lors de traitements multi-threadés.

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::ffi::CString;
use std::sync::Mutex;
use std::fs::File;
use std::io::Write;
use once_cell::sync::Lazy;
use libc::{c_int, c_void, c_char};

// Inclusion des binaires natifs directement dans l'exécutable à la compilation
const PDFIUM_DLL_BYTES: &[u8] = include_bytes!("../../lib/pdfium.dll");
const PDFIUM_SO_BYTES: &[u8] = include_bytes!("../../lib/libpdfium.so");

// =========================================================================
// STRUCTURES ET ENCAPSULATION SÉCURISÉE (THREAD-SAFE)
// =========================================================================

/// Enveloppe de sécurité autour du pointeur brut de la bibliothèque partagée (`*mut c_void`).
#[allow(dead_code)]
struct LibraryHandle(*mut c_void);
unsafe impl Send for LibraryHandle {}
unsafe impl Sync for LibraryHandle {}

/// Table de correspondance centralisant l'ensemble des pointeurs de fonctions
/// exportés par la bibliothèque native Pdfium.
#[allow(dead_code)]
struct PdfiumFunctions {
    fpdf_init_library: unsafe extern "C" fn(),
    fpdf_destroy_library: unsafe extern "C" fn(),
    fpdf_load_document: unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_void,
    fpdf_close_document: unsafe extern "C" fn(*mut c_void),
    fpdf_get_page_count: unsafe extern "C" fn(*mut c_void) -> c_int,
    fpdf_load_page: unsafe extern "C" fn(*mut c_void, c_int) -> *mut c_void,
    fpdf_close_page: unsafe extern "C" fn(*mut c_void),
    fpdf_get_page_width_f: unsafe extern "C" fn(*mut c_void) -> f32,
    fpdf_get_page_height_f: unsafe extern "C" fn(*mut c_void) -> f32,
    fpdf_bitmap_create_ex: unsafe extern "C" fn(c_int, c_int, c_int, *mut c_void, c_int) -> *mut c_void,
    fpdf_bitmap_fill_rect: unsafe extern "C" fn(*mut c_void, c_int, c_int, c_int, c_int, u32),
    fpdf_render_page_bitmap: unsafe extern "C" fn(*mut c_void, *mut c_void, c_int, c_int, c_int, c_int, c_int, c_int),
    fpdf_bitmap_get_buffer: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    fpdf_bitmap_destroy: unsafe extern "C" fn(*mut c_void),
}

unsafe impl Send for PdfiumFunctions {}
unsafe impl Sync for PdfiumFunctions {}

// =========================================================================
// DÉCLARATIONS DES SYSTÈMES ET CHARGEMENT DYNAMIQUE CIBLÉ
// =========================================================================

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "C" {
    fn LoadLibraryA(lpLibFileName: *const c_char) -> *mut c_void;
    fn GetProcAddress(hModule: *mut c_void, lpProcName: *const c_char) -> *mut c_void;
    #[allow(dead_code)]
    fn FreeLibrary(hModule: *mut c_void) -> c_int;
}

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    #[allow(dead_code)]
    fn dlclose(handle: *mut c_void) -> c_int;
}

/// Extrait les octets inclus en mémoire dans un fichier temporaire sur le disque.
///
/// # Paramètres
/// * `bytes` - Le slice d'octets de la bibliothèque native stocké en mémoire RAM.
/// * `filename` - Le nom du fichier cible à créer dans le répertoire temporaire.
fn extract_embedded_lib(bytes: &[u8], filename: &str) -> Result<PathBuf> {
    let mut temp_path = std::env::temp_dir();
    // Utilisation d'un sous-dossier propre à l'application pour éviter les collisions
    temp_path.push("RustyLibraryShrinker_cache");
    std::fs::create_dir_all(&temp_path)?;
    temp_path.push(filename);

    // On n'écrit le fichier que s'il n'existe pas déjà ou si sa taille diffère (optimisation)
    if !temp_path.exists() || std::fs::metadata(&temp_path)?.len() != bytes.len() as u64 {
        let mut file = File::create(&temp_path).context("Impossible de créer le fichier d'extraction temporaire")?;
        file.write_all(bytes).context("Échec d'écriture des octets de la bibliothèque embarquée")?;
        file.flush()?;
    }
    Ok(temp_path)
}

/// Charge l'instance de Pdfium adaptée au système d'exploitation hôte depuis la mémoire embarquée.
///
/// # Retour
/// * `Result<(LibraryHandle, PdfiumFunctions)>` - Le handle système et la table des fonctions initialisées.
unsafe fn load_pdfium_platform() -> Result<(LibraryHandle, PdfiumFunctions)> {
    #[cfg(target_os = "windows")]
    {
        let target_path = extract_embedded_lib(PDFIUM_DLL_BYTES, "pdfium.dll")?;
        let path_str = target_path.to_string_lossy();
        let dll_name = CString::new(path_str.as_ref()).unwrap();

        let handle = unsafe { LoadLibraryA(dll_name.as_ptr()) };
        if handle.is_null() {
            return Err(anyhow::anyhow!("Échec du chargement de la DLL extraite à l'emplacement : {}", path_str));
        }
        let get_proc = |name: &str| {
            let c_name = CString::new(name).unwrap();
            unsafe { GetProcAddress(handle, c_name.as_ptr()) }
        };
        let fns = unsafe { build_functions_struct(get_proc)? };
        Ok((LibraryHandle(handle), fns))
    }

    #[cfg(target_os = "linux")]
    {
        let target_path = extract_embedded_lib(PDFIUM_SO_BYTES, "libpdfium.so")?;
        let path_str = target_path.to_string_lossy();
        let so_name = CString::new(path_str.as_ref()).unwrap();

        let handle = unsafe { dlopen(so_name.as_ptr(), 1) }; // 1 = RTLD_LAZY
        if handle.is_null() {
            return Err(anyhow::anyhow!("Échec du chargement du .so extrait à l'emplacement : {}", path_str));
        }
        let get_proc = |name: &str| {
            let c_name = CString::new(name).unwrap();
            unsafe { dlsym(handle, c_name.as_ptr()) }
        };
        let fns = unsafe { build_functions_struct(get_proc)? };
        Ok((LibraryHandle(handle), fns))
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        return Err(anyhow::anyhow!("Système d'exploitation non pris en charge pour l'extraction PDF."));
    }
}

/// Associe les symboles extraits de la bibliothèque binaire aux pointeurs de fonctions Rust.
///
/// # Paramètres
/// * `get_proc` - Fermeture (closure) chargée d'interroger l'OS pour trouver l'adresse mémoire d'un symbole C.
unsafe fn build_functions_struct<F>(get_proc: F) -> Result<PdfiumFunctions>
where
    F: Fn(&str) -> *mut c_void
{
    let mut missing = Vec::new();
    let mut load = |name: &str| -> *mut c_void {
        let addr = get_proc(name);
        if addr.is_null() { missing.push(name.to_string()); }
        addr
    };

    let mut close_doc_ptr = get_proc("FPDF_CloseDocument");
    if close_doc_ptr.is_null() {
        close_doc_ptr = load("FPDF_CloseDocument");
        if close_doc_ptr.is_null() {
            return Err(anyhow::anyhow!("Symbole critique manquant : FPDF_CloseDocument"));
        }
    }

    let fns = unsafe {
        PdfiumFunctions {
            fpdf_init_library: std::mem::transmute(load("FPDF_InitLibrary")),
            fpdf_destroy_library: std::mem::transmute(load("FPDF_DestroyLibrary")),
            fpdf_load_document: std::mem::transmute(load("FPDF_LoadDocument")),
            fpdf_close_document: std::mem::transmute(close_doc_ptr),
            fpdf_get_page_count: std::mem::transmute(load("FPDF_GetPageCount")),
            fpdf_load_page: std::mem::transmute(load("FPDF_LoadPage")),
            fpdf_close_page: std::mem::transmute(load("FPDF_ClosePage")),
            fpdf_get_page_width_f: std::mem::transmute(load("FPDF_GetPageWidthF")),
            fpdf_get_page_height_f: std::mem::transmute(load("FPDF_GetPageHeightF")),
            fpdf_bitmap_create_ex: std::mem::transmute(load("FPDFBitmap_CreateEx")),
            fpdf_bitmap_fill_rect: std::mem::transmute(load("FPDFBitmap_FillRect")),
            fpdf_render_page_bitmap: std::mem::transmute(load("FPDF_RenderPageBitmap")),
            fpdf_bitmap_get_buffer: std::mem::transmute(load("FPDFBitmap_GetBuffer")),
            fpdf_bitmap_destroy: std::mem::transmute(load("FPDFBitmap_Destroy")),
        }
    };

    if !missing.is_empty() {
        return Err(anyhow::anyhow!("Symboles Pdfium essentiels manquants : {:?}", missing));
    }

    Ok(fns)
}

// =========================================================================
// ÉTAT GLOBAL SYNCHRONE (MUTEX & ONCE_CELL)
// =========================================================================

/// Verrou de synchronisation globale empêchant l'accès concurrent aux APIs non thread-safe de Pdfium.
static PDFIUM_MUTEX: Mutex<()> = Mutex::new(());

/// Moteur Pdfium initialisé de manière paresseuse, partagé par l'ensemble du cycle de vie du processus.
static PDFIUM_ENGINE: Lazy<Option<(LibraryHandle, PdfiumFunctions)>> = Lazy::new(|| {
    unsafe {
        let (handle, fns) = load_pdfium_platform().ok()?;
        (fns.fpdf_init_library)();
        Some((handle, fns))
    }
});

// =========================================================================
// API PUBLIQUE DU MODULE
// =========================================================================

/// Parcourt un document PDF et extrait chaque page sous forme d'image PNG haute résolution.
///
/// # Paramètres
/// * `pdf_path` - Le chemin d'accès système vers le fichier `.pdf` à traiter.
/// * `temp_dir` - Le dossier temporaire cible dans lequel enregistrer les images PNG générées.
pub fn extract_pdf(pdf_path: &Path, temp_dir: &Path) -> Result<()> {
    let _lock = PDFIUM_MUTEX.lock().map_err(|_| anyhow::anyhow!("Le verrou de synchronisation Pdfium a été corrompu."))?;

    let (_, fns) = PDFIUM_ENGINE.as_ref().context("Impossible de charger ou d'initialiser le moteur Pdfium embarqué.")?;

    let path_str = pdf_path.to_string_lossy();
    let c_path = CString::new(path_str.as_ref()).context("Erreur d'encodage du chemin du fichier PDF.")?;

    unsafe {
        let document = (fns.fpdf_load_document)(c_path.as_ptr(), std::ptr::null());

        if document.is_null() {
            return Err(anyhow::anyhow!("Pdfium n'a pas pu ouvrir le document (format invalide ou fichier corrompu)."));
        }

        let page_count = (fns.fpdf_get_page_count)(document);
        let target_height = 2650.0;

        for index in 0..page_count {
            let page = (fns.fpdf_load_page)(document, index);
            if page.is_null() { continue; }

            let orig_width = (fns.fpdf_get_page_width_f)(page);
            let orig_height = (fns.fpdf_get_page_height_f)(page);

            if orig_height <= 0.0 || orig_width <= 0.0 {
                let _ = (fns.fpdf_close_page)(page);
                continue;
            }

            let ratio = orig_width / orig_height;
            let w = (target_height * ratio) as c_int;
            let h = target_height as c_int;

            let bitmap = (fns.fpdf_bitmap_create_ex)(w, h, 4, std::ptr::null_mut(), 0);
            if !bitmap.is_null() {
                (fns.fpdf_bitmap_fill_rect)(bitmap, 0, 0, w, h, 0xFFFFFFFF);
                (fns.fpdf_render_page_bitmap)(bitmap, page, 0, 0, w, h, 0, 0);

                let buffer_ptr = (fns.fpdf_bitmap_get_buffer)(bitmap);
                let buffer_size = (w * h * 4) as usize;

                if !buffer_ptr.is_null() {
                    let bgra_slice = std::slice::from_raw_parts(buffer_ptr as *const u8, buffer_size);
                    let mut rgba_data = vec![0u8; buffer_size];

                    for chunk_idx in (0..buffer_size).step_by(4) {
                        if chunk_idx + 3 < buffer_size {
                            rgba_data[chunk_idx]     = bgra_slice[chunk_idx + 2];
                            rgba_data[chunk_idx + 1] = bgra_slice[chunk_idx + 1];
                            rgba_data[chunk_idx + 2] = bgra_slice[chunk_idx];
                            rgba_data[chunk_idx + 3] = bgra_slice[chunk_idx + 3];
                        }
                    }

                    if let Some(img) = image::RgbaImage::from_raw(w as u32, h as u32, rgba_data) {
                        let file_name = format!("page_{:04}.png", index + 1);
                        let output_path = temp_dir.join(file_name);
                        let _ = img.save(&output_path);
                    }
                }
                (fns.fpdf_bitmap_destroy)(bitmap);
            }
            (fns.fpdf_close_page)(page);
        }

        (fns.fpdf_close_document)(document);
    }

    Ok(())
}