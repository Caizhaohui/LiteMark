/**
 * Third-party notices / licenses viewer (M3).
 */

interface LicensesDialogProps {
  text: string;
  onClose: () => void;
}

export function LicensesDialog({ text, onClose }: LicensesDialogProps): JSX.Element {
  return (
    <div className="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="licenses-title">
      <div className="modal modal--wide modal--tall">
        <h2 id="licenses-title" className="modal__title">
          Third-party notices
        </h2>
        <pre className="licenses__body">{text}</pre>
        <div className="modal__actions">
          <button type="button" className="btn btn--primary" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
