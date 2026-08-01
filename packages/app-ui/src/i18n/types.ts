import type { en } from "./locales/en";

/** Supported UI locales (BCP 47 tags). Default: en. */
export const LOCALES = ["en", "zh-CN", "zh-TW", "ja"] as const;

export type LocaleId = (typeof LOCALES)[number];

export const DEFAULT_LOCALE: LocaleId = "en";

export const LOCALE_STORAGE_KEY = "litemark.locale";

/** Native names for the language picker. */
export const LOCALE_LABELS: Record<LocaleId, string> = {
  en: "English",
  "zh-CN": "简体中文",
  "zh-TW": "繁體中文",
  ja: "日本語",
};

/** Widen string literal leaves so catalogs can use any translation string. */
type WidenStrings<T> = {
  [K in keyof T]: T[K] extends string
    ? string
    : T[K] extends object
      ? WidenStrings<T[K]>
      : T[K];
};

/** Message tree shape (English catalog is the source of keys). */
export type MessageTree = WidenStrings<typeof en>;

/** Deep partial for non-English catalogs (missing keys fall back to English). */
export type DeepPartial<T> = {
  [K in keyof T]?: T[K] extends string
    ? string
    : T[K] extends object
      ? DeepPartial<T[K]>
      : T[K];
};
