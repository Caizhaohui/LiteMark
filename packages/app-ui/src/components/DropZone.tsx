/**
 * Drag-and-drop "open" overlay. Intercepts file drops at the app root and
 * forwards markdown files to the document store. Non-markdown drops are
 * ignored. Visual feedback is shown while dragging over the window.
 *
 * Note: in a Tauri webview the drop event's `dataTransfer.files` are available
 * to JS, but the *paths* are not exposed for security. The authoritative path
 * arrives via the Tauri file-drop event, so this component reads paths from
 * `onDropPaths` which the App wires to the Tauri webview drag-drop event.
 */

import { useEffect, useState } from "react";
import { useT } from "../i18n/I18nProvider";

interface DropZoneProps {
  /** Called with absolute file paths when a drop is accepted. */
  onDropPaths: (paths: string[]) => void;
  /** When true, the drop zone is armed (a drag is in progress over the window). */
  armed: boolean;
}

export function DropZone({ onDropPaths, armed }: DropZoneProps): JSX.Element | null {
  const t = useT();
  const [over, setOver] = useState(false);

  // Reset overlay if the drag leaves entirely.
  useEffect(() => {
    if (!armed) setOver(false);
  }, [armed]);

  if (!armed) return null;

  return (
    <div
      className={over ? "dropzone dropzone--over" : "dropzone"}
      onDragEnter={(e) => {
        e.preventDefault();
        setOver(true);
      }}
      onDragOver={(e) => {
        e.preventDefault();
      }}
      onDragLeave={(e) => {
        e.preventDefault();
        setOver(false);
      }}
      onDrop={(e) => {
        e.preventDefault();
        setOver(false);
        // Path forwarding happens via the Tauri event; this is a no-op fallback.
        onDropPaths([]);
      }}
    >
      <div className="dropzone__label">{t("dropzone.label")}</div>
    </div>
  );
}
