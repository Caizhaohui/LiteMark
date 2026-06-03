# LiteMark Development

## Current State

LiteMark `v1.0.0` is the first stable CLI release. The shipped product is:

- A Rust CLI
- A local preview server
- A browser-based live preview
- HTML export
- PDF export through an installed Chromium-based browser

This is not yet a native desktop editor.

## Release Workflow

Release validation:

```bat
scripts\cargo-msvc.cmd check
scripts\cargo-msvc.cmd test
scripts\build-release.cmd
scripts\package-portable.cmd
```

Portable output:

```text
dist/
  LiteMark-v1.0.0-windows-x64-portable/
  LiteMark-v1.0.0-windows-x64-portable.zip
```

## v1.0.0 Scope

Included:

- Config-driven preview and export
- HTML export with optional image embedding
- PDF export with `--browser`, `--page-size`, `--print-background`
- Test coverage for parser, config, HTML export, and PDF failure paths

Not included:

- Native editor window
- File tree sidebar
- Menus and dialog integration
- Installer packaging

## Tauri Migration Plan

The next stage is a real desktop GUI based on Tauri. The migration rule is:

- Keep Markdown parsing and export logic in Rust
- Build the editor and shell as a separate GUI layer
- Avoid forking render logic between CLI and desktop

Proposed structure:

```text
LiteMark/
  src/          # current CLI + preview implementation
  desktop/      # web UI for the Tauri shell
  src-tauri/    # Tauri shell and commands
```

## Immediate Desktop Priorities

1. Create a minimal Tauri shell that can open the current preview workflow.
2. Expose Rust commands for open-file, export-html, export-pdf.
3. Add a desktop frontend with file picker, preview pane, and export actions.
4. Decide whether embedded webview preview replaces the external browser preview path.
