//! CLI logic.

use std::fs;
use std::path::{Path, PathBuf};

use futures::StreamExt;
use grobid_core::corpus::{self, Work};
use grobid_core::grobid_client;
use grobid_core::metadata::{extract_metadata_from_tei, Metadata};
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

/// Outcome of processing one PDF: success flag, corpus record, and what
/// happened to the Markdown output (written this run vs. cache hit).
struct PdfOutcome {
    ok: bool,
    work: Option<Work>,
    pdf_name: String,
    wrote_md: bool,
}

/// Resolve metadata for a TEI file, optionally enriching it via Crossref.
async fn resolve_metadata(tpath: &Path, enrich_client: Option<&reqwest::Client>) -> Metadata {
    let mut meta = match extract_metadata_from_tei(tpath) {
        Ok(m) => m,
        Err(e) => {
            log().warn(&format!("Metadata extraction failed for {tpath:?}: {e}"));
            Metadata::default()
        }
    };
    if let (Some(client), Some(doi)) = (enrich_client, meta.doi.clone()) {
        if let Some(enrichment) = grobid_core::enrich::lookup_logged(client, &doi).await {
            grobid_core::enrich::apply(&mut meta, &enrichment);
        }
    }
    meta
}

/// Build the corpus record for a converted file (TEI must exist).
fn build_work(pdf_path: &Path, tpath: &Path, mpath: &Path, meta: &Metadata) -> Option<Work> {
    let pdf_name = pdf_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string());
    let md_name = mpath.file_name().map(|n| n.to_string_lossy().to_string());
    match corpus::work_from_tei(tpath, pdf_name.as_deref(), md_name.as_deref(), meta) {
        Ok(w) => Some(w),
        Err(e) => {
            log().warn(&format!("Corpus indexing failed for {tpath:?}: {e}"));
            None
        }
    }
}

/// Process a single PDF: fetch TEI if missing, then render Markdown if missing.
/// Returns the outcome including the corpus record for the SQLite index.
async fn process_pdf(
    index: usize,
    total: usize,
    pdf_path: &Path,
    client: &reqwest::Client,
    enrich_client: Option<&reqwest::Client>,
) -> PdfOutcome {
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
                return PdfOutcome {
                    ok: false,
                    work: None,
                    pdf_name: name.to_string(),
                    wrote_md: false,
                };
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

    // Metadata is needed for the corpus index even when the Markdown file is
    // already cached, so resolve it up front. Crossref enrichment only runs
    // when we are (re)writing the Markdown — cached files keep their stored
    // enrichment instead of re-hitting the API on every run.
    let meta = if mpath.exists() {
        resolve_metadata(&tpath, None).await
    } else {
        resolve_metadata(&tpath, enrich_client).await
    };

    if mpath.exists() {
        log().info(&format!(
            "[{index}/{total}] Using existing MD: {}",
            mpath.file_name().unwrap_or_default().to_string_lossy()
        ));
        let work = build_work(pdf_path, &tpath, &mpath, &meta);
        return PdfOutcome {
            ok: true,
            work,
            pdf_name: name.to_string(),
            wrote_md: false,
        };
    }

    log().info(&format!(
        "[{index}/{total}] TEI → MD: {}",
        tpath.file_name().unwrap_or_default().to_string_lossy()
    ));
    let ok = match convert_tei_to_md(&tpath, Some(&meta)) {
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
    };
    let work = if ok {
        build_work(pdf_path, &tpath, &mpath, &meta)
    } else {
        None
    };
    PdfOutcome {
        ok,
        work,
        pdf_name: name.to_string(),
        wrote_md: ok,
    }
}

/// Upsert all collected works into the corpus DB and export index.json.
fn index_corpus(outcomes: &[PdfOutcome]) {
    let works: Vec<&Work> = outcomes.iter().filter_map(|o| o.work.as_ref()).collect();
    if works.is_empty() {
        return;
    }
    let db_path = paths::db_path();
    let mut db = match corpus::open_db(&db_path) {
        Ok(db) => db,
        Err(e) => {
            log().warn(&format!(
                "Could not open corpus DB {}: {e}",
                db_path.display()
            ));
            return;
        }
    };
    let mut indexed = 0usize;
    for work in &works {
        match db.upsert_work(work) {
            Ok(()) => indexed += 1,
            Err(e) => log().warn(&format!("Corpus upsert failed for {}: {e}", work.key)),
        }
    }
    log().info(&format!(
        "Indexed {indexed} work(s) into {}",
        db_path.display()
    ));
    corpus::log_corpus_summary(&db);
    if let Err(e) = db.export_index(&paths::index_path()) {
        log().warn(&format!("Could not export index.json: {e}"));
    }

    // Per-work metadata JSON files alongside the other corpus outputs.
    let meta_dir = paths::metadata_dir();
    let mut written = 0usize;
    for work in works {
        match corpus::write_metadata_file(&meta_dir, work) {
            Ok(_) => written += 1,
            Err(e) => log().warn(&format!("Metadata export failed for {}: {e}", work.key)),
        }
    }
    if written > 0 {
        log().info(&format!(
            "Wrote {written} metadata file(s) to {}",
            meta_dir.display()
        ));
    }

    // Regenerate the corpus dashboard (index.html) on every index pass.
    match corpus::render_graph_html(&db) {
        Ok(html) => {
            let out = paths::graph_path();
            let write_result = (|| -> std::io::Result<()> {
                if let Some(parent) = out.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&out, html)
            })();
            match write_result {
                Ok(()) => log().info(&format!("Wrote dashboard {}", out.display())),
                Err(e) => log().warn(&format!("Could not write dashboard {}: {e}", out.display())),
            }
        }
        Err(e) => log().warn(&format!("Could not render dashboard: {e}")),
    }
}

pub async fn convert_pdf_to_md(enrich: bool) -> anyhow::Result<i32> {
    let run_started = std::time::Instant::now();
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

    let enrich_client = if enrich {
        log().info("Crossref enrichment enabled");
        Some(
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .user_agent(concat!("grobid.rs/", env!("CARGO_PKG_VERSION")))
                .build()?,
        )
    } else {
        None
    };

    let total = all_pdfs.len();
    let workers = concurrency().min(total);
    log().info(&format!(
        "Processing {total} file(s) with {workers} concurrent request(s)"
    ));

    let outcomes = futures::stream::iter(all_pdfs.iter().enumerate().map(|(idx, pdf_path)| {
        process_pdf(idx + 1, total, pdf_path, &client, enrich_client.as_ref())
    }))
    .buffer_unordered(workers)
    .collect::<Vec<PdfOutcome>>()
    .await;

    let failures = outcomes.iter().filter(|o| !o.ok).count();
    index_corpus(&outcomes);
    write_process_manifest(&outcomes, enrich, run_started.elapsed());

    log().info(&"=".repeat(60).to_string());
    if failures > 0 {
        log().error(&format!(
            "Conversion completed with {failures} failure(s) out of {total} file(s)"
        ));
        return Ok(1);
    }
    log().info("Conversion completed");
    log().info(&format!("Corpus DB: {}", paths::db_path().display()));
    log().info(&format!("Dashboard: {}", paths::graph_path().display()));
    log().info(&"=".repeat(60).to_string());
    Ok(0)
}

/// Record this run's metadata (per-file statuses, counts, config) into
/// `.grobid/process.json`.
fn write_process_manifest(outcomes: &[PdfOutcome], enrich: bool, elapsed: std::time::Duration) {
    use grobid_core::corpus::{FileResult, RunRecord};
    let files: Vec<FileResult> = outcomes
        .iter()
        .map(|o| FileResult {
            pdf: o.pdf_name.clone(),
            status: if !o.ok {
                "failed".to_string()
            } else if o.wrote_md {
                "converted".to_string()
            } else {
                "cached".to_string()
            },
        })
        .collect();
    let count = |status: &str| files.iter().filter(|f| f.status == status).count();
    let record = RunRecord {
        started_at: chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::from_std(elapsed).unwrap_or_default())
            .map(|t| t.to_rfc3339())
            .unwrap_or_default(),
        finished_at: chrono::Utc::now().to_rfc3339(),
        duration_ms: elapsed.as_millis() as u64,
        total: files.len(),
        converted: count("converted"),
        cached: count("cached"),
        failed: count("failed"),
        enrich,
        concurrency: concurrency(),
        grobid_url: grobid_core::config::grobid_url(),
        files,
    };
    if let Err(e) = corpus::append_process_manifest(&paths::process_path(), &record) {
        log().warn(&format!("Could not write process manifest: {e}"));
    }
}

/// `grobid graph` — write the interactive citation graph HTML.
pub fn graph_cmd(db_arg: Option<PathBuf>, out_arg: Option<PathBuf>) -> anyhow::Result<i32> {
    let db_path = db_arg.unwrap_or_else(paths::db_path);
    let out_path = out_arg.unwrap_or_else(paths::graph_path);
    let db = corpus::open_db(&db_path)?;
    let html = corpus::render_graph_html(&db)?;
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, html)?;
    log().info(&format!("Wrote {}", out_path.display()));
    Ok(0)
}

/// `grobid search <query>` — search the corpus index.
pub fn search_cmd(query: &str, limit: usize, db_arg: Option<PathBuf>) -> anyhow::Result<i32> {
    let db_path = db_arg.unwrap_or_else(paths::db_path);
    if !db_path.exists() {
        log().error(&format!(
            "No corpus DB at {} — run `grobid` first",
            db_path.display()
        ));
        return Ok(1);
    }
    let db = corpus::open_db(&db_path)?;
    let rows = db.search(query, limit)?;
    if rows.is_empty() {
        log().info(&format!("No works matching \"{query}\""));
        return Ok(0);
    }
    for r in &rows {
        let doi = r.doi.as_deref().unwrap_or("—");
        let year = if r.year.is_empty() { "—" } else { &r.year };
        let cited = r
            .cited_by
            .map(|c| format!(" · cited {c}"))
            .unwrap_or_default();
        println!("{year}  {doi}  {}  — {}{}", r.title, r.journal, cited);
    }
    log().info(&format!("{} result(s)", rows.len()));
    Ok(0)
}

/// `grobid stats` — corpus statistics.
pub fn stats_cmd(db_arg: Option<PathBuf>) -> anyhow::Result<i32> {
    let db_path = db_arg.unwrap_or_else(paths::db_path);
    if !db_path.exists() {
        log().error(&format!(
            "No corpus DB at {} — run `grobid` first",
            db_path.display()
        ));
        return Ok(1);
    }
    let db = corpus::open_db(&db_path)?;
    let s = db.stats()?;
    println!("Works:            {}", s.works);
    println!("Titled:           {}", s.titled);
    println!("Distinct authors: {}", s.distinct_authors);
    println!("References:       {}", s.references);
    println!("  with DOI:        {}", s.doi_refs);
    println!("  unresolved:      {}", s.stub_refs);
    println!("  self-citations:  {}", s.self_refs);
    if !s.min_year.is_empty() {
        println!("Year span:        {}–{}", s.min_year, s.max_year);
    }
    if !s.year_hist.is_empty() {
        println!("\nWorks per year:");
        for (year, count) in &s.year_hist {
            println!("  {year}: {count}");
        }
    }
    if !s.top_authors.is_empty() {
        println!("\nTop authors:");
        for (authors, count) in s.top_authors.iter().take(5) {
            println!("  {count}×  {authors}");
        }
    }
    if !s.top_cited.is_empty() {
        println!("\nMost cited (Crossref):");
        for (title, doi, cited) in &s.top_cited {
            let d = doi.as_deref().unwrap_or("—");
            println!("  {cited:>5}  {d}  {title}");
        }
    }
    if !s.top_in_cited.is_empty() {
        println!("\nMost cited within corpus (in-degree):");
        for (title, doi, cited) in &s.top_in_cited {
            let d = doi.as_deref().unwrap_or("—");
            if title.is_empty() {
                println!("  {cited:>5}  {d}");
            } else {
                println!("  {cited:>5}  {d}  {title}");
            }
        }
    }
    if !s.top_citing.is_empty() {
        println!("\nMost citing (out-degree):");
        for (title, doi, cited) in &s.top_citing {
            let d = doi.as_deref().unwrap_or("—");
            println!("  {cited:>5}  {d}  {title}");
        }
    }
    Ok(0)
}
