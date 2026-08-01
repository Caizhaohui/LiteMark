# LiteMark

<p align="center">
  <img src="assets/brand/litemark-icon-256.png" width="128" height="128" alt="LiteMark icon" />
</p>

<p align="center">
  <strong>Local-first Markdown desktop editor for Windows</strong><br/>
  Typora-inspired workflow · Monaco source · Milkdown hybrid · crossnote preview · HTML/PDF export
</p>

<p align="center">
  <a href="https://github.com/Caizhaohui-tib/LiteMark/releases"><img alt="Release" src="https://img.shields.io/github/v/release/Caizhaohui-tib/LiteMark?style=flat-square" /></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" /></a>
  <a href="docs/privacy.md"><img alt="Privacy" src="https://img.shields.io/badge/telemetry-none-success?style=flat-square" /></a>
</p>

---

## Highlights

| | |
|---|---|
| **Source mode** | Monaco Editor — full Markdown control, find/replace, multi-cursor |
| **Hybrid mode** | Milkdown / ProseMirror with roundtrip safety (blocks lossy switches) |
| **Live preview** | crossnote sidecar — Mermaid, KaTeX, GFM; unsaved content stays in memory |
| **Export** | HTML (offline package), PDF (system Edge/Chrome), optional Pandoc DOCX/EPUB/LaTeX |
| **Safety** | No webview FS permission; atomic saves; crash recovery; DOMPurify; trusted workspaces |
| **Privacy** | No accounts, no telemetry, no cloud |

## Screenshots / UI

- Multi-tab document shell with dirty indicators  
- Source / Split / Preview layouts (`Ctrl+1` / `2` / `3`)  
- Source ↔ Hybrid editor mode with data-loss guard  
- Export dialogs with progress & cancel  
- Settings: trusted folders, Pandoc path, crash report  

## Download

**[Latest release (v2.0.0)](https://github.com/Caizhaohui-tib/LiteMark/releases/tag/v2.0.0)**

| Asset | Description |
|-------|-------------|
| `LiteMark_2.0.0_x64-setup.exe` | Windows 10/11 x64 NSIS installer (current user) |

> **Note for preview/export after install:** the render sidecar needs Node.js. During development you can point at a built sidecar via:
>
> ```text
> LITEMARK_NODE=C:\Path\To\node.exe
> LITEMARK_SIDECAR_ENTRY=...\packages\render-sidecar\dist\index.js
> ```
>
> A fully self-contained Node bundle is planned; see `scripts/SIDECAR-BUNDLE.txt`.

## Quick start (developers)

### Requirements

- Windows 10/11 x64  
- [Rust](https://rustup.rs/) stable (`x86_64-pc-windows-msvc`) + Visual Studio Build Tools  
- [Node.js](https://nodejs.org/) 22+ and [pnpm](https://pnpm.io/) 11  
- WebView2 Runtime (usually preinstalled on modern Windows)

### Run

```powershell
corepack enable
pnpm install
pnpm --filter @litemark/render-sidecar build
pnpm dev
```

### Build installer

```powershell
pnpm tauri build
# → src-tauri/target/release/bundle/nsis/LiteMark_2.0.0_x64-setup.exe
```

### Tests

```powershell
pnpm typecheck
pnpm test
pnpm test:rust
```

## Architecture

```text
React WebView (Monaco / Milkdown / Preview)
        │  Tauri invoke (no fs/dialog permission)
        ▼
Rust core (sessions, atomic save, recovery, path auth, export)
        │  stdin/stdout JSON Lines
        ▼
Node sidecar + crossnote (render / HTML / PDF)
```

Details: [docs/architecture.md](docs/architecture.md) · [docs/security.md](docs/security.md) · [docs/adr/](docs/adr/)

## Repository layout

| Path | Role |
|------|------|
| `packages/app-ui` | React + Vite frontend |
| `packages/render-sidecar` | crossnote IPC process |
| `packages/shared-protocol` | Shared IPC & Tauri contracts |
| `packages/markdown-core` | Hybrid roundtrip / normalize |
| `src-tauri` | Tauri 2 / Rust application |
| `LiteMark_DEVELOPMENT_PLAN.md` | Milestone plan (M0–M6) |

## Features by milestone

| Milestone | Status | Summary |
|-----------|--------|---------|
| M0 | Done | Tauri shell, sidecar ping, crossnote spike |
| M1 | Done | Open/save, tabs, atomic write, recovery |
| M2 | Done | Monaco, live preview, TOC, large-file degrade |
| M3 | Done | HTML/PDF export, NSIS, file associations |
| M4 | Done | Milkdown hybrid + roundtrip guards |
| M5 | Done | Pandoc, trusted workspaces, settings |
| M6 | Done | CI, privacy docs, SBOM hooks, a11y |

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` / `O` / `S` | New / Open / Save |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+1` / `2` / `3` | Source / Split / Preview layout |
| `Ctrl+,` | Settings |

## Documentation

- [User guide](docs/user/getting-started.md)  
- [Privacy](docs/privacy.md)  
- [Performance baselines](docs/performance.md)  
- [Development plan](LiteMark_DEVELOPMENT_PLAN.md)  

## License

MIT — see [LICENSE](LICENSE).  
Third-party notices: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

LiteMark is **not** affiliated with Typora, Markdown Preview Enhanced, or crossnote authors.

## Contributing

Issues and PRs welcome. Please keep milestone scope and security defaults in mind (untrusted Markdown, no silent FS access from the webview).
