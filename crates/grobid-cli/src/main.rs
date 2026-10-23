mod cli;

use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(name = "grobid", about = "Convert PDFs to Markdown via GROBID")]
struct Args {
    /// Print version information.
    #[arg(long, short = 'V')]
    version: bool,

    /// Increase log verbosity (-v debug, -vv trace). Also honors GROBID_LOG.
    #[arg(long, short = 'v', action = ArgAction::Count)]
    verbose: u8,
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

    let exit_code = cli::convert_pdf_to_md().await?;
    std::process::exit(exit_code);
}
