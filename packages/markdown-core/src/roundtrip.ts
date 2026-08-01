/**
 * Hybrid mode roundtrip guard (M4 §7.3).
 *
 * source → (protect raw) → hybrid load → serialize → restore raw → normalize
 * compare. If normalized forms differ, mode switch is blocked.
 */

import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkGfm from "remark-gfm";
import remarkStringify from "remark-stringify";
import { normalizeMarkdown } from "./normalize.js";
import { protectRawRegions, restoreRawRegions } from "./raw-blocks.js";

export interface RoundtripResult {
  ok: boolean;
  /** Serialized markdown after hybrid path (when ok, safe to use). */
  serialized: string;
  /** Human-readable reasons when not ok. */
  risks: string[];
  /** Unsupported/raw regions detected before hybrid parse. */
  rawCount: number;
}

const processor = unified()
  .use(remarkParse)
  .use(remarkGfm)
  .use(remarkStringify, {
    bullet: "-",
    fence: "`",
    fences: true,
    incrementListMarker: true,
    rule: "-",
    tightDefinitions: true,
  });

/**
 * Simulate the hybrid editor's parse → serialize cycle using remark.
 * Milkdown uses a compatible CommonMark/GFM subset; remark is our oracle for
 * whether structured editing would rewrite the document.
 */
export function hybridRoundtrip(markdown: string): RoundtripResult {
  const risks: string[] = [];
  const { protectedMarkdown, slots } = protectRawRegions(markdown);
  let serialized: string;
  try {
    const tree = processor.parse(protectedMarkdown);
    serialized = String(processor.stringify(tree));
  } catch (e) {
    return {
      ok: false,
      serialized: markdown,
      risks: [
        `Hybrid parser failed: ${e instanceof Error ? e.message : String(e)}`,
      ],
      rawCount: slots.length,
    };
  }

  serialized = restoreRawRegions(serialized, slots);
  const a = normalizeMarkdown(markdown);
  const b = normalizeMarkdown(serialized);

  if (a !== b) {
    // Provide a short diagnostic rather than a full diff dump.
    risks.push(
      "Hybrid mode would rewrite the Markdown in a way that is not byte-equivalent after normalization. Stay in source mode to avoid data loss.",
    );
    const aLines = a.split("\n");
    const bLines = b.split("\n");
    if (aLines.length !== bLines.length) {
      risks.push(
        `Line count differs after roundtrip (${aLines.length} → ${bLines.length}).`,
      );
    } else {
      for (let i = 0; i < aLines.length; i++) {
        if (aLines[i] !== bLines[i]) {
          risks.push(
            `First difference near line ${i + 1}:\n  was: ${aLines[i].slice(0, 120)}\n  now: ${bLines[i].slice(0, 120)}`,
          );
          break;
        }
      }
    }
    return { ok: false, serialized, risks, rawCount: slots.length };
  }

  return {
    ok: true,
    serialized: normalizeMarkdown(serialized),
    risks: [],
    rawCount: slots.length,
  };
}

/**
 * Serialize from hybrid editor output and check against last known source.
 * Used when leaving hybrid mode: always allowed (hybrid → source), but we still
 * normalize.
 */
export function serializeFromHybrid(markdownFromEditor: string): string {
  return normalizeMarkdown(markdownFromEditor);
}
