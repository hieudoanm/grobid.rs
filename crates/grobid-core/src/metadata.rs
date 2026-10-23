//! Metadata extraction from TEI XML.

use roxmltree::Node;

use crate::config::META_FIELDS;
use crate::render::{author_name, descendant_named, descendants_named, inline, tei_root};
use crate::utils::{text_content, yaml_string};

/// Build YAML frontmatter string from a metadata map. Empty fields are
/// omitted so consumers don't have to distinguish "" from "absent".
pub fn frontmatter(meta: &Metadata) -> String {
    let mut lines: Vec<String> = vec!["---".to_string()];
    for field in META_FIELDS {
        let value = match field.to_string().as_str() {
            "authors" => meta.authors.join(", "),
            "year" => meta.year.clone().unwrap_or_default(),
            "title" => meta.title.clone().unwrap_or_default(),
            "journal" => meta.journal.clone().unwrap_or_default(),
            "volume" => meta.volume.clone().unwrap_or_default(),
            "issue" => meta.issue.clone().unwrap_or_default(),
            "pages" => meta.pages.clone().unwrap_or_default(),
            "doi" => meta.doi.clone().unwrap_or_default(),
            "doi_link" => meta.doi_link.clone().unwrap_or_default(),
            _ => String::new(),
        };
        if value.is_empty() {
            continue;
        }
        if *field == "year" && value.chars().all(|c| c.is_ascii_digit()) {
            lines.push(format!("year: {value}"));
        } else {
            lines.push(format!("{}: {}", field, yaml_string(&value)));
        }
    }
    lines.push("---".to_string());
    lines.join("\n")
}

/// Metadata extracted from a TEI document.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub year: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub doi_link: Option<String>,
}

pub(crate) fn extract_metadata_doc(doc: &roxmltree::Document) -> Metadata {
    let mut meta = Metadata::default();
    let tei = tei_root(doc.root());
    if tei.tag_name().name() != "TEI" {
        return meta;
    }
    let Some(header) = descendant_named(tei, "teiHeader") else {
        return meta;
    };

    // Title from fileDesc/titleStmt/title (prefer type="main" if present).
    if let Some(file_desc) = descendant_named(header, "fileDesc") {
        if let Some(title_stmt) = descendant_named(file_desc, "titleStmt") {
            let mut titles: Vec<Node> = descendants_named(title_stmt, "title");
            titles.sort_by_key(|n| {
                n.attribute("type")
                    .map(|t| if t.eq_ignore_ascii_case("main") { 0 } else { 1 })
                    .unwrap_or(1)
            });
            for node in titles {
                let t = inline(node);
                if !t.is_empty() {
                    meta.title = Some(t);
                    break;
                }
            }
        }
    }
    // Authors: only within the header so reference-list authors are excluded.
    let author_nodes = descendants_named(header, "author");
    for author in author_nodes {
        let name = author_name(author);
        if !name.is_empty() {
            meta.authors.push(name);
        }
    }

    // Journal/monograph metadata from sourceDesc biblStruct/monogr.
    if let Some(source_desc) = descendant_named(header, "sourceDesc") {
        if let Some(bibl_struct) = descendant_named(source_desc, "biblStruct") {
            if let Some(monogr) = descendant_named(bibl_struct, "monogr") {
                for title in descendants_named(monogr, "title") {
                    if title
                        .attribute("level")
                        .is_some_and(|l| l.eq_ignore_ascii_case("j"))
                    {
                        let journal = inline(title);
                        if !journal.is_empty() {
                            meta.journal = Some(journal);
                            break;
                        }
                    }
                }
                if let Some(imprint) = descendant_named(monogr, "imprint") {
                    for date in descendants_named(imprint, "date") {
                        let when = date
                            .attribute("when")
                            .or_else(|| date.attribute("notBefore"))
                            .unwrap_or("");
                        if when.len() >= 4 {
                            meta.year = Some(when[..4].to_string());
                            break;
                        }
                    }
                    for scope in descendants_named(imprint, "biblScope") {
                        let unit = scope.attribute("unit").unwrap_or("");
                        let value = text_content(&scope);
                        if value.is_empty() {
                            continue;
                        }
                        match unit {
                            "volume" => meta.volume = Some(value),
                            "issue" => meta.issue = Some(value),
                            "page" => meta.pages = Some(value),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // DOI from idno[@type='DOI'] anywhere in the header.
    for idno in descendants_named(header, "idno") {
        let is_doi = idno
            .attribute("type")
            .is_some_and(|t| t.eq_ignore_ascii_case("doi"));
        if !is_doi {
            continue;
        }
        let doi = text_content(&idno);
        if !doi.is_empty() {
            meta.doi_link = Some(if doi.starts_with("http") {
                doi.clone()
            } else {
                format!("https://doi.org/{doi}")
            });
            meta.doi = Some(doi);
            break;
        }
    }

    meta
}

/// Extract metadata from a TEI XML file.
pub fn extract_metadata_from_tei(tei_path: &std::path::Path) -> anyhow::Result<Metadata> {
    let content = std::fs::read_to_string(tei_path)?;
    let doc = roxmltree::Document::parse(&content)?;
    Ok(extract_metadata_doc(&doc))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEI: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<TEI xmlns="http://www.tei-c.org/ns/1.0">
  <teiHeader>
    <fileDesc>
      <titleStmt>
        <title level="a" type="main">A Study of Things</title>
        <title level="a" type="sub">A Study of Things: Supplement</title>
      </titleStmt>
      <publicationStmt><publisher>Test Publisher</publisher></publicationStmt>
      <sourceDesc>
        <biblStruct>
          <analytic>
            <author><forename>Ada</forename><surname>Lovelace</surname></author>
            <author><forename>Alan</forename><surname>Turing</surname></author>
          </analytic>
          <monogr>
            <title level="j">Journal of Tests</title>
            <imprint>
              <date when="2021-05-01">May 2021</date>
              <biblScope unit="volume">12</biblScope>
              <biblScope unit="issue">3</biblScope>
              <biblScope unit="page">45-67</biblScope>
            </imprint>
          </monogr>
        </biblStruct>
      </sourceDesc>
    </fileDesc>
    <profileDesc>
      <abstract><p>Short abstract.</p></abstract>
    </profileDesc>
    <idno type="DOI">10.1234/test.2021</idno>
  </teiHeader>
  <text>
    <body>
      <div><head>Introduction</head><p>Hello world.</p></div>
    </body>
  </text>
  <back>
    <div type="references">
      <listBibl>
        <biblStruct>
          <analytic><author><forename>Ref</forename><surname>Author</surname></author></analytic>
          <monogr><title>Some Reference</title></monogr>
        </biblStruct>
      </listBibl>
    </div>
  </back>
</TEI>"#;

    #[test]
    fn extracts_all_metadata_fields() {
        let doc = roxmltree::Document::parse(TEI).unwrap();
        let meta = extract_metadata_doc(&doc);
        assert_eq!(meta.title.as_deref(), Some("A Study of Things"));
        assert_eq!(meta.authors, vec!["Ada Lovelace", "Alan Turing"]);
        assert_eq!(meta.journal.as_deref(), Some("Journal of Tests"));
        assert_eq!(meta.year.as_deref(), Some("2021"));
        assert_eq!(meta.volume.as_deref(), Some("12"));
        assert_eq!(meta.issue.as_deref(), Some("3"));
        assert_eq!(meta.pages.as_deref(), Some("45-67"));
        assert_eq!(meta.doi.as_deref(), Some("10.1234/test.2021"));
        assert_eq!(
            meta.doi_link.as_deref(),
            Some("https://doi.org/10.1234/test.2021")
        );
    }

    #[test]
    fn does_not_pick_up_reference_authors() {
        let doc = roxmltree::Document::parse(TEI).unwrap();
        let meta = extract_metadata_doc(&doc);
        assert!(!meta.authors.iter().any(|a| a.contains("Ref")));
    }

    #[test]
    fn handles_missing_header() {
        let doc = roxmltree::Document::parse("<TEI/>").unwrap();
        let meta = extract_metadata_doc(&doc);
        assert!(meta.title.is_none());
        assert!(meta.authors.is_empty());
    }

    #[test]
    fn frontmatter_contains_fields() {
        let meta = Metadata {
            title: Some("T".into()),
            authors: vec!["A B".into()],
            year: Some("2020".into()),
            doi: Some("10.1/x".into()),
            ..Default::default()
        };
        let fm = frontmatter(&meta);
        assert!(fm.starts_with("---\n"));
        assert!(fm.contains("title: \"T\""));
        assert!(fm.contains("authors: \"A B\""));
        assert!(fm.contains("year: 2020"));
        assert!(fm.contains("doi: \"10.1/x\""));
    }
}
