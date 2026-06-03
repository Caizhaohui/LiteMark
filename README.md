# LiteMark

LiteMark is a Windows-first Rust tool for offline Markdown preview and export. It runs as a CLI, starts a local preview server, opens the browser automatically, and supports self-contained HTML and Chromium-based PDF export.

Current stable release: `v1.0.0`

## Features

- Local Markdown preview with live refresh
- Table of contents, front matter, callout, task list, and footnote support
- KaTeX, Mermaid, code highlighting, emoji, and lightbox toggles
- Self-contained HTML export
- PDF export through Chrome, Edge, or Chromium
- Project configuration via `.litemark.toml`

## Quick Start

```bat
cd LiteMark
scripts\cargo-msvc.cmd run -- path\to\note.md
```

Export HTML:

```bat
scripts\cargo-msvc.cmd run -- export html path\to\note.md -o path\to\note.html
```

Export PDF:

```bat
scripts\cargo-msvc.cmd run -- export pdf path\to\note.md -o path\to\note.pdf --browser edge --page-size Letter --print-background true
```

Show help and version:

```bat
scripts\cargo-msvc.cmd run -- --help
scripts\cargo-msvc.cmd run -- --version
```

## Configuration

Primary config file:

```text
.litemark.toml
```

Legacy fallback, still accepted:

```text
.stillmark.toml
```

Example config:

```text
.litemark.toml.example
```

Active configuration keys:

- `preview.theme`
- `preview.debounce_ms`
- `preview.scroll_sync`
- `render.math`
- `render.mermaid`
- `render.highlight`
- `render.callout`
- `render.emoji`
- `render.lightbox`
- `export.embed_images`

## Build And Release

Check and test:

```bat
scripts\cargo-msvc.cmd check
scripts\cargo-msvc.cmd test
```

Build release executable:

```bat
scripts\build-release.cmd
```

Package the Windows portable distribution:

```bat
scripts\package-portable.cmd
```

Generated portable layout:

```text
dist/
  LiteMark-v1.0.0-windows-x64-portable/
    litemark.exe
    README.md
    .litemark.toml.example
  LiteMark-v1.0.0-windows-x64-portable.zip
```

## PDF Export Options

- `--browser`
  Accepts `edge`, `chrome`, `chromium`, or a full executable path
- `--page-size`
  Accepts values such as `A4`, `Letter`, `Legal`
- `--print-background`
  Controls whether background colors and images are printed into the PDF

## GitHub Release

Repository:

```text
https://github.com/Caizhaohui/LiteMark
```

Release target:

```text
v1.0.0
```

## Desktop GUI Plan

The current stable release is CLI + browser preview. Tauri GUI work starts in `desktop/` and `src-tauri/` so the existing Rust parsing and export logic can be reused instead of rewritten.

See [DEVELOPMENT.md](DEVELOPMENT.md) for the current desktop migration plan.
