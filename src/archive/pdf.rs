//! Extraction et traitement des images au sein des fichiers PDF.
//!
//! Ce module permet de parcourir un document PDF, de charger dynamiquement la bibliothèque
//! native Pdfium (`pdfium.dll` sous Windows, `libpdfium.so` sous Linux), et de convertir
//! chaque page en image PNG haute définition.
//!
//! L'accès à la bibliothèque FFI est entièrement sécurisé par un Mutex global afin
//! de garantir la stabilité de l'application lors de traitements multi-threadés.

use anyhow::{Result, Context};
use std::path::Path;
use std::ffi::CString;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use libc::{c_int, c_void, c_char};

// =========================================================================
// STRUCTURES ET ENCAPSULATION SÉCURISÉE (THREAD-SAFE)
// =========================================================================

/// Enveloppe de sécurité autour du pointeur brut de la bibliothèque partagée (`*mut c_void`).
/// Permet d'indiquer explicitement au compilateur Rust que le handle peut être partagé
/// et transféré entre les threads en toute sécurité sous couvert du Mutex global.
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

/// Charge l'instance de Pdfium adaptée au système d'exploitation hôte.
///
/// # Retour
/// * `Result<(LibraryHandle, PdfiumFunctions)>` - Le handle système de la ressource et la table des fonctions initialisées.
unsafe fn load_pdfium_platform() -> Result<(LibraryHandle, PdfiumFunctions)> {
    #[cfg(target_os = "windows")]
    {
        let dll_name = CString::new("pdfium.dll").unwrap();
        let handle = unsafe { LoadLibraryA(dll_name.as_ptr()) };
        if handle.is_null() {
            return Err(anyhow::anyhow!("Le fichier 'pdfium.dll' est introuvable à côté de l'exécutable Windows."));
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
        let so_name = CString::new("./libpdfium.so").unwrap();
        let mut handle = unsafe { dlopen(so_name.as_ptr(), 1) }; // 1 = RTLD_LAZY
        if handle.is_null() {
            let fallback_name = CString::new("libpdfium.so").unwrap();
            handle = unsafe { dlopen(fallback_name.as_ptr(), 1) };
        }
        if handle.is_null() {
            return Err(anyhow::anyhow!("Le fichier 'libpdfium.so' est introuvable (chemin local ou /usr/lib)."));
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

    // On vérifie le symbole de fermeture avant la construction pour éviter le warning de non-nullité
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
/// Cette méthode est entièrement thread-safe et peut être invoquée simultanément
/// depuis plusieurs threads (via Rayon, par exemple).
///
/// # Paramètres
/// * `pdf_path` - Le chemin d'accès système vers le fichier `.pdf` à traiter.
/// * `temp_dir` - Le dossier temporaire cible dans lequel enregistrer les images PNG générées.
///
/// # Erreurs
/// Renvoie une erreur si la bibliothèque Pdfium ne peut pas être chargée, si le document
/// est corrompu/verrouillé, ou si l'écriture des fichiers PNG échoue sur le support disque.
pub fn extract_pdf(pdf_path: &Path, temp_dir: &Path) -> Result<()> {
    // 1. Acquisition sécurisée du verrou global pour isoler le thread courant
    let _lock = PDFIUM_MUTEX.lock().map_err(|_| anyhow::anyhow!("Le verrou de synchronisation Pdfium a été corrompu."))?;

    // 2. Extraction du moteur unique de rendu
    let (_, fns) = PDFIUM_ENGINE.as_ref().context("Impossible de charger ou d'initialiser le moteur Pdfium natif.")?;

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

            // Protection élémentaire contre les divisions par zéro sur documents malformés
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

                    // Transposition rapide de l'espace de couleur BGRA (Pdfium) vers RGBA (Crate Image)
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