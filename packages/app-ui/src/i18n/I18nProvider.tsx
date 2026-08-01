/**
 * LiteMark UI internationalization.
 * Default locale: English. Preference stored in localStorage.
 */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { en } from "./locales/en";
import { catalogs } from "./locales";
import {
  DEFAULT_LOCALE,
  LOCALES,
  LOCALE_LABELS,
  LOCALE_STORAGE_KEY,
  type LocaleId,
  type MessageTree,
} from "./types";
import { deepMerge, translate } from "./resolve";

type TFunction = (key: string, params?: Record<string, string | number>) => string;

interface I18nContextValue {
  locale: LocaleId;
  setLocale: (id: LocaleId) => void;
  t: TFunction;
  messages: MessageTree;
  locales: readonly LocaleId[];
  labels: typeof LOCALE_LABELS;
}

const I18nContext = createContext<I18nContextValue | null>(null);

function readStoredLocale(): LocaleId {
  try {
    const raw = localStorage.getItem(LOCALE_STORAGE_KEY);
    if (raw && (LOCALES as readonly string[]).includes(raw)) {
      return raw as LocaleId;
    }
  } catch {
    /* ignore */
  }
  return DEFAULT_LOCALE;
}

export function I18nProvider({ children }: { children: ReactNode }): JSX.Element {
  const [locale, setLocaleState] = useState<LocaleId>(() => readStoredLocale());

  const setLocale = useCallback((id: LocaleId) => {
    setLocaleState(id);
    try {
      localStorage.setItem(LOCALE_STORAGE_KEY, id);
    } catch {
      /* ignore */
    }
  }, []);

  const messages = useMemo(() => {
    if (locale === "en") return en;
    return deepMerge(en, catalogs[locale]);
  }, [locale]);

  const t = useCallback<TFunction>(
    (key, params) => translate(messages, key, params),
    [messages],
  );

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  const value = useMemo(
    () => ({
      locale,
      setLocale,
      t,
      messages,
      locales: LOCALES,
      labels: LOCALE_LABELS,
    }),
    [locale, setLocale, t, messages],
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error("useI18n must be used within I18nProvider");
  }
  return ctx;
}

/** Shorthand for t() only. */
export function useT(): TFunction {
  return useI18n().t;
}
