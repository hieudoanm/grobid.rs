//! Optional Crossref enrichment: resolve a work's DOI against the public
//! Crossref API to fill fields GROBID may miss (type, citation count,
//! abstract, alternative metadata). Opt-in via `--enrich`/`GROBID_ENRICH`.

use serde_json::Value;

use crate::metadata::Metadata;
use crate::utils::{collapse_whitespace, log};

const CROSSREF_API: &str = "https://api.crossref.org/works";

/// Fields fetched from Crossref for a single DOI.
#[derive(Debug, Clone, Default)]
pub struct Enrichment {
    pub kind: Option<String>,
    pub cited_by: Option<i64>,
    pub abstract_text: Option<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
}

/// Merge enrichment results into extracted metadata. Only fills fields that
/// are still empty — TEI/GROBID data always wins.
pub fn apply(meta: &mut Metadata, enrichment: &Enrichment) {
    if meta.kind.is_none() {
        meta.kind = enrichment.kind.clone();
    }
    if meta.cited_by.is_none() {
        meta.cited_by = enrichment.cited_by;
    }
    if meta.abstract_text.is_none() {
        meta.abstract_text = enrichment.abstract_text.clone();
    }
    if meta.year.is_none() {
        meta.year = enrichment.year.clone();
    }
    if meta.journal.is_none() {
        meta.journal = enrichment.journal.clone();
    }
}

/// Look up a DOI on Crossref. Returns `None` on any failure (network, 404,
/// malformed payload) — enrichment is best-effort and never blocks conversion.
pub async fn lookup(client: &reqwest::Client, doi: &str) -> Option<Enrichment> {
    let url = format!("{CROSSREF_API}/{}", urlencoding::encode(doi.trim()));
    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Value = resp.json().await.ok()?;
    parse_message(&body)
}

fn parse_message(body: &Value) -> Option<Enrichment> {
    let msg = body.get("message")?;
    let mut out = Enrichment {
        kind: msg.get("type").and_then(Value::as_str).map(str::to_string),
        cited_by: msg.get("is-referenced-by-count").and_then(Value::as_i64),
        abstract_text: msg
            .get("abstract")
            .and_then(Value::as_str)
            .and_then(strip_jats),
        ..Default::default()
    };
    // Year from issued/published-print/published-online date-parts.
    for key in ["published-print", "published", "issued", "published-online"] {
        if let Some(year) = msg
            .get(key)
            .and_then(|d| d.get("date-parts"))
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(Value::as_array)
            .and_then(|a| a.first())
            .and_then(Value::as_i64)
        {
            out.year = Some(year.to_string());
            break;
        }
    }
    out.journal = msg
        .get("container-title")
        .and_then(Value::as_array)
        .and_then(|a| a.first())
        .and_then(Value::as_str)
        .map(str::to_string);
    Some(out)
}

/// Crossref abstracts are JATS fragments (`<jats:p>…</jats:p>`); reduce them
/// to plain text.
fn strip_jats(raw: &str) -> Option<String> {
    let mut text = String::with_capacity(raw.len());
    let mut in_tag = false;
    for ch in raw.chars() {
        match ch {
            '<' => in_tag = true,
            // Space after each tag keeps block-level siblings (</jats:p><jats:p>)
            // separated; collapse_whitespace normalizes the extras.
            '>' => {
                in_tag = false;
                text.push(' ');
            }
            _ if !in_tag => text.push(ch),
            _ => {}
        }
    }
    let text = collapse_whitespace(&text);
    let text = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x000a0;", " ")
        .replace("&#160;", " ");
    let text = collapse_whitespace(&text).trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// Whether enrichment is enabled: `--enrich` flag or `GROBID_ENRICH` env.
pub fn enabled(flag: bool) -> bool {
    flag || std::env::var("GROBID_ENRICH").is_ok_and(|v| v != "0" && !v.is_empty())
}

/// Convenience: look up and log failures at debug level.
pub async fn lookup_logged(client: &reqwest::Client, doi: &str) -> Option<Enrichment> {
    let result = lookup(client, doi).await;
    if result.is_none() {
        log().debug(&format!("Crossref: no result for {doi}"));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_crossref_message() {
        let body = json!({
            "message": {
                "DOI": "10.1/x",
                "type": "journal-article",
                "is-referenced-by-count": 123,
                "title": ["A Paper"],
                "container-title": ["Nature"],
                "published-print": {"date-parts": [[2020, 5, 1]]},
                "abstract": "<jats:p>First &amp; foremost.</jats:p><jats:p>Second.</jats:p>"
            }
        });
        let e = parse_message(&body).unwrap();
        assert_eq!(e.kind.as_deref(), Some("journal-article"));
        assert_eq!(e.cited_by, Some(123));
        assert_eq!(e.year.as_deref(), Some("2020"));
        assert_eq!(e.journal.as_deref(), Some("Nature"));
        assert_eq!(
            e.abstract_text.as_deref(),
            Some("First & foremost. Second.")
        );
    }

    #[test]
    fn parse_message_handles_minimal_payload() {
        let body = json!({"message": {"DOI": "10.1/y"}});
        let e = parse_message(&body).unwrap();
        assert!(e.kind.is_none());
        assert!(e.abstract_text.is_none());
    }

    #[test]
    fn apply_only_fills_empty_fields() {
        let mut meta = Metadata {
            year: Some("1999".into()),
            ..Default::default()
        };
        let e = Enrichment {
            kind: Some("proceedings-article".into()),
            year: Some("2020".into()),
            ..Default::default()
        };
        apply(&mut meta, &e);
        assert_eq!(meta.kind.as_deref(), Some("proceedings-article"));
        assert_eq!(meta.year.as_deref(), Some("1999"));
    }

    #[test]
    fn strip_jats_removes_markup() {
        assert_eq!(
            strip_jats("<jats:p>Hello <italic>world</italic></jats:p>"),
            Some("Hello world".into())
        );
        assert_eq!(strip_jats("<jats:p>   </jats:p>"), None);
    }
}
