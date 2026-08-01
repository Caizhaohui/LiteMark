/**
 * Unsaved-close confirmation: Save / Discard / Cancel.
 */

import { useT } from "../i18n/I18nProvider";

interface ConfirmCloseProps {
  displayName: string;
  onChoose: (choice: "save" | "discard" | "cancel") => void;
}

export function ConfirmClose({ displayName, onChoose }: ConfirmCloseProps): JSX.Element {
  const t = useT();

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal">
        <h2 className="modal__title">{t("confirmClose.title")}</h2>
        <p className="modal__body">{t("confirmClose.body", { name: displayName })}</p>
        <div className="modal__actions">
          <button type="button" className="btn btn--primary" onClick={() => onChoose("save")}>
            {t("confirmClose.save")}
          </button>
          <button type="button" className="btn" onClick={() => onChoose("discard")}>
            {t("confirmClose.discard")}
          </button>
          <button type="button" className="btn" onClick={() => onChoose("cancel")}>
            {t("confirmClose.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
