/**
 * Markdown normalization for roundtrip comparison (M4 §7.3).
 *
 * We deliberately do NOT pretty-print the whole document. Normalization only
 * collapses semantically equivalent surface forms so hybrid ↔ source checks
 * ignore incidental whitespace while still catching real content loss.
 */

/** Normalize line endings to LF and strip a single trailing BOM. */
export function normalizeLineEndings(input: string): string {
  return input.replace(/^\uFEFF/, "").replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

/**
 * Semantic normalize used before comparing source → hybrid → source results.
 * - LF endings
 * - Trim trailing whitespace per line
 * - Collapse 3+ blank lines to 2
 * - Ensure file ends with at most one trailing newline
 * - Do NOT reorder blocks or reformat tables
 */
export function normalizeMarkdown(input: string): string {
  let s = normalizeLineEndings(input);
  const lines = s.split("\n").map((line) => line.replace(/[ \t]+$/g, ""));
  s = lines.join("\n");
  s = s.replace(/\n{3,}/g, "\n\n");
  s = s.replace(/^\n+/, "");
  if (s.length > 0 && !s.endsWith("\n")) {
    s += "\n";
  }
  return s;
}

/**
 * Stricter byte-preserving check for raw_markdown_block payloads:
 * only line-ending normalize, keep all other characters.
 */
export function normalizeRawBlock(input: string): string {
  return normalizeLineEndings(input);
}
