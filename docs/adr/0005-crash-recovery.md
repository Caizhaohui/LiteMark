# ADR 0005 — Crash recovery and external-modification handling

- **Status:** Accepted
- **Date:** 2026-07-29
- **Milestone:** M1

## Context

Two M1 acceptance criteria concern durability against surprise:

1. "应用异常退出后可恢复 dirty 内容" — after a crash/kill, unsaved edits must
   be recoverable on the next launch.
2. "外部修改时提示 Reload/Keep Mine/Compare Later，不静默覆盖" — if the file
   changes on disk while open, LiteMark must never silently overwrite it; it
   must ask the user.

Both are about not losing the user's work or the on-disk truth.

## Decision

**D1 — Recovery snapshots on disk (§6.3).** Every edit is followed (debounced
in the UI; eagerly in Rust) by `recovery::write_snapshot`, which atomically
writes a JSON snapshot to `%LOCALAPPDATA%\LiteMark\LiteMark\recovery\<key>.json`.
The file is keyed by `recovery_key` (a content-hash of the path, or `new-<uuid>`
for unsaved documents), so each document keeps exactly its newest snapshot.
On the next launch, `get_pending_recovery` returns them and the UI offers
Restore / Discard. A successful save deletes the snapshot; housekeeping caps
the directory at 50 files globally (oldest evicted).

**D2 — Snapshot shape is minimal and self-describing.** Each snapshot stores
`sessionId`, `originalPath`, `capturedAt` (ISO-8601), `revision`, `content`,
and `recoveryKey`. Corrupt snapshots are skipped on read (a single bad file
does not block restoring the others).

**D3 — Recovery is best-effort, never fatal.** Snapshot write failures are
logged and swallowed — a recovery-disk error must not crash the editor or fail
an edit. Recovery is a safety net, not a transactional guarantee.

**D4 — External change detection by mtime polling.** Each session records the
file's mtime (epoch ms) at open/save in `externalMtimeMs`. On window-focus, the
UI calls `check_external_change`, which compares the current on-disk mtime to
the recorded one. A mismatch surfaces `ExternalChangePrompt`
(Reload / Keep mine / Compare later). LiteMark never writes over a file that
changed underneath it without the user's explicit choice.

## Consequences

- ✅ A crash loses at most one debounce window (~400ms) of typing.
- ✅ On-disk external edits are never silently clobbered.
- ⚠️ mtime polling is not a guarantee under sub-millisecond edit races or
  filesystems with coarse mtime granularity; this is acceptable for M1 and
  matches typical editor behavior. Hash-based detection is a future option.

## Alternatives considered

- **In-memory-only recovery (write on close).** Rejected: a hard crash loses
  everything.
- **Hash-based external-change detection (compare full file hash on focus).**
  More accurate than mtime but requires re-reading the whole file on every
  focus event; deferred. mtime is the pragmatic M1 choice.
- **A dedicated recovery process / WAL.** Overkill for a single-user desktop
  editor; the per-edit snapshot is simpler and sufficient.
