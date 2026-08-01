/**
 * Detect constructs that hybrid mode should preserve as raw source cards
 * rather than structured ProseMirror nodes (M4 §7.2).
 *
 * Policy: known GFM/common structures go through the hybrid editor; fenced
 * blocks with special languages (mermaid, plantuml, graphviz, …), MPE
 * directives, and raw HTML blocks stay as raw_markdown_block so bytes are
 * never silently rewritten.
 */

export interface RawRegion {
  /** Inclusive start offset in the original string. */
  start: number;
  /** Exclusive end offset. */
  end: number;
  kind: "fence" | "html" | "directive" | "front_matter";
  language?: string;
  text: string;
}

const SPECIAL_FENCE_LANGS = new Set([
  "mermaid",
  "plantuml",
  "puml",
  "graphviz",
  "dot",
  "viz",
  "wavedrom",
  "vega",
  "vega-lite",
  "echarts",
  "flow",
  "sequence",
  "abc",
  "tikz",
  "math",
  "latex",
  "katex",
  "js",
  "javascript",
  "ts",
  "typescript", // code stays in hybrid as code_block; special ones listed above for NodeView
]);

/** Fences that hybrid keeps as structured code_block (editable) not raw. */
const STRUCTURED_CODE_LANGS = new Set([
  "",
  "text",
  "plain",
  "md",
  "markdown",
  "json",
  "yaml",
  "yml",
  "toml",
  "xml",
  "html",
  "css",
  "scss",
  "bash",
  "sh",
  "shell",
  "powershell",
  "ps1",
  "python",
  "py",
  "rust",
  "rs",
  "go",
  "java",
  "c",
  "cpp",
  "csharp",
  "cs",
  "js",
  "javascript",
  "ts",
  "typescript",
  "sql",
  "ruby",
  "rb",
  "php",
  "swift",
  "kotlin",
  "mermaid", // mermaid: hybrid shows as special card but content preserved
]);

const FENCE_RE =
  /^([ \t]{0,3})(`{3,}|~{3,})([^\n`]*)\n([\s\S]*?)^\1\2[ \t]*$/gm;
const HTML_BLOCK_RE = new RegExp(
  "^[ \\t]{0,3}<(?:script|style|pre|div|iframe|object|embed|table|details|section|article)(?:\\s|>)[\\s\\S]*?<\\/[a-zA-Z]+>[ \\t]*$",
  "gim",
);
const DIRECTIVE_RE = /^[ \t]{0,3}@import\b.*$/gm;
const FRONT_MATTER_RE = /^---\r?\n[\s\S]*?\r?\n---[ \t]*\r?\n/;

/**
 * Find regions that must survive hybrid mode as raw source.
 * For M4, mermaid/math fences are treated as "preserve language" code blocks
 * (structured), while HTML blocks and @import directives become raw cards.
 */
export function findRawRegions(markdown: string): RawRegion[] {
  const regions: RawRegion[] = [];

  const fm = FRONT_MATTER_RE.exec(markdown);
  if (fm && fm.index === 0) {
    regions.push({
      start: 0,
      end: fm[0].length,
      kind: "front_matter",
      text: fm[0],
    });
  }

  FENCE_RE.lastIndex = 0;
  let m: RegExpExecArray | null;
  while ((m = FENCE_RE.exec(markdown)) !== null) {
    const lang = (m[3] ?? "").trim().split(/\s+/)[0]?.toLowerCase() ?? "";
    // Only mark truly exotic fences as raw; standard + mermaid stay structured.
    if (lang && !STRUCTURED_CODE_LANGS.has(lang) && SPECIAL_FENCE_LANGS.has(lang)) {
      regions.push({
        start: m.index,
        end: m.index + m[0].length,
        kind: "fence",
        language: lang,
        text: m[0],
      });
    } else if (lang && !STRUCTURED_CODE_LANGS.has(lang)) {
      // Unknown language fence → raw to avoid lossy rewrite.
      regions.push({
        start: m.index,
        end: m.index + m[0].length,
        kind: "fence",
        language: lang,
        text: m[0],
      });
    }
  }

  HTML_BLOCK_RE.lastIndex = 0;
  while ((m = HTML_BLOCK_RE.exec(markdown)) !== null) {
    regions.push({
      start: m.index,
      end: m.index + m[0].length,
      kind: "html",
      text: m[0],
    });
  }

  DIRECTIVE_RE.lastIndex = 0;
  while ((m = DIRECTIVE_RE.exec(markdown)) !== null) {
    regions.push({
      start: m.index,
      end: m.index + m[0].length,
      kind: "directive",
      text: m[0],
    });
  }

  regions.sort((a, b) => a.start - b.start);
  return regions;
}

/**
 * Protect raw regions with placeholders so a hybrid serializer cannot rewrite
 * their bytes. Returns protected markdown + restore map.
 */
export function protectRawRegions(markdown: string): {
  protectedMarkdown: string;
  slots: string[];
} {
  const regions = findRawRegions(markdown);
  if (regions.length === 0) {
    return { protectedMarkdown: markdown, slots: [] };
  }
  const slots: string[] = [];
  let out = "";
  let cursor = 0;
  for (const r of regions) {
    if (r.start < cursor) continue; // overlap skip
    out += markdown.slice(cursor, r.start);
    const idx = slots.length;
    slots.push(r.text);
    out += `\n\n\`\`\`litemark-raw-${idx}\n${r.text}\n\`\`\`\n\n`;
    cursor = r.end;
  }
  out += markdown.slice(cursor);
  return { protectedMarkdown: out, slots };
}

/** Restore placeholders after hybrid serialize. */
export function restoreRawRegions(markdown: string, slots: string[]): string {
  let out = markdown;
  for (let i = 0; i < slots.length; i++) {
    const re = new RegExp(
      "```litemark-raw-" + i + "\\n[\\s\\S]*?\\n```",
      "g",
    );
    out = out.replace(re, () => slots[i]);
  }
  return out;
}
