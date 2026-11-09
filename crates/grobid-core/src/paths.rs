//! Path resolution.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::config::get_data_dir;
use crate::utils::log;

/// Base data directory, resolved lazily from the environment on first use.
fn base_dir() -> PathBuf {
    static BASE: OnceLock<PathBuf> = OnceLock::new();
    BASE.get_or_init(get_data_dir).clone()
}

/// PDF, TEI, and MD directories.
pub fn dirs() -> (PathBuf, PathBuf, PathBuf) {
    let base = base_dir();
    (base.join("pdf"), base.join("tei"), base.join("md"))
}

/// Hidden working directory for all generated corpus artifacts
/// (`<data-dir>/.grobid/`), sibling to `pdf/`, `tei/`, and `md/`.
pub fn grobid_dir() -> PathBuf {
    base_dir().join(".grobid")
}

/// Corpus SQLite database (`<data-dir>/.grobid/grobid.db`).
pub fn db_path() -> PathBuf {
    grobid_dir().join("grobid.db")
}

/// Per-work metadata JSON directory (`<data-dir>/.grobid/metadata/`).
pub fn metadata_dir() -> PathBuf {
    grobid_dir().join("metadata")
}

/// Portable JSON search index (`<data-dir>/.grobid/index.json`).
pub fn index_path() -> PathBuf {
    grobid_dir().join("index.json")
}

/// Process/run manifest (`<data-dir>/.grobid/process.json`).
pub fn process_path() -> PathBuf {
    grobid_dir().join("process.json")
}

/// Interactive corpus dashboard output (`<data-dir>/.grobid/index.html`).
pub fn graph_path() -> PathBuf {
    grobid_dir().join("index.html")
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
