/**
 * M3 export settings dialog — HTML (offline toggle) or PDF (page options).
 * Choosing Export opens a native save dialog, then runs the Rust export command.
 */

import { useEffect, useState } from "react";
import type { ExportFormat, PdfPageOptions } from "@litemark/shared-protocol";
import { DEFAULT_PDF_PAGE_OPTIONS } from "@litemark/shared-protocol";
import { useT } from "../i18n/I18nProvider";

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
  const t = useT();
  const [offline, setOffline] = useState(true);
  const [page, setPage] = useState<PdfPageOptions>({ ...DEFAULT_PDF_PAGE_OPTIONS });

  useEffect(() => {
    setOffline(true);
    setPage({ ...DEFAULT_PDF_PAGE_OPTIONS });
  }, [format]);

  const title = format === "html" ? t("export.htmlTitle") : t("export.pdfTitle");

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="export-title">
      <div className="modal modal--wide">
        <h2 id="export-title" className="modal__title">
          {title}
        </h2>
        <p className="modal__body">{t("export.body", { name: displayName })}</p>

        {format === "html" && (
          <div className="export-form">
            <label className="export-form__row">
              <input
                type="checkbox"
                checked={offline}
                disabled={busy}
                onChange={(e) => setOffline(e.target.checked)}
              />
              <span>{t("export.offline")}</span>
            </label>
            <p className="export-form__hint">{t("export.offlineHint")}</p>
          </div>
        )}

        {format === "pdf" && (
          <div className="export-form">
            {browserAvailable === false && (
              <div className="export-form__warn" role="alert">
                {t("export.noBrowser")}
              </div>
            )}
            {browserAvailable && browserName && (
              <p className="export-form__hint">{t("export.usingBrowser", { name: browserName })}</p>
            )}

            <label className="export-form__field">
              <span>{t("export.pageSize")}</span>
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
              <span>{t("export.landscape")}</span>
            </label>

            <label className="export-form__row">
              <input
                type="checkbox"
                checked={page.printBackground}
                disabled={busy}
                onChange={(e) => setPage((p) => ({ ...p, printBackground: e.target.checked }))}
              />
              <span>{t("export.printBackground")}</span>
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
              <span>{t("export.headerFooter")}</span>
            </label>

            <div className="export-form__margins">
              <span className="export-form__margins-label">{t("export.margins")}</span>
              {(["marginTopMm", "marginRightMm", "marginBottomMm", "marginLeftMm"] as const).map(
                (key) => {
                  const label =
                    key === "marginTopMm"
                      ? t("export.top")
                      : key === "marginRightMm"
                        ? t("export.right")
                        : key === "marginBottomMm"
                          ? t("export.bottom")
                          : t("export.left");
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
              {progressMessage ?? t("export.exporting")}{" "}
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
              {t("export.cancelExport")}
            </button>
          ) : (
            <>
              <button type="button" className="btn" onClick={onCancel}>
                {t("export.close")}
              </button>
              <button
                type="button"
                className="btn btn--primary"
                disabled={format === "pdf" && browserAvailable === false}
                onClick={() => onConfirm({ format, offline, page })}
              >
                {t("export.export")}
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
