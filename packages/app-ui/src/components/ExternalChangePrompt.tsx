/**
 * External-modification prompt: Reload / Keep Mine / Compare Later.
 * The file changed on disk since it was opened; LiteMark never silently
 * overwrites (M1 acceptance).
 */

import { useT } from "../i18n/I18nProvider";

interface ExternalChangePromptProps {
  displayName: string;
  onChoose: (choice: "reload" | "keep" | "compare") => void;
}

export function ExternalChangePrompt({
  displayName,
  onChoose,
}: ExternalChangePromptProps): JSX.Element {
  const t = useT();

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal">
        <h2 className="modal__title">{t("externalChange.title")}</h2>
        <p className="modal__body">{t("externalChange.body", { name: displayName })}</p>
        <div className="modal__actions">
          <button type="button" className="btn btn--primary" onClick={() => onChoose("reload")}>
            {t("externalChange.reload")}
          </button>
          <button type="button" className="btn" onClick={() => onChoose("keep")}>
            {t("externalChange.keep")}
          </button>
          <button type="button" className="btn" onClick={() => onChoose("compare")}>
            {t("externalChange.compare")}
          </button>
        </div>
      </div>
    </div>
  );
}
