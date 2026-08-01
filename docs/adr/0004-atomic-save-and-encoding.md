# ADR 0004 — Atomic save, encoding, and line-ending preservation

- **Status:** Accepted
- **Date:** 2026-07-29
- **Milestone:** M1

## Context

M1 must reliably save Markdown files and must never corrupt them, even if the
process or machine dies mid-write (acceptance: "文件保存使用原子替换" and
"打开 UTF-8、UTF-8 BOM、LF、CRLF 文件后保存不发生意外编码变化"). A naive
`fs::write` truncates the target first and then writes; a crash between those
steps leaves a half-empty file and lost work.

Additionally, files may be UTF-8 or UTF-8-with-BOM, with LF or CRLF endings.
Round-tripping must preserve whichever combination was originally present — a
silent change (e.g. always writing LF, or always stripping the BOM) would
produce noisy diffs and could confuse other tools.

## Decision

**D1 — Atomic save via temp-file + rename (§6.2).** `files::atomic_save`
writes the full content to a unique temp file *in the same directory* as the
target, flushes, `sync_all`s (best-effort), preserves the existing file's
permissions, then atomically renames temp → target. On any failure it deletes
the temp file and leaves the target byte-for-byte untouched. The directory is
never truncate-then-written. Same-directory placement guarantees the rename is
atomic on a single volume.

**D2 — Encoding limited to UTF-8 / UTF-8-with-BOM; everything else rejected.**
`files::encoding::sniff_encoding` recognizes the BOM and validates the
payload as UTF-8. Non-UTF-8 bytes return `FILE_ENCODING_UNSUPPORTED` rather
than being lossily transcoded (transcoding GBK/Shift-JIS/Latin-1 silently
would risk data loss on round-trip and violate "no lossy auto-fix"). This is
deliberately conservative; users with legacy encodings must convert externally.

**D3 — Line endings detected and preserved.** On read, `detect_line_ending`
classifies the file as CRLF if any `\r\n` is present, else LF, and the content
is normalized to `\n` in memory. On write, `apply_line_ending` restores the
detected style. A file that mixed endings is normalized to CRLF on write (the
presence of any CRLF wins) — this matches "preserve, don't silently *change*
style" while still guaranteeing a single consistent ending.

**D4 — Long-path safety.** `paths::normalize_long_path` applies the `\\?\`
verbatim prefix to absolute Windows paths longer than 248 chars, so saves and
reads to deep paths work without per-machine `LongPathsEnabled` registry. The
prefix is only added when genuinely needed (short paths keep working with the
normal Win32 path layer).

## Consequences

- ✅ A crash during save never truncates the user's file (the worst case is a
  leftover `.litemark-tmp-*` file, which `atomic_save` cleans up).
- ✅ Encoding and line-ending style survive open→edit→save.
- ⚠️ Non-UTF-8 files cannot be opened (by design). This is surfaced as a clear
  `FILE_ENCODING_UNSUPPORTED` error, not a silent mangle.

## Alternatives considered

- **`fs::write` directly.** Rejected: truncates-then-writes; not crash-safe.
- **Auto-detect and transcode legacy encodings.** Rejected: lossy, surprising.
- **Persist the raw bytes and only re-encode if the user changed encoding.**
  Considered for M2; M1 keeps the model simple (always re-encode from the
  normalized in-memory string with the detected style).
