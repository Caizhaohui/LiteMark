# Performance baselines (M6)

Recorded on a typical Windows 11 x64 development machine. Numbers are guidance,
not contractual SLAs.

| Scenario | Target | Notes |
|----------|--------|--------|
| Cold start to interactive UI | &lt; 2 s | Excludes first-time WebView2 install |
| Preview render (normal doc &lt; 1 MiB) | &lt; 500 ms perceived | 250 ms debounce + render |
| Hybrid open (roundtrip check) | &lt; 200 ms | remark-based guard |
| 10 open documents idle | baseline TBD on release hardware | Measure RSS after 5 min idle |
| Open/close 100 docs | no unbounded growth | Session map + recovery cleanup |

## Large files

| Size | Behavior |
|------|----------|
| &lt; 1 MiB | Full live preview |
| 1–5 MiB | Reduced debounce (750 ms) |
| &gt; 5 MiB | Preview paused until manual request |

## Memory hygiene

- Closing a tab calls `closeSession` on the sidecar and drops the Rust session.
- PDF export kills the browser child and deletes temp dirs on cancel/complete.
- Recovery snapshots are capped by housekeeping in the recovery store.
