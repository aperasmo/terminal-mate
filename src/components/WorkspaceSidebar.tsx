import {
  Check,
  FolderPlus,
  Pencil,
  Search,
  SquareTerminal,
  Trash2,
  TriangleAlert,
  X,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import type { FormEvent } from "react";

import type { Workspace } from "../lib/types";

interface WorkspaceSidebarProps {
  workspaces: Workspace[];
  activeWorkspaceId: string | null;
  busy: boolean;
  /** Workspace ids whose session's most recent run failed (not user-stopped),
   * so a warning can surface even for a workspace the user isn't currently
   * looking at — same idea as the tab warning badge in terminal apps like
   * Warp. */
  warnings?: Record<string, boolean>;
  onAdd: () => void;
  onSelect: (workspaceId: string) => void;
  onRemove: (workspaceId: string) => void;
  onRename: (workspaceId: string, alias: string) => string | null;
}

export function WorkspaceSidebar({
  workspaces,
  activeWorkspaceId,
  busy,
  warnings,
  onAdd,
  onSelect,
  onRemove,
  onRename,
}: WorkspaceSidebarProps) {
  const [query, setQuery] = useState("");
  const [editingWorkspaceId, setEditingWorkspaceId] = useState<string | null>(
    null,
  );
  const [aliasDraft, setAliasDraft] = useState("");
  const [aliasError, setAliasError] = useState<string | null>(null);
  const aliasInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editingWorkspaceId) {
      aliasInputRef.current?.focus();
      aliasInputRef.current?.select();
    }
  }, [editingWorkspaceId]);

  const visibleWorkspaces = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) {
      return workspaces;
    }

    return workspaces.filter((workspace) =>
      `${workspace.name} ${workspace.path}`.toLowerCase().includes(normalized),
    );
  }, [query, workspaces]);

  function beginRename(workspace: Workspace) {
    onSelect(workspace.id);
    setEditingWorkspaceId(workspace.id);
    setAliasDraft(workspace.name);
    setAliasError(null);
  }

  function cancelRename() {
    setEditingWorkspaceId(null);
    setAliasDraft("");
    setAliasError(null);
  }

  function submitRename(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!editingWorkspaceId) {
      return;
    }

    const error = onRename(editingWorkspaceId, aliasDraft);
    if (error) {
      setAliasError(error);
      aliasInputRef.current?.focus();
      return;
    }

    cancelRename();
  }

  return (
    <aside className="workspace-sidebar">
      <div className="panel-heading-row">
        <h2>Workspaces</h2>
        <button
          className="compact-button"
          type="button"
          onClick={onAdd}
          disabled={busy}
        >
          <FolderPlus size={16} />
          {busy ? "Opening..." : "Add"}
        </button>
      </div>

      <label className="search-field">
        <Search size={16} />
        <span className="sr-only">Filter workspaces</span>
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Filter workspaces..."
        />
      </label>

      <div className="workspace-list">
        {visibleWorkspaces.map((workspace) => {
          const active = workspace.id === activeWorkspaceId;
          const editing = workspace.id === editingWorkspaceId;
          return (
            <div
              className={`workspace-row${active ? " active" : ""}${editing ? " editing" : ""}`}
              key={workspace.id}
              title={editing ? undefined : workspace.path}
            >
              {editing ? (
                <form className="workspace-rename-form" onSubmit={submitRename}>
                  <label>
                    <span>Workspace alias</span>
                    <input
                      ref={aliasInputRef}
                      value={aliasDraft}
                      maxLength={60}
                      onChange={(event) => {
                        setAliasDraft(event.target.value);
                        setAliasError(null);
                      }}
                      onKeyDown={(event) => {
                        if (event.key === "Escape") {
                          cancelRename();
                        }
                      }}
                      aria-invalid={Boolean(aliasError)}
                      aria-describedby={
                        aliasError ? `alias-error-${workspace.id}` : undefined
                      }
                    />
                  </label>
                  <div className="workspace-rename-actions">
                    <button
                      type="submit"
                      aria-label="Save workspace alias"
                      title="Save alias"
                    >
                      <Check size={15} />
                    </button>
                    <button
                      type="button"
                      onClick={cancelRename}
                      aria-label="Cancel workspace rename"
                      title="Cancel rename"
                    >
                      <X size={15} />
                    </button>
                  </div>
                  <small className="workspace-rename-path">
                    {workspace.path}
                  </small>
                  {aliasError ? (
                    <small
                      className="workspace-alias-error"
                      id={`alias-error-${workspace.id}`}
                      role="alert"
                    >
                      {aliasError}
                    </small>
                  ) : null}
                </form>
              ) : (
                <>
                  <button
                    className="workspace-select"
                    type="button"
                    onClick={() => onSelect(workspace.id)}
                  >
                    <span className="workspace-avatar">
                      {workspace.name.charAt(0).toUpperCase()}
                    </span>
                    <span className="workspace-copy">
                      <span className="workspace-name-row">
                        <strong>{workspace.name}</strong>
                        {warnings?.[workspace.id] ? (
                          <span title="Last run failed">
                            <TriangleAlert
                              className="workspace-warning-icon"
                              size={12}
                              aria-label="Last run failed"
                            />
                          </span>
                        ) : null}
                      </span>
                      <span>{workspace.profile.runtimeName}</span>
                      <small>
                        <SquareTerminal size={12} />
                        {workspace.profile.shell}
                      </small>
                    </span>
                    <span
                      className="workspace-state"
                      aria-label={active ? "Active" : "Saved"}
                    />
                  </button>
                  <div className="workspace-actions">
                    <button
                      className="workspace-action rename-workspace"
                      type="button"
                      onClick={() => beginRename(workspace)}
                      aria-label={`Rename ${workspace.name}`}
                      title="Rename workspace alias"
                    >
                      <Pencil size={14} />
                    </button>
                    <button
                      className="workspace-action remove-workspace"
                      type="button"
                      onClick={() => onRemove(workspace.id)}
                      aria-label={`Remove ${workspace.name}`}
                      title="Remove workspace"
                    >
                      <Trash2 size={15} />
                    </button>
                  </div>
                </>
              )}
            </div>
          );
        })}

        {visibleWorkspaces.length === 0 ? (
          <div className="sidebar-empty">
            <SquareTerminal size={24} />
            <p>
              {workspaces.length === 0 ? "No saved workspaces." : "No matches."}
            </p>
          </div>
        ) : null}
      </div>

      <div className="sidebar-footer">
        <span className="status-dot" />
        Local workspace storage
      </div>
    </aside>
  );
}
