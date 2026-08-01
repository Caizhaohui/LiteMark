/**
 * Unsaved-close confirmation: Save / Discard / Cancel.
 */

interface ConfirmCloseProps {
  displayName: string;
  onChoose: (choice: "save" | "discard" | "cancel") => void;
}

export function ConfirmClose({ displayName, onChoose }: ConfirmCloseProps): JSX.Element {
  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal">
        <h2 className="modal__title">Unsaved changes</h2>
        <p className="modal__body">
          <strong>{displayName}</strong> has unsaved changes. Save before closing?
        </p>
        <div className="modal__actions">
          <button type="button" className="btn btn--primary" onClick={() => onChoose("save")}>
            Save
          </button>
          <button type="button" className="btn" onClick={() => onChoose("discard")}>
            Discard
          </button>
          <button type="button" className="btn" onClick={() => onChoose("cancel")}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
