# LiteMark

Lightweight offline Markdown previewer with math (KaTeX), diagrams (Mermaid), callouts, YAML frontmatter, auto TOC, live reload, interactive task lists, and export to self-contained HTML/PDF.

## Features

- Live preview server with WebSocket hot reload and file watching
- GitHub-style callouts (`[!NOTE]`, `[!TIP]`, etc.)
- YAML frontmatter with nice rendering
- Automatic table of contents
- Math typesetting via KaTeX
- Mermaid diagrams
- Syntax highlighting
- Interactive task list checkboxes (click in preview to toggle and save back to source)
- Image lightbox
- Theme toggle (Ctrl+D)
- Export to self-contained single-file HTML
- Export to PDF via headless Chromium/Edge
- Theming (github-light / github-dark)
- Sidebar Table of Contents (auto, updates live)
- Per-directory `.litemark.toml` config (searching ancestors)

## Quick Start

```bash
# After clone, fetch full offline assets (recommended)
pwsh -File scripts/fetch-vendor-assets.ps1

# Preview (uses config from .litemark.toml or ancestors)
litemark examples/test.md

# Or a complex doc
litemark examples/pscl4-report.md
```

## Usage

```bash
# Preview
litemark path/to/note.md

# Export
litemark export html note.md
litemark export pdf report.md -o report.pdf
```

Config file: `.litemark.toml` in the same directory as your markdown (or parent).

See source for supported render options.

## Building

```bash
cargo build --release
```

## Vendored Assets

For full offline support (math, diagrams, highlight), place these in `assets/vendor/`:

- katex.min.js + katex.min.css + auto-render.min.js (from KaTeX)
- mermaid.min.js
- highlight.min.js + highlight.min.css

A placeholder for katex is included; replace with real files for production use.

## License

MIT (or your choice)
