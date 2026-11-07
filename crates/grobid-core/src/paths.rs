//! Path resolution.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::config::get_data_dir;

/// Resolve the pdf, tei, and md directories from the data directory.
fn resolve_paths() -> (PathBuf, PathBuf, PathBuf) {
    let base = get_data_dir();
    (base.join("pdf"), base.join("tei"), base.join("md"))
}

/// PDF, TEI, and MD directories (resolved at startup).
pub fn dirs() -> (PathBuf, PathBuf, PathBuf) {
    let state = STATE.lock().unwrap();
    (
        state.pdf_dir.clone(),
        state.tei_dir.clone(),
        state.md_dir.clone(),
    )
}

/// Initialize the path state from environment.
pub fn init_dirs() {
    let (pdf, tei, md) = resolve_paths();
    let mut state = STATE.lock().unwrap();
    state.pdf_dir = pdf;
    state.tei_dir = tei;
    state.md_dir = md;
}

struct PathState {
    pdf_dir: PathBuf,
    tei_dir: PathBuf,
    md_dir: PathBuf,
}

static STATE: std::sync::LazyLock<Mutex<PathState>> = std::sync::LazyLock::new(|| {
    Mutex::new(PathState {
        pdf_dir: PathBuf::new(),
        tei_dir: PathBuf::new(),
        md_dir: PathBuf::new(),
    })
});

/// Compute the TEI XML path for a given PDF path.
pub fn tei_path(pdf_path: &Path) -> PathBuf {
    let state = STATE.lock().unwrap();
    state.tei_dir.join(format!(
        "{}.tei.xml",
        pdf_path.file_stem().unwrap_or_default().to_string_lossy()
    ))
}

/// Compute the Markdown path for a given TEI XML path.
pub fn md_path(tei_path: &Path) -> PathBuf {
    let state = STATE.lock().unwrap();
    let name = tei_path.file_name().unwrap_or_default().to_string_lossy();
    let stem = name.strip_suffix(".tei.xml").unwrap_or(&name);
    state.md_dir.join(format!("{}.md", stem))
}

/// List all PDF files in the PDF directory, sorted by name.
pub fn pdfs() -> Vec<PathBuf> {
    let state = STATE.lock().unwrap();
    if !state.pdf_dir.is_dir() {
        return vec![];
    }
    let mut entries: Vec<PathBuf> = match std::fs::read_dir(&state.pdf_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "pdf"))
            .collect(),
        Err(_) => return vec![],
    };
    entries.sort();
    entries
}
