/**
 * Source / Split / Preview layout switcher (M2).
 * Distinct from DocumentSession.mode (source|hybrid|preview) which is for
 * Milkdown in M4 — this only controls the UI chrome layout.
 */

import type { PreviewLayout } from "@litemark/shared-protocol";

interface ViewModeBarProps {
  layout: PreviewLayout;
  onChange: (layout: PreviewLayout) => void;
  disabled?: boolean;
}

const OPTIONS: { id: PreviewLayout; label: string; title: string }[] = [
  { id: "source", label: "Source", title: "Source only (Monaco)" },
  { id: "split", label: "Split", title: "Source + preview side by side" },
  { id: "preview", label: "Preview", title: "Preview only" },
];

export function ViewModeBar({ layout, onChange, disabled }: ViewModeBarProps): JSX.Element {
  return (
    <div className="viewmode" role="toolbar" aria-label="View layout">
      {OPTIONS.map((opt) => (
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
