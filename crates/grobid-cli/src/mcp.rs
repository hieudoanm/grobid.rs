//! MCP server over stdio (`grobid mcp`) exposing the local corpus:
//! stats, search, work listing, and graph generation.

use std::path::PathBuf;

use grobid_core::{corpus, paths};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, ErrorData as McpError, ServerCapabilities, ServerConfig,
};
use rmcp::schemars::JsonSchema;
use rmcp::{tool, tool_handler, tool_router, ServerHandler, ServiceExt};
use serde::Deserialize;

#[derive(Clone)]
pub struct GrobidMcp {
    db_path: PathBuf,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchParams {
    /// Substring query matched against title, authors, journal, DOI,
    /// abstract, and type.
    pub query: String,
    /// Maximum results (default 20).
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LimitParams {
    /// Maximum results (default 50).
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GraphParams {
    /// Output HTML path (default: <data-dir>/.grobid/index.html).
    #[serde(default)]
    pub out: Option<String>,
}

fn internal_error(e: impl std::fmt::Display) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

fn json_result<T: serde::Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value).map_err(internal_error)?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

#[tool_router]
impl GrobidMcp {
    #[tool(description = "Corpus statistics: works, references, DOI coverage, \
            year span, distinct authors, and top in/out-degree rankings.")]
    async fn grobid_stats(&self) -> Result<CallToolResult, McpError> {
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let s = db.stats().map_err(internal_error)?;
        json_result(&serde_json::json!({
            "works": s.works,
            "titled": s.titled,
            "untitled": s.works - s.titled,
            "references": s.references,
            "doi_refs": s.doi_refs,
            "stub_refs": s.stub_refs,
            "self_refs": s.self_refs,
            "distinct_authors": s.distinct_authors,
            "min_year": s.min_year,
            "max_year": s.max_year,
            "year_hist": s.year_hist,
            "top_authors": s.top_authors,
            "top_in_cited": s.top_in_cited,
            "top_citing": s.top_citing,
            "top_cited_crossref": s.top_cited,
        }))
    }

    #[tool(description = "Search the corpus index. Substring match across \
            title, authors, journal, DOI, abstract, and type.")]
    async fn grobid_search(
        &self,
        Parameters(p): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let rows = db
            .search(&p.query, p.limit.unwrap_or(20))
            .map_err(internal_error)?;
        json_result(
            &rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "key": r.key,
                        "doi": r.doi,
                        "title": r.title,
                        "authors": r.authors,
                        "year": r.year,
                        "journal": r.journal,
                        "kind": r.kind,
                        "cited_by": r.cited_by,
                        "md_file": r.md_file,
                    })
                })
                .collect::<Vec<_>>(),
        )
    }

    #[tool(description = "List works in the corpus, most recent metadata first.")]
    async fn grobid_works(
        &self,
        Parameters(p): Parameters<LimitParams>,
    ) -> Result<CallToolResult, McpError> {
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let rows = db
            .search("", p.limit.unwrap_or(50))
            .map_err(internal_error)?;
        json_result(
            &rows
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "key": r.key,
                        "doi": r.doi,
                        "title": r.title,
                        "authors": r.authors,
                        "year": r.year,
                        "journal": r.journal,
                        "kind": r.kind,
                    })
                })
                .collect::<Vec<_>>(),
        )
    }

    #[tool(description = "Generate the interactive citation-graph dashboard \
            (self-contained HTML with d3-force, stats, and unified search) \
            and return its path.")]
    async fn grobid_graph(
        &self,
        Parameters(p): Parameters<GraphParams>,
    ) -> Result<CallToolResult, McpError> {
        let db = corpus::open_db(&self.db_path).map_err(internal_error)?;
        let html = corpus::render_graph_html(&db).map_err(internal_error)?;
        let out = p.out.map(PathBuf::from).unwrap_or_else(paths::graph_path);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(internal_error)?;
        }
        std::fs::write(&out, html).map_err(internal_error)?;
        json_result(&serde_json::json!({ "path": out.display().to_string() }))
    }
}

#[tool_handler]
impl ServerHandler for GrobidMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Local GROBID corpus tools: statistics, full-text search over \
             converted papers, work listing, and citation-graph dashboard \
             generation. Requires a corpus DB built by running `grobid`."
                .to_string(),
        )
    }
}

/// Serve MCP over stdio until the client disconnects.
pub async fn serve(db_arg: Option<PathBuf>) -> anyhow::Result<()> {
    let db_path = db_arg.unwrap_or_else(paths::db_path);
    let service = GrobidMcp { db_path };
    let running = service
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await?;
    running.waiting().await?;
    Ok(())
}
