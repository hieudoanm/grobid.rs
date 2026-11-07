//! Configuration.

use std::path::PathBuf;

/// TEI XML namespace.
pub const NS: &str = "http://www.tei-c.org/ns/1.0";

/// Metadata fields emitted in YAML frontmatter.
pub const META_FIELDS: &[&str] = &[
    "title", "authors", "year", "journal", "volume", "issue", "pages", "doi", "doi_link",
];

/// GROBID service name.
pub const GROBID_SERVICE: &str = "processFulltextDocument";

/// Whether to consolidate the header when calling GROBID.
pub const GROBID_CONSOLIDATE_HEADER: bool = true;

/// GROBID server port.
pub fn grobid_port() -> u16 {
    std::env::var("GROBID_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8070)
}

/// GROBID base URL.
pub fn grobid_url() -> String {
    format!("http://localhost:{}", grobid_port())
}

/// GROBID health check URL.
pub fn health_url() -> String {
    format!("{}/api/isalive", grobid_url())
}

/// Docker image used for GROBID.
pub fn grobid_image() -> String {
    std::env::var("GROBID_IMAGE").unwrap_or_else(|_| "grobid/grobid:0.9.1-crf".to_string())
}

/// Docker container name.
pub fn grobid_container() -> String {
    std::env::var("GROBID_CONTAINER").unwrap_or_else(|_| "grobid-cli".to_string())
}

/// Resolve the data directory from environment variables.
pub fn get_data_dir() -> PathBuf {
    if let Ok(dir) =
        std::env::var("GROBID_DATA_DIR").or_else(|_| std::env::var("ARTICLES_DATA_DIR"))
    {
        return PathBuf::from(dir);
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
