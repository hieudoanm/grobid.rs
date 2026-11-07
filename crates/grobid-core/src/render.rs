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
    collapse_spaces(&parts.join(" ")).trim().to_string()
}

fn find_element_by_tag<'a>(node: Node<'a, 'a>, tag: &str) -> Option<Node<'a, 'a>> {
    node.children()
        .find(|child| child.is_element() && child.tag_name().name() == tag)
}

pub fn find_element_by_tag_pub<'a>(node: Node<'a, 'a>, tag: &str) -> Option<Node<'a, 'a>> {
    find_element_by_tag(node, tag)
}

pub fn first_element<'a>(node: Node<'a, 'a>, xpath: &str) -> Option<Node<'a, 'a>> {
    // Simplified xpath handling for common patterns used
    if xpath.contains("/tei:TEI/tei:teiHeader/tei:fileDesc/tei:titleStmt/tei:title") {
        // Look in teiHeader/fileDesc/titleStmt/title
        let tei = if node.is_element() && node.tag_name().name() == "TEI" {
            Some(node)
        } else {
            node.children()
                .find(|n| n.is_element() && n.tag_name().name() == "TEI")
        };
        if let Some(tei) = tei {
            if let Some(header) = find_element_by_tag_pub(tei, "teiHeader") {
                if let Some(file_desc) = find_element_by_tag_pub(header, "fileDesc") {
                    if let Some(title_stmt) = find_element_by_tag_pub(file_desc, "titleStmt") {
                        if let Some(title) = find_element_by_tag_pub(title_stmt, "title") {
                            return Some(title);
                        }
                    }
                }
            }
        }
        return None;
    }
    if xpath == "//tei:profileDesc/tei:abstract" || xpath.contains("profileDesc/tei:abstract") {
        let tei = if node.is_element() && node.tag_name().name() == "TEI" {
            Some(node)
        } else {
            node.children()
                .find(|n| n.is_element() && n.tag_name().name() == "TEI")
        };
        let tei = tei?;
        let header = find_element_by_tag_pub(tei, "teiHeader")?;
        let profile = find_element_by_tag_pub(header, "profileDesc")?;
        return find_element_by_tag_pub(profile, "abstract");
    }
    if xpath == "//tei:text/tei:body" || xpath.contains("text/tei:body") {
        let tei = if node.is_element() && node.tag_name().name() == "TEI" {
            Some(node)
        } else {
            node.children()
                .find(|n| n.is_element() && n.tag_name().name() == "TEI")
        };
        let tei = tei?;
        let text = find_element_by_tag_pub(tei, "text")?;
        return find_element_by_tag_pub(text, "body");
    }
    None
}

pub fn find_all<'a>(node: Node<'a, 'a>, xpath: &str) -> Vec<Node<'a, 'a>> {
    let mut results = Vec::new();
    // For now, handle common cases
    if xpath.contains("author") {
        // search recursively
        fn search_authors<'a>(n: Node<'a, 'a>, results: &mut Vec<Node<'a, 'a>>) {
            if n.is_element() && n.tag_name().name() == "author" {
                results.push(n);
                return;
            }
            for child in n.children() {
                search_authors(child, results);
            }
        }
        search_authors(node, &mut results);
    }
    if xpath.contains("listBibl") {
        fn search_listbibl<'a>(n: Node<'a, 'a>, results: &mut Vec<Node<'a, 'a>>) {
            if n.is_element() && (n.tag_name().name() == "listBibl") {
                for child in n.children() {
                    if child.is_element()
                        && (child.tag_name().name() == "biblStruct"
                            || child.tag_name().name() == "bibl")
                    {
                        results.push(child);
                    }
                }
                return;
            }
            for child in n.children() {
                search_listbibl(child, results);
            }
        }
        search_listbibl(node, &mut results);
    }
    results
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
        if let Some(header) = find_element_by_tag_pub(tei, "teiHeader") {
            if let Some(file_desc) = find_element_by_tag_pub(header, "fileDesc") {
                if let Some(title_stmt) = find_element_by_tag_pub(file_desc, "titleStmt") {
                    for child in title_stmt.children() {
                        if child.is_element() && child.tag_name().name() == "title" {
                            let t2 = inline(child);
                            if !t2.is_empty() {
                                title = t2;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    if title.is_empty() {
        if let Some(title_node) = first_element(
            tei,
            "/tei:TEI/tei:teiHeader/tei:fileDesc/tei:titleStmt/tei:title",
        ) {
            let t2 = inline(title_node);
            if !t2.is_empty() {
                title = t2;
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
        let author_nodes = find_element_by_tag_pub(tei, "teiHeader")
            .map(|header| descendants_named(header, "author"))
            .unwrap_or_default();
        for author in author_nodes {
            let name = author_name(author);
            if !name.is_empty() {
                authors.push(name);
            }
        }
    }

    let abstract_node = first_element(tei, "//tei:profileDesc/tei:abstract");
    let body_node = first_element(tei, "//tei:text/tei:body");

    let meta_resolved = match meta {
        Some(m) => m.clone(),
        None => extract_metadata_doc(doc),
    };
    let mut sections: Vec<String> = Vec::new();
    sections.push(frontmatter(&meta_resolved));
    // Deduplicate title if duplicated
    let mut title_clean = title.clone();
    if title_clean.len() > 10 {
        let mid = title_clean.len() / 2;
        // check if first half equals second half (simple)
        let (a, b) = title_clean.split_at(mid);
        if a.trim() == b.trim() && !a.trim().is_empty() {
            title_clean = a.trim().to_string();
        }
    }
    sections.push(format!("# {}", title_clean));
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

    // References: find_all returns the biblStruct/bibl entries inside listBibl.
    let mut references: Vec<String> = Vec::new();
    for ref_node in find_all(tei, "listBibl") {
        let ref_text = text_content(&ref_node);
        if !ref_text.is_empty() {
            references.push(format!("- {}", ref_text));
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
