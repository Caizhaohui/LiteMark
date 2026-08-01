/**
 * Shown when source → hybrid roundtrip detects potential data loss (M4 §7.3).
 */

import { useT } from "../i18n/I18nProvider";

interface ModeSwitchWarningProps {
  risks: string[];
  onStaySource: () => void;
  onForceHybrid?: () => void;
}

export function ModeSwitchWarning({
  risks,
  onStaySource,
}: ModeSwitchWarningProps): JSX.Element {
  const t = useT();

  return (
    <div className="modal-overlay" role="alertdialog" aria-modal="true" aria-labelledby="mode-warn-title">
      <div className="modal modal--wide">
        <h2 id="mode-warn-title" className="modal__title">
          {t("modeWarn.title")}
        </h2>
        <p className="modal__body">{t("modeWarn.body")}</p>
        <ul className="mode-warn__list">
          {risks.map((r, i) => (
            <li key={i}>
              <pre className="mode-warn__risk">{r}</pre>
            </li>
          ))}
        </ul>
        <div className="modal__actions">
          <button type="button" className="btn btn--primary" onClick={onStaySource}>
            {t("modeWarn.stay")}
          </button>
        </div>
      </div>
    </div>
  );
}
