/**
 * Source vs Hybrid editor mode switch (M4). Distinct from layout (source/split/preview).
 */

import { useT } from "../i18n/I18nProvider";

export type EditorMode = "source" | "hybrid";

interface EditorModeBarProps {
  mode: EditorMode;
  disabled?: boolean;
  onChange: (mode: EditorMode) => void;
}

export function EditorModeBar({ mode, disabled, onChange }: EditorModeBarProps): JSX.Element {
  const t = useT();

  return (
    <div className="viewmode" role="toolbar" aria-label={t("editorMode.label")}>
      <button
        type="button"
        className={`viewmode__btn ${mode === "source" ? "viewmode__btn--active" : ""}`}
        disabled={disabled}
        aria-pressed={mode === "source"}
        title={t("editorMode.sourceTitle")}
        onClick={() => onChange("source")}
      >
        {t("editorMode.source")}
      </button>
      <button
        type="button"
        className={`viewmode__btn ${mode === "hybrid" ? "viewmode__btn--active" : ""}`}
        disabled={disabled}
        aria-pressed={mode === "hybrid"}
        title={t("editorMode.hybridTitle")}
        onClick={() => onChange("hybrid")}
      >
        {t("editorMode.hybrid")}
      </button>
    </div>
  );
}
