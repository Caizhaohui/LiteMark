use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfExportOptions {
    pub page_size: String,
    pub print_background: bool,
    pub browser: Option<String>,
}

impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            page_size: "A4".to_string(),
            print_background: true,
            browser: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BrowserAttempt {
    browser: String,
    detail: String,
}

pub fn export_pdf(md_path: &Path, out_path: &Path, options: &PdfExportOptions) -> anyhow::Result<()> {
    export_pdf_with_printer(md_path, out_path, options, try_print_to_pdf)
}

fn export_pdf_with_printer<F>(
    md_path: &Path,
    out_path: &Path,
    options: &PdfExportOptions,
    printer: F,
) -> anyhow::Result<()>
where
    F: Fn(&str, &Path, &PdfExportOptions) -> anyhow::Result<()>,
{
    let content = std::fs::read_to_string(md_path)?;
    let config = crate::config::RuntimeConfig::for_file(md_path, 0, String::new(), false);
    let doc = crate::markdown::parse_with_render_config(&content, &config.file_config.render);
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

    let result = printer(html_url.as_str(), &abs_out, options);

    let _ = std::fs::remove_file(&temp_html);

    result
}

fn default_candidate_browsers() -> &'static [&'static str] {
    &[
        "chrome",
        "msedge",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        "google-chrome",
        "chromium",
        "chromium-browser",
    ]
}

fn resolve_browser_candidates(browser: Option<&str>) -> Vec<String> {
    match browser.map(str::trim).filter(|s| !s.is_empty()) {
        None => default_candidate_browsers().iter().map(|s| (*s).to_string()).collect(),
        Some("edge") => vec![
            "msedge".to_string(),
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe".to_string(),
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe".to_string(),
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge".to_string(),
        ],
        Some("chrome") => vec![
            "chrome".to_string(),
            r"C:\Program Files\Google\Chrome\Application\chrome.exe".to_string(),
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe".to_string(),
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string(),
        ],
        Some("chromium") => vec![
            "chromium".to_string(),
            "chromium-browser".to_string(),
            "google-chrome".to_string(),
        ],
        Some(other) => vec![other.to_string()],
    }
}

fn build_browser_args(html_url: &str, out_path: &Path, options: &PdfExportOptions) -> Vec<String> {
    vec![
        "--headless".to_string(),
        "--disable-gpu".to_string(),
        "--no-sandbox".to_string(),
        format!("--print-to-pdf={}", out_path.display()),
        "--print-to-pdf-no-header".to_string(),
        format!(
            "--print-to-pdf-page-config={{\"pageSize\":\"{}\",\"printBackground\":{}}}",
            options.page_size, options.print_background
        ),
        html_url.to_string(),
    ]
}

fn try_print_to_pdf(html_url: &str, out_path: &Path, options: &PdfExportOptions) -> anyhow::Result<()> {
    let mut attempts = Vec::new();

    for browser in resolve_browser_candidates(options.browser.as_deref()) {
        let output = Command::new(&browser)
            .args(build_browser_args(html_url, out_path, options))
            .output();

        match output {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let detail = if !stderr.trim().is_empty() {
                    format!("exit {} stderr: {}", output.status, stderr.trim())
                } else if !stdout.trim().is_empty() {
                    format!("exit {} stdout: {}", output.status, stdout.trim())
                } else {
                    format!("exit {}", output.status)
                };
                attempts.push(BrowserAttempt {
                    browser: browser.clone(),
                    detail,
                });
            }
            Err(err) => attempts.push(BrowserAttempt {
                browser: browser.clone(),
                detail: err.to_string(),
            }),
        }
    }

    Err(build_browser_error(&attempts))
}

fn build_browser_error(attempts: &[BrowserAttempt]) -> anyhow::Error {
    let mut message = String::from(
        "PDF export failed.\n\
         Tried Chromium-based browsers in this order:\n",
    );

    for attempt in attempts {
        message.push_str(&format!("  - {}: {}\n", attempt.browser, attempt.detail));
    }

    message.push_str(
        "Install Chrome/Edge/Chromium or use HTML export instead:\n  litemark export html <file.md>",
    );

    anyhow::anyhow!(message)
}

#[cfg(test)]
mod tests {
    use super::{
        build_browser_args, build_browser_error, export_pdf_with_printer,
        resolve_browser_candidates, BrowserAttempt, PdfExportOptions,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn browser_args_include_page_size_and_background() {
        let options = PdfExportOptions {
            page_size: "Letter".to_string(),
            print_background: false,
            browser: None,
        };
        let args = build_browser_args("file:///tmp/demo.html", Path::new("out.pdf"), &options);
        let joined = args.join(" ");
        assert!(joined.contains("--print-to-pdf=out.pdf"));
        assert!(joined.contains("\"pageSize\":\"Letter\""));
        assert!(joined.contains("\"printBackground\":false"));
    }

    #[test]
    fn browser_error_lists_attempts() {
        let err = build_browser_error(&[
            BrowserAttempt {
                browser: "chrome".to_string(),
                detail: "not found".to_string(),
            },
            BrowserAttempt {
                browser: "msedge".to_string(),
                detail: "exit code 1".to_string(),
            },
        ]);
        let text = err.to_string();
        assert!(text.contains("PDF export failed."));
        assert!(text.contains("chrome: not found"));
        assert!(text.contains("msedge: exit code 1"));
        assert!(text.contains("litemark export html"));
    }

    #[test]
    fn default_pdf_options_are_stable() {
        let options = PdfExportOptions::default();
        assert_eq!(options.page_size, "A4");
        assert!(options.print_background);
        assert!(options.browser.is_none());
    }

    #[test]
    fn resolve_browser_candidates_supports_named_aliases_and_custom_path() {
        assert!(resolve_browser_candidates(Some("edge"))
            .iter()
            .any(|candidate| candidate.contains("msedge")));
        assert!(resolve_browser_candidates(Some("chrome"))
            .iter()
            .any(|candidate| candidate.contains("chrome")));
        assert_eq!(
            resolve_browser_candidates(Some(r"C:\Custom\Browser.exe")),
            vec![r"C:\Custom\Browser.exe".to_string()]
        );
    }

    fn temp_fixture_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("litemark-pdf-{name}-{unique}"));
        fs::create_dir_all(&dir).expect("create temp fixture dir");
        dir
    }

    #[test]
    fn export_pdf_cleans_up_temp_html_on_printer_failure() {
        let dir = temp_fixture_dir("cleanup-failure");
        let md_path = dir.join("note.md");
        let out_path = dir.join("note.pdf");
        let temp_html = out_path.with_extension("litemark-temp.html");

        fs::write(&md_path, "# Title\n\nBody\n").expect("write markdown fixture");

        let err = export_pdf_with_printer(
            &md_path,
            &out_path,
            &PdfExportOptions::default(),
            |_html_url, _out_path, _options| Err(anyhow::anyhow!("printer failed")),
        )
        .expect_err("printer failure should bubble up");

        assert!(err.to_string().contains("printer failed"));
        assert!(!temp_html.exists(), "temporary html should be removed on failure");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_pdf_passes_expected_inputs_to_printer() {
        let dir = temp_fixture_dir("printer-inputs");
        let md_path = dir.join("note.md");
        let out_path = dir.join("note.pdf");
        let seen: Arc<Mutex<Vec<(String, PathBuf, PdfExportOptions)>>> =
            Arc::new(Mutex::new(Vec::new()));

        fs::write(&md_path, "# Title\n\nBody\n").expect("write markdown fixture");

        let seen_clone = Arc::clone(&seen);
        export_pdf_with_printer(
            &md_path,
            &out_path,
            &PdfExportOptions {
                page_size: "Letter".to_string(),
                print_background: false,
                browser: Some("edge".to_string()),
            },
            move |html_url, pdf_path, options| {
                seen_clone.lock().expect("lock seen").push((
                    html_url.to_string(),
                    pdf_path.to_path_buf(),
                    options.clone(),
                ));
                Ok(())
            },
        )
        .expect("export should succeed with stub printer");

        let seen = seen.lock().expect("lock seen");
        assert_eq!(seen.len(), 1);
        assert!(seen[0].0.starts_with("file:///"));
        assert_eq!(seen[0].1, out_path);
        assert_eq!(seen[0].2.page_size, "Letter");
        assert!(!seen[0].2.print_background);
        assert_eq!(seen[0].2.browser.as_deref(), Some("edge"));
        assert!(
            !out_path.with_extension("litemark-temp.html").exists(),
            "temporary html should be removed on success"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    #[ignore = "requires a local Chromium-based browser and writes a real PDF"]
    fn export_pdf_end_to_end_with_real_browser() {
        if std::env::var("LITEMARK_RUN_PDF_E2E").ok().as_deref() != Some("1") {
            return;
        }

        let dir = temp_fixture_dir("pdf-e2e");
        let md_path = dir.join("note.md");
        let out_path = dir.join("note.pdf");

        fs::write(&md_path, "# Title\n\nHello LiteMark\n").expect("write markdown fixture");
        super::export_pdf(&md_path, &out_path, &PdfExportOptions::default())
            .expect("real browser pdf export should succeed");

        let meta = fs::metadata(&out_path).expect("pdf output should exist");
        assert!(meta.len() > 0, "pdf output should be non-empty");
        assert!(
            !out_path.with_extension("litemark-temp.html").exists(),
            "temporary html should be removed after real browser export"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
