/**
 * Status bar: encoding, line-ending, dirty, read-only, char count, preview status.
 */

import { useT } from "../i18n/I18nProvider";

interface StatusBarProps {
  encoding: string;
  lineEnding: string;
  readOnly: boolean;
  dirty: boolean;
  charCount: number;
  busy: boolean;
  /** Preview render status text (e.g. "42 ms" or "Rendering…"). */
  previewLabel?: string | null;
  reduced?: boolean;
  degraded?: boolean;
}

export function StatusBar({
  encoding,
  lineEnding,
  readOnly,
  dirty,
  charCount,
  busy,
  previewLabel,
  reduced,
  degraded,
}: StatusBarProps): JSX.Element {
  const t = useT();

  return (
    <footer className="statusbar">
      <span className="statusbar__item">{dirty ? t("status.unsaved") : t("status.saved")}</span>
      <span className="statusbar__item">{encoding}</span>
      <span className="statusbar__item">{lineEnding.toUpperCase()}</span>
      {readOnly && <span className="statusbar__item">{t("status.readOnly")}</span>}
      <span className="statusbar__item">{t("status.chars", { n: charCount })}</span>
      {degraded && <span className="statusbar__item">{t("preview.largeFileMode")}</span>}
      {!degraded && reduced && <span className="statusbar__item">{t("preview.reduced")}</span>}
      {previewLabel && <span className="statusbar__item">{previewLabel}</span>}
      {busy && <span className="statusbar__item statusbar__busy">{t("status.working")}</span>}
    </footer>
  );
}
