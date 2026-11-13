# TEI → Markdown

After GROBID produces TEI XML, `grobid-core` renders it to Markdown. The same
parsed document feeds both metadata extraction and body rendering, so the XML is
read and parsed exactly once per file.

## Parsing model

- `roxmltree` parses the document into a DOM; parsing is non-validating.
- Elements are matched by **local name**, not qualified name — TEI documents
  with or without the `tei:` namespace prefix both work.
- Helpers in `render.rs`:
  - `tei_root` — finds the `TEI` root element
  - `descendant_named(node, name)` — first descendant with the given local name
  - `descendants_named(node, name)` — all descendants with the given local name
  - `inline(node)` — flattens inline markup (`hi`, `ref`, `lb`, nested elements)
    to plain text
  - `text_content(node)` — all descendant text, whitespace-collapsed

## Metadata extraction (`metadata.rs`)

`extract_metadata_doc` walks `teiHeader` only:

| Field                        | Source                                                                |
| ---------------------------- | --------------------------------------------------------------------- |
| `title`                      | `fileDesc/titleStmt/title`, preferring `type="main"`                  |
| `authors`                    | every `author` element under the header (via `author_name`)           |
| `journal`                    | `sourceDesc/biblStruct/monogr/title` with `level="j"`                 |
| `year`                       | `imprint/date[@when]` (or `notBefore`), first 4 chars                 |
| `volume` / `issue` / `pages` | `imprint/biblScope` by `unit` attribute                               |
| `doi`                        | first `idno[@type='DOI']` under the header                            |
| `doi_link`                   | the DOI itself if it starts with `http`, else `https://doi.org/<doi>` |

Scoping to `teiHeader` is deliberate: document-wide searches pick up authors
from `listBibl` reference entries, which must not appear in frontmatter.

`frontmatter()` emits a YAML block containing exactly the keys in
`config::META_FIELDS`, omitting empty values (authors are joined with `", "`,
and a numeric `year` is emitted unquoted):

```yaml
---
title: '...'
authors: '...'
year: 2024
journal: '...'
volume: '10'
issue: '2'
pages: '100-115'
doi: '10.1234/example'
doi_link: 'https://doi.org/10.1234/example'
---
```

## Rendering (`render.rs`)

`convert_tei_xml`/`convert_tei_to_md` produce:

```markdown
---
<frontmatter>
---

# <title or filename stem>

**Authors:** ... (when present)

## Abstract (when present)

...

## Article (body sections, preserving heading levels)

...

## References (from listBibl/biblStruct or bibl entries)

- ...
```

Rendered constructs include headings, paragraphs, inline emphasis, lists,
tables, formulas, figures, and the reference list. When no title is found in the
TEI, the Markdown `#` heading falls back to the input file's stem.

## References

Entries inside `listBibl` (`biblStruct`/`bibl`) are parsed by `parse_reference`
into a `ParsedReference` (authors, analytic title, container title preferring
`level="j"`, imprint year, and `idno[@type='DOI']`) and rendered as
`Authors. Title. Journal. Year`. When a DOI is present the title becomes a
`[Title](https://doi.org/…)` link; unstructured entries fall back to flattened
text. `extract_references` exposes the same parsed list to the corpus indexer
(`corpus.rs`), which stores citation edges — including DOIs of works not present
in the corpus — in `grobid.db`.

## Tests

Unit tests live inline in `render.rs`/`metadata.rs` and cover: full-document
rendering, metadata field extraction, missing-header tolerance, reference-author
exclusion, filename fallback, and invalid-XML rejection. The real GROBID fixture
in `crates/grobid-core/tests/fixtures/` is rendered end-to-end by
`converts_real_grobid_tei_fixture`.
