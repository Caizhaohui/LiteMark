/**
 * External-modification prompt: Reload / Keep Mine / Compare Later.
 * The file changed on disk since it was opened; LiteMark never silently
 * overwrites (M1 acceptance).
 */

interface ExternalChangePromptProps {
  displayName: string;
  onChoose: (choice: "reload" | "keep" | "compare") => void;
}

export function ExternalChangePrompt({
  displayName,
  onChoose,
}: ExternalChangePromptProps): JSX.Element {
  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal">
        <h2 className="modal__title">File changed on disk</h2>
        <p className="modal__body">
          <strong>{displayName}</strong> was modified outside LiteMark. Reload the file (discarding
          your unsaved edits), keep your version, or decide later.
        </p>
        <div className="modal__actions">
          <button type="button" className="btn btn--primary" onClick={() => onChoose("reload")}>
            Reload
          </button>
          <button type="button" className="btn" onClick={() => onChoose("keep")}>
            Keep mine
          </button>
          <button type="button" className="btn" onClick={() => onChoose("compare")}>
            Compare later
          </button>
        </div>
      </div>
    </div>
  );
}
