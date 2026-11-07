//! CLI logic.

use std::fs;

use grobid_core::grobid_client;
use grobid_core::paths::{self, md_path, pdfs, tei_path};
use grobid_core::render::convert_tei_to_md;
use grobid_core::utils::log;

pub async fn convert_pdf_to_md() -> anyhow::Result<i32> {
    paths::init_dirs();
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

    if !grobid_client::start_grobid().await && !grobid_client::is_grobid_alive().await {
        log().error("GROBID is not available");
        return Ok(1);
    }

    log().info("Connecting to GROBID...");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let total = all_pdfs.len();
    let mut failures = 0usize;
    for (idx, pdf_path) in all_pdfs.iter().enumerate() {
        let index = idx + 1;
        let mut tpath = tei_path(pdf_path);
        if !tpath.exists() {
            log().info(&format!(
                "[{}/{}] PDF → TEI: {}",
                index,
                total,
                pdf_path.file_name().unwrap_or_default().to_string_lossy()
            ));
            let (status, out_path, elapsed) =
                grobid_client::pdf_to_tei(&client, pdf_path, &tei_path).await;
            if out_path.is_none() {
                log().error(&format!(
                    "  FAILED (status={}) after {:.1}s",
                    status, elapsed
                ));
                failures += 1;
                continue;
            }
            tpath = out_path.unwrap();
            log().info(&format!(
                "  wrote {} ({} bytes)",
                tpath.file_name().unwrap_or_default().to_string_lossy(),
                tpath.metadata().map(|m| m.len()).unwrap_or(0)
            ));
        } else {
            log().info(&format!(
                "[{}/{}] Using existing TEI: {}",
                index,
                total,
                tpath.file_name().unwrap_or_default().to_string_lossy()
            ));
        }

        let mpath = md_path(&tpath);
        if mpath.exists() {
            log().info(&format!(
                "[{}/{}] Using existing MD: {}",
                index,
                total,
                mpath.file_name().unwrap_or_default().to_string_lossy()
            ));
            continue;
        }

        log().info(&format!(
            "[{}/{}] TEI → MD: {}",
            index,
            total,
            tpath.file_name().unwrap_or_default().to_string_lossy()
        ));
        match convert_tei_to_md(&tpath, None) {
            Ok(md_content) => {
                if let Err(e) = fs::write(&mpath, md_content) {
                    log().error(&format!("  FAILED: {}", e));
                    failures += 1;
                    continue;
                }
                log().info(&format!(
                    "  wrote {} ({} bytes)",
                    mpath.file_name().unwrap_or_default().to_string_lossy(),
                    mpath.metadata().map(|m| m.len()).unwrap_or(0)
                ));
            }
            Err(e) => {
                log().error(&format!("  FAILED: {}", e));
                failures += 1;
                continue;
            }
        }
    }

    log().info(&"=".repeat(60).to_string());
    if failures > 0 {
        log().error(&format!(
            "Conversion completed with {} failure(s) out of {} file(s)",
            failures, total
        ));
        return Ok(1);
    }
    log().info("Conversion completed");
    log().info(&"=".repeat(60).to_string());
    Ok(0)
}
