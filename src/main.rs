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
#[command(name = "litemark", version, about = "LiteMark: Lightweight offline Markdown previewer")]
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
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Export { format }) => match format {
            ExportFormat::Html { file, output } => {
                let md = std::fs::read_to_string(&file)?;
                let parsed = markdown::parse(&md);
                let file_dir = file.parent().unwrap_or_else(|| std::path::Path::new("."));
                let file_cfg = config::FileConfig::load(file_dir);
                let cfg = RuntimeConfig {
                    file_config: file_cfg,
                    theme: cli.theme,
                    ..Default::default()
                };
                let html = export::html::export_html(&parsed, &cfg, &file);
                let out = output.unwrap_or_else(|| file.with_extension("html"));
                std::fs::write(&out, html)?;
                println!("Exported: {}", out.display());
            }
            ExportFormat::Pdf { file, output } => {
                let out = output.unwrap_or_else(|| file.with_extension("pdf"));
                export::pdf::export_pdf(&file, &out)?;
                println!("Exported: {}", out.display());
            }
        },
        None => {
            let file = cli.file.unwrap_or_else(|| {
                eprintln!("Usage: litemark <file.md>");
                eprintln!("       litemark export html <file.md>");
                std::process::exit(1);
            });

            let file = std::fs::canonicalize(&file)?;
            if !file.exists() {
                anyhow::bail!("File not found: {}", file.display());
            }

            let file_dir = file.parent().unwrap_or_else(|| std::path::Path::new("."));
            let file_cfg = config::FileConfig::load(file_dir);
            let cfg = RuntimeConfig {
                file_config: file_cfg,
                port: cli.port,
                theme: cli.theme,
                open_browser: !cli.no_open,
            };

            server::run_server(file, cfg).await?;
        }
    }

    Ok(())
}
