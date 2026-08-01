# LiteMark

<p align="center">
  <img src="assets/brand/litemark-icon-256.png" width="128" height="128" alt="LiteMark icon" />
</p>

<p align="center">
  <strong>Local-first Markdown desktop editor for Windows</strong><br/>
  Typora-inspired workflow · Monaco source · Milkdown hybrid · crossnote preview · multi-format export
</p>

<p align="center">
  <a href="https://github.com/Caizhaohui/LiteMark/releases"><img alt="Release" src="https://img.shields.io/github/v/release/Caizhaohui/LiteMark?style=flat-square" /></a>
  <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" /></a>
  <a href="docs/privacy.md"><img alt="Privacy" src="https://img.shields.io/badge/telemetry-none-success?style=flat-square" /></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-Windows%2010%2F11%20x64-0078D6?style=flat-square" />
  <img alt="Stack" src="https://img.shields.io/badge/stack-Tauri%202%20%7C%20React%20%7C%20Rust%20%7C%20Node-informational?style=flat-square" />
</p>

---

## What is LiteMark?

**LiteMark** is a Windows desktop Markdown editor that keeps your files local, your edits under control, and your preview faithful. It combines:

- **Monaco** for precise source editing  
- **Milkdown** for optional hybrid (WYSIWYM) editing with data-loss guards  
- **crossnote** (Node sidecar) for live preview, Mermaid, KaTeX, GFM, and export  

The webview **never** has raw filesystem access. All open/save/export/render traffic goes through a Rust core that authorizes paths, runs atomic writes, and brokers a JSON-Lines IPC protocol to the render sidecar.

> LiteMark is **not** affiliated with Typora, Markdown Preview Enhanced, or the crossnote authors.

---

## Highlights

| Area | What you get |
|------|----------------|
| **Source mode** | Monaco: syntax highlight, word wrap, find/replace, multi-cursor, undo/redo |
| **Hybrid mode** | Milkdown/ProseMirror; switch blocked when roundtrip would rewrite content unsafely |
| **Live preview** | crossnote pipeline — GFM, code fences, KaTeX, Mermaid; **unsaved buffer only** (no silent disk write) |
| **Layouts** | Source / Split / Preview; resizable split panes (ratio remembered) |
| **Export** | Offline HTML package; PDF via system Edge/Chrome; optional Pandoc DOCX / EPUB / LaTeX |
| **i18n** | English (default), 简体中文, 繁體中文, 日本語 — toolbar language picker + Settings |
| **Safety** | Atomic saves, crash recovery snapshots, external-change prompts, DOMPurify, trusted workspaces for experiments |
| **Privacy** | No accounts, no telemetry, no cloud sync required |
| **Windows UX** | `.md` file association, single-instance open, cold-start CLI paths, no console flash for the sidecar |

---

## Download

**[Latest release (v2.1.0)](https://github.com/Caizhaohui/LiteMark/releases/tag/v2.1.0)**

| Asset | Description |
|-------|-------------|
| `LiteMark_2.1.0_x64-setup.exe` | Windows 10/11 x64 NSIS installer (per-user) |

### After install

1. Launch **LiteMark** from the Start Menu.  
2. Open a `.md` file (`Ctrl+O`) or double-click a Markdown file (if associated).  
3. Use **Source / Split / Preview** and export from the toolbar.  
4. Change UI language from the toolbar dropdown or **Settings → Language**.

### Runtime note (render / export)

Preview and export use a **Node.js** process hosting crossnote. For development builds you can point at a local sidecar:

```text
LITEMARK_NODE=C:\Path\To\node.exe
LITEMARK_SIDECAR_ENTRY=E:\path\to\LiteMark\packages\render-sidecar\dist\index.js
```

A fully self-contained Node bundle is tracked as follow-up work (`scripts/SIDECAR-BUNDLE.txt`).  
PDF export needs **Microsoft Edge** or **Google Chrome** installed (or a configured browser path).

---

## Screenshots / UI tour

| Surface | Behavior |
|---------|----------|
| Tab bar | Multi-document tabs, dirty ·, read-only mark, close with unsaved confirm |
| Toolbar | New / Open / Save · HTML · PDF · DOCX · EPUB · Settings · Licenses · language · Source/Hybrid · layout |
| Split view | Drag the center handle; double-click resets 50%; ratio stored in `localStorage` |
| Status bar | Dirty state, encoding, line endings, char count, preview timing, busy indicator |
| Settings | Language, trusted workspaces, Pandoc path, wiki-links, custom CSS path, crash report, update endpoint stub |

---

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New document |
| `Ctrl+O` | Open |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+1` / `2` / `3` | Source / Split / Preview layout |
| `Ctrl+,` | Settings |

Monaco’s own chords (find, multi-cursor, etc.) work in source mode.

---

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│  React webview (Tauri 2)                                    │
│  Monaco · Milkdown · Preview pane · Export / Settings UI    │
│  i18n (en / zh-CN / zh-TW / ja)                             │
└───────────────────────────┬─────────────────────────────────┘
                            │  invoke only (no webview FS)
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  Rust core                                                  │
│  sessions · atomic save · recovery · path auth              │
│  file dialogs · export prefs · sidecar manager (warm)       │
│  lmlocal:// assets · CLI / file-association open            │
└───────────────────────────┬─────────────────────────────────┘
                            │  stdin/stdout JSON Lines
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  Node render sidecar + crossnote                            │
│  parseMD (preview) · HTML export · PDF (headless browser)   │
└─────────────────────────────────────────────────────────────┘
```

### Design principles

1. **Markdown is the source of truth** — disk format is always `.md` (or related Markdown extensions).  
2. **In-memory preview** — the sidecar renders the editor buffer; it does not write the source file for preview.  
3. **Defense in depth** — Rust path checks + safe crossnote defaults + DOMPurify in the UI.  
4. **Optional tools never block editing** — missing Pandoc/Graphviz only disables related actions.  
5. **Fast open path** — render sidecar is **warmed at startup** so double-click open is not blocked by a cold Node/crossnote load; first preview after open skips the typing debounce.

More detail: [docs/architecture.md](docs/architecture.md) · [docs/security.md](docs/security.md) · [docs/adr/](docs/adr/)

---

## Feature map (milestones)

| Milestone | Status | Scope |
|-----------|--------|--------|
| **M0** | Done | Tauri shell, sidecar protocol, crossnote spike |
| **M1** | Done | Open/save, tabs, atomic write, recovery, recent files |
| **M2** | Done | Monaco, live preview, TOC, scroll sync, large-file degrade |
| **M3** | Done | HTML/PDF export, NSIS installer, file associations |
| **M4** | Done | Milkdown hybrid + hybridRoundtrip guards |
| **M5** | Done | Pandoc, trusted workspaces, settings |
| **M6** | Done | Privacy docs, diagnostics, a11y basics, release polish |
| **Post-M6** | Ongoing | i18n, resizable split, preview warm path (v2.1), icon branding |

---

## Internationalization

Default language is **English**. Preference is stored in `localStorage` (`litemark.locale`) and applied immediately.

| Code | Language |
|------|----------|
| `en` | English (default) |
| `zh-CN` | 简体中文 |
| `zh-TW` | 繁體中文 |
| `ja` | 日本語 |

UI catalogs live under `packages/app-ui/src/i18n/`. Missing keys fall back to English.

---

## Export formats

| Format | Engine | Notes |
|--------|--------|--------|
| **HTML** | crossnote | Optional offline package (inline CSS/fonts/images) |
| **PDF** | crossnote + Edge/Chrome headless | Page size, margins, landscape, header/footer |
| **DOCX / EPUB / LaTeX** | Pandoc (optional) | Detected in Settings; path override supported |

Export never modifies the source Markdown. Default save names use the document stem (e.g. `notes.md` → `notes.pdf`).

---

## Performance (preview)

Measured on a real ~25 KB technical plan with many code fences (no Mermaid/math):

| Stage | Typical |
|-------|---------|
| Sidecar cold start (Node + crossnote) | ~1.5–1.8 s (mitigated by **startup warm**) |
| First `parseMD` after warm | ~150–170 ms |
| Subsequent `parseMD` | ~40–50 ms |
| Typing debounce | 250 ms (0 ms on first open / tab switch) |

Bench scripts (developers):

```powershell
node scripts/bench-preview-ipc.mjs path\to\file.md
```

---

## Repository layout

```text
LiteMark/
├── packages/
│   ├── app-ui/              # React + Vite frontend (Monaco, Milkdown, i18n)
│   ├── render-sidecar/      # Node IPC + crossnote adapter
│   ├── shared-protocol/     # Shared types & IPC contracts
│   └── markdown-core/       # Hybrid roundtrip / normalize
├── src-tauri/               # Tauri 2 + Rust core
│   ├── icons/               # App icons (generated)
│   └── resources/sidecar/   # Bundled sidecar dist for release
├── assets/brand/            # Branding / master icon
├── docs/                    # Architecture, security, privacy, ADRs
├── scripts/                 # Build helpers, icon, benches
└── LiteMark_DEVELOPMENT_PLAN.md
```

---

## Quick start (developers)

### Requirements

- Windows 10/11 **x64**  
- [Rust](https://rustup.rs/) stable (`x86_64-pc-windows-msvc`) + VS Build Tools (C++)  
- [Node.js](https://nodejs.org/) **22+** and [pnpm](https://pnpm.io/) **11**  
- WebView2 Runtime (usually preinstalled)  
- Optional: [Pandoc](https://pandoc.org/), Edge/Chrome for PDF  

### Install & run

```powershell
corepack enable
pnpm install
pnpm --filter @litemark/render-sidecar build
pnpm dev
```

### Build installer

```powershell
pnpm tauri build
# → src-tauri\target\release\bundle\nsis\LiteMark_2.1.0_x64-setup.exe
```

### Tests & checks

```powershell
pnpm typecheck
pnpm test
pnpm test:rust
```

### Regenerate app icons

Provide a **square** PNG (preferably 1024×1024), then:

```powershell
pnpm tauri icon path\to\master.png -o src-tauri\icons
```

Brand masters are also kept under `assets/brand/`.

---

## Security model (short)

- Webview has **no** direct `fs` / shell access.  
- Sidecar config forces `enableScriptExecution: false`, no `file://` protocol whitelist for untrusted content.  
- Preview HTML is sanitized with **DOMPurify** before injection.  
- Local images use the authorized `lmlocal://` scheme after path checks under the document directory.  
- **Trusted workspaces** gate experimental features; new/downloaded docs are untrusted by default.  

See [docs/security.md](docs/security.md) and [docs/privacy.md](docs/privacy.md).

---

## Configuration & data locations

| Item | Location / key |
|------|----------------|
| Install (typical) | `%LOCALAPPDATA%\LiteMark\` |
| UI language | `localStorage` → `litemark.locale` |
| Split ratio | `localStorage` → `litemark.splitRatio` |
| App settings / trusted paths | App data JSON (via Rust settings module) |
| Recovery snapshots | Managed by Rust recovery store |

---

## Changelog (v2.1.0)

- New application icon and branding assets  
- UI languages: English, Simplified Chinese, Traditional Chinese, Japanese  
- Resizable source/preview split  
- Preview performance: startup sidecar warm + zero debounce on first open  
- File association / CLI open reliability improvements  
- PDF/DOCX export default names from document stem  
- CREATE_NO_WINDOW for sidecar (no console flash on Windows)  

---

## Documentation

| Doc | Topic |
|-----|--------|
| [docs/user/getting-started.md](docs/user/getting-started.md) | End-user getting started |
| [docs/architecture.md](docs/architecture.md) | System architecture |
| [docs/security.md](docs/security.md) | Security model |
| [docs/privacy.md](docs/privacy.md) | Privacy / telemetry policy |
| [docs/performance.md](docs/performance.md) | Performance notes |
| [LiteMark_DEVELOPMENT_PLAN.md](LiteMark_DEVELOPMENT_PLAN.md) | Milestone plan |
| [docs/adr/](docs/adr/) | Architecture decision records |
| [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) | Third-party licenses |

---

## Contributing

Issues and PRs are welcome. Please:

- Keep milestone scope and **security defaults** (untrusted Markdown, no silent FS from the webview).  
- Prefer small, reviewable changes with a clear motivation.  
- Run `pnpm typecheck` and relevant tests before opening a PR.  

---

## License

**MIT** — see [LICENSE](LICENSE).  
Third-party notices: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

---

<p align="center">
  Made for people who want Markdown that stays on their machine.
</p>
