/**
 * Defense-in-depth HTML sanitization for the preview pane (§8.4).
 *
 * Crossnote already sanitizes server-side, but the frontend MUST still run
 * DOMPurify before inserting HTML into the DOM. Never pass raw sidecar HTML
 * to `dangerouslySetInnerHTML`.
 */

import DOMPurify from "dompurify";

/** Schemes allowed on href/src after sanitization. */
const ALLOWED_URI_REGEXP =
  /^(?:(?:https?|mailto|tel|lmlocal):|[^a-z]|[a-z+.\-]+(?:[^a-z+.\-:]|$))/i;

/**
 * Sanitize rendered Markdown HTML for safe insertion into the preview.
 * Keeps SVG (Mermaid) and math markup; strips scripts, event handlers,
 * iframes, and javascript: URLs.
 */
export function sanitizePreviewHtml(dirty: string): string {
  return DOMPurify.sanitize(dirty, {
    USE_PROFILES: { html: true, svg: true, svgFilters: true },
    // Keep common Markdown/math/code classes and ids for themes + TOC anchors.
    ADD_ATTR: ["target", "class", "id", "style", "viewBox", "xmlns", "fill", "stroke"],
    ADD_TAGS: [
      "foreignObject",
      // KaTeX uses these annotation elements.
      "annotation",
      "semantics",
      "math",
      "mrow",
      "mi",
      "mo",
      "mn",
      "msup",
      "msub",
      "mfrac",
      "msqrt",
      "mtext",
    ],
    FORBID_TAGS: ["script", "iframe", "object", "embed", "form", "input", "button", "base", "meta", "link"],
    FORBID_ATTR: [
      "onerror",
      "onload",
      "onclick",
      "onmouseover",
      "onfocus",
      "onblur",
      "onsubmit",
      "formaction",
      "xlink:href",
    ],
    ALLOW_UNKNOWN_PROTOCOLS: false,
    ALLOWED_URI_REGEXP,
    // Never keep HTML comments that could hide scripts.
    ALLOW_DATA_ATTR: false,
  });
}

// Hook: force-rel noreferrer on external anchors; block javascript: leftovers.
DOMPurify.addHook("afterSanitizeAttributes", (node) => {
  if (node instanceof HTMLElement) {
    if (node.tagName === "A") {
      const href = node.getAttribute("href") ?? "";
      if (/^\s*javascript:/i.test(href)) {
        node.removeAttribute("href");
      } else if (/^https?:/i.test(href) || /^mailto:/i.test(href) || /^tel:/i.test(href)) {
        node.setAttribute("rel", "noopener noreferrer");
        // Leave target unset; we intercept clicks and open via Rust.
      }
    }
    // Strip any residual event-handler attributes DOMPurify might leave on SVG.
    for (const attr of Array.from(node.attributes)) {
      if (attr.name.toLowerCase().startsWith("on")) {
        node.removeAttribute(attr.name);
      }
    }
  }
});
