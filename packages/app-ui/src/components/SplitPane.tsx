/**
 * Horizontal split: left | drag handle | right.
 * Drag the handle to resize; ratio is clamped and optional persistence key
 * stores the last width in localStorage.
 */

import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type ReactNode,
  type PointerEvent as ReactPointerEvent,
  type KeyboardEvent as ReactKeyboardEvent,
} from "react";
import { useT } from "../i18n/I18nProvider";

const MIN_RATIO = 0.18;
const MAX_RATIO = 0.82;
const DEFAULT_RATIO = 0.5;
const STORAGE_KEY = "litemark.splitRatio";

interface SplitPaneProps {
  left: ReactNode;
  right: ReactNode;
  /** Initial left pane ratio (0–1). Defaults to last saved or 0.5. */
  defaultRatio?: number;
  /** Persist ratio under this key (default: litemark.splitRatio). */
  storageKey?: string;
}

function loadRatio(key: string, fallback: number): number {
  try {
    const raw = localStorage.getItem(key);
    if (raw == null) return fallback;
    const n = Number(raw);
    if (!Number.isFinite(n)) return fallback;
    return Math.min(MAX_RATIO, Math.max(MIN_RATIO, n));
  } catch {
    return fallback;
  }
}

export function SplitPane({
  left,
  right,
  defaultRatio = DEFAULT_RATIO,
  storageKey = STORAGE_KEY,
}: SplitPaneProps): JSX.Element {
  const t = useT();
  const [ratio, setRatio] = useState(() => loadRatio(storageKey, defaultRatio));
  const rootRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);

  const persist = useCallback(
    (r: number) => {
      try {
        localStorage.setItem(storageKey, String(r));
      } catch {
        /* ignore quota / private mode */
      }
    },
    [storageKey],
  );

  const onPointerDown = (e: ReactPointerEvent<HTMLDivElement>) => {
    e.preventDefault();
    dragging.current = true;
    e.currentTarget.setPointerCapture(e.pointerId);
    document.body.classList.add("split-pane--dragging");
  };

  const onPointerMove = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (!dragging.current || !rootRef.current) return;
    const rect = rootRef.current.getBoundingClientRect();
    if (rect.width <= 0) return;
    const x = e.clientX - rect.left;
    const next = Math.min(MAX_RATIO, Math.max(MIN_RATIO, x / rect.width));
    setRatio(next);
  };

  const endDrag = (e: ReactPointerEvent<HTMLDivElement>) => {
    if (!dragging.current) return;
    dragging.current = false;
    document.body.classList.remove("split-pane--dragging");
    try {
      e.currentTarget.releasePointerCapture(e.pointerId);
    } catch {
      /* already released */
    }
    setRatio((r) => {
      persist(r);
      return r;
    });
  };

  // Double-click handle → reset to 50%.
  const onDoubleClick = () => {
    setRatio(DEFAULT_RATIO);
    persist(DEFAULT_RATIO);
  };

  // Keyboard: focus handle and use arrows.
  const onKeyDown = (e: ReactKeyboardEvent<HTMLDivElement>) => {
    const step = e.shiftKey ? 0.08 : 0.03;
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      setRatio((r) => {
        const n = Math.max(MIN_RATIO, r - step);
        persist(n);
        return n;
      });
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      setRatio((r) => {
        const n = Math.min(MAX_RATIO, r + step);
        persist(n);
        return n;
      });
    } else if (e.key === "Home") {
      e.preventDefault();
      setRatio(DEFAULT_RATIO);
      persist(DEFAULT_RATIO);
    }
  };

  useEffect(() => {
    return () => {
      document.body.classList.remove("split-pane--dragging");
    };
  }, []);

  // Account for the fixed handle width so panes fill 100%.
  const handlePx = 6;
  const leftStyle = {
    width: `calc((100% - ${handlePx}px) * ${ratio})`,
  };
  const rightStyle = {
    width: `calc((100% - ${handlePx}px) * ${1 - ratio})`,
  };

  return (
    <div className="split-pane" ref={rootRef}>
      <div className="split-pane__pane split-pane__pane--left" style={leftStyle}>
        {left}
      </div>
      <div
        className="split-pane__handle"
        role="separator"
        aria-orientation="vertical"
        aria-valuenow={Math.round(ratio * 100)}
        aria-valuemin={Math.round(MIN_RATIO * 100)}
        aria-valuemax={Math.round(MAX_RATIO * 100)}
        aria-label={t("split.resize")}
        tabIndex={0}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={endDrag}
        onPointerCancel={endDrag}
        onDoubleClick={onDoubleClick}
        onKeyDown={onKeyDown}
      >
        <span className="split-pane__grip" aria-hidden />
      </div>
      <div className="split-pane__pane split-pane__pane--right" style={rightStyle}>
        {right}
      </div>
    </div>
  );
}
