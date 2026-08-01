# LiteMark Architecture

This document summarizes the runtime architecture. See
`LiteMark_DEVELOPMENT_PLAN.md` §5 for the full design and `docs/adr/` for
decision records. M0 established the shell + sidecar skeleton; **M1 adds the
document-lifecycle layer** (open / save / recover) entirely brokered by Rust.

## Layers

```
┌─────────────────────────────────────────────────────────────┐
│ React UI (packages/app-ui)                                   │
│ M2–M6: AppShell / TabBar / EditorModeBar (source|hybrid) /   │
│ ViewModeBar (source|split|preview) / Monaco + Milkdown /     │
│ PreviewPane / Export / Settings / StatusBar / dialogs        │
├─────────────────────────────────────────────────────────────┤
│ Tauri Commands (src-tauri/src/commands)                      │
│ M0: ping_sidecar.                                            │
│ M1: documents / file_dialogs / recent / recovery.            │
│ M2: render_markdown / release_render_session /               │
│     open_external_url / resolve_document_asset.              │
│ M3: export_html / export_pdf / cancel_export /               │
│     probe_export_tools / get_third_party_notices.            │
│ The webview has NO fs/dialog permission — everything is       │
│ brokered here (see security.md).                             │
├─────────────────────────────────────────────────────────────┤
│ Rust Core (src-tauri)                                        │
│ session::SessionManager (in-memory DocumentSession map)      │
│ files::{atomic_save, encoding, paths, recent}                │
│ recovery::store (crash snapshots under %LOCALAPPDATA%)       │
│ assets::lmlocal protocol (authorized local preview images)   │
│ SidecarManager → Sidecar client (JSON Lines, timeout, crash) │
│ error.rs (ErrorCode ↔ shared-protocol §14)                   │
├─────────────────────────────────────────────────────────────┤
│ Node Render Sidecar (packages/render-sidecar)                │
│ stdin/stdout JSON Lines; stderr = logs only                  │
│ crossnote-adapter → render / exportHtml / exportPdf          │
├─────────────────────────────────────────────────────────────┤
│ crossnote 0.9.31 (+ puppeteer-core for PDF)                  │
└─────────────────────────────────────────────────────────────┘
```

## Live preview (M2)

```
Monaco (onChange)
  │ debounce 250 ms (750 ms if 1–5 MiB; paused if >5 MiB)
  │ revision++
  ▼
invoke render_markdown { sessionId, markdown, revision }
  ▼
Rust: createSession + sidecar render (in-memory; no disk write)
  ▼
DOMPurify.sanitize → PreviewPane.innerHTML
  │ relative img → resolve_document_asset → lmlocal://
  │ <a href> → open_external_url (OS) or in-page #anchor
  ▼
close_document → sidecar closeSession
```

## Document lifecycle (M1)

```
React (textarea)
  │ invoke set_document_content / save_document / …
  ▼
commands::documents ──▶ SessionManager ──▶ files::atomic_save  (atomic temp+rename)
                   └──▶ recovery::write_snapshot (per-edit, debounced)
                   └──▶ files::recent::record_opened
```

- `DocumentSession` (session/model.rs) is the single in-memory representation;
  its `dirty` flag is **derived from the content hash** on every read, never
  set by the UI (§6.1).
- **Atomic save** writes a same-directory temp file, fsyncs, preserves
  permissions, then renames temp→target (§6.2, ADR 0004). The target is never
  truncate-then-written.
- **Encoding/line-endings** are sniffed on read (UTF-8 / UTF-8-BOM; LF/CRLF)
  and restored on write. Non-UTF-8 → `FILE_ENCODING_UNSUPPORTED` (no lossy
  transcoding).
- **Recovery** snapshots every edit to `%LOCALAPPDATA%\LiteMark\LiteMark\
  recovery\`; offered on next launch (§6.3, ADR 0005).
- **External changes** detected by mtime on window-focus →
  ExternalChangePrompt (Reload / Keep mine / Compare). Never silent overwrite.
- **Single instance**: a second launch forwards its file-path args to the
  running instance via `tauri-plugin-single-instance` + an `open-files` event.
- **Native dialogs** via `rfd` (Rust), so no `dialog` permission is exposed to
  the webview.

## IPC protocol

Defined once in `packages/shared-protocol/src/index.ts` (the single source of
truth). Two protocols live there:

1. **Sidecar** (Rust ⇄ Node) JSON Lines:
   - Request: `{"id","method","params"}`
   - Success: `{"id","ok":true,"result"}`
   - Error: `{"id","ok":false,"error":{"code","message","details"}}`
   - Event: `{"event","payload"}` (e.g. `ready`, `exportProgress`)
   - Method set is a **static enum** (`SIDECAR_METHODS`); no `exec`/`shell`.

2. **Tauri commands** (webview ⇄ Rust), enumerated in `TAURI_COMMANDS`. M1
   adds the document/dialog/recent/recovery commands and their typed params
   and results (`DocumentSession`, `SessionSummary`, `RecoveryEntry`, …).

The Rust `ErrorCode` enum mirrors the shared error-code set (§14) and is
validated by a unit test.

## Build toolchain

See ADR 0001 (original GNU choice) and **ADR 0003 (reversal to MSVC)**. LiteMark
now builds with `x86_64-pc-windows-msvc`, Tauri's preferred Windows target. The
self-contained GNU toolchain broke (`dlltool` CreateProcess) and MSVC became
available; `scripts/setup-env.{sh,ps1}` now only sanity-check the MSVC linker
rather than emitting a machine-specific cargo config.

## Packages

| Package | Role |
|---|---|
| `@litemark/shared-protocol` | Sidecar JSON Lines protocol **+** M1 Tauri command contracts (TS) |
| `@litemark/app-ui` | React/Vite webview (Tauri frontend) |
| `@litemark/render-sidecar` | Node process wrapping crossnote |
| `src-tauri` | Rust/Tauri 2 application core (files, session, recovery, sidecar) |
