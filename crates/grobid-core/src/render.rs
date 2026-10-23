//! TEI rendering.

use roxmltree::Node;

use crate::metadata::{extract_metadata_doc, frontmatter};
use crate::utils::text_content;

/// Find the TEI root element from a parsed document's root node.
pub(crate) fn tei_root<'a>(root: Node<'a, 'a>) -> Node<'a, 'a> {
    if root.is_element() && root.tag_name().name() == "TEI" {
        return root;
    }
    root.children()
        .find(|n| n.is_element() && n.tag_name().name() == "TEI")
        .unwrap_or(root)
}

/// First descendant element (depth-first) with the given local name.
pub(crate) fn descendant_named<'a>(node: Node<'a, 'a>, name: &str) -> Option<Node<'a, 'a>> {
    for child in node.children() {
        if !child.is_element() {
            continue;
        }
        if child.tag_name().name() == name {
            return Some(child);
        }
        if let Some(found) = descendant_named(child, name) {
            return Some(found);
        }
    }
    None
}

/// All descendant elements with the given local name (depth-first).
pub(crate) fn descendants_named<'a>(node: Node<'a, 'a>, name: &str) -> Vec<Node<'a, 'a>> {
    let mut out = Vec::new();
    for child in node.children() {
        if !child.is_element() {
            continue;
        }
        if child.tag_name().name() == name {
            out.push(child);
        }
        out.extend(descendants_named(child, name));
    }
    out
}

/// Format an author element's forename/surname parts into a display name.
pub(crate) fn author_name(author: Node) -> String {
    let mut parts: Vec<String> = Vec::new();
    for n in author.descendants() {
        if n.is_element() && (n.tag_name().name() == "forename" || n.tag_name().name() == "surname")
        {
            if let Some(t) = n.text() {
                parts.push(t.to_string());
            }
        }
    }
    let name = collapse_spaces(&parts.join(" ")).trim().to_string();
    if name.is_empty() {
        // Corporate authors, <name> elements, etc.
        return text_content(&author);
    }
    name
}

pub fn inline(element: Node) -> String {
    let mut parts: Vec<String> = Vec::new();
    // Don't use element.text() as it concatenates all descendants causing duplication
    // when we also process children
    for child in element.children() {
        if child.is_element() {
            let tag = child.tag_name().name();
            if tag == "hi" {
                let content = inline(child);
                let rend = child.attribute("rend").unwrap_or("").to_lowercase();
                if rend.contains("italic") {
                    parts.push(format!("*{}*", content));
                } else if rend.contains("bold") {
                    parts.push(format!("**{}**", content));
                } else {
                    parts.push(content);
                }
            } else if tag == "ref" {
                let content = inline(child);
                let target = child.attribute("target").unwrap_or("");
                if target.starts_with("https://") || target.starts_with("http://") {
                    if !content.is_empty() {
                        parts.push(format!("[{}]({})", content, target));
                    } else {
                        parts.push(content);
                    }
                } else {
                    parts.push(content);
                }
            } else if tag == "lb" {
                parts.push("  \n".to_string());
            } else {
                parts.push(inline(child));
            }
            if let Some(tail) = child.tail() {
                parts.push(tail.to_string());
            }
        } else if child.is_text() {
            parts.push(child.text().unwrap_or("").to_string());
        }
    }
    // If element has no children but has direct text (rare case), try text()
    if parts.is_empty() {
        if let Some(text) = element.text() {
            return collapse_spaces(text).trim().to_string();
        }
    }
    let joined = parts.join("");
    collapse_spaces(&joined).trim().to_string()
}

fn collapse_spaces(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for ch in s.chars() {
        if ch.is_whitespace() && ch != '\n' {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = ch == ' ';
        }
    }
    result
}

fn render_list(element: Node, depth: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    for item in element.children() {
        if item.is_element() && item.tag_name().name() == "item" {
            // look for p or direct content
            let mut content = String::new();
            let mut has_p = false;
            for p in item.children() {
                if p.is_element() && p.tag_name().name() == "p" {
                    content.push_str(&inline(p));
                    content.push(' ');
                    has_p = true;
                }
            }
            if !has_p {
                content = inline(item);
            }
            let content = content.trim().to_string();
            lines.push(format!("{}- {}", "  ".repeat(depth), content));
            for nested in item.children() {
                if nested.is_element() && nested.tag_name().name() == "list" {
                    lines.push(render_list(nested, depth + 1));
                }
            }
        }
    }
    lines.join("\n")
}

fn render_table(element: Node) -> String {
    let mut rows: Vec<Vec<String>> = Vec::new();
    for row in element.descendants() {
        if row.is_element() && row.tag_name().name() == "row" {
            let mut cells: Vec<String> = Vec::new();
            for cell in row.children() {
                if cell.is_element() && cell.tag_name().name() == "cell" {
                    cells.push(inline(cell).replace("|", "\\|"));
                }
            }
            if !cells.is_empty() {
                rows.push(cells);
            }
        }
    }
    if rows.is_empty() {
        return text_content(&element);
    }
    let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let rows_padded: Vec<Vec<String>> = rows
        .into_iter()
        .map(|mut r| {
            while r.len() < width {
                r.push(String::new());
            }
            r
        })
        .collect();
    let mut output: Vec<String> = Vec::new();
    if !rows_padded.is_empty() {
        output.push(format!("| {} |", rows_padded[0].join(" | ")));
        output.push(format!("| {} |", vec!["---"; width].join(" | ")));
        for row in rows_padded[1..].iter() {
            output.push(format!("| {} |", row.join(" | ")));
        }
    }
    output.join("\n")
}

fn render_node(element: Node, depth: usize) -> String {
    let tag = element.tag_name().name();
    if tag == "head" {
        let heading = inline(element);
        let level = if depth < 6 { depth } else { 6 };
        return format!("{} {}", "#".repeat(level), heading);
    }
    if tag == "p" {
        return inline(element);
    }
    if tag == "list" {
        return render_list(element, 0);
    }
    if tag == "table" {
        return render_table(element);
    }
    if tag == "formula" {
        return format!("$$\n{}\n$$", text_content(&element));
    }
    if tag == "figure" {
        let mut label_parts: Vec<String> = Vec::new();
        for l in element.children() {
            if l.is_element() && l.tag_name().name() == "label" {
                label_parts.push(text_content(&l));
            }
        }
        let mut caption_parts: Vec<String> = Vec::new();
        for c in element.children() {
            if c.is_element() && (c.tag_name().name() == "figDesc" || c.tag_name().name() == "head")
            {
                caption_parts.push(text_content(&c));
            }
        }
        let mut graphic_url = String::new();
        for g in element.descendants() {
            if g.is_element() && g.tag_name().name() == "graphic" {
                if let Some(url) = g.attribute("url") {
                    graphic_url = url.to_string();
                    break;
                }
            }
        }
        let description: String = [label_parts.join(" "), caption_parts.join(" ")]
            .into_iter()
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !graphic_url.is_empty() {
            return format!("![{}]({})", description, graphic_url);
        }
        if !description.is_empty() {
            return format!("> {}", description);
        }
        return String::new();
    }
    if tag == "div" {
        return render_children(element, depth + 1);
    }
    String::new()
}

fn render_children(element: Node, depth: usize) -> String {
    let mut blocks: Vec<String> = Vec::new();
    for child in element.children() {
        if child.is_element() {
            let tag = child.tag_name().name();
            if matches!(
                tag,
                "head" | "p" | "div" | "list" | "table" | "formula" | "figure"
            ) {
                let result = render_node(child, depth);
                if !result.is_empty() {
                    blocks.push(result);
                }
            }
        }
    }
    blocks.join("\n\n")
}

/// Render a single bibliography entry (biblStruct or bibl) as a readable line:
/// "Authors. Title. Journal. Year" when the structure is present, otherwise a
/// flattened text fallback.
fn render_reference(entry: Node) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Authors and analytic (work) title.
    if let Some(analytic) = descendant_named(entry, "analytic") {
        let authors: Vec<String> = descendants_named(analytic, "author")
            .iter()
            .map(|a| author_name(*a))
            .filter(|s| !s.is_empty())
            .collect();
        if !authors.is_empty() {
            parts.push(authors.join(", "));
        }
        if let Some(title) = descendant_named(analytic, "title") {
            let t = inline(title);
            if !t.is_empty() {
                parts.push(t);
            }
        }
    }

    // Container title (journal preferred via level="j") and imprint year.
    if let Some(monogr) = descendant_named(entry, "monogr") {
        let mut titles = descendants_named(monogr, "title");
        titles.sort_by_key(|n| {
            n.attribute("level")
                .map(|l| if l.eq_ignore_ascii_case("j") { 0 } else { 1 })
                .unwrap_or(1)
        });
        for t in titles {
            let s = inline(t);
            if !s.is_empty() {
                parts.push(s);
                break;
            }
        }
        if let Some(imprint) = descendant_named(monogr, "imprint") {
            if let Some(date) = descendant_named(imprint, "date") {
                let year = date
                    .attribute("when")
                    .filter(|w| w.chars().count() >= 4)
                    .map(|w| w.chars().take(4).collect::<String>())
                    .unwrap_or_else(|| text_content(&date));
                if !year.is_empty() {
                    parts.push(year);
                }
            }
        }
    }

    if !parts.is_empty() {
        return parts.join(". ");
    }
    text_content(&entry)
}

fn convert_doc(
    doc: &roxmltree::Document,
    meta: Option<&crate::metadata::Metadata>,
    fallback_title: &str,
) -> anyhow::Result<String> {
    let root = doc.root();

    // Find TEI element
    let tei = tei_root(root);

    // Title - prefer titleStmt title
    let title_from_meta = match meta {
        Some(m) => m.title.clone(),
        None => None,
    };
    let mut title = String::new();
    if let Some(t) = title_from_meta.as_ref() {
        if !t.is_empty() {
            title = t.clone();
        }
    }
    if title.is_empty() {
        if let Some(header) = descendant_named(tei, "teiHeader") {
            if let Some(file_desc) = descendant_named(header, "fileDesc") {
                if let Some(title_stmt) = descendant_named(file_desc, "titleStmt") {
                    let mut titles = descendants_named(title_stmt, "title");
                    titles.sort_by_key(|n| {
                        n.attribute("type")
                            .map(|t| if t.eq_ignore_ascii_case("main") { 0 } else { 1 })
                            .unwrap_or(1)
                    });
                    for t in titles {
                        let t2 = inline(t);
                        if !t2.is_empty() {
                            title = t2;
                            break;
                        }
                    }
                }
            }
        }
    }
    if title.is_empty() {
        title = fallback_title.to_string();
    }

    // Authors
    let mut authors: Vec<String> = Vec::new();
    if let Some(m) = meta {
        authors = m.authors.clone();
    }
    if authors.is_empty() {
        // Scope to teiHeader so authors cited in the reference list are excluded.
        let author_nodes = descendant_named(tei, "teiHeader")
            .map(|header| descendants_named(header, "author"))
            .unwrap_or_default();
        for author in author_nodes {
            let name = author_name(author);
            if !name.is_empty() {
                authors.push(name);
            }
        }
    }

    let abstract_node = descendant_named(tei, "teiHeader")
        .and_then(|header| descendant_named(header, "profileDesc"))
        .and_then(|profile| descendant_named(profile, "abstract"));
    let body_node = descendant_named(tei, "text").and_then(|text| descendant_named(text, "body"));

    let meta_resolved = match meta {
        Some(m) => m.clone(),
        None => extract_metadata_doc(doc),
    };
    let mut sections: Vec<String> = Vec::new();
    sections.push(frontmatter(&meta_resolved));
    sections.push(format!("# {}", title));
    if !authors.is_empty() {
        sections.push(format!("**Authors:** {}", authors.join(", ")));
    }
    if let Some(abs) = abstract_node {
        let abstract_md = render_children(abs, 2);
        if !abstract_md.is_empty() {
            sections.push("## Abstract".to_string());
            sections.push(abstract_md);
        }
    }
    if let Some(body) = body_node {
        let body_md = render_children(body, 2);
        if !body_md.is_empty() {
            sections.push("## Article".to_string());
            sections.push(body_md);
        }
    }

    // References: biblStruct/bibl entries inside listBibl.
    let mut references: Vec<String> = Vec::new();
    for list in descendants_named(tei, "listBibl") {
        for child in list.children() {
            if child.is_element() && matches!(child.tag_name().name(), "biblStruct" | "bibl") {
                let reference = render_reference(child);
                if !reference.is_empty() {
                    references.push(format!("- {reference}"));
                }
            }
        }
    }
    if !references.is_empty() {
        sections.push("## References".to_string());
        sections.push(references.join("\n"));
    }

    let result = sections.join("\n\n").trim().to_string();
    Ok(format!("{}\n", result))
}

/// Convert a TEI XML file to Markdown.
pub fn convert_tei_to_md(
    input_path: &std::path::Path,
    meta: Option<&crate::metadata::Metadata>,
) -> anyhow::Result<String> {
    let content = std::fs::read_to_string(input_path)?;
    let doc = roxmltree::Document::parse(&content)?;
    let fallback_title = input_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .trim_end_matches(".tei")
        .to_string();
    convert_doc(&doc, meta, &fallback_title)
}

/// Convert a TEI XML string to Markdown.
pub fn convert_tei_xml(xml: &str) -> anyhow::Result<String> {
    let doc = roxmltree::Document::parse(xml)?;
    convert_doc(&doc, None, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEI: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<TEI xmlns="http://www.tei-c.org/ns/1.0">
  <teiHeader>
    <fileDesc>
      <titleStmt><title level="a" type="main">Render Test Title</title></titleStmt>
      <sourceDesc>
        <biblStruct>
          <analytic><author><forename>Ann</forename><surname>Lee</surname></author></analytic>
          <monogr>
            <title level="j">Journal of Rendering</title>
            <imprint>
              <date when="2020">2020</date>
              <biblScope unit="volume">7</biblScope>
              <biblScope unit="page">1-9</biblScope>
            </imprint>
          </monogr>
        </biblStruct>
      </sourceDesc>
    </fileDesc>
    <profileDesc><abstract><p>Abstract text here.</p></abstract></profileDesc>
  </teiHeader>
  <text>
    <body>
      <div>
        <head>Introduction</head>
        <p>First paragraph with <hi rend="italic">emphasis</hi>.</p>
        <list><item><p>Item one</p></item><item><p>Item two</p></item></list>
        <table><row><cell>A</cell><cell>B</cell></row><row><cell>1</cell><cell>2</cell></row></table>
      </div>
    </body>
  </text>
  <back>
    <div type="references">
      <listBibl>
        <biblStruct><monogr><title>Reference One</title></monogr></biblStruct>
        <bibl><title>Reference Two</title></bibl>
      </listBibl>
    </div>
  </back>
</TEI>"#;

    #[test]
    fn renders_full_document() {
        let md = convert_tei_xml(TEI).unwrap();
        assert!(md.starts_with("---\n"));
        assert!(md.contains("title: \"Render Test Title\""));
        assert!(md.contains("journal: \"Journal of Rendering\""));
        assert!(md.contains("year: 2020"));
        assert!(md.contains("volume: \"7\""));
        assert!(md.contains("pages: \"1-9\""));
        assert!(md.contains("# Render Test Title"));
        assert!(md.contains("**Authors:** Ann Lee"));
        assert!(md.contains("## Abstract"));
        assert!(md.contains("## Article"));
        assert!(md.contains("## Introduction"));
        assert!(md.contains("*emphasis*"));
        assert!(md.contains("- Item one"));
        assert!(md.contains("| A | B |"));
        assert!(md.contains("## References"));
        assert!(md.contains("- Reference One"));
        assert!(md.contains("- Reference Two"));
    }

    #[test]
    fn falls_back_to_filename_title() {
        let xml = TEI.replace(
            "<titleStmt><title level=\"a\" type=\"main\">Render Test Title</title></titleStmt>",
            "<titleStmt/>",
        );
        let doc = roxmltree::Document::parse(&xml).unwrap();
        let md = convert_doc(&doc, None, "fallback-name").unwrap();
        assert!(md.contains("# fallback-name"));
    }

    #[test]
    fn rejects_invalid_xml() {
        assert!(convert_tei_xml("<TEI><unclosed>").is_err());
    }

    #[test]
    fn converts_real_grobid_tei_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/tei/Predicting language outcome after stroke using machine learning - in search of the big data benefit.tei.xml"
        );
        let md = convert_tei_to_md(std::path::Path::new(path), None).unwrap();
        assert!(md.starts_with("---\n"));
        assert!(md.contains("# "));
        assert!(md.contains("## Article"));
        // Real GROBID output includes a reference list.
        assert!(md.contains("## References"));
    }
}
