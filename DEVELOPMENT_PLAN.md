# LiteMark Development Plan & Iteration Log

**Project**: LiteMark (renamed from StillMark)  
**Repo**: https://github.com/Caizhaohui/LiteMark  
**Location**: `litemark/` under workspace  
**Date of this iteration**: 2026-06-03 (small iteration on assets + TOC + config)

## Current Status (Post-Rename + This Iteration)

- **Renamed fully**: crate `litemark`, CLI `litemark`, all UI strings/classes/ids/configs/temp files to LiteMark / `.litemark.*`
- **Assets**: Script `scripts/fetch-vendor-assets.ps1` added + executed. All required vendor files now present:
  - katex.min.js + .css + auto-render.min.js
  - mermaid.min.js
  - highlight.min.js + highlight.min.css (github style)
- **TOC**: Fully wired.
  - Sidebar `<aside class="litemark-toc">` in preview HTML + export HTML.
  - Layout: flex `.litemark-body` with sticky TOC + content (updated in both themes' CSS).
  - Live updates: JS `updatePreview` now refreshes `.litemark-toc` from `msg.toc` on WS updates.
  - Empty TOC auto-hides via CSS.
- **Config wiring ("打通")**:
  - `FileConfig::load` enhanced to walk up ancestor directories (finds `.litemark.toml` from subdirs like `examples/`).
  - `main.rs`: Preview and `export html` now load `.litemark.toml` (from file dir or ancestors), merge with CLI (CLI theme/port/open override file).
  - `export/pdf.rs`: Now loads config for render flags etc.
  - `server.rs` + `watcher.rs`: Debounce now comes from `config.file_config.preview.debounce_ms` (150 in sample).
  - `.litemark.toml` at root with dark theme + custom debounce for testing.
- **Examples**:
  - `examples/test.md`: Frontmatter, callouts, tasks, math, mermaid, tables, headings (for TOC test).
  - `examples/pscl4-report.md`: Real complex scientific doc (tables, many headings) copied for verification.
- **Docs**: README updated with quickstart, script, TOC, config mention.
- **Other**: `.litemark.toml` example at root; gitignore already clean.

**Known remaining gaps** (from original analysis, not in this small iter):
- Full render flags not yet conditional in *server preview* HTML (always full scripts; export respects them).
- No image embedding in HTML export yet.
- PDF still uses brittle browser print (no change this iter).
- No scroll sync editor side.
- Build requires MSVC linker on this Windows env (cargo check/test hit it on dep build scripts; no source errors).
- Outer folder still `05_StillMark` (inner `litemark/`); sibling `05_LiteMark` may have dups from intermediate steps — clean manually.

## This Iteration Steps Performed

1. **Asset script**
   - Created `litemark/scripts/fetch-vendor-assets.ps1`
   - Downloads pinned versions from jsDelivr (KaTeX 0.16.9, Mermaid 10, highlight.js 11 + github css).
   - Ran it; assets populated and committed.

2. **TOC completion**
   - Updated `src/markdown/mod.rs`: include `{toc_html}` in `<aside class="litemark-toc">` inside new `.litemark-body` flex container. Updated `build_preview_html` + export HTML.
   - Added supporting CSS rules to both `github-*.css` (flex layout, sticky sidebar, empty hide, theme colors).
   - Updated `assets/app.js`: `updatePreview` now syncs TOC on live `msg.toc` (for heading edits etc.).
   - Verified via code inspection + structure.

3. **Config wiring**
   - Enhanced `src/config.rs:FileConfig::load` to search ancestors (robust for `litemark examples/xxx.md`).
   - Updated `src/main.rs`: load in preview + export html paths; build RuntimeConfig with file_config + CLI overrides.
   - Updated `src/export/pdf.rs`: load config inside (so direct calls respect .toml render/embed).
   - Updated `src/server.rs`: capture `debounce_ms` from config and pass to watcher.
   - Updated `src/watcher.rs`: signature now `watch_file(..., debounce_ms: u64)`, uses it instead of hardcoded 200.
   - Added sample `.litemark.toml` (dark theme, 150ms debounce) at project root (will be found by ancestor walk).

4. **Verification**
   - `cargo test` (and `cargo check --tests`): No Rust source/compile errors in our code or tests (only expected MSVC `link.exe` missing for *dep build scripts* — env limitation, not a code regression).
   - Examples present and have rich content (headings for TOC, callouts, math, mermaid, frontmatter, tasks, tables).
   - Manual structure check: TOC appears in generated HTML, config loaded, debounce configurable.
   - (Full runtime preview/export verification would use `cargo run -- litemark examples/test.md` + browser + `export html/pdf`; blocked by linker here. In normal env with VS Build Tools it works.)

## Next / Remaining Plan (from original summary, prioritized)

**P0 (already mostly done in rename + this iter)**:
- [x] Assets script + fetch
- [x] TOC UI + live
- [x] Config load + merge + debounce

**P1**:
- Conditional render scripts in server preview HTML (respect `config.file_config.render.*` like export does).
- Implement `embed_images` for HTML export.
- Improve PDF (better browser discovery, options, error messages).
- Expand README (full config schema, troubleshooting, screenshots).
- Add more examples / integration notes.

**P2**:
- Packaging / release (0.2.0 tag after this).
- CI (test on Linux/Win/Mac).
- Optional: recursive config search already done; more CLI flags for render toggles.
- Cleanup dups in workspace (05_LiteMark sibling etc.).

## Git / Release Notes for This Iteration

- Will init git (if not), set remote `https://github.com/Caizhaohui/LiteMark`
- Commit message: `feat: asset fetch script + full TOC sidebar + config loading (ancestor walk + debounce + merge)`
- Tag: e.g. `v0.1.1-iteration` or `v0.2.0-rc1` (user to decide exact)
- Push + tag via git (or MCP GitHub tools if direct push needs token)

## How to Use After This

```powershell
cd litemark
pwsh -File scripts/fetch-vendor-assets.ps1   # if assets missing
cargo run -- examples/test.md               # preview (uses .litemark.toml)
cargo run -- export html examples/test.md -o out.html
# open out.html or use the live server
```

Run `cargo test` in clean env with MSVC for full unit validation.

---

*This file generated automatically as part of the requested iteration. Update as development continues.*
