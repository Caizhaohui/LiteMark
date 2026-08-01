# LiteMark User Guide (quick start)

## Install

1. Run the NSIS installer `LiteMark_x.y.z_x64-setup.exe`.
2. Optionally associate `.md` files during install (file associations are registered by the installer).
3. Launch **LiteMark** from the Start menu.

## Open and edit

- **Open**: toolbar **Open**, or `Ctrl+O`, or drag a `.md` file onto the window.
- **New**: **＋** or `Ctrl+N`.
- **Save / Save As**: `Ctrl+S` / `Ctrl+Shift+S`.
- **Source mode**: Monaco editor with full Markdown control.
- **Hybrid mode**: structured editing (Milkdown). LiteMark blocks the switch if it would rewrite your Markdown unsafely.
- **Layout**: Source / Split / Preview (`Ctrl+1` / `2` / `3`).

## Preview

The right-hand preview uses crossnote (Mermaid, KaTeX, GFM). Unsaved content is rendered from memory — your file is not overwritten for preview.

## Export

| Button | Output | Notes |
|--------|--------|--------|
| HTML | `.html` | Offline package option inlines CSS/images |
| PDF | `.pdf` | Needs Microsoft Edge or Chrome |
| DOCX / EPUB | via Pandoc | Install [Pandoc](https://pandoc.org/) or set path in Settings |

## Settings (`Ctrl+,`)

- **Trusted workspaces**: folders you explicitly trust for advanced features
- **Pandoc / Graphviz / PlantUML** probe status (optional tools)
- **Wiki links**, custom CSS path
- **Crash report** export
- **Update endpoint** (empty = updates disabled)

## Privacy

See [docs/privacy.md](../privacy.md). No telemetry.
