/**
 * Shown when source → hybrid roundtrip detects potential data loss (M4 §7.3).
 */

interface ModeSwitchWarningProps {
  risks: string[];
  onStaySource: () => void;
  onForceHybrid?: () => void;
}

export function ModeSwitchWarning({
  risks,
  onStaySource,
}: ModeSwitchWarningProps): JSX.Element {
  return (
    <div className="modal-overlay" role="alertdialog" aria-modal="true" aria-labelledby="mode-warn-title">
      <div className="modal modal--wide">
        <h2 id="mode-warn-title" className="modal__title">
          Hybrid mode may change this document
        </h2>
        <p className="modal__body">
          Switching to hybrid editing would rewrite Markdown in a way that is not
          equivalent after normalization. LiteMark stays in source mode to protect
          your content.
        </p>
        <ul className="mode-warn__list">
          {risks.map((r, i) => (
            <li key={i}>
              <pre className="mode-warn__risk">{r}</pre>
            </li>
          ))}
        </ul>
        <div className="modal__actions">
          <button type="button" className="btn btn--primary" onClick={onStaySource}>
            Stay in source mode
          </button>
        </div>
      </div>
    </div>
  );
}
