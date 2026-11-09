mod cli;
mod mcp;

use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "grobid", about = "Convert PDFs to Markdown via GROBID")]
struct Args {
    /// Print version information.
    #[arg(long, short = 'V')]
    version: bool,

    /// Increase log verbosity (-v debug, -vv trace). Also honors GROBID_LOG.
    #[arg(long, short = 'v', action = ArgAction::Count)]
    verbose: u8,

    /// Enrich metadata via the Crossref API (adds type/cited_by to
    /// frontmatter, abstract/type/citations to the corpus index).
    /// Also enabled by GROBID_ENRICH=1. Only applies when converting.
    #[arg(long)]
    enrich: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate the corpus dashboard (index.html) from the corpus DB.
    Graph {
        /// Corpus DB path (default: <data-dir>/.grobid/grobid.db).
        #[arg(long)]
        db: Option<PathBuf>,
        /// Output HTML path (default: <data-dir>/.grobid/index.html).
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Search the corpus index (title/authors/journal/DOI/abstract/type).
    Search {
        /// Search query (substring match).
        query: String,
        /// Maximum results.
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// Corpus DB path (default: <data-dir>/.grobid/grobid.db).
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Print corpus statistics (works, references, years, top authors/cited).
    Stats {
        /// Corpus DB path (default: <data-dir>/.grobid/grobid.db).
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Serve the corpus over MCP (stdio) for AI agents.
    Mcp {
        /// Corpus DB path (default: <data-dir>/.grobid/grobid.db).
        #[arg(long)]
        db: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    if args.verbose > 0 {
        grobid_core::utils::bump_level(args.verbose);
    }
    if args.version {
        println!(
            "grobid {} (build {})",
            env!("CARGO_PKG_VERSION"),
            grobid_core::version::get_version()
        );
        return Ok(());
    }

    let exit_code = match args.command {
        Some(Command::Graph { db, out }) => cli::graph_cmd(db, out)?,
        Some(Command::Search { query, limit, db }) => cli::search_cmd(&query, limit, db)?,
        Some(Command::Stats { db }) => cli::stats_cmd(db)?,
        Some(Command::Mcp { db }) => {
            mcp::serve(db).await?;
            return Ok(());
        }
        None => cli::convert_pdf_to_md(args.enrich).await?,
    };
    std::process::exit(exit_code);
}
