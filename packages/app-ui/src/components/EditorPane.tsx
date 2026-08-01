/**
 * M2 source editor: Monaco with Markdown language support.
 *
 * Replaces the M1 temporary textarea. Features required by §7.1:
 * syntax highlight, line numbers, word wrap, find/replace (built-in),
 * multi-cursor, undo/redo. Ctrl+S is handled at the App level.
 */

import { useEffect, useRef, useState } from "react";
import Editor, { loader, type OnMount } from "@monaco-editor/react";
import * as monaco from "monaco-editor";

// Bundle Monaco from node_modules (no CDN) so the offline desktop app works.
loader.config({ monaco });

interface EditorPaneProps {
  value: string;
  readOnly: boolean;
  disabled: boolean;
  onChange: (value: string) => void;
  /** Scroll percentage 0–1 from the preview pane (basic scroll sync). */
  scrollRatio?: number | null;
  /** Called when the editor scrolls so the preview can follow. */
  onScrollRatio?: (ratio: number) => void;
}

export function EditorPane({
  value,
  readOnly,
  disabled,
  onChange,
  scrollRatio,
  onScrollRatio,
}: EditorPaneProps): JSX.Element {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const applyingRemoteScroll = useRef(false);
  const dark = usePrefersDark();

  const handleMount: OnMount = (editor) => {
    editorRef.current = editor;
    editor.onDidScrollChange(() => {
      if (applyingRemoteScroll.current) return;
      if (!onScrollRatio) return;
      const dom = editor.getDomNode();
      if (!dom) return;
      const top = editor.getScrollTop();
      const height = editor.getScrollHeight() - dom.clientHeight;
      if (height <= 0) {
        onScrollRatio(0);
        return;
      }
      onScrollRatio(Math.min(1, Math.max(0, top / height)));
    });
  };

  // Apply scroll sync from the preview.
  useEffect(() => {
    if (scrollRatio == null) return;
    const editor = editorRef.current;
    if (!editor) return;
    const dom = editor.getDomNode();
    if (!dom) return;
    const height = editor.getScrollHeight() - dom.clientHeight;
    if (height <= 0) return;
    applyingRemoteScroll.current = true;
    editor.setScrollTop(scrollRatio * height);
    requestAnimationFrame(() => {
      applyingRemoteScroll.current = false;
    });
  }, [scrollRatio]);

  if (disabled) {
    return (
      <div className="editor">
        <div className="editor__placeholder">
          No document open. Press ＋ to create or Open to load a file.
        </div>
      </div>
    );
  }

  return (
    <div className="editor">
      <Editor
        language="markdown"
        theme={dark ? "vs-dark" : "light"}
        value={value}
        onChange={(v) => onChange(v ?? "")}
        onMount={handleMount}
        options={{
          readOnly,
          wordWrap: "on",
          lineNumbers: "on",
          minimap: { enabled: false },
          fontSize: 14,
          fontFamily:
            '"Cascadia Code", "JetBrains Mono", Consolas, "Microsoft YaHei", monospace',
          scrollBeyondLastLine: false,
          automaticLayout: true,
          tabSize: 2,
          renderWhitespace: "selection",
          unicodeHighlight: { ambiguousCharacters: false },
          quickSuggestions: false,
          suggestOnTriggerCharacters: false,
          wordBasedSuggestions: "off",
          padding: { top: 12, bottom: 12 },
        }}
        loading={<div className="editor__placeholder">Loading editor…</div>}
      />
    </div>
  );
}

function usePrefersDark(): boolean {
  const [dark, setDark] = useState(
    () =>
      typeof window !== "undefined" &&
      !!window.matchMedia?.("(prefers-color-scheme: dark)").matches,
  );
  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => setDark(mq.matches);
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, []);
  return dark;
}
