/**
 * M3 export settings dialog — HTML (offline toggle) or PDF (page options).
 * Choosing Export opens a native save dialog, then runs the Rust export command.
 */

import { useEffect, useState } from "react";
import type { ExportFormat, PdfPageOptions } from "@litemark/shared-protocol";
import { DEFAULT_PDF_PAGE_OPTIONS } from "@litemark/shared-protocol";

export interface ExportDialogResult {
  format: ExportFormat;
  offline: boolean;
  page: PdfPageOptions;
}

interface ExportDialogProps {
  format: ExportFormat;
  displayName: string;
  browserAvailable: boolean | null;
  browserName?: string | null;
  busy: boolean;
  progress: number | null;
  progressMessage: string | null;
  error: string | null;
  onConfirm: (opts: ExportDialogResult) => void;
  onCancel: () => void;
  onAbort: () => void;
}

export function ExportDialog({
  format,
  displayName,
  browserAvailable,
  browserName,
  busy,
  progress,
  progressMessage,
  error,
  onConfirm,
  onCancel,
  onAbort,
}: ExportDialogProps): JSX.Element {
  const [offline, setOffline] = useState(true);
  const [page, setPage] = useState<PdfPageOptions>({ ...DEFAULT_PDF_PAGE_OPTIONS });

  useEffect(() => {
    setOffline(true);
    setPage({ ...DEFAULT_PDF_PAGE_OPTIONS });
  }, [format]);

  const title = format === "html" ? "Export HTML" : "Export PDF";

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="export-title">
      <div className="modal modal--wide">
        <h2 id="export-title" className="modal__title">
          {title}
        </h2>
        <p className="modal__body">
          Export <strong>{displayName}</strong> without modifying the source Markdown file.
        </p>

        {format === "html" && (
          <div className="export-form">
            <label className="export-form__row">
              <input
                type="checkbox"
                checked={offline}
                disabled={busy}
                onChange={(e) => setOffline(e.target.checked)}
              />
              <span>Offline package (inline CSS, fonts, and images)</span>
            </label>
            <p className="export-form__hint">
              Offline mode produces a single HTML file that opens without a network connection.
            </p>
          </div>
        )}

        {format === "pdf" && (
          <div className="export-form">
            {browserAvailable === false && (
              <div className="export-form__warn" role="alert">
                No Microsoft Edge or Google Chrome was found. PDF export requires one of these
                browsers. Install Edge (recommended on Windows) and try again.
              </div>
            )}
            {browserAvailable && browserName && (
              <p className="export-form__hint">Using: {browserName}</p>
            )}

            <label className="export-form__field">
              <span>Page size</span>
              <select
                value={page.pageSize}
                disabled={busy}
                onChange={(e) =>
                  setPage((p) => ({
                    ...p,
                    pageSize: e.target.value as PdfPageOptions["pageSize"],
                  }))
                }
              >
                <option value="A4">A4</option>
                <option value="Letter">Letter</option>
                <option value="Legal">Legal</option>
              </select>
            </label>

            <label className="export-form__row">
              <input
                type="checkbox"
                checked={page.landscape}
                disabled={busy}
                onChange={(e) => setPage((p) => ({ ...p, landscape: e.target.checked }))}
              />
              <span>Landscape</span>
            </label>

            <label className="export-form__row">
              <input
                type="checkbox"
                checked={page.printBackground}
                disabled={busy}
                onChange={(e) => setPage((p) => ({ ...p, printBackground: e.target.checked }))}
              />
              <span>Print background colors</span>
            </label>

            <label className="export-form__row">
              <input
                type="checkbox"
                checked={page.displayHeaderFooter}
                disabled={busy}
                onChange={(e) =>
                  setPage((p) => ({ ...p, displayHeaderFooter: e.target.checked }))
                }
              />
              <span>Header / footer</span>
            </label>

            <div className="export-form__margins">
              <span className="export-form__margins-label">Margins (mm)</span>
              {(["marginTopMm", "marginRightMm", "marginBottomMm", "marginLeftMm"] as const).map(
                (key) => {
                  const label =
                    key === "marginTopMm"
                      ? "Top"
                      : key === "marginRightMm"
                        ? "Right"
                        : key === "marginBottomMm"
                          ? "Bottom"
                          : "Left";
                  return (
                    <label key={key} className="export-form__field export-form__field--sm">
                      <span>{label}</span>
                      <input
                        type="number"
                        min={0}
                        max={50}
                        step={1}
                        disabled={busy}
                        value={page[key]}
                        onChange={(e) =>
                          setPage((p) => ({
                            ...p,
                            [key]: Number(e.target.value) || 0,
                          }))
                        }
                      />
                    </label>
                  );
                },
              )}
            </div>
          </div>
        )}

        {busy && (
          <div className="export-progress" aria-live="polite">
            <div className="export-progress__bar">
              <div
                className="export-progress__fill"
                style={{ width: `${Math.round((progress ?? 0) * 100)}%` }}
              />
            </div>
            <div className="export-progress__msg">
              {progressMessage ?? "Exporting…"}{" "}
              {progress != null && `(${Math.round(progress * 100)}%)`}
            </div>
          </div>
        )}

        {error && (
          <div className="export-form__warn" role="alert">
            {error}
          </div>
        )}

        <div className="modal__actions">
          {busy ? (
            <button type="button" className="btn" onClick={onAbort}>
              Cancel export
            </button>
          ) : (
            <>
              <button type="button" className="btn" onClick={onCancel}>
                Close
              </button>
              <button
                type="button"
                className="btn btn--primary"
                disabled={format === "pdf" && browserAvailable === false}
                onClick={() => onConfirm({ format, offline, page })}
              >
                Export…
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
