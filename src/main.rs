mod assets;
mod config;
mod export;
mod markdown;
mod server;
mod watcher;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use config::RuntimeConfig;

#[derive(Parser)]
#[command(
    name = "litemark",
    version,
    author = "Caizhaohui",
    about = "Lightweight offline Markdown previewer and exporter",
    long_about = "LiteMark is an offline Markdown previewer for local writing workflows. It starts a local preview server, opens the browser automatically, and exports self-contained HTML or PDF.",
    after_help = "Examples:\n  litemark note.md\n  litemark export html note.md -o note.html\n  litemark export pdf note.md -o note.pdf --browser edge --page-size Letter"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Markdown file to preview
    file: Option<PathBuf>,

    /// Server port (0 = auto)
    #[arg(short, long, default_value_t = 0)]
    port: u16,

    /// Preview theme
    #[arg(short, long, default_value = "github-light")]
    theme: String,

    /// Don't open browser automatically
    #[arg(long)]
    no_open: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Export markdown to other formats
    Export {
        #[command(subcommand)]
        format: ExportFormat,
    },
}

#[derive(Subcommand)]
enum ExportFormat {
    /// Export as self-contained HTML
    Html {
        /// Input markdown file
        file: PathBuf,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Export as PDF
    Pdf {
        /// Input markdown file
        file: PathBuf,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// PDF page size, e.g. A4 or Letter
        #[arg(long, default_value = "A4")]
        page_size: String,
        /// Print page backgrounds into the PDF
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        print_background: bool,
        /// Browser to use: edge, chrome, chromium, or a full executable path
        #[arg(long)]
        browser: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Export { format }) => match format {
            ExportFormat::Html { file, output } => {
                let md = std::fs::read_to_string(&file)?;
                let cfg = RuntimeConfig::for_file(&file, cli.port, cli.theme.clone(), !cli.no_open);
                let parsed = markdown::parse_with_render_config(&md, &cfg.file_config.render);
                let html = export::html::export_html(&parsed, &cfg, &file);
                let out = output.unwrap_or_else(|| file.with_extension("html"));
                std::fs::write(&out, html)?;
                println!("Exported: {}", out.display());
            }
            ExportFormat::Pdf {
                file,
                output,
                page_size,
                print_background,
                browser,
            } => {
                let out = output.unwrap_or_else(|| file.with_extension("pdf"));
                let options = export::pdf::PdfExportOptions {
                    page_size,
                    print_background,
                    browser,
                };
                export::pdf::export_pdf(&file, &out, &options)?;
                println!("Exported: {}", out.display());
            }
        },
        None => {
            let file = cli.file.unwrap_or_else(|| {
                eprintln!("Usage: litemark <file.md>");
                eprintln!("       litemark export html <file.md>");
                eprintln!("       litemark --help");
                std::process::exit(1);
            });

            let file = std::fs::canonicalize(&file)?;
            if !file.exists() {
                anyhow::bail!("File not found: {}", file.display());
            }

            let cfg = RuntimeConfig::for_file(&file, cli.port, cli.theme, !cli.no_open);

            server::run_server(file, cfg).await?;
        }
    }

    Ok(())
}
