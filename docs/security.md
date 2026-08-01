# LiteMark Security Model

All Markdown is treated as **untrusted input** by default
(DEVELOPMENT_PLAN.md §0.9, §8.4).

## Trust boundaries

```
[ Markdown (untrusted) ]
      │
      ▼
React webview ──invoke──▶ Rust core ──JSON Lines──▶ Node sidecar ──▶ crossnote
   ▲                         │                         │
   │                         │                         └─ enableScriptExecution=false
   └─ DOMPurify before        │                            protocolsWhiteList: http(s),mailto,tel
      inserting HTML           │                            printBackground=true
                               └─ path authorization, timeouts, crash handling
```

## M0-enforced controls

### Sidecar / crossnote
- **Static method whitelist** — no generic `exec`/`shell`. Unknown methods are
  rejected with `PROTOCOL_INVALID` (see `packages/render-sidecar/src/index.ts`).
- **stdout = protocol, stderr = logs** — a framing invariant that prevents log
  injection from corrupting the protocol stream (§5.3).
- **Scripts always off** — `enableScriptExecution: false` is hard-enforced in
  `resolveConfig()`; caller-supplied `trusted`/`enableScriptExecution` are
  ignored. crossnote zeroes JS/CSS `@import` and code-chunk execution.
- **No `file://` in rendered output** — `protocolsWhiteList` drops `file://`
  and `atom://`; local images are served via the Rust-controlled `lmlocal://`
  custom protocol (M2), which only allows paths under an open document's
  parent directory.
- **No untrusted parser hooks** — `notebookPath` defaults to the crossnote
  build dir, not the user's document dir, so no `.crossnote/parser.js` from an
  untrusted location is loaded (§12.5).

### Rust core
- **No broad Tauri permissions** — `capabilities/default.json` grants only
  core window/event permissions. The webview has **no** `shell`/`fs`/`http`
  permission; all sidecar traffic **and all file access** is brokered by Rust.
- **Sidecar lifecycle** — `kill_on_drop`, crash detection → `SIDECAR_CRASHED`,
  per-request timeout (default 10s) → `SIDECAR_TIMEOUT`, automatic restart
  after crash.
- **Path authorization** — export output paths are validated by Rust before
  being forwarded (stub in M0; enforced in M1+).

### Document lifecycle (M1)
- **No `fs` or `dialog` permission for the webview.** Open/save file *dialogs*
  are shown by Rust via `rfd` (native), not the Tauri `dialog` plugin. The
  webview only ever receives a path string back; it cannot read or write any
  file directly. Every read/write goes through a Rust command
  (`open_file`, `save_document`, `save_as_document`, …).
- **Atomic save** — files are never truncate-then-written; a temp file in the
  same directory is fsynced and renamed over the target (§6.2, ADR 0004). A
  failed save leaves the original untouched.
- **Encoding safety** — only UTF-8 / UTF-8-BOM is accepted; any other encoding
  returns `FILE_ENCODING_UNSUPPORTED` instead of lossy transcoding.
- **No silent overwrite of external changes** — on-disk mtime is recorded at
  open/save; a later mismatch prompts Reload/Keep/Compare before any write
  (ADR 0005).

### Preview (M2)
- **DOMPurify before DOM insert** — sidecar HTML is sanitized again in the
  webview (`sanitizePreviewHtml`). Scripts, event handlers, iframes, and
  `javascript:` URLs are stripped.
- **Links never navigate the webview** — click handlers call
  `open_external_url` (http/https/mailto/tel only) or scroll to in-page
  anchors. `file://` and other schemes are refused.
- **Local assets authorized** — `resolve_document_asset` + `lmlocal://` only
  serve files under the document directory; `..` escapes are rejected.

### Export (M3)
- **Output path is user-chosen only** via the native save dialog, then
  re-validated by Rust (`authorize_export_path`): absolute path, allowed
  extension, never overwrites the source Markdown.
- **Sidecar never invents paths** — it only writes the authorized `outputPath`.
- **Cancel cleans browser processes** — PDF cancel kills the Edge/Chrome child
  and removes temp dirs so no zombie Chromium remains.
- **No bundled Chromium download** — PDF uses system Edge/Chrome; missing
  browser → `BROWSER_NOT_FOUND` with a clear UI message.
- **Recovery snapshots** live under the app's own `%LOCALAPPDATA%` directory,
  never in the user's document tree; they are best-effort and never fatal.

### Frontend (M2+)
- DOMPurify sanitization before any HTML insertion.
- No `dangerouslySetInnerHTML` with unsanitized strings.
- Links opened via Rust → system browser (never in-webview navigation).

## Verified at M0

- Render spike output contains **no `<script>` tags**.
- Malicious-input sample: `<script>`/`onerror=` stripped; `<iframe>` sandboxed;
  `javascript:` link rendered as inert text.
- PDF export writes only to the caller-authorized path; temp files cleaned up.

## Verified at M1

- `capabilities/default.json` still grants **no** `fs`/`dialog` permission; a
  capability-audit confirms the webview cannot touch the filesystem directly.
- Atomic-save unit tests assert a failed save leaves the original intact and
  leaves no temp-file residue.
- Non-UTF-8 byte sequences are rejected with `FILE_ENCODING_UNSUPPORTED`.
- The full app builds and bundles (`pnpm tauri build` → NSIS installer).

## Open items for later milestones

- Full DOMPurify defense-in-depth in the preview pane (M2).
- Authorized-path enforcement for exports (M3).
- Trusted-workspace opt-in for script execution (M5, ADR 0007).
