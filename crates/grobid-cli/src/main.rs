mod cli;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "grobid", about = "Convert PDFs to Markdown via GROBID")]
struct Args {
    #[arg(long, short)]
    version: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    if args.version {
        println!("grobid {}", grobid_core::version::get_version());
        return Ok(());
    }

    let exit_code = cli::convert_pdf_to_md().await?;
    std::process::exit(exit_code);
}
