/**
 * Startup recovery prompt. If LiteMark finds crash-recovery snapshots on disk,
 * it offers to restore each (or discard all). M1: a simple list.
 */

import type { RecoveryEntry } from "@litemark/shared-protocol";

interface RecoveryPromptProps {
  entries: RecoveryEntry[];
  onRestore: (recoveryKey: string) => void;
  onDiscardOne: (recoveryKey: string) => void;
  onDiscardAll: () => void;
}

export function RecoveryPrompt({
  entries,
  onRestore,
  onDiscardOne,
  onDiscardAll,
}: RecoveryPromptProps): JSX.Element {
  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal modal--wide">
        <h2 className="modal__title">Recover documents?</h2>
        <p className="modal__body">
          The previous session ended with unsaved work. LiteMark kept a copy — restore a document to
          continue editing, or discard it.
        </p>
        <ul className="recovery__list">
          {entries.map((e) => (
            <li key={e.recoveryKey} className="recovery__item">
              <div className="recovery__meta">
                <span className="recovery__name">
                  {e.originalPath ?? "Untitled"}
                </span>
                <span className="recovery__sub">
                  rev {e.revision} · {e.capturedAt}
                </span>
              </div>
              <div className="recovery__actions">
                <button
                  type="button"
                  className="btn btn--primary"
                  onClick={() => onRestore(e.recoveryKey)}
                >
                  Restore
                </button>
                <button
                  type="button"
                  className="btn"
                  onClick={() => onDiscardOne(e.recoveryKey)}
                >
                  Discard
                </button>
              </div>
            </li>
          ))}
        </ul>
        <div className="modal__actions">
          <button type="button" className="btn" onClick={onDiscardAll}>
            Discard all
          </button>
        </div>
      </div>
    </div>
  );
}
