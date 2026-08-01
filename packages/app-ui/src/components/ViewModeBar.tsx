/**
 * Source / Split / Preview layout switcher (M2).
 * Distinct from DocumentSession.mode (source|hybrid|preview) which is for
 * Milkdown in M4 — this only controls the UI chrome layout.
 */

import type { PreviewLayout } from "@litemark/shared-protocol";
import { useT } from "../i18n/I18nProvider";

interface ViewModeBarProps {
  layout: PreviewLayout;
  onChange: (layout: PreviewLayout) => void;
  disabled?: boolean;
}

export function ViewModeBar({ layout, onChange, disabled }: ViewModeBarProps): JSX.Element {
  const t = useT();

  const options: { id: PreviewLayout; label: string; title: string }[] = [
    { id: "source", label: t("layout.source"), title: t("layout.sourceTitle") },
    { id: "split", label: t("layout.split"), title: t("layout.splitTitle") },
    { id: "preview", label: t("layout.preview"), title: t("layout.previewTitle") },
  ];

  return (
    <div className="viewmode" role="toolbar" aria-label={t("layout.viewLayout")}>
      {options.map((opt) => (
        <button
          key={opt.id}
          type="button"
          className={`viewmode__btn ${layout === opt.id ? "viewmode__btn--active" : ""}`}
          title={opt.title}
          disabled={disabled}
          aria-pressed={layout === opt.id}
          onClick={() => onChange(opt.id)}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
}
