import { Bot, KeyRound, Save, ShieldCheck, X } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState, type FormEvent } from "react";

interface SettingsModalProps {
  onClose: () => void;
  onSaved?: (settings: AiPlannerSettings) => void;
}

export interface AiPlannerSettings {
  enabled: boolean;
  endpoint: string;
  model: string;
  apiKeySet: boolean;
}

function errorText(error: unknown, fallback: string) {
  if (error instanceof Error) return error.message;
  if (typeof error === "string" && error.trim()) return error;
  return fallback;
}

export function SettingsModal({ onClose, onSaved }: SettingsModalProps) {
  const [settings, setSettings] = useState<AiPlannerSettings | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [clearApiKey, setClearApiKey] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    void invoke<AiPlannerSettings>("get_ai_planner_settings")
      .then(setSettings)
      .catch((reason) =>
        setError(errorText(reason, "Could not load AI planner settings.")),
      );
  }, []);

  async function save(event: FormEvent) {
    event.preventDefault();
    if (!settings) return;
    setSaving(true);
    setSaved(false);
    setError(null);
    try {
      const updated = await invoke<AiPlannerSettings>(
        "save_ai_planner_settings",
        {
          request: {
            enabled: settings.enabled,
            endpoint: settings.endpoint,
            model: settings.model,
            apiKey: apiKey || null,
            clearApiKey,
          },
        },
      );
      setSettings(updated);
      setApiKey("");
      setClearApiKey(false);
      setSaved(true);
      onSaved?.(updated);
    } catch (reason) {
      setError(errorText(reason, "Could not save AI planner settings."));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div
      className="help-backdrop"
      onMouseDown={(event) => event.target === event.currentTarget && onClose()}
    >
      <section
        className="settings-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
      >
        <header className="help-header">
          <div>
            <p className="help-eyebrow">TerminalMate configuration</p>
            <h2 id="settings-title">
              <Bot size={18} aria-hidden="true" /> Generative AI planner
            </h2>
          </div>
          <button
            className="icon-button"
            type="button"
            onClick={onClose}
            aria-label="Close settings"
            title="Close settings"
          >
            <X size={19} />
          </button>
        </header>

        <div className="help-safety-note">
          <ShieldCheck size={18} aria-hidden="true" />
          <span>
            Local matching runs first. AI can select only a supported structured
            action; Rust renders it and the normal risk and approval policy still
            decides whether it may run.
          </span>
        </div>

        {settings ? (
          <form className="settings-form" onSubmit={(event) => void save(event)}>
            <label className="settings-toggle">
              <span>
                <strong>Use generative AI for unmatched requests</strong>
                <small>Results are marked AI PLAN before review.</small>
              </span>
              <input
                type="checkbox"
                checked={settings.enabled}
                onChange={(event) => {
                  setSettings({ ...settings, enabled: event.target.checked });
                  setSaved(false);
                }}
              />
            </label>

            <label className="settings-field">
              <span>OpenAI-compatible chat completions endpoint</span>
              <input
                type="url"
                value={settings.endpoint}
                onChange={(event) => {
                  setSettings({ ...settings, endpoint: event.target.value });
                  setSaved(false);
                }}
                required
              />
            </label>

            <label className="settings-field">
              <span>Model</span>
              <input
                value={settings.model}
                onChange={(event) => {
                  setSettings({ ...settings, model: event.target.value });
                  setSaved(false);
                }}
                required
              />
            </label>

            <label className="settings-field">
              <span>
                <KeyRound size={14} aria-hidden="true" /> API key
              </span>
              <input
                type="password"
                value={apiKey}
                onChange={(event) => {
                  setApiKey(event.target.value);
                  setClearApiKey(false);
                  setSaved(false);
                }}
                placeholder={
                  settings.apiKeySet
                    ? "Key stored for this session - enter to replace"
                    : "Optional for local endpoints"
                }
                autoComplete="off"
              />
            </label>

            {settings.apiKeySet ? (
              <label className="settings-clear-key">
                <input
                  type="checkbox"
                  checked={clearApiKey}
                  onChange={(event) => setClearApiKey(event.target.checked)}
                />
                Remove the stored session key
              </label>
            ) : null}

            <p className="settings-privacy">
              The API key stays in Rust memory only. It is never returned to the
              interface, written to disk, or included in command logs, and is
              cleared when TerminalMate closes.
            </p>

            {error ? <p className="settings-error" role="alert">{error}</p> : null}
            <footer className="settings-footer">
              <span className="settings-saved" role="status">
                {saved ? "Saved for this session" : ""}
              </span>
              <button type="button" className="approval-cancel" onClick={onClose}>
                Cancel
              </button>
              <button type="submit" className="approval-approve" disabled={saving}>
                <Save size={14} /> {saving ? "Saving..." : "Save"}
              </button>
            </footer>
          </form>
        ) : error ? (
          <p className="settings-error" role="alert">{error}</p>
        ) : (
          <p className="settings-loading" role="status">Loading settings...</p>
        )}
      </section>
    </div>
  );
}
