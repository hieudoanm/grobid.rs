//! Path resolution.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::config::get_data_dir;
use crate::utils::log;

/// PDF, TEI, and MD directories, resolved lazily from the environment on
/// first use so callers never have to remember an init step.
pub fn dirs() -> (PathBuf, PathBuf, PathBuf) {
    static DIRS: OnceLock<(PathBuf, PathBuf, PathBuf)> = OnceLock::new();
    DIRS.get_or_init(|| {
        let base = get_data_dir();
        (base.join("pdf"), base.join("tei"), base.join("md"))
    })
    .clone()
}

/// Compute the TEI XML path for a given PDF path.
pub fn tei_path(pdf_path: &Path) -> PathBuf {
    let (_, tei_dir, _) = dirs();
    tei_dir.join(format!(
        "{}.tei.xml",
        pdf_path.file_stem().unwrap_or_default().to_string_lossy()
    ))
}

/// Compute the Markdown path for a given TEI XML path.
pub fn md_path(tei_path: &Path) -> PathBuf {
    let (_, _, md_dir) = dirs();
    let name = tei_path.file_name().unwrap_or_default().to_string_lossy();
    let stem = name.strip_suffix(".tei.xml").unwrap_or(&name);
    md_dir.join(format!("{}.md", stem))
}

/// List all PDF files in the PDF directory, sorted by name.
pub fn pdfs() -> Vec<PathBuf> {
    let (pdf_dir, _, _) = dirs();
    if !pdf_dir.is_dir() {
        return vec![];
    }
    let entries = match std::fs::read_dir(&pdf_dir) {
        Ok(rd) => rd,
        Err(e) => {
            log().warn(&format!(
                "Cannot read PDF directory {}: {e}",
                pdf_dir.display()
            ));
            return vec![];
        }
    };
    let mut pdfs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
        })
        .collect();
    pdfs.sort();
    pdfs
}
