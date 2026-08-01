/**
 * Source vs Hybrid editor mode switch (M4). Distinct from layout (source/split/preview).
 */

export type EditorMode = "source" | "hybrid";

interface EditorModeBarProps {
  mode: EditorMode;
  disabled?: boolean;
  onChange: (mode: EditorMode) => void;
}

export function EditorModeBar({ mode, disabled, onChange }: EditorModeBarProps): JSX.Element {
  return (
    <div className="viewmode" role="toolbar" aria-label="Editor mode">
      <button
        type="button"
        className={`viewmode__btn ${mode === "source" ? "viewmode__btn--active" : ""}`}
        disabled={disabled}
        aria-pressed={mode === "source"}
        title="Source mode (Monaco)"
        onClick={() => onChange("source")}
      >
        Source
      </button>
      <button
        type="button"
        className={`viewmode__btn ${mode === "hybrid" ? "viewmode__btn--active" : ""}`}
        disabled={disabled}
        aria-pressed={mode === "hybrid"}
        title="Hybrid mode (Milkdown)"
        onClick={() => onChange("hybrid")}
      >
        Hybrid
      </button>
    </div>
  );
}
