import { invoke } from "@tauri-apps/api/core";
import { Check, FileEdit, Save, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";

interface FileEditorModalProps {
  path: string;
  workingDirectory: string;
  onClose: () => void;
}

function errorText(error: unknown, fallback: string) {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  return fallback;
}

export function FileEditorModal({
  path,
  workingDirectory,
  onClose,
}: FileEditorModalProps) {
  const [content, setContent] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [savedJustNow, setSavedJustNow] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    let cancelled = false;
    setContent(null);
    setLoadError(null);

    invoke<string>("read_editable_file", { workingDirectory, path })
      .then((text) => {
        if (!cancelled) {
          setContent(text);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setLoadError(errorText(error, "Could not open this file."));
        }
      });

    return () => {
      cancelled = true;
    };
  }, [path, workingDirectory]);

  useEffect(() => {
    if (content !== null) {
      textareaRef.current?.focus();
    }
  }, [content !== null]);

  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);

  async function save() {
    if (content === null) {
      return;
    }
    setSaving(true);
    setSaveError(null);
    setSavedJustNow(false);
    try {
      await invoke("write_editable_file", { workingDirectory, path, content });
      setSavedJustNow(true);
    } catch (error) {
      setSaveError(errorText(error, "Could not save this file."));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div
      className="help-backdrop"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <section
        aria-labelledby="file-editor-title"
        aria-modal="true"
        className="editor-dialog"
        role="dialog"
      >
        <header className="help-header">
          <div>
            <p className="help-eyebrow">Built-in editor</p>
            <h2 id="file-editor-title">
              <FileEdit size={16} aria-hidden="true" /> {path}
            </h2>
          </div>
          <button
            className="icon-button"
            type="button"
            onClick={onClose}
            aria-label="Close editor"
            title="Close editor"
          >
            <X size={19} />
          </button>
        </header>

        <div className="help-safety-note">
          <FileEdit size={18} aria-hidden="true" />
          <span>
            nano, vim, and other full-screen editors need a real terminal
            TerminalMate does not provide. This edits the file directly —
            no shell, no terminal — and Save overwrites{" "}
            <code>{path}</code> in <code>{workingDirectory}</code>.
          </span>
        </div>

        {loadError ? (
          <div className="editor-status editor-status-error" role="alert">
            {loadError}
          </div>
        ) : content === null ? (
          <div className="editor-status" role="status">
            Opening {path}...
          </div>
        ) : (
          <>
            <textarea
              ref={textareaRef}
              className="editor-textarea"
              value={content}
              onChange={(event) => {
                setContent(event.target.value);
                setSavedJustNow(false);
              }}
              spellCheck={false}
              autoCapitalize="off"
              autoComplete="off"
            />
            <footer className="editor-footer">
              {saveError ? (
                <span className="editor-save-error" role="alert">
                  {saveError}
                </span>
              ) : savedJustNow ? (
                <span className="editor-saved" role="status">
                  <Check size={14} /> Saved
                </span>
              ) : (
                <span className="editor-hint">
                  Changes are not saved until you press Save.
                </span>
              )}
              <button
                className="editor-save"
                type="button"
                disabled={saving}
                onClick={() => void save()}
              >
                <Save size={14} />
                {saving ? "Saving..." : "Save"}
              </button>
            </footer>
          </>
        )}
      </section>
    </div>
  );
}
