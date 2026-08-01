/**
 * M4 hybrid (WYSIWYM) editor powered by Milkdown / ProseMirror.
 *
 * Markdown remains the only on-disk format. This component:
 * - loads markdown via defaultValueCtx
 * - reports markdown on change via listener
 * - supports undo/redo (history) and clipboard paste (sanitized by Milkdown)
 *
 * Mode-switch data-loss checks live outside (hybridRoundtrip).
 */

import { useCallback, useEffect, useRef } from "react";
import { Editor, rootCtx, defaultValueCtx, editorViewCtx } from "@milkdown/kit/core";
import { commonmark } from "@milkdown/kit/preset/commonmark";
import { gfm } from "@milkdown/kit/preset/gfm";
import { history } from "@milkdown/kit/plugin/history";
import { clipboard } from "@milkdown/kit/plugin/clipboard";
import { listener, listenerCtx } from "@milkdown/kit/plugin/listener";
import { getMarkdown, replaceAll } from "@milkdown/kit/utils";
import { Milkdown, MilkdownProvider, useEditor, useInstance } from "@milkdown/react";
import "@milkdown/kit/prose/view/style/prosemirror.css";
import "@milkdown/kit/prose/tables/style/tables.css";

interface HybridEditorInnerProps {
  value: string;
  readOnly: boolean;
  onChange: (markdown: string) => void;
  /** Bumps when the document id changes so the editor reloads content. */
  documentKey: string;
}

function HybridEditorInner({
  value,
  readOnly,
  onChange,
  documentKey,
}: HybridEditorInnerProps): JSX.Element {
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const lastEmitted = useRef(value);
  const initialValue = useRef(value);

  // Capture the first value for this documentKey as defaultValue.
  const defaultForKey = useRef({ key: documentKey, value });
  if (defaultForKey.current.key !== documentKey) {
    defaultForKey.current = { key: documentKey, value };
    initialValue.current = value;
    lastEmitted.current = value;
  }

  useEditor(
    (root) => {
      return Editor.make()
        .config((ctx) => {
          ctx.set(rootCtx, root);
          ctx.set(defaultValueCtx, initialValue.current);
          ctx.get(listenerCtx).markdownUpdated((_ctx, markdown) => {
            if (markdown === lastEmitted.current) return;
            lastEmitted.current = markdown;
            onChangeRef.current(markdown);
          });
        })
        .use(commonmark)
        .use(gfm)
        .use(history)
        .use(clipboard)
        .use(listener);
    },
    [documentKey],
  );

  const [loading, getEditor] = useInstance();

  // Apply external value updates (e.g. recovery / reload) without looping.
  useEffect(() => {
    if (loading) return;
    if (value === lastEmitted.current) return;
    const editor = getEditor();
    if (!editor) return;
    editor.action(replaceAll(value));
    lastEmitted.current = value;
  }, [value, loading, getEditor]);

  // Read-only toggle via ProseMirror editable prop.
  useEffect(() => {
    if (loading) return;
    const editor = getEditor();
    if (!editor) return;
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      view.setProps({ editable: () => !readOnly });
    });
  }, [readOnly, loading, getEditor]);

  return (
    <div className={`hybrid ${readOnly ? "hybrid--readonly" : ""}`}>
      <Milkdown />
    </div>
  );
}

export interface HybridEditorProps {
  value: string;
  readOnly: boolean;
  disabled: boolean;
  documentKey: string;
  onChange: (markdown: string) => void;
}

export function HybridEditor(props: HybridEditorProps): JSX.Element {
  if (props.disabled) {
    return (
      <div className="editor">
        <div className="editor__placeholder">
          No document open. Press ＋ to create or Open to load a file.
        </div>
      </div>
    );
  }

  return (
    <div className="editor editor--hybrid">
      <MilkdownProvider>
        <HybridEditorInner
          key={props.documentKey}
          value={props.value}
          readOnly={props.readOnly}
          onChange={props.onChange}
          documentKey={props.documentKey}
        />
      </MilkdownProvider>
    </div>
  );
}

/** Read current markdown from a live milkdown instance (helper for tests). */
export function readHybridMarkdown(getEditor: () => Editor | undefined): string {
  const editor = getEditor();
  if (!editor) return "";
  return editor.action(getMarkdown());
}

export const HybridToolbar = ({
  disabled,
  onCommand,
}: {
  disabled?: boolean;
  onCommand: (cmd: HybridToolbarCommand) => void;
}): JSX.Element => {
  const btn = useCallback(
    (label: string, cmd: HybridToolbarCommand, title: string) => (
      <button
        type="button"
        className="btn btn--small"
        disabled={disabled}
        title={title}
        onClick={() => onCommand(cmd)}
      >
        {label}
      </button>
    ),
    [disabled, onCommand],
  );

  return (
    <div className="hybrid-toolbar" role="toolbar" aria-label="Hybrid formatting">
      {btn("B", "toggleStrong", "Bold")}
      {btn("I", "toggleEmphasis", "Italic")}
      {btn("S", "toggleInlineCode", "Inline code")}
      {btn("H1", "wrapHeading1", "Heading 1")}
      {btn("H2", "wrapHeading2", "Heading 2")}
      {btn("•", "wrapBulletList", "Bullet list")}
      {btn("1.", "wrapOrderedList", "Ordered list")}
      {btn("☐", "wrapTaskList", "Task list")}
      {btn("“", "wrapBlockquote", "Quote")}
      {btn("—", "insertHr", "Horizontal rule")}
    </div>
  );
};

export type HybridToolbarCommand =
  | "toggleStrong"
  | "toggleEmphasis"
  | "toggleInlineCode"
  | "wrapHeading1"
  | "wrapHeading2"
  | "wrapBulletList"
  | "wrapOrderedList"
  | "wrapTaskList"
  | "wrapBlockquote"
  | "insertHr";

/**
 * Toolbar actions applied as markdown wrappers when the full Milkdown command
 * surface is awkward from outside. Keeps hybrid editing useful even if a
 * ProseMirror command fails.
 */
export function applyToolbarMarkdown(
  markdown: string,
  cmd: HybridToolbarCommand,
  selection?: { start: number; end: number },
): string {
  const start = selection?.start ?? markdown.length;
  const end = selection?.end ?? markdown.length;
  const selected = markdown.slice(start, end) || "text";
  let replacement = selected;
  switch (cmd) {
    case "toggleStrong":
      replacement = `**${selected}**`;
      break;
    case "toggleEmphasis":
      replacement = `*${selected}*`;
      break;
    case "toggleInlineCode":
      replacement = `\`${selected}\``;
      break;
    case "wrapHeading1":
      replacement = `# ${selected.replace(/^#+\s*/, "")}`;
      break;
    case "wrapHeading2":
      replacement = `## ${selected.replace(/^#+\s*/, "")}`;
      break;
    case "wrapBulletList":
      replacement = selected
        .split("\n")
        .map((l) => (l.trim() ? `- ${l.trim()}` : l))
        .join("\n");
      break;
    case "wrapOrderedList":
      replacement = selected
        .split("\n")
        .map((l, i) => (l.trim() ? `${i + 1}. ${l.trim()}` : l))
        .join("\n");
      break;
    case "wrapTaskList":
      replacement = selected
        .split("\n")
        .map((l) => (l.trim() ? `- [ ] ${l.trim()}` : l))
        .join("\n");
      break;
    case "wrapBlockquote":
      replacement = selected
        .split("\n")
        .map((l) => `> ${l}`)
        .join("\n");
      break;
    case "insertHr":
      replacement = `${selected}\n\n---\n`;
      break;
  }
  return markdown.slice(0, start) + replacement + markdown.slice(end);
}
