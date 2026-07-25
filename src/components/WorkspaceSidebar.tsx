import {
  FolderPlus,
  Search,
  SquareTerminal,
  Trash2
} from "lucide-react";
import { useMemo, useState } from "react";

import type { Workspace } from "../lib/types";

interface WorkspaceSidebarProps {
  workspaces: Workspace[];
  activeWorkspaceId: string | null;
  busy: boolean;
  onAdd: () => void;
  onSelect: (workspaceId: string) => void;
  onRemove: (workspaceId: string) => void;
}

export function WorkspaceSidebar({
  workspaces,
  activeWorkspaceId,
  busy,
  onAdd,
  onSelect,
  onRemove
}: WorkspaceSidebarProps) {
  const [query, setQuery] = useState("");
  const visibleWorkspaces = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    if (!normalized) {
      return workspaces;
    }

    return workspaces.filter((workspace) =>
      `${workspace.name} ${workspace.path}`.toLowerCase().includes(normalized)
    );
  }, [query, workspaces]);

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
          return (
            <div
              className={`workspace-row${active ? " active" : ""}`}
              key={workspace.id}
            >
              <button
                className="workspace-select"
                type="button"
                onClick={() => onSelect(workspace.id)}
              >
                <span className="workspace-avatar">
                  {workspace.name.charAt(0).toUpperCase()}
                </span>
                <span className="workspace-copy">
                  <strong>{workspace.name}</strong>
                  <span>{workspace.profile.runtimeName}</span>
                  <small>
                    <SquareTerminal size={12} />
                    {workspace.profile.shell}
                  </small>
                </span>
                <span className="workspace-state" aria-label={active ? "Active" : "Saved"} />
              </button>
              <button
                className="remove-workspace"
                type="button"
                onClick={() => onRemove(workspace.id)}
                aria-label={`Remove ${workspace.name}`}
                title="Remove workspace"
              >
                <Trash2 size={15} />
              </button>
            </div>
          );
        })}

        {visibleWorkspaces.length === 0 ? (
          <div className="sidebar-empty">
            <SquareTerminal size={24} />
            <p>{workspaces.length === 0 ? "No saved workspaces." : "No matches."}</p>
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

