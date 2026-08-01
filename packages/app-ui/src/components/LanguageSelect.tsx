/**
 * Compact language picker — toolbar and Settings share this control.
 */

import { useI18n } from "../i18n/I18nProvider";
import type { LocaleId } from "../i18n/types";

interface LanguageSelectProps {
  /** Extra class on the wrapper. */
  className?: string;
  /** Show the "Language" label next to the select. */
  showLabel?: boolean;
}

export function LanguageSelect({
  className = "",
  showLabel = false,
}: LanguageSelectProps): JSX.Element {
  const { t, locale, setLocale, locales, labels } = useI18n();

  return (
    <label className={`lang-select ${className}`.trim()}>
      {showLabel && <span className="lang-select__label">{t("settings.language")}</span>}
      <select
        className="lang-select__control"
        value={locale}
        onChange={(e) => setLocale(e.target.value as LocaleId)}
        title={t("settings.language")}
        aria-label={t("settings.language")}
      >
        {locales.map((id) => (
          <option key={id} value={id}>
            {labels[id]}
          </option>
        ))}
      </select>
    </label>
  );
}
