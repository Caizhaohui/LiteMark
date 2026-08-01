/**
 * Third-party notices / licenses viewer (M3).
 */

import { useT } from "../i18n/I18nProvider";

interface LicensesDialogProps {
  text: string;
  onClose: () => void;
}

export function LicensesDialog({ text, onClose }: LicensesDialogProps): JSX.Element {
  const t = useT();

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="licenses-title">
      <div className="modal modal--wide modal--tall">
        <h2 id="licenses-title" className="modal__title">
          {t("licenses.title")}
        </h2>
        <pre className="licenses__body">{text}</pre>
        <div className="modal__actions">
          <button type="button" className="btn btn--primary" onClick={onClose}>
            {t("licenses.close")}
          </button>
        </div>
      </div>
    </div>
  );
}
