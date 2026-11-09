//! Corpus index: a SQLite database of all converted works plus their
//! reference edges, searchable via `grobid search`/`stats`/`graph`.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::metadata::Metadata;
use crate::render::{extract_references, ParsedReference};
use crate::utils::{log, text_content};

/// A single converted work plus its parsed references. Serializes to the
/// per-work metadata JSON written under `<data-dir>/metadata/`.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Work {
    /// Stable identity: DOI when present, otherwise the PDF file stem.
    pub key: String,
    pub doi: Option<String>,
    pub doi_link: Option<String>,
    pub title: String,
    pub authors: Vec<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub kind: Option<String>,
    pub abstract_text: Option<String>,
    pub cited_by: Option<i64>,
    pub md_file: Option<String>,
    pub pdf_file: Option<String>,
    pub references: Vec<ParsedReference>,
}

impl Work {
    /// Number of references lacking both a title and a DOI (unresolved stubs).
    pub fn stub_refs(&self) -> usize {
        self.references
            .iter()
            .filter(|r| r.title.as_deref().unwrap_or("").is_empty() && r.doi.is_none())
            .count()
    }

    /// Number of references whose DOI equals this work's DOI (self-citations).
    pub fn self_refs(&self) -> usize {
        match &self.doi {
            Some(doi) => self
                .references
                .iter()
                .filter(|r| r.doi.as_deref() == Some(doi.as_str()))
                .count(),
            None => 0,
        }
    }
}

/// Open (creating if needed) the corpus database and run migrations.
pub fn open_db(path: &Path) -> anyhow::Result<CorpusDb> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS works(
           id INTEGER PRIMARY KEY,
           key TEXT NOT NULL UNIQUE,
           doi TEXT,
           title TEXT NOT NULL DEFAULT '',
           authors TEXT NOT NULL DEFAULT '',
           year TEXT NOT NULL DEFAULT '',
           journal TEXT NOT NULL DEFAULT '',
           kind TEXT NOT NULL DEFAULT '',
           abstract TEXT NOT NULL DEFAULT '',
           cited_by INTEGER,
           md_file TEXT NOT NULL DEFAULT '',
           pdf_file TEXT NOT NULL DEFAULT '',
           ref_count INTEGER NOT NULL DEFAULT 0,
           stub_refs INTEGER NOT NULL DEFAULT 0,
           self_refs INTEGER NOT NULL DEFAULT 0
         );
         CREATE INDEX IF NOT EXISTS idx_works_doi ON works(doi);
         CREATE TABLE IF NOT EXISTS refs(
           id INTEGER PRIMARY KEY,
           work_key TEXT NOT NULL,
           ref_doi TEXT,
           ref_title TEXT NOT NULL DEFAULT '',
           ref_container TEXT NOT NULL DEFAULT '',
           ref_year TEXT NOT NULL DEFAULT '',
           is_stub INTEGER NOT NULL DEFAULT 0,
           is_self INTEGER NOT NULL DEFAULT 0
         );
         CREATE INDEX IF NOT EXISTS idx_refs_work ON refs(work_key);
         CREATE INDEX IF NOT EXISTS idx_refs_doi ON refs(ref_doi);",
    )?;
    Ok(CorpusDb { conn })
}

pub struct CorpusDb {
    conn: Connection,
}

impl CorpusDb {
    /// Insert or replace a work and rewrite its reference edges.
    pub fn upsert_work(&mut self, work: &Work) -> anyhow::Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO works(key, doi, title, authors, year, journal, kind, abstract,
                                cited_by, md_file, pdf_file, ref_count, stub_refs, self_refs)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
             ON CONFLICT(key) DO UPDATE SET
               doi=excluded.doi, title=excluded.title, authors=excluded.authors,
               year=excluded.year, journal=excluded.journal, kind=excluded.kind,
               abstract=excluded.abstract, cited_by=excluded.cited_by,
               md_file=excluded.md_file, pdf_file=excluded.pdf_file,
               ref_count=excluded.ref_count, stub_refs=excluded.stub_refs,
               self_refs=excluded.self_refs",
            params![
                work.key.clone(),
                work.doi.clone(),
                work.title.clone(),
                work.authors.join(", "),
                work.year.clone().unwrap_or_default(),
                work.journal.clone().unwrap_or_default(),
                work.kind.clone().unwrap_or_default(),
                work.abstract_text.clone().unwrap_or_default(),
                work.cited_by,
                work.md_file.clone().unwrap_or_default(),
                work.pdf_file.clone().unwrap_or_default(),
                work.references.len() as i64,
                work.stub_refs() as i64,
                work.self_refs() as i64,
            ],
        )?;
        tx.execute("DELETE FROM refs WHERE work_key = ?1", params![work.key])?;
        for r in &work.references {
            let is_stub = r.title.as_deref().unwrap_or("").is_empty() && r.doi.is_none();
            let is_self = work
                .doi
                .as_deref()
                .is_some_and(|d| r.doi.as_deref() == Some(d));
            tx.execute(
                "INSERT INTO refs(work_key, ref_doi, ref_title, ref_container, ref_year,
                                        is_stub, is_self)
                 VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![
                    work.key,
                    r.doi,
                    r.title.clone().unwrap_or_default(),
                    r.container.clone().unwrap_or_default(),
                    r.year.clone().unwrap_or_default(),
                    is_stub as i64,
                    is_self as i64,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Case-insensitive substring search across title/authors/journal/doi/
    /// abstract/kind — the same unified-filter semantics as the doi app.
    pub fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<WorkRow>> {
        let pattern = format!("%{}%", escape_like(query));
        let mut stmt = self.conn.prepare(
            "SELECT key, doi, title, authors, year, journal, kind, cited_by, md_file
             FROM works
             WHERE title LIKE ?1 ESCAPE '\\'
                OR authors LIKE ?1 ESCAPE '\\'
                OR journal LIKE ?1 ESCAPE '\\'
                OR doi LIKE ?1 ESCAPE '\\'
                OR abstract LIKE ?1 ESCAPE '\\'
                OR kind LIKE ?1 ESCAPE '\\'
             ORDER BY year DESC, title
             LIMIT ?2",
        )?;
        let rows = stmt
            .query_map(params![pattern, limit as i64], WorkRow::from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn stats(&self) -> anyhow::Result<Stats> {
        let mut stmt = self.conn.prepare(
            "SELECT year, COUNT(*) FROM works WHERE year != '' GROUP BY year ORDER BY year",
        )?;
        let year_hist: Vec<(String, i64)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?;

        let scalar =
            |sql: &str| -> anyhow::Result<i64> { Ok(self.conn.query_row(sql, [], |r| r.get(0))?) };

        // Per-author counts: works.authors is a joined string, so split on
        // the separators GROBID emits (comma, semicolon, pipe, "and") and
        // count individuals case-insensitively.
        let author_rows = {
            let mut stmt = self
                .conn
                .prepare("SELECT authors FROM works WHERE authors != ''")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        let mut author_counts: std::collections::HashMap<String, (String, i64)> =
            std::collections::HashMap::new();
        for row in &author_rows {
            for name in split_authors(row) {
                let key = name.to_lowercase();
                let entry = author_counts
                    .entry(key)
                    .or_insert_with(|| (name.clone(), 0));
                entry.1 += 1;
            }
        }
        let distinct_authors = author_counts.len() as i64;
        let mut top_authors: Vec<(String, i64)> = author_counts.into_values().collect();
        top_authors.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        top_authors.truncate(10);

        let top_cited = {
            let mut stmt = self.conn.prepare(
                "SELECT title, doi, cited_by FROM works
                 WHERE cited_by IS NOT NULL AND title != ''
                 ORDER BY cited_by DESC LIMIT 10",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        // Corpus-derived rankings (doi app parity): in-degree = works citing
        // this work, out-degree = DOI-resolved references this work cites.
        // Self-citations and untitled stub works are excluded.
        // In-degree over every cited DOI (corpus works AND external stubs):
        // which references does this corpus cite most? Titles resolve via
        // LEFT JOIN; external-only DOIs fall back to the bare DOI.
        let top_in_cited = {
            let mut stmt = self.conn.prepare(
                "SELECT COALESCE(w.title, ''), r.ref_doi, COUNT(*) AS c
                 FROM refs r LEFT JOIN works w ON w.doi = r.ref_doi
                 WHERE r.is_self = 0 AND r.ref_doi IS NOT NULL AND r.ref_doi != ''
                 GROUP BY r.ref_doi ORDER BY c DESC LIMIT 10",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let top_citing = {
            let mut stmt = self.conn.prepare(
                "SELECT w.title, w.doi, COUNT(*) AS c
                 FROM refs r JOIN works w ON w.key = r.work_key
                 WHERE r.ref_doi IS NOT NULL AND r.ref_doi != '' AND w.title != ''
                 GROUP BY w.id ORDER BY c DESC LIMIT 10",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let (min_year, max_year) = year_hist
            .first()
            .zip(year_hist.last())
            .map(|(a, b)| (a.0.clone(), b.0.clone()))
            .unwrap_or_default();

        Ok(Stats {
            works: scalar("SELECT COUNT(*) FROM works")?,
            titled: scalar("SELECT COUNT(*) FROM works WHERE title != ''")?,
            references: scalar("SELECT COUNT(*) FROM refs")?,
            doi_refs: scalar(
                "SELECT COUNT(*) FROM refs WHERE ref_doi IS NOT NULL AND ref_doi != ''",
            )?,
            stub_refs: {
                let mut stmt = self
                    .conn
                    .prepare("SELECT COALESCE(SUM(stub_refs),0) FROM works")?;
                stmt.query_row([], |r| r.get(0))?
            },
            self_refs: {
                let mut stmt = self
                    .conn
                    .prepare("SELECT COALESCE(SUM(self_refs),0) FROM works")?;
                stmt.query_row([], |r| r.get(0))?
            },
            min_year,
            max_year,
            year_hist,
            top_authors,
            top_cited,
            distinct_authors,
            top_in_cited,
            top_citing,
        })
    }

    /// Nodes and edges for the citation graph: corpus works plus external
    /// DOI-only stub nodes for referenced works not in the corpus.
    pub fn graph_data(&self) -> anyhow::Result<(Vec<GraphNode>, Vec<GraphEdge>)> {
        let mut nodes: Vec<GraphNode> = Vec::new();
        // Untitled stub works are excluded from the graph (doi app parity);
        // they still count in corpus stats and search.
        let mut stmt = self.conn.prepare(
            "SELECT key, doi, title, authors, year, journal, kind, cited_by
             FROM works WHERE title != '' ORDER BY key",
        )?;
        let works = stmt
            .query_map([], |r| {
                Ok(GraphNode {
                    id: r.get::<_, String>(0)?,
                    doi: r.get(1)?,
                    title: r.get::<_, String>(2)?,
                    authors: r.get::<_, String>(3)?,
                    year: r.get::<_, String>(4)?,
                    journal: r.get::<_, String>(5)?,
                    kind: r.get::<_, String>(6)?,
                    cited_by: r.get(7)?,
                    external: false,
                    degree: 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        nodes.extend(works);

        // External nodes: referenced DOIs that are not corpus works.
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT r.ref_doi FROM refs r
             LEFT JOIN works w ON w.doi = r.ref_doi
             WHERE r.ref_doi IS NOT NULL AND r.ref_doi != '' AND w.id IS NULL",
        )?;
        let external_dois: Vec<String> = stmt
            .query_map([], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        for doi in external_dois {
            nodes.push(GraphNode {
                id: format!("ext:{doi}"),
                doi: Some(doi.clone()),
                title: String::new(),
                authors: String::new(),
                year: String::new(),
                journal: String::new(),
                kind: String::new(),
                cited_by: None,
                external: true,
                degree: 0,
            });
        }

        // Edges: work -> referenced DOI. Self-loops and stub refs skipped.
        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut stmt = self.conn.prepare(
            "SELECT work_key, ref_doi FROM refs
             WHERE ref_doi IS NOT NULL AND ref_doi != '' AND is_self = 0",
        )?;
        let raw_edges: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_, _>>()?;
        let node_ids: std::collections::HashSet<&str> =
            nodes.iter().map(|n| n.id.as_str()).collect();
        let work_doi: std::collections::HashMap<String, String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT key, doi FROM works WHERE doi IS NOT NULL")?;
            let rows =
                stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
            rows.collect::<Result<_, _>>()?
        };
        for (work_key, ref_doi) in raw_edges {
            // Edge source is the work's DOI node id when it has one, else key.
            let source = work_doi
                .get(&work_key)
                .cloned()
                .unwrap_or_else(|| work_key.clone());
            let target = format!("ext:{ref_doi}");
            let target = if node_ids.contains(target.as_str()) {
                target
            } else {
                ref_doi.clone()
            };
            // Drop edges from stub-excluded works or to missing nodes.
            if source != target && node_ids.contains(source.as_str()) {
                edges.push(GraphEdge { source, target });
            }
        }

        // In-degree per node for radius scaling.
        let mut degree: std::collections::HashMap<String, i64> = Default::default();
        for e in &edges {
            *degree.entry(e.target.clone()).or_insert(0) += 1;
        }
        for n in &mut nodes {
            n.degree = *degree.get(&n.id).unwrap_or(&0);
        }

        Ok((nodes, edges))
    }

    /// Write a portable JSON search index (works only, no edges).
    pub fn export_index(&self, path: &Path) -> anyhow::Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT doi, title, authors, year, journal, kind, cited_by, md_file
             FROM works ORDER BY year DESC, title",
        )?;
        let items: Vec<IndexEntry> = stmt
            .query_map([], |r| {
                Ok(IndexEntry {
                    doi: r.get(0)?,
                    title: r.get(1)?,
                    authors: r
                        .get::<_, String>(2)?
                        .split(", ")
                        .map(str::to_string)
                        .collect(),
                    year: r.get::<_, String>(3)?,
                    journal: r.get::<_, String>(4)?,
                    kind: r.get::<_, String>(5)?,
                    cited_by: r.get(6)?,
                    md_file: r.get::<_, String>(7)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(&items)?)?;
        Ok(())
    }
}

fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// A work row returned by search.
#[derive(Debug)]
pub struct WorkRow {
    pub key: String,
    pub doi: Option<String>,
    pub title: String,
    pub authors: String,
    pub year: String,
    pub journal: String,
    pub kind: String,
    pub cited_by: Option<i64>,
    pub md_file: String,
}

impl WorkRow {
    fn from_row(r: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Self {
            key: r.get(0)?,
            doi: r.get(1)?,
            title: r.get(2)?,
            authors: r.get(3)?,
            year: r.get(4)?,
            journal: r.get(5)?,
            kind: r.get(6)?,
            cited_by: r.get(7)?,
            md_file: r.get(8)?,
        })
    }
}

/// Aggregate corpus statistics.
pub struct Stats {
    pub works: i64,
    pub titled: i64,
    pub references: i64,
    pub doi_refs: i64,
    pub stub_refs: i64,
    pub self_refs: i64,
    pub min_year: String,
    pub max_year: String,
    pub year_hist: Vec<(String, i64)>,
    pub top_authors: Vec<(String, i64)>,
    pub top_cited: Vec<(String, Option<String>, i64)>,
    /// Count of distinct author strings (doi app parity).
    pub distinct_authors: i64,
    /// Works ranked by in-degree: how many corpus works cite them.
    pub top_in_cited: Vec<(String, Option<String>, i64)>,
    /// Works ranked by out-degree: how many DOI-resolved refs they cite.
    pub top_citing: Vec<(String, Option<String>, i64)>,
}

#[derive(Serialize)]
pub struct GraphNode {
    pub id: String,
    pub doi: Option<String>,
    pub title: String,
    pub authors: String,
    pub year: String,
    pub journal: String,
    pub kind: String,
    pub cited_by: Option<i64>,
    pub external: bool,
    pub degree: i64,
}

#[derive(Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

#[derive(Serialize)]
struct IndexEntry {
    doi: Option<String>,
    title: String,
    authors: Vec<String>,
    year: String,
    journal: String,
    kind: String,
    cited_by: Option<i64>,
    md_file: String,
}

/// Build a `Work` record from a parsed TEI document's metadata, abstract, and
/// reference list. `meta` may carry Crossref-enriched fields.
pub fn work_from_tei(
    tei_path: &Path,
    pdf_file: Option<&str>,
    md_file: Option<&str>,
    meta: &Metadata,
) -> anyhow::Result<Work> {
    let content = std::fs::read_to_string(tei_path)?;
    let doc = roxmltree::Document::parse(&content)?;
    let tei = crate::render::tei_root(doc.root());

    let abstract_text = crate::render::descendant_named(tei, "teiHeader")
        .and_then(|h| crate::render::descendant_named(h, "profileDesc"))
        .and_then(|p| crate::render::descendant_named(p, "abstract"))
        .map(|a| text_content(&a))
        .filter(|s| !s.is_empty());

    let key = meta
        .doi
        .clone()
        .or_else(|| {
            pdf_file
                .map(|f| f.trim_end_matches(".pdf").to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| {
            tei_path
                .file_stem()
                .map(|s| s.to_string_lossy().trim_end_matches(".tei").to_string())
        })
        .unwrap_or_default();

    Ok(Work {
        key,
        doi: meta.doi.clone(),
        doi_link: meta.doi_link.clone(),
        title: meta.title.clone().unwrap_or_default(),
        authors: meta.authors.clone(),
        year: meta.year.clone(),
        journal: meta.journal.clone(),
        volume: meta.volume.clone(),
        issue: meta.issue.clone(),
        pages: meta.pages.clone(),
        kind: meta.kind.clone(),
        abstract_text: abstract_text.or_else(|| meta.abstract_text.clone()),
        cited_by: meta.cited_by,
        md_file: md_file.map(str::to_string),
        pdf_file: pdf_file.map(str::to_string),
        references: extract_references(tei),
    })
}

/// Split a joined author string into individual names. GROBID output uses
/// commas, semicolons, pipes, and "and" as separators; names themselves
/// never contain those.
fn split_authors(authors: &str) -> Vec<String> {
    authors
        .split([',', ';', '|'])
        .flat_map(|part| part.split(" and "))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

/// Log a one-line corpus summary after an upsert pass.
pub fn log_corpus_summary(db: &CorpusDb) {
    match db.stats() {
        Ok(s) => log().info(&format!(
            "Corpus: {} works, {} reference edges ({} with DOI)",
            s.works, s.references, s.doi_refs
        )),
        Err(e) => log().warn(&format!("Could not read corpus stats: {e}")),
    }
}

/// Write a work's full metadata (including references) as pretty JSON.
/// Returns the written path. Filename derives from the PDF stem so it lines
/// up with `md/<name>.md`.
pub fn write_metadata_file(dir: &Path, work: &Work) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let stem = work
        .pdf_file
        .as_deref()
        .map(|f| {
            Path::new(f)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| work.key.clone())
        })
        .unwrap_or_else(|| work.key.clone());
    let path = dir.join(format!("{stem}.json"));
    std::fs::write(&path, serde_json::to_string_pretty(work)?)?;
    Ok(path)
}

/// Per-file outcome recorded in the process manifest.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileResult {
    pub pdf: String,
    /// `converted` (Markdown written this run), `cached` (outputs reused),
    /// or `failed`.
    pub status: String,
}

/// One conversion run's metadata, appended to `.grobid/process.json`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RunRecord {
    pub started_at: String,
    pub finished_at: String,
    pub duration_ms: u64,
    pub total: usize,
    pub converted: usize,
    pub cached: usize,
    pub failed: usize,
    pub enrich: bool,
    pub concurrency: usize,
    pub grobid_url: String,
    pub files: Vec<FileResult>,
}

/// Append a run record to the process manifest, keeping the last 50 runs.
pub fn append_process_manifest(path: &Path, record: &RunRecord) -> anyhow::Result<()> {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    struct Manifest {
        #[serde(default)]
        runs: Vec<serde_json::Value>,
    }
    let mut manifest = if path.exists() {
        let raw = std::fs::read_to_string(path)?;
        serde_json::from_str::<Manifest>(&raw).unwrap_or_default()
    } else {
        Manifest::default()
    };
    manifest.runs.push(serde_json::to_value(record)?);
    if manifest.runs.len() > 50 {
        let excess = manifest.runs.len() - 50;
        manifest.runs.drain(..excess);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}

const GRAPH_TEMPLATE: &str = include_str!("templates/graph.html");

/// Render the corpus as a self-contained Obsidian-style dashboard page:
/// citation graph plus corpus stats, year chart, and unified search filter.
pub fn render_graph_html(db: &CorpusDb) -> anyhow::Result<String> {
    let (nodes, edges) = db.graph_data()?;
    let stats = db.stats()?;
    let stats_json = serde_json::json!({
        "works": stats.works,
        "titled": stats.titled,
        "references": stats.references,
        "doi_refs": stats.doi_refs,
        "stub_refs": stats.stub_refs,
        "self_refs": stats.self_refs,
        "distinct_authors": stats.distinct_authors,
        "min_year": stats.min_year,
        "max_year": stats.max_year,
        "years": stats.year_hist,
        "top_authors": stats.top_authors,
        "top_in_cited": stats.top_in_cited,
        "top_citing": stats.top_citing,
        "top_cited_crossref": stats.top_cited,
    });
    let nodes_json = serde_json::to_string(&nodes)?;
    let edges_json = serde_json::to_string(&edges)?;
    Ok(GRAPH_TEMPLATE
        .replace("__NODES__", &nodes_json)
        .replace("__EDGES__", &edges_json)
        .replace("__STATS__", &stats_json.to_string())
        .replace("__TITLE__", "grobid.rs corpus"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_work() -> Work {
        Work {
            key: "10.1/test".into(),
            doi: Some("10.1/test".into()),
            doi_link: Some("https://doi.org/10.1/test".into()),
            title: "Test Paper".into(),
            authors: vec!["Ada Lovelace".into()],
            year: Some("2024".into()),
            journal: Some("Journal of Tests".into()),
            volume: Some("3".into()),
            issue: Some("2".into()),
            pages: Some("10-20".into()),
            kind: Some("journal-article".into()),
            abstract_text: Some("An abstract.".into()),
            cited_by: Some(42),
            md_file: Some("test.md".into()),
            pdf_file: Some("test.pdf".into()),
            references: vec![
                ParsedReference {
                    title: Some("Other Paper".into()),
                    doi: Some("10.2/other".into()),
                    year: Some("2020".into()),
                    ..Default::default()
                },
                ParsedReference {
                    title: None,
                    doi: None,
                    ..Default::default()
                },
            ],
        }
    }

    fn temp_db() -> (CorpusDb, PathBuf) {
        let path = std::env::temp_dir().join(format!(
            "grobid-test-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let db = open_db(&path).unwrap();
        (db, path)
    }

    #[test]
    fn upsert_and_search_roundtrip() {
        let (mut db, path) = temp_db();
        db.upsert_work(&sample_work()).unwrap();

        let hits = db.search("test paper", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].doi.as_deref(), Some("10.1/test"));

        let hits = db.search("lovelace", 10).unwrap();
        assert_eq!(hits.len(), 1);

        let hits = db.search("nonexistent", 10).unwrap();
        assert!(hits.is_empty());

        // LIKE wildcards in the query are escaped, not treated as patterns.
        let hits = db.search("%", 10).unwrap();
        assert!(hits.is_empty());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn stats_and_graph_data() {
        let (mut db, path) = temp_db();
        db.upsert_work(&sample_work()).unwrap();

        let s = db.stats().unwrap();
        assert_eq!(s.works, 1);
        assert_eq!(s.references, 2);
        assert_eq!(s.doi_refs, 1);
        assert_eq!(s.stub_refs, 1);
        assert_eq!(s.self_refs, 0);
        assert_eq!(s.min_year, "2024");
        assert_eq!(s.top_cited.len(), 1);
        assert_eq!(s.top_cited[0].2, 42);

        let (nodes, edges) = db.graph_data().unwrap();
        // 1 work node + 1 external DOI node.
        assert_eq!(nodes.len(), 2);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].source, "10.1/test");
        assert_eq!(edges[0].target, "ext:10.2/other");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn upsert_replaces_references() {
        let (mut db, path) = temp_db();
        let mut w = sample_work();
        db.upsert_work(&w).unwrap();
        w.references.clear();
        db.upsert_work(&w).unwrap();
        let s = db.stats().unwrap();
        assert_eq!(s.references, 0);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn export_index_writes_json() {
        let (mut db, path) = temp_db();
        db.upsert_work(&sample_work()).unwrap();
        let index = std::env::temp_dir().join(format!("grobid-idx-{}.json", std::process::id()));
        db.export_index(&index).unwrap();
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&index).unwrap()).unwrap();
        assert_eq!(json.as_array().unwrap().len(), 1);
        assert_eq!(json[0]["doi"], "10.1/test");
        let _ = std::fs::remove_file(&index);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn graph_html_contains_data() {
        let (mut db, path) = temp_db();
        db.upsert_work(&sample_work()).unwrap();
        let html = render_graph_html(&db).unwrap();
        assert!(html.contains("d3@7"));
        assert!(html.contains("10.1/test"));
        assert!(html.contains("10.2/other"));
        let _ = std::fs::remove_file(&path);
    }
}
