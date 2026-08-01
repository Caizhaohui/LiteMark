/**
 * M2 Crossnote preview pane.
 *
 * Inserts DOMPurify-sanitized HTML only. Intercepts link clicks so navigation
 * never leaves the webview: external URLs go through Rust `open_external_url`,
 * in-document anchors scroll locally, local relative assets are rewritten to
 * the `lmlocal://` protocol.
 */

import { useEffect, useRef, type MouseEvent } from "react";
import type { Diagnostic, TocEntry } from "@litemark/shared-protocol";
import { useT } from "../i18n/I18nProvider";
import * as cmd from "../services/tauriCommands";

interface PreviewPaneProps {
  html: string;
  toc: TocEntry[];
  diagnostics: Diagnostic[];
  status: "idle" | "pending" | "ready" | "error" | "degraded";
  error: string | null;
  renderMs: number | null;
  sessionId: string | null;
  filePath: string | null;
  degraded: boolean;
  onRequestPreview: () => void;
  scrollRatio?: number | null;
  onScrollRatio?: (ratio: number) => void;
  /** Jump to a TOC id (from the outline). */
  scrollToId?: string | null;
  onTocNavigate?: (id: string) => void;
}

export function PreviewPane({
  html,
  toc,
  diagnostics,
  status,
  error,
  renderMs,
  sessionId,
  filePath,
  degraded,
  onRequestPreview,
  scrollRatio,
  onScrollRatio,
  scrollToId,
  onTocNavigate,
}: PreviewPaneProps): JSX.Element {
  const t = useT();
  const bodyRef = useRef<HTMLDivElement>(null);
  const applyingRemoteScroll = useRef(false);

  // Inject sanitized HTML and rewrite local asset URLs.
  useEffect(() => {
    const el = bodyRef.current;
    if (!el) return;
    el.innerHTML = html;
    void rewriteLocalAssets(el, sessionId, filePath);
  }, [html, sessionId, filePath]);

  // Scroll sync from editor.
  useEffect(() => {
    if (scrollRatio == null) return;
    const el = bodyRef.current;
    if (!el) return;
    const max = el.scrollHeight - el.clientHeight;
    if (max <= 0) return;
    applyingRemoteScroll.current = true;
    el.scrollTop = scrollRatio * max;
    requestAnimationFrame(() => {
      applyingRemoteScroll.current = false;
    });
  }, [scrollRatio]);

  // TOC jump.
  useEffect(() => {
    if (!scrollToId) return;
    const el = bodyRef.current;
    if (!el) return;
    const target =
      el.querySelector(`#${cssEscape(scrollToId)}`) ??
      el.querySelector(`[id="${scrollToId.replace(/"/g, "")}"]`);
    if (target instanceof HTMLElement) {
      target.scrollIntoView({ block: "start", behavior: "smooth" });
    }
  }, [scrollToId]);

  const handleScroll = () => {
    if (applyingRemoteScroll.current) return;
    const el = bodyRef.current;
    if (!el || !onScrollRatio) return;
    const max = el.scrollHeight - el.clientHeight;
    if (max <= 0) {
      onScrollRatio(0);
      return;
    }
    onScrollRatio(Math.min(1, Math.max(0, el.scrollTop / max)));
  };

  const handleClick = (e: MouseEvent) => {
    const target = e.target as HTMLElement | null;
    if (!target) return;
    const anchor = target.closest("a");
    if (!anchor) return;
    e.preventDefault();
    e.stopPropagation();
    const href = anchor.getAttribute("href") ?? "";
    if (!href) return;

    if (href.startsWith("#")) {
      const id = decodeURIComponent(href.slice(1));
      onTocNavigate?.(id);
      const el = bodyRef.current;
      const node = el?.querySelector(`#${cssEscape(id)}`);
      if (node instanceof HTMLElement) {
        node.scrollIntoView({ block: "start", behavior: "smooth" });
      }
      return;
    }

    if (/^https?:/i.test(href) || /^mailto:/i.test(href) || /^tel:/i.test(href)) {
      void cmd.openExternalUrl(href).catch(() => {
        /* non-fatal */
      });
      return;
    }

    // Relative / local file link: only open if we can resolve under the doc dir
    // — and even then we open via the OS, not by navigating the webview.
    if (sessionId && filePath) {
      void (async () => {
        try {
          const resolved = await cmd.resolveDocumentAsset(sessionId, href);
          // Opening a local file via OS is out of M2 scope for safety; show as
          // external open of file path only if scheme is http. For images we
          // already rewrite src. For .md links, ignore for now.
          void resolved;
        } catch {
          /* ignore unauthorized */
        }
      })();
    }
  };

  return (
    <div className="preview">
      <div className="preview__toolbar">
        <span className="preview__status">
          {status === "pending" && t("preview.rendering")}
          {status === "ready" && renderMs != null && `${renderMs} ms`}
          {status === "error" && t("preview.renderFailed")}
          {status === "degraded" && t("preview.largeFile")}
          {status === "idle" && t("preview.title")}
        </span>
        {degraded && (
          <button type="button" className="btn btn--small" onClick={onRequestPreview}>
            {t("preview.previewNow")}
          </button>
        )}
      </div>

      {status === "error" && error && (
        <div className="preview__error" role="alert">
          {error}
        </div>
      )}

      {status === "degraded" && !html && (
        <div className="preview__degraded">
          <p>{t("preview.largeFileBody1")}</p>
          <p>{t("preview.largeFileBody2")}</p>
          <button type="button" className="btn btn--primary" onClick={onRequestPreview}>
            {t("preview.renderOnce")}
          </button>
        </div>
      )}

      <div className="preview__body-wrap">
        {toc.length > 0 && (
          <aside className="preview__toc" aria-label={t("preview.contents")}>
            <div className="preview__toc-title">{t("preview.contents")}</div>
            <ul className="preview__toc-list">
              {toc.map((entry) => (
                <li
                  key={`${entry.id}-${entry.text}`}
                  className={`preview__toc-item preview__toc-item--l${Math.min(entry.level, 4)}`}
                >
                  <button
                    type="button"
                    className="preview__toc-link"
                    onClick={() => {
                      onTocNavigate?.(entry.id);
                      const el = bodyRef.current;
                      const node = el?.querySelector(`#${cssEscape(entry.id)}`);
                      if (node instanceof HTMLElement) {
                        node.scrollIntoView({ block: "start", behavior: "smooth" });
                      }
                    }}
                  >
                    {entry.text || entry.id}
                  </button>
                </li>
              ))}
            </ul>
          </aside>
        )}

        {/* HTML is injected via ref + sanitizePreviewHtml (never raw sidecar HTML). */}
        <div
          ref={bodyRef}
          className="preview__body markdown-preview"
          onScroll={handleScroll}
          onClick={handleClick}
        />
      </div>

      {diagnostics.length > 0 && (
        <div className="preview__diagnostics" role="status">
          {diagnostics.map((d, i) => (
            <div key={i} className={`preview__diag preview__diag--${d.level}`}>
              {d.line != null ? `L${d.line}: ` : ""}
              {d.message}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

async function rewriteLocalAssets(
  root: HTMLElement,
  sessionId: string | null,
  filePath: string | null,
): Promise<void> {
  if (!sessionId || !filePath) return;
  const nodes = root.querySelectorAll("img[src], source[src], a[href]");
  for (const node of Array.from(nodes)) {
    const attr = node.tagName === "A" ? "href" : "src";
    const value = node.getAttribute(attr);
    if (!value) continue;
    if (
      /^(https?:|data:|mailto:|tel:|lmlocal:|#)/i.test(value) ||
      value.startsWith("//")
    ) {
      continue;
    }
    try {
      const resolved = await cmd.resolveDocumentAsset(sessionId, value);
      node.setAttribute(attr, resolved.url);
    } catch {
      // Leave the attribute as-is (or strip src so broken images don't hit file://).
      if (attr === "src") {
        node.removeAttribute("src");
        node.setAttribute("data-missing-src", value);
      }
    }
  }
}

/** CSS.escape polyfill for WebView2. */
function cssEscape(value: string): string {
  if (typeof CSS !== "undefined" && typeof CSS.escape === "function") {
    return CSS.escape(value);
  }
  return value.replace(/([ !"#$%&'()*+,./:;<=>?@[\\\]^`{|}~])/g, "\\$1");
}
