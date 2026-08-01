/**
 * Extract a structured table-of-contents from crossnote's rendered `tocHTML`.
 *
 * crossnote emits the TOC as an unordered list of anchor links, e.g.:
 *   <ul><li><a href="#heading-id">Heading</a> ... <ul>...</ul></li></ul>
 *
 * We walk it without a DOM library (the structure is simple and stable) to
 * keep the sidecar dependency-light. Each entry's nesting level is derived
 * from the count of `data-line` / `<ul>` nesting is complex; instead we infer
 * level from the anchor href / text heuristics only when present. For M0 the
 * goal is a non-empty, structured TOC; richer leveling arrives in M2.
 */

export interface TocEntry {
  level: number;
  text: string;
  id: string;
}

const ANCHOR_RE = /<a[^>]*href="#([^"]*)"[^>]*>([\s\S]*?)<\/a>/gi;

/**
 * Parse TOC anchors out of the rendered TOC HTML. Returns entries in document
 * order. `level` defaults to 1 when no nesting signal is available.
 */
export function extractToc(tocHTML: string): TocEntry[] {
  const entries: TocEntry[] = [];
  if (!tocHTML) return entries;
  let match: RegExpExecArray | null;
  ANCHOR_RE.lastIndex = 0;
  while ((match = ANCHOR_RE.exec(tocHTML)) !== null) {
    const id = decodeHtmlEntities(match[1]);
    const text = stripTags(decodeHtmlEntities(match[2])).trim();
    if (id) {
      entries.push({ level: inferLevel(id, match.index, tocHTML), text, id });
    }
  }
  return entries;
}

/** Strip HTML tags and collapse whitespace. */
function stripTags(html: string): string {
  return html.replace(/<[^>]*>/g, "").replace(/\s+/g, " ");
}

/** Decode the handful of HTML entities crossnote is known to emit in anchors. */
function decodeHtmlEntities(s: string): string {
  return s
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
}

/**
 * Infer a heading level. crossnote does not encode level in the anchor id, so
 * we approximate by counting how many `<ul>` opens precede the anchor (each
 * nested ul = +1 level). The first/top-level list is level 1.
 */
function inferLevel(id: string, anchorOffset: number, fullHTML: string): number {
  void id;
  const before = fullHTML.slice(0, anchorOffset);
  const opens = (before.match(/<ul/gi) ?? []).length;
  const closes = (before.match(/<\/ul>/gi) ?? []).length;
  const depth = Math.max(1, opens - closes + 1);
  return depth;
}
