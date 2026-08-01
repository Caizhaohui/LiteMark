import type { DeepPartial, MessageTree } from "./types";
import { en } from "./locales/en";

/** Look up a dotted key in a message tree. */
export function lookup(tree: unknown, path: string): string | undefined {
  const parts = path.split(".");
  let cur: unknown = tree;
  for (const p of parts) {
    if (cur == null || typeof cur !== "object") return undefined;
    cur = (cur as Record<string, unknown>)[p];
  }
  return typeof cur === "string" ? cur : undefined;
}

/** Merge overlay onto base (deep). */
export function deepMerge(
  base: MessageTree,
  overlay: DeepPartial<MessageTree> | undefined,
): MessageTree {
  if (!overlay) return base;
  return mergeObj(base as unknown as Record<string, unknown>, overlay as Record<string, unknown>) as unknown as MessageTree;
}

function mergeObj(
  base: Record<string, unknown>,
  over: Record<string, unknown>,
): Record<string, unknown> {
  const out: Record<string, unknown> = { ...base };
  for (const [k, v] of Object.entries(over)) {
    if (v && typeof v === "object" && !Array.isArray(v) && typeof base[k] === "object") {
      out[k] = mergeObj(base[k] as Record<string, unknown>, v as Record<string, unknown>);
    } else if (v !== undefined) {
      out[k] = v;
    }
  }
  return out;
}

/** Interpolate `{name}` placeholders. */
export function interpolate(
  template: string,
  params?: Record<string, string | number>,
): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (_, key: string) =>
    params[key] != null ? String(params[key]) : `{${key}}`,
  );
}

/** Translate with English fallback. */
export function translate(
  catalog: MessageTree,
  key: string,
  params?: Record<string, string | number>,
): string {
  const raw = lookup(catalog, key) ?? lookup(en, key) ?? key;
  return interpolate(raw, params);
}
