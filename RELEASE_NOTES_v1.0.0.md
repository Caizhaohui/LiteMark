# LiteMark v1.0.0

## Highlights

- First stable Windows CLI release
- Local Markdown preview with live refresh
- Self-contained HTML export
- PDF export through Edge, Chrome, or Chromium
- Config-driven render and export pipeline through `.litemark.toml`

## Included In This Release

- `litemark.exe`
- Example project config: `.litemark.toml.example`
- Portable package zip for Windows x64

## PDF Export

Supported options:

- `--browser`
- `--page-size`
- `--print-background`

## Validation

- `scripts\cargo-msvc.cmd check`
- `scripts\cargo-msvc.cmd test`
- `scripts\build-release.cmd`
- `scripts\package-portable.cmd`
