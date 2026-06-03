use std::path::Path;
use std::process::Command;

pub fn export_pdf(md_path: &Path, out_path: &Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(md_path)?;
    let doc = crate::markdown::parse(&content);
    let file_dir = md_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let file_cfg = crate::config::FileConfig::load(file_dir);
    let config = crate::config::RuntimeConfig {
        file_config: file_cfg,
        ..Default::default()
    };
    let html = super::html::export_html(&doc, &config, md_path);

    let temp_html = out_path.with_extension("litemark-temp.html");
    std::fs::write(&temp_html, &html)?;

    let abs_html = std::fs::canonicalize(&temp_html)?;
    let abs_out = if out_path.is_absolute() {
        out_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(out_path)
    };

    let html_url = url::Url::from_file_path(&abs_html)
        .map_err(|_| anyhow::anyhow!("Failed to create file URL for {}", abs_html.display()))?;

    let result = try_print_to_pdf(html_url.as_str(), &abs_out);

    let _ = std::fs::remove_file(&temp_html);

    result
}

fn try_print_to_pdf(html_url: &str, out_path: &Path) -> anyhow::Result<()> {
    let browsers = [
        "chrome",
        "msedge",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        "google-chrome",
        "chromium",
        "chromium-browser",
    ];

    for browser in &browsers {
        let output = Command::new(browser)
            .args([
                "--headless",
                "--disable-gpu",
                "--no-sandbox",
                &format!("--print-to-pdf={}", out_path.display()),
                "--print-to-pdf-no-header",
                html_url,
            ])
            .output();

        match output {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    eprintln!("{} stderr: {}", browser, stderr.trim());
                }
            }
            Err(_) => continue,
        }
    }

    anyhow::bail!(
        "Could not find Chrome/Edge/Chromium for PDF export.\n\
         Please install a Chromium-based browser or use HTML export instead:\n\
           litemark export html <file.md>"
    )
}
