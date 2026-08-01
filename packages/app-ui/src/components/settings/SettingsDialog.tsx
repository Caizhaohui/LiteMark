/**
 * M5/M6 settings: trusted workspaces, pandoc, wiki links, custom CSS path,
 * experimental flags, crash log export, update status.
 */

import { useEffect, useState } from "react";
import type { AppSettings, OptionalToolsStatus, UpdateStatus } from "@litemark/shared-protocol";
import * as cmd from "../../services/tauriCommands";

interface SettingsDialogProps {
  onClose: () => void;
}

export function SettingsDialog({ onClose }: SettingsDialogProps): JSX.Element {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [tools, setTools] = useState<OptionalToolsStatus | null>(null);
  const [update, setUpdate] = useState<UpdateStatus | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [trustPath, setTrustPath] = useState("");

  useEffect(() => {
    void (async () => {
      try {
        const [s, t, u] = await Promise.all([
          cmd.getSettings(),
          cmd.probeOptionalTools(),
          cmd.getUpdateStatus(),
        ]);
        setSettings(s);
        setTools(t);
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
      setNotice("Settings saved.");
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
      setNotice("Workspace trusted.");
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
      setNotice(`Crash report written to:\n${path}`);
    } catch (e) {
      setError(cmd.toCoreError(e).message);
    }
  };

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="settings-title">
      <div className="modal modal--wide modal--tall">
        <h2 id="settings-title" className="modal__title">
          Settings
        </h2>

        {!settings && !error && <p className="modal__body">Loading…</p>}
        {error && (
          <div className="export-form__warn" role="alert">
            {error}
          </div>
        )}
        {notice && <p className="export-form__hint">{notice}</p>}

        {settings && (
          <div className="settings">
            <section className="settings__section">
              <h3 className="settings__h">Trusted workspaces</h3>
              <p className="export-form__hint">
                New and downloaded documents are untrusted by default. Trusting a
                folder is required before experimental features can run there.
              </p>
              <ul className="settings__list">
                {settings.trustedWorkspaces.length === 0 && (
                  <li className="export-form__hint">No trusted workspaces yet.</li>
                )}
                {settings.trustedWorkspaces.map((p) => (
                  <li key={p} className="settings__row">
                    <code className="settings__code">{p}</code>
                    <button type="button" className="btn btn--small" onClick={() => void removeTrust(p)}>
                      Revoke
                    </button>
                  </li>
                ))}
              </ul>
              <div className="settings__row">
                <input
                  className="settings__input"
                  placeholder="Absolute folder path"
                  value={trustPath}
                  onChange={(e) => setTrustPath(e.target.value)}
                  aria-label="Workspace path to trust"
                />
                <button type="button" className="btn btn--small" onClick={() => void addTrust()}>
                  Trust
                </button>
              </div>
            </section>

            <section className="settings__section">
              <h3 className="settings__h">Optional tools</h3>
              <p className="export-form__hint">
                Missing tools never block the editor. Probes run only when Settings opens.
              </p>
              <ul className="settings__list">
                <li>
                  Pandoc:{" "}
                  {tools?.pandoc.available
                    ? `available (${tools.pandoc.version ?? tools.pandoc.path})`
                    : "not found"}
                </li>
                <li>
                  Graphviz:{" "}
                  {tools?.graphviz.available ? `available (${tools.graphviz.path})` : "not found"}
                </li>
                <li>
                  PlantUML:{" "}
                  {tools?.plantuml.available ? `available (${tools.plantuml.path})` : "not found"}
                </li>
              </ul>
              <label className="export-form__field">
                <span>Pandoc path (optional override)</span>
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
              <h3 className="settings__h">Preview / syntax</h3>
              <label className="export-form__row">
                <input
                  type="checkbox"
                  checked={settings.enableWikiLinks}
                  onChange={(e) =>
                    setSettings({ ...settings, enableWikiLinks: e.target.checked })
                  }
                />
                <span>Enable wiki-link syntax (preview)</span>
              </label>
              <label className="export-form__field">
                <span>Custom CSS path (sanitized; scripts blocked)</span>
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
                <span>Show experimental code-execution UI (never runs without confirm)</span>
              </label>
            </section>

            <section className="settings__section">
              <h3 className="settings__h">Updates & diagnostics</h3>
              <p className="export-form__hint">{update?.message}</p>
              <label className="export-form__field">
                <span>Update endpoint (empty = disabled)</span>
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
                Export crash report
              </button>
            </section>
          </div>
        )}

        <div className="modal__actions">
          <button type="button" className="btn" onClick={onClose}>
            Close
          </button>
          <button type="button" className="btn btn--primary" onClick={() => void save()} disabled={!settings}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
