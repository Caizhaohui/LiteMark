/**
 * M5/M6 settings: language, trusted workspaces, pandoc, wiki links, custom CSS path,
 * experimental flags, crash log export, update status.
 */

import { useEffect, useState } from "react";
import type { AppSettings, OptionalToolsStatus, UpdateStatus } from "@litemark/shared-protocol";
import { useI18n } from "../../i18n/I18nProvider";
import { LanguageSelect } from "../LanguageSelect";
import * as cmd from "../../services/tauriCommands";

interface SettingsDialogProps {
  onClose: () => void;
}

export function SettingsDialog({ onClose }: SettingsDialogProps): JSX.Element {
  const { t } = useI18n();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [tools, setTools] = useState<OptionalToolsStatus | null>(null);
  const [update, setUpdate] = useState<UpdateStatus | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [trustPath, setTrustPath] = useState("");

  useEffect(() => {
    void (async () => {
      try {
        const [s, tStatus, u] = await Promise.all([
          cmd.getSettings(),
          cmd.probeOptionalTools(),
          cmd.getUpdateStatus(),
        ]);
        setSettings(s);
        setTools(tStatus);
        setUpdate(u);
      } catch (e) {
        setError(cmd.toCoreError(e).message);
      }
    })();
  }, []);

  const save = async () => {
    if (!settings) return;
    try {
      await cmd.setSettings(settings);
      setNotice(t("settings.saved"));
      setError(null);
    } catch (e) {
      setError(cmd.toCoreError(e).message);
    }
  };

  const addTrust = async () => {
    if (!trustPath.trim()) return;
    try {
      const s = await cmd.trustWorkspace(trustPath.trim());
      setSettings(s);
      setTrustPath("");
      setNotice(t("settings.workspaceTrusted"));
    } catch (e) {
      setError(cmd.toCoreError(e).message);
    }
  };

  const removeTrust = async (path: string) => {
    try {
      const s = await cmd.untrustWorkspace(path);
      setSettings(s);
    } catch (e) {
      setError(cmd.toCoreError(e).message);
    }
  };

  const dumpLog = async () => {
    try {
      const path = await cmd.exportCrashLog();
      setNotice(t("settings.crashWritten", { path }));
    } catch (e) {
      setError(cmd.toCoreError(e).message);
    }
  };

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="settings-title">
      <div className="modal modal--wide modal--tall">
        <h2 id="settings-title" className="modal__title">
          {t("settings.title")}
        </h2>

        {/* Language is client-side only — always shown, even if backend settings fail. */}
        <div className="settings">
          <section className="settings__section">
            <h3 className="settings__h">{t("settings.language")}</h3>
            <p className="export-form__hint">{t("settings.languageHint")}</p>
            <LanguageSelect showLabel className="lang-select--block" />
          </section>
        </div>

        {!settings && !error && <p className="modal__body">{t("settings.loading")}</p>}
        {error && (
          <div className="export-form__warn" role="alert">
            {error}
          </div>
        )}
        {notice && <p className="export-form__hint">{notice}</p>}

        {settings && (
          <div className="settings">
            <section className="settings__section">
              <h3 className="settings__h">{t("settings.trustedTitle")}</h3>
              <p className="export-form__hint">{t("settings.trustedHint")}</p>
              <ul className="settings__list">
                {settings.trustedWorkspaces.length === 0 && (
                  <li className="export-form__hint">{t("settings.noTrusted")}</li>
                )}
                {settings.trustedWorkspaces.map((p) => (
                  <li key={p} className="settings__row">
                    <code className="settings__code">{p}</code>
                    <button type="button" className="btn btn--small" onClick={() => void removeTrust(p)}>
                      {t("settings.revoke")}
                    </button>
                  </li>
                ))}
              </ul>
              <div className="settings__row">
                <input
                  className="settings__input"
                  placeholder={t("settings.trustPlaceholder")}
                  value={trustPath}
                  onChange={(e) => setTrustPath(e.target.value)}
                  aria-label={t("settings.trustAria")}
                />
                <button type="button" className="btn btn--small" onClick={() => void addTrust()}>
                  {t("settings.trust")}
                </button>
              </div>
            </section>

            <section className="settings__section">
              <h3 className="settings__h">{t("settings.toolsTitle")}</h3>
              <p className="export-form__hint">{t("settings.toolsHint")}</p>
              <ul className="settings__list">
                <li>
                  {t("settings.pandoc")}:{" "}
                  {tools?.pandoc.available
                    ? t("settings.available", {
                        detail: tools.pandoc.version ?? tools.pandoc.path ?? "",
                      })
                    : t("settings.notFound")}
                </li>
                <li>
                  {t("settings.graphviz")}:{" "}
                  {tools?.graphviz.available
                    ? t("settings.available", { detail: tools.graphviz.path ?? "" })
                    : t("settings.notFound")}
                </li>
                <li>
                  {t("settings.plantuml")}:{" "}
                  {tools?.plantuml.available
                    ? t("settings.available", { detail: tools.plantuml.path ?? "" })
                    : t("settings.notFound")}
                </li>
              </ul>
              <label className="export-form__field">
                <span>{t("settings.pandocPath")}</span>
                <input
                  className="settings__input"
                  value={settings.pandocPath ?? ""}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      pandocPath: e.target.value || null,
                    })
                  }
                />
              </label>
            </section>

            <section className="settings__section">
              <h3 className="settings__h">{t("settings.previewTitle")}</h3>
              <label className="export-form__row">
                <input
                  type="checkbox"
                  checked={settings.enableWikiLinks}
                  onChange={(e) =>
                    setSettings({ ...settings, enableWikiLinks: e.target.checked })
                  }
                />
                <span>{t("settings.wikiLinks")}</span>
              </label>
              <label className="export-form__field">
                <span>{t("settings.customCss")}</span>
                <input
                  className="settings__input"
                  value={settings.customCssPath ?? ""}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      customCssPath: e.target.value || null,
                    })
                  }
                />
              </label>
              <label className="export-form__row">
                <input
                  type="checkbox"
                  checked={settings.experimentalCodeExecution}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      experimentalCodeExecution: e.target.checked,
                    })
                  }
                />
                <span>{t("settings.experimental")}</span>
              </label>
            </section>

            <section className="settings__section">
              <h3 className="settings__h">{t("settings.updatesTitle")}</h3>
              <p className="export-form__hint">{update?.message}</p>
              <label className="export-form__field">
                <span>{t("settings.updateEndpoint")}</span>
                <input
                  className="settings__input"
                  value={settings.updateEndpoint ?? ""}
                  onChange={(e) =>
                    setSettings({
                      ...settings,
                      updateEndpoint: e.target.value || null,
                    })
                  }
                  placeholder="https://…"
                />
              </label>
              <button type="button" className="btn" onClick={() => void dumpLog()}>
                {t("settings.exportCrash")}
              </button>
            </section>
          </div>
        )}

        <div className="modal__actions">
          <button type="button" className="btn" onClick={onClose}>
            {t("settings.close")}
          </button>
          <button type="button" className="btn btn--primary" onClick={() => void save()} disabled={!settings}>
            {t("settings.save")}
          </button>
        </div>
      </div>
    </div>
  );
}
