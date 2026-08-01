/**
 * Tab bar + toolbars for document actions, export, editor mode, and layout.
 */

import type { PreviewLayout, SessionSummary } from "@litemark/shared-protocol";
import { ViewModeBar } from "./ViewModeBar";
import { EditorModeBar, type EditorMode } from "./EditorModeBar";

interface TabBarProps {
  sessions: SessionSummary[];
  busy: boolean;
  layout: PreviewLayout;
  editorMode: EditorMode;
  hasActive: boolean;
  onLayoutChange: (layout: PreviewLayout) => void;
  onEditorModeChange: (mode: EditorMode) => void;
  onActivate: (id: string) => void;
  onClose: (id: string) => void;
  onNew: () => void;
  onOpen: () => void;
  onSave: () => void;
  onExportHtml: () => void;
  onExportPdf: () => void;
  onExportPandoc: (format: "docx" | "epub" | "latex") => void;
  onSettings: () => void;
  onLicenses: () => void;
}

export function TabBar({
  sessions,
  busy,
  layout,
  editorMode,
  hasActive,
  onLayoutChange,
  onEditorModeChange,
  onActivate,
  onClose,
  onNew,
  onOpen,
  onSave,
  onExportHtml,
  onExportPdf,
  onExportPandoc,
  onSettings,
  onLicenses,
}: TabBarProps): JSX.Element {
  return (
    <div className="tabbar">
      <div className="tabbar__toolbar">
        <button type="button" className="btn" onClick={onNew} disabled={busy} title="New (Ctrl+N)">
          ＋
        </button>
        <button type="button" className="btn" onClick={onOpen} disabled={busy} title="Open (Ctrl+O)">
          Open
        </button>
        <button type="button" className="btn" onClick={onSave} disabled={busy} title="Save (Ctrl+S)">
          Save
        </button>
        <button
          type="button"
          className="btn"
          onClick={onExportHtml}
          disabled={busy || !hasActive}
          title="Export HTML"
        >
          HTML
        </button>
        <button
          type="button"
          className="btn"
          onClick={onExportPdf}
          disabled={busy || !hasActive}
          title="Export PDF"
        >
          PDF
        </button>
        <button
          type="button"
          className="btn"
          onClick={() => onExportPandoc("docx")}
          disabled={busy || !hasActive}
          title="Export DOCX via Pandoc"
        >
          DOCX
        </button>
        <button
          type="button"
          className="btn"
          onClick={() => onExportPandoc("epub")}
          disabled={busy || !hasActive}
          title="Export EPUB via Pandoc"
        >
          EPUB
        </button>
        <button type="button" className="btn" onClick={onSettings} disabled={busy} title="Settings">
          ⚙
        </button>
        <button type="button" className="btn" onClick={onLicenses} disabled={busy} title="Licenses">
          ©
        </button>
        <div className="tabbar__spacer" />
        <EditorModeBar mode={editorMode} onChange={onEditorModeChange} disabled={busy || !hasActive} />
        <ViewModeBar layout={layout} onChange={onLayoutChange} disabled={busy} />
      </div>
      <div className="tabbar__tabs" role="tablist">
        {sessions.length === 0 && <span className="tabbar__empty">No documents open</span>}
        {sessions.map((s) => (
          <button
            key={s.id}
            type="button"
            role="tab"
            aria-selected={s.active}
            className={`tab ${s.active ? "tab--active" : ""}`}
            onClick={() => onActivate(s.id)}
            title={s.filePath ?? s.displayName}
          >
            <span className="tab__name">{s.displayName}</span>
            {s.dirty && <span className="tab__dot" aria-label="unsaved changes" />}
            {s.readOnly && (
              <span className="tab__ro" title="read-only">
                ⊘
              </span>
            )}
            <span
              className="tab__close"
              role="button"
              tabIndex={0}
              aria-label={`Close ${s.displayName}`}
              onClick={(e) => {
                e.stopPropagation();
                onClose(s.id);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.stopPropagation();
                  onClose(s.id);
                }
              }}
            >
              ×
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
