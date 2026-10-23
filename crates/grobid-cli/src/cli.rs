//! CLI logic.

use std::fs;
use std::path::Path;

use futures::StreamExt;
use grobid_core::grobid_client;
use grobid_core::paths::{self, md_path, pdfs, tei_path};
use grobid_core::render::convert_tei_to_md;
use grobid_core::utils::log;

/// Maximum concurrent PDF→TEI requests; GROBID handles parallel requests but
/// each one is JVM-heavy, so keep this modest. Override with GROBID_CONCURRENCY.
fn concurrency() -> usize {
    std::env::var("GROBID_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(4)
        .max(1)
}

/// Process a single PDF: fetch TEI if missing, then render Markdown if missing.
/// Returns true on success (including already-converted skips).
async fn process_pdf(
    index: usize,
    total: usize,
    pdf_path: &Path,
    client: &reqwest::Client,
) -> bool {
    let name = pdf_path.file_name().unwrap_or_default().to_string_lossy();
    let mut tpath = tei_path(pdf_path);

    if !tpath.exists() {
        log().info(&format!("[{index}/{total}] PDF → TEI: {name}"));
        let (status, out_path, elapsed) =
            grobid_client::pdf_to_tei(client, pdf_path, &tei_path).await;
        match out_path {
            None => {
                log().error(&format!(
                    "[{index}/{total}]   FAILED (status={status}) after {elapsed:.1}s"
                ));
                return false;
            }
            Some(p) => {
                tpath = p;
                log().info(&format!(
                    "[{index}/{total}]   wrote {} ({} bytes)",
                    tpath.file_name().unwrap_or_default().to_string_lossy(),
                    tpath.metadata().map(|m| m.len()).unwrap_or(0)
                ));
            }
        }
    } else {
        log().info(&format!(
            "[{index}/{total}] Using existing TEI: {}",
            tpath.file_name().unwrap_or_default().to_string_lossy()
        ));
    }

    let mpath = md_path(&tpath);
    if mpath.exists() {
        log().info(&format!(
            "[{index}/{total}] Using existing MD: {}",
            mpath.file_name().unwrap_or_default().to_string_lossy()
        ));
        return true;
    }

    log().info(&format!(
        "[{index}/{total}] TEI → MD: {}",
        tpath.file_name().unwrap_or_default().to_string_lossy()
    ));
    match convert_tei_to_md(&tpath, None) {
        Ok(md_content) => match fs::write(&mpath, md_content) {
            Ok(()) => {
                log().info(&format!(
                    "[{index}/{total}]   wrote {} ({} bytes)",
                    mpath.file_name().unwrap_or_default().to_string_lossy(),
                    mpath.metadata().map(|m| m.len()).unwrap_or(0)
                ));
                true
            }
            Err(e) => {
                log().error(&format!("[{index}/{total}]   FAILED: {e}"));
                false
            }
        },
        Err(e) => {
            log().error(&format!("[{index}/{total}]   FAILED: {e}"));
            false
        }
    }
}

pub async fn convert_pdf_to_md() -> anyhow::Result<i32> {
    let (pdf_dir, tei_dir, md_dir) = paths::dirs();
    fs::create_dir_all(&tei_dir)?;
    fs::create_dir_all(&md_dir)?;

    log().info(&"=".repeat(60).to_string());
    log().info("Converting PDFs to Markdown via GROBID");
    log().info(&"=".repeat(60).to_string());
    log().info(&format!("PDF dir: {}", pdf_dir.display()));
    log().info(&format!("TEI dir: {}", tei_dir.display()));
    log().info(&format!("MD dir: {}", md_dir.display()));

    let all_pdfs = pdfs();
    if all_pdfs.is_empty() {
        log().warn(&format!("No PDFs found in {}", pdf_dir.display()));
        return Ok(0);
    }

    // Only contact GROBID when at least one PDF still needs TEI conversion;
    // fully-cached runs (and TEI→MD-only work) need no server at all.
    let needs_grobid = all_pdfs.iter().any(|p| !tei_path(p).exists());
    if needs_grobid
        && !grobid_client::start_grobid().await
        && !grobid_client::is_grobid_alive().await
    {
        log().error("GROBID is not available");
        return Ok(1);
    }
    if !needs_grobid {
        log().info("All TEI files present; skipping GROBID startup");
    } else {
        log().info("Connecting to GROBID...");
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let total = all_pdfs.len();
    let workers = concurrency().min(total);
    log().info(&format!(
        "Processing {total} file(s) with {workers} concurrent request(s)"
    ));

    let results = futures::stream::iter(
        all_pdfs
            .iter()
            .enumerate()
            .map(|(idx, pdf_path)| process_pdf(idx + 1, total, pdf_path, &client)),
    )
    .buffer_unordered(workers)
    .collect::<Vec<bool>>()
    .await;

    let failures = results.iter().filter(|ok| !**ok).count();

    log().info(&"=".repeat(60).to_string());
    if failures > 0 {
        log().error(&format!(
            "Conversion completed with {failures} failure(s) out of {total} file(s)"
        ));
        return Ok(1);
    }
    log().info("Conversion completed");
    log().info(&"=".repeat(60).to_string());
    Ok(0)
}
