/**
 * Status bar: encoding, line-ending, dirty, read-only, char count, preview status.
 */

interface StatusBarProps {
  encoding: string;
  lineEnding: string;
  readOnly: boolean;
  dirty: boolean;
  charCount: number;
  busy: boolean;
  /** Preview render status text (e.g. "42 ms" or "Rendering…"). */
  previewLabel?: string | null;
  reduced?: boolean;
  degraded?: boolean;
}

export function StatusBar({
  encoding,
  lineEnding,
  readOnly,
  dirty,
  charCount,
  busy,
  previewLabel,
  reduced,
  degraded,
}: StatusBarProps): JSX.Element {
  return (
    <footer className="statusbar">
      <span className="statusbar__item">
        {dirty ? "● Unsaved" : "Saved"}
      </span>
      <span className="statusbar__item">{encoding}</span>
      <span className="statusbar__item">{lineEnding.toUpperCase()}</span>
      {readOnly && <span className="statusbar__item">Read-only</span>}
      <span className="statusbar__item">{charCount} chars</span>
      {degraded && <span className="statusbar__item">Preview: large-file mode</span>}
      {!degraded && reduced && <span className="statusbar__item">Preview: reduced</span>}
      {previewLabel && <span className="statusbar__item">{previewLabel}</span>}
      {busy && <span className="statusbar__item statusbar__busy">Working…</span>}
    </footer>
  );
}
