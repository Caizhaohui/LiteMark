# ADR 0002 — crossnote in-memory rendering

- **Status:** Accepted (spike-verified)
- **Date:** 2026-07-15
- **Milestone:** M0

## Context

DEVELOPMENT_PLAN.md §5.4 requires rendering **unsaved** editor content to HTML
without ever writing to (or reading from) the user's source file. The plan
called for a `renderMarkdownText({ markdown, logicalFilePath, notebookPath,
config })` entry point, with a fallback shadow-file approach allowed only for
spike validation.

crossnote 0.9.31 (verified locally) exposes:
- `Notebook.init({ notebookPath, config, fs })` → `Notebook`
- `notebook.getNoteMarkdownEngine(filePath)` → `MarkdownEngine` (filePath must
  be absolute; used only for relative-resource/import/link resolution)
- `engine.parseMD(inputString, opts)` → `{ html, markdown, tocHTML, yamlConfig,
  JSAndCssFiles }` — **`inputString` is a parameter**; when non-empty, the
  engine never reads the file from disk.
- `engine.generateHTMLTemplateForExport(html, yamlConfig, opts)` → full
  `<head>`-wrapped document string (offline inlining supported).

crossnote's own `htmlExport`/`chromeExport` are unsuitable directly: they read
from `engine.filePath` and write next to the source — both forbidden by the
LiteMark security model.

## Decision

Build a **thin public-API adapter** in `packages/render-sidecar/src/crossnote-adapter/`:

1. `renderMarkdownText()` → `Notebook.init` + `getNoteMarkdownEngine` +
   `engine.parseMD(markdownString, { isForPreview:true, useRelativeFilePath:false,
   hideFrontMatter:true })`. The in-memory markdown string is passed directly;
   the user's file is never touched. ✅ No fork required.
2. `exportHtml()` → `parseMD` + `generateHTMLTemplateForExport({offline:true,
   embedLocalImages:true})`, returning the string; the Rust core writes it to
   the caller-authorized path. Bypasses `htmlExport` entirely.
3. `exportPdf()` → replicate the export flow over the public helpers and drive
   `puppeteer-core` ourselves against the discovered browser. See "Browser
   launch" below.

**Safe defaults** (merged into every Notebook config, enforced regardless of
caller input — §8.1, §12.5):
- `enableScriptExecution: false` (disables code-chunk execution + JS/CSS imports)
- `enableHTML5Embed: false`
- `printBackground: true`
- `protocolsWhiteList: "http://, https://, mailto:, tel:"` (drops `file://`,
  `atom://`)
- `enableWikiLinkSyntax: false` (avoids touching the document dir)
- A fallback `notebookPath` pointing at the crossnote build dir (not the user's
  document dir), so no untrusted `.crossnote/parser.js` is loaded.

`logicalFilePath` is used only for relative-resource resolution; it does **not**
need to exist on disk because the markdown string is supplied explicitly.

## Spike verification (M0)

Test input: `testdata/markdown/sample.md` (YAML front matter, headings, bold/
italic, Mermaid graph, inline + block KaTeX, code block, task list, blockquote).

- **Render spike** — HTML length 6340 chars, **renderMs = 104 ms** (well under
  the 500 ms perception budget). Contains `<h1>`, Mermaid container, KaTeX
  markup. **No `<script>` tags** in output with scripts disabled. TOC extracted
  (5 entries, correct levels/ids).
- **PDF spike** — same input exported via system **Edge 143** to PDF:
  **164,542 bytes**, valid `%PDF-1.4` header, contains the rendered Mermaid
  diagram, KaTeX, and code highlighting.
- **Malicious-input check** (`testdata/malicious/script-injection.md`):
  `<script>` removed; `onerror=` removed; `<iframe>` retained but with
  `sandbox=""`; `javascript:` link rendered as inert literal text (not a
  clickable `href`). crossnote's preview-time sanitization is effective;
  defense-in-depth DOMPurify on the frontend remains required for M2 (§8.4).

### Browser launch finding

`puppeteer.launch()` against Edge 143 on this Windows environment fails
silently: the browser process exits with code 0 and empty stderr before the
DevTools handshake completes (both `pipe:true` and `pipe:false`). Root cause:
Edge's process-relaunch behavior under puppeteer's pipe/port management.

**Workaround** (implemented in `export-pdf.ts`): spawn Edge ourselves with
`--remote-debugging-port=<ephemeral>` + a fresh `--user-data-dir`, wait for the
`/json/version` endpoint, then `puppeteer.connect({ browserURL })`. This binds
to 127.0.0.1 only and uses an OS-assigned port (`--remote-debugging-port=0`
semantics via a pre-reserved listener) — no public port is opened. Verified
working; produces the 164 KB PDF above.

## Alternatives considered

- **Shadow file** (write unsaved markdown to an app-cache file, render that):
  rejected — the public-API adapter makes it unnecessary, and the plan forbids
  creating hidden preview files in the user's document dir.
- **Forking crossnote** to add a first-class `renderMarkdownText`: rejected;
  the thin adapter is sufficient and avoids a maintenance fork.
- **crossnote `chromeExport`**: rejected — reads/writes next to the source file
  and has no Edge detection or programmatic PDF options.

## Consequences

- ✅ Unsaved content renders in memory; the source file is never written.
- ✅ Relative-resource resolution works via `logicalFilePath`/`notebookPath`.
- ⚠️ Full wikilink/backlink indexing touches the filesystem (`refreshNotes`),
  so it is disabled by default (`enableWikiLinkSyntax:false`) until the trusted-
  workspace concept lands (M5, ADR 0007).
- ⚠️ The Edge launch-on-port workaround is environment-specific; revisit if a
  future puppeteer-core/Edge combination restores reliable `launch()`.
- 📌 Frontend must still sanitize inserted HTML with DOMPurify before display
  (M2, §8.4) — crossnote's sanitization is necessary but not sufficient for
  defense in depth.

## Status

Accepted and spike-verified for M0. The adapter is the single integration point
with crossnote; bumping crossnote requires re-running the render + PDF spikes.
